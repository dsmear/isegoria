//! A tiered network of four members, `t = 3`, running the answer review on cuts (`docs/05`
//! §Distributed integration): its feeds, persons, proofs in either scope, and cuts.
#![allow(dead_code)]

use ed25519_dalek::Signature;
use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::beacon::{BeaconRound, RoundId};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Replica, SignedEntry};
use network::tier::{Tier, Tiers};
use protocol::answer::{commit_answer, Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::{AnswerLedger, AnswerLedgerError, CutReport};
use protocol::answer_path::{commit_context, network_context, AnswerConfig, POOL_CONTEXT};
use protocol::answer_rule::ConsensusRule;
use protocol::deposit::deposit_context;
use std::collections::{BTreeMap, BTreeSet};

pub const NET: [u8; 32] = [0x79; 32];
pub const OTHER: [u8; 32] = [0x80; 32];
pub const N: usize = 4;
pub const T: usize = 3;

pub fn members() -> Vec<Member> {
    (0..N)
        .map(|i| Member::from_seed([10 + i as u8; 32]))
        .collect()
}

pub fn consortium() -> Consortium {
    Consortium::new(members().iter().map(Member::public).collect(), T)
}

fn writer(m: usize, tier: Tier) -> FeedWriter {
    let base = if tier == Tier::Public { 10 } else { 20 };
    FeedWriter::from_seed(NET, [base + m as u8; 32])
}

/// Every writer key of the tiered set: the members' consortium keys, then their circle keys.
pub fn writer_keys() -> Vec<ed25519_dalek::VerifyingKey> {
    let public = (0..N).map(|m| writer(m, Tier::Public).public());
    public
        .chain((0..N).map(|m| writer(m, Tier::Circle).public()))
        .collect()
}

pub fn tiers_on(network: [u8; 32]) -> Tiers {
    let circle: Vec<_> = (0..N).map(|m| writer(m, Tier::Circle).public()).collect();
    Tiers::new(network, consortium(), &circle).expect("a valid tiered set")
}

pub fn tiers() -> Tiers {
    tiers_on(NET)
}

pub fn round(epoch: u64) -> RoundId {
    RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch,
    }
}

/// Round `epoch`'s value with every member committing and `revealing` revealing, in process.
pub fn value(epoch: u64, revealing: &BTreeSet<usize>) -> Option<[u8; 32]> {
    let c = consortium();
    let mut r = BeaconRound::open(&c, NET, epoch);
    let pairs: Vec<_> = (members().iter().enumerate())
        .map(|(i, m)| m.beacon_commit(r.id(), i))
        .collect();
    for (commit, _) in &pairs {
        r.commit(commit).unwrap();
    }
    r.close_commits().unwrap();
    r.close_deposits().unwrap();
    for &i in revealing {
        r.reveal(&pairs[i].1).unwrap();
    }
    r.finish().unwrap().value()
}

/// A group of 3, a quorum of 2, three quarters: smaller rounds than D49's, for speed.
pub fn small() -> AnswerConfig {
    AnswerConfig {
        rule: ConsensusRule::new(3, 2, 3, 4).unwrap(),
        ..AnswerConfig::default()
    }
}

pub struct Person {
    pub cred: AnonymousCredential,
    pub judge: Nym,
    pub propose: Nym,
}

pub fn person(issuer: &Issuer, secret: u8) -> Person {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    let cred = pending.finalize(issuer.issue(&req).unwrap());
    let pk = issuer.public();
    let judge = prove(&cred, &pk, Role::Judge, b"nym").id();
    let propose = prove(&cred, &pk, Role::Propose, b"nym").id();
    Person {
        cred,
        judge,
        propose,
    }
}

pub fn question(tag: &str) -> Question {
    Question {
        context: b"civic education, 2026".to_vec(),
        stem: tag.as_bytes().to_vec(),
        alternatives: vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()],
        key: 0,
    }
}

/// `offline` as network `scope` binds it; unchanged for `None`, the offline path.
pub fn context(scope: Option<&[u8; 32]>, offline: &[u8]) -> Vec<u8> {
    scope.map_or_else(|| offline.to_vec(), |n| network_context(n, offline))
}

pub fn join(issuer: &Issuer, p: &Person, scope: Option<&[u8; 32]>) -> AnswerEvent {
    let ctx = context(scope, POOL_CONTEXT);
    AnswerEvent::JoinPool {
        proof: prove(&p.cred, &issuer.public(), Role::Judge, &ctx),
    }
}

pub fn deposit(
    issuer: &Issuer,
    a: &Person,
    q: &Question,
    epoch: u64,
    scope: Option<&[u8; 32]>,
) -> AnswerEvent {
    let ctx = context(scope, &deposit_context(q.content_id(), epoch));
    AnswerEvent::Deposit {
        epoch,
        question: q.clone(),
        proof: prove(&a.cred, &issuer.public(), Role::Propose, &ctx),
    }
}

pub fn commit(
    issuer: &Issuer,
    p: &Person,
    q: u64,
    answer: Answer,
    nonce: [u8; 32],
    scope: Option<&[u8; 32]>,
) -> AnswerEvent {
    let commitment = commit_answer(answer, &nonce, p.judge, q);
    let ctx = context(scope, &commit_context(q, &commitment));
    AnswerEvent::Commit {
        question: q,
        commitment,
        proof: prove(&p.cred, &issuer.public(), Role::Judge, &ctx),
    }
}

pub fn reveal(p: &Person, q: u64, answer: Answer, nonce: [u8; 32]) -> AnswerEvent {
    AnswerEvent::Reveal {
        question: q,
        reviewer: p.judge,
        answer,
        nonce,
    }
}

/// A cut applied, the view its signatures were given in, and the signatures.
pub type Applied = (Cut, u64, Vec<(usize, Signature)>);

/// The members' feeds on one replica and a ledger over its cuts; each epoch's beacon commits
/// are published as it opens, its reveals after its closing cut by members not in `withhold`.
pub struct Net {
    pub issuer: Issuer,
    feeds: BTreeMap<(usize, bool), (FeedWriter, TransparencyLog)>,
    pub replica: Replica,
    pub ledger: AnswerLedger,
    pub cuts: Vec<Applied>,
    pub withhold: BTreeSet<usize>,
    pub beacon: BTreeSet<EntryId>,
}

impl Net {
    pub fn new(config: AnswerConfig) -> Net {
        let issuer = Issuer::new([1u8; 32]);
        let mut feeds = BTreeMap::new();
        for m in 0..N {
            for tier in [Tier::Public, Tier::Circle] {
                let feed = (writer(m, tier), TransparencyLog::new());
                feeds.insert((m, tier == Tier::Circle), feed);
            }
        }
        let ledger = AnswerLedger::new(tiers(), issuer.public(), config).expect("a valid config");
        let mut net = Net {
            replica: Replica::new(tiers().writers().clone()),
            issuer,
            feeds,
            ledger,
            cuts: Vec::new(),
            withhold: BTreeSet::new(),
            beacon: BTreeSet::new(),
        };
        net.commits(0);
        net
    }

    pub fn publish(&mut self, m: usize, tier: Tier, object: Vec<u8>) -> EntryId {
        let (writer, log) = self.feeds.get_mut(&(m, tier == Tier::Circle)).unwrap();
        let entry = log.append(cid(&object)).clone();
        let signed = writer.sign(&entry);
        let id = signed.id();
        self.replica
            .insert(signed, object)
            .expect("a writer of the set");
        id
    }

    /// `event` on member `m`'s circle feed.
    pub fn log(&mut self, m: usize, event: &AnswerEvent) -> EntryId {
        self.publish(m, Tier::Circle, event.encode())
    }

    pub fn request(
        &mut self,
        by: impl IntoIterator<Item = usize>,
        e: &AnswerEvent,
    ) -> Vec<EntryId> {
        by.into_iter().map(|m| self.log(m, e)).collect()
    }

    fn commits(&mut self, epoch: u64) {
        for (i, m) in members().iter().enumerate() {
            let object = MemberObject::Commit(m.beacon_commit(round(epoch), i).0);
            let id = self.publish(i, Tier::Public, object.encode());
            self.beacon.insert(id);
        }
    }

    fn reveals(&mut self, epoch: u64) {
        for (i, m) in members().iter().enumerate() {
            if !self.withhold.contains(&i) {
                let reveal = m.beacon_commit(round(epoch), i).1;
                let object = MemberObject::Reveal { epoch, reveal };
                let id = self.publish(i, Tier::Public, object.encode());
                self.beacon.insert(id);
            }
        }
    }

    /// The next cut as the ledger's checks require it.
    pub fn propose(&self) -> Cut {
        let next = self.ledger.next().expect("a next cut");
        Cut::next(&self.replica, self.ledger.last(), next.epoch, next.closes)
    }

    /// `T` members' signatures over `cut` in view 0, none of a member rule 6 excludes.
    pub fn sign(&self, cut: &Cut) -> Vec<(usize, Signature)> {
        self.sign_in(cut, 0)
    }

    pub fn sign_in(&self, cut: &Cut, view: u64) -> Vec<(usize, Signature)> {
        let excluded = self.ledger.excluded(cut);
        let signers = (0..N).filter(|i| !excluded.contains(i)).take(T);
        self.signed_by(cut, view, signers)
    }

    pub fn signed_by(
        &self,
        cut: &Cut,
        view: u64,
        by: impl IntoIterator<Item = usize>,
    ) -> Vec<(usize, Signature)> {
        let cp = cut.checkpoint_in(view, NET, consortium().member_set_hash());
        by.into_iter()
            .map(|i| (i, members()[i].sign(&cp)))
            .collect()
    }

    pub fn apply(&mut self, cut: Cut) -> Result<CutReport, AnswerLedgerError> {
        self.apply_in(cut, 0)
    }

    pub fn apply_in(&mut self, cut: Cut, view: u64) -> Result<CutReport, AnswerLedgerError> {
        let sigs = self.sign_in(&cut, view);
        let report = self
            .ledger
            .apply_in_view(&self.replica, &cut, view, &sigs)?;
        if cut.closes {
            self.reveals(cut.epoch);
            self.commits(cut.epoch + 1);
        }
        self.cuts.push((cut, view, sigs));
        Ok(report)
    }

    /// Applies every cut so far, in its view, to `ledger` on `replica`.
    pub fn replay(&self, ledger: &mut AnswerLedger, replica: &Replica) {
        for (cut, view, sigs) in &self.cuts {
            ledger.apply_in_view(replica, cut, *view, sigs).unwrap();
        }
    }

    pub fn cut(&mut self) -> CutReport {
        let cut = self.propose();
        self.apply(cut).expect("the proposal applies")
    }

    /// `T` members ask to close the open epoch; the cuts that count them, close it and finish
    /// its round. The last cut's report.
    pub fn close_epoch(&mut self) -> CutReport {
        let epoch = self.ledger.state().epoch();
        self.request(0..T, &AnswerEvent::RequestCloseEpoch { epoch });
        self.cut();
        assert_eq!(self.cut().sealed, Some(epoch), "the closing cut");
        self.cut()
    }

    /// Every signed entry and its object, in the replica's order.
    pub fn entries(&self) -> Vec<(SignedEntry, Vec<u8>)> {
        let ids: Vec<EntryId> = self.replica.ids().collect();
        let get = |id: &EntryId| self.replica.get(id).unwrap();
        ids.iter()
            .map(|id| (get(id).0.clone(), get(id).1.to_vec()))
            .collect()
    }
}
