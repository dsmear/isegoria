//! A node's admission state rebuilt from its log (`docs/04` §Events and replay, `docs/08`
//! PROTO-014, T73): after a restart, or on another node, replay gives the same state.

use identity::credential::{AnonymousCredential, Credential, Issuer, IssuerPublic};
use identity::enrollment::Label;
use identity::nullifier::{prove, NullifierProof};
use identity::nym::Role;
use network::cid::{cid, Cid};
use network::store::{DurableLog, ObjectStore};
use protocol::admission::Unproven;
use protocol::deposit::{deposit_context, DepositRejected, Draft};
use protocol::events::NodeEvent;
use protocol::node::{Node, NodeError, NodeState, Rejection};
use protocol::pilot::{response_context, ResponseRejected};
use protocol::review::{review_context, ReviewRejected};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const EPOCH: u64 = 4;

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("node_replay")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn issuer() -> Issuer {
    Issuer::new([1u8; 32])
}

fn person(issuer: &Issuer, secret: u8) -> AnonymousCredential {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    pending.finalize(issuer.issue(&req).unwrap())
}

fn draft(tag: &str) -> Draft {
    Draft {
        item: tag.as_bytes().to_vec(),
        primary_source: b"Gazzetta Ufficiale".to_vec(),
    }
}

fn deposit(issuer: &Issuer, who: &AnonymousCredential, tag: &str) -> NodeEvent {
    let d = draft(tag);
    let ctx = deposit_context(d.content_id(), EPOCH);
    NodeEvent::Deposit {
        epoch: EPOCH,
        quota: 1,
        proof: prove(who, &issuer.public(), Role::Propose, &ctx),
        draft: d,
    }
}

fn reviewer(issuer: &Issuer, who: &AnonymousCredential, item: Cid) -> NodeEvent {
    let ctx = review_context(item, EPOCH);
    NodeEvent::AdmitReviewer {
        item,
        epoch: EPOCH,
        proof: prove(who, &issuer.public(), Role::Judge, &ctx),
    }
}

fn respondent(issuer: &Issuer, who: &AnonymousCredential, batch: Cid) -> NodeEvent {
    let ctx = response_context(batch, EPOCH);
    NodeEvent::AdmitRespondent {
        batch,
        epoch: EPOCH,
        proof: prove(who, &issuer.public(), Role::Respond, &ctx),
    }
}

/// A node that accepted two deposits, two reviewers and two respondents.
fn busy_node(dir: &Path, issuer: &Issuer) -> (Vec<AnonymousCredential>, Cid, Cid) {
    let people: Vec<AnonymousCredential> = (2..5).map(|s| person(issuer, s)).collect();
    let (item, batch) = (draft("item A").content_id(), cid(b"batch 1"));
    let mut node = Node::open(dir, issuer.public()).unwrap();
    node.submit(deposit(issuer, &people[0], "item A")).unwrap();
    node.submit(deposit(issuer, &people[1], "item B")).unwrap();
    node.submit(reviewer(issuer, &people[1], item)).unwrap();
    node.submit(reviewer(issuer, &people[2], item)).unwrap();
    node.submit(respondent(issuer, &people[0], batch)).unwrap();
    node.submit(respondent(issuer, &people[2], batch)).unwrap();
    assert_eq!(node.log_len(), 6);
    (people, item, batch)
}

/// AT-PRO-10: after a restart the node replays its log to the state it had.
#[test]
fn at_pro_10_a_restarted_node_replays_to_the_state_it_had() {
    let (issuer, dir) = (issuer(), scratch("restart"));
    let (people, item, batch) = busy_node(&dir, &issuer);
    let before = {
        let node = Node::open(&dir, issuer.public()).unwrap();
        assert_eq!(node.log_len(), 6);
        let s = node.state();
        assert_eq!(s.drafts().len(), 2);
        assert!(s.drafts().contains(&item));
        assert_eq!(s.panel(item, EPOCH).map(|p| p.len()), Some(2));
        assert_eq!(s.respondents(batch, EPOCH).map(|p| p.len()), Some(2));
        assert!(s.panel(item, EPOCH + 1).is_none());
        node
    };
    let proposer = prove(&people[0], &issuer.public(), Role::Propose, b"any").id();
    assert_eq!(before.state().quota_used(EPOCH, &proposer), 1);
    assert_eq!(before.state().quota_used(EPOCH + 1, &proposer), 0);
    drop(before);

    let mut node = Node::open(&dir, issuer.public()).unwrap();
    let rejected = |r| Err(NodeError::Rejected(r));
    assert_eq!(
        node.submit(deposit(&issuer, &people[0], "item A")),
        rejected(Rejection::Deposit(DepositRejected::DuplicateCid))
    );
    assert_eq!(
        node.submit(deposit(&issuer, &people[0], "item C")),
        rejected(Rejection::Deposit(DepositRejected::OverQuota))
    );
    assert_eq!(
        node.submit(reviewer(&issuer, &people[1], item)),
        rejected(Rejection::Review(ReviewRejected::Duplicate))
    );
    assert_eq!(
        node.submit(respondent(&issuer, &people[2], batch)),
        rejected(Rejection::Response(ResponseRejected::Duplicate))
    );
    assert_eq!(node.log_len(), 6, "a rejected event leaves no trace");
    node.submit(deposit(&issuer, &people[2], "item C")).unwrap();
    assert_eq!(node.log_len(), 7);
}

/// AT-PRO-10: the same files replay to the same state on another node.
#[test]
fn at_pro_10_the_same_log_replays_to_the_same_state_anywhere() {
    let (issuer, dir) = (issuer(), scratch("origin"));
    busy_node(&dir, &issuer);
    let copy = scratch("copy");
    fs::create_dir_all(&copy).unwrap();
    for f in ["log", "objects"] {
        fs::copy(dir.join(f), copy.join(f)).unwrap();
    }
    let a = Node::open(&dir, issuer.public()).unwrap();
    let b = Node::open(&copy, issuer.public()).unwrap();
    assert_eq!(a.state(), b.state());
    assert_ne!(
        a.state(),
        Node::open(&scratch("empty"), issuer.public())
            .unwrap()
            .state()
    );
}

/// AT-PRO-10: every event is checked as the entry point checks it, proofs included.
#[test]
fn at_pro_10_events_are_checked_like_the_entry_points() {
    let (issuer, dir) = (issuer(), scratch("checks"));
    let who = person(&issuer, 2);
    let mut node = Node::open(&dir, issuer.public()).unwrap();
    let mut no_source = deposit(&issuer, &who, "item");
    if let NodeEvent::Deposit { draft, .. } = &mut no_source {
        draft.primary_source.clear();
    }
    assert_eq!(
        node.submit(no_source),
        Err(NodeError::Rejected(Rejection::Deposit(
            DepositRejected::NoPrimarySource
        )))
    );
    let item = cid(b"item");
    let wrong_role = NodeEvent::AdmitReviewer {
        item,
        epoch: EPOCH,
        proof: prove(
            &who,
            &issuer.public(),
            Role::Respond,
            &review_context(item, EPOCH),
        ),
    };
    assert_eq!(
        node.submit(wrong_role),
        Err(NodeError::Rejected(Rejection::Review(
            ReviewRejected::Unproven(Unproven::WrongRole)
        )))
    );
    let other_epoch = NodeEvent::AdmitRespondent {
        batch: item,
        epoch: EPOCH + 1,
        proof: prove(
            &who,
            &issuer.public(),
            Role::Respond,
            &response_context(item, EPOCH),
        ),
    };
    assert_eq!(
        node.submit(other_epoch),
        Err(NodeError::Rejected(Rejection::Response(
            ResponseRejected::Unproven(Unproven::BadProof)
        )))
    );
    assert_eq!(node.log_len(), 0);
    assert_eq!(node.state(), &NodeState::default(), "nor in memory");
}

/// AT-PRO-10: a log that names a missing object, a non-event, or an event this node rejects
/// refuses the node, at that entry.
#[test]
fn at_pro_10_a_log_this_node_cannot_replay_refuses_it() {
    let issuer = issuer();
    let dir = scratch("missing");
    busy_node(&dir, &issuer);
    fs::remove_file(dir.join("objects")).unwrap();
    assert_eq!(
        Node::open(&dir, issuer.public()).err(),
        Some(NodeError::MissingObject { entry: 0 })
    );

    let dir = scratch("junk");
    busy_node(&dir, &issuer);
    let junk = {
        let (mut objects, _) = ObjectStore::open(&dir.join("objects")).unwrap();
        objects.put(b"not an event").unwrap()
    };
    DurableLog::open(&dir.join("log"))
        .unwrap()
        .0
        .append(junk)
        .unwrap();
    assert_eq!(
        Node::open(&dir, issuer.public()).err(),
        Some(NodeError::Undecodable { entry: 6 })
    );

    let dir = scratch("other-issuer");
    busy_node(&dir, &issuer);
    assert_eq!(
        Node::open(&dir, Issuer::new([2u8; 32]).public()).err(),
        Some(NodeError::ReplayRejected {
            entry: 0,
            rejection: Rejection::Deposit(DepositRejected::Unproven(Unproven::BadProof)),
        })
    );
}

/// AT-PRO-10: every event survives its encoding, and hostile bytes decode to nothing.
#[test]
fn at_pro_10_events_round_trip_and_hostile_bytes_are_refused() {
    let issuer = issuer();
    let who = person(&issuer, 2);
    let item = cid(b"item");
    let events = [
        deposit(&issuer, &who, "item"),
        reviewer(&issuer, &who, item),
        respondent(&issuer, &who, item),
    ];
    for event in &events {
        let bytes = event.encode();
        let decoded = NodeEvent::decode(&bytes).expect("an encoded event decodes");
        assert_eq!(decoded.encode(), bytes);
        for cut in 0..bytes.len() {
            assert!(NodeEvent::decode(&bytes[..cut]).is_none(), "cut at {cut}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(NodeEvent::decode(&longer).is_none());
        let mut other_version = bytes.clone();
        other_version[0] = 2;
        assert!(NodeEvent::decode(&other_version).is_none());
    }
    let mut unknown = events[1].encode();
    unknown[1] = 9;
    assert!(NodeEvent::decode(&unknown).is_none());
    assert!(NodeEvent::decode(&[]).is_none());
}

/// AT-PRO-10: a proof keeps its role through its encoding, and verifies after it.
#[test]
fn at_pro_10_a_proof_keeps_its_role_through_its_encoding() {
    let issuer = issuer();
    let who = person(&issuer, 3);
    for role in [Role::Propose, Role::Judge, Role::Respond] {
        let proof = prove(&who, &issuer.public(), role, b"ctx");
        let bytes = proof.encode();
        assert_eq!(bytes[0], role as u8);
        let decoded = NullifierProof::decode(&bytes).unwrap();
        assert_eq!((decoded.role(), decoded.id()), (role, proof.id()));
        assert!(identity::nullifier::verify(
            &decoded,
            &issuer.public(),
            b"ctx"
        ));
        assert_eq!(decoded.encode(), bytes);
    }
    let bytes = prove(&who, &issuer.public(), Role::Judge, b"ctx").encode();
    let mut bad_role = bytes.clone();
    bad_role[0] = 3;
    assert!(NullifierProof::decode(&bad_role).is_none());
    assert!(NullifierProof::decode(&bytes[..bytes.len() - 1]).is_none());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(NullifierProof::decode(&trailing).is_none());
    let _: &IssuerPublic = &issuer.public();
}

/// AT-PRO-10: an event whose proof decodes from a non-canonical encoding (a repeated entry
/// inside the BBS+ proof, found by fuzzing) is refused, so one event has one CID.
#[test]
fn at_pro_10_a_non_canonical_encoding_is_refused() {
    let bytes = include_bytes!("fixtures/noncanonical_event.bin");
    assert!(NodeEvent::decode(bytes).is_none());
}
