//! The replicated set's tiers (`docs/04` §Who reads, `docs/01` D54, `docs/10` T77 step 1): the
//! tiered writer set, the transport binding, the filtered pull sync and the public cut reader.

use ed25519_dalek::{Signature, VerifyingKey};
use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{
    added, collect, collect_with, member_objects, observed, sign_cut, Collected, Cut, CutError,
    Mark, MemberObject, Observed,
};
use network::log::TransparencyLog;
use network::observer::{ObserveError, Observer};
use network::replica::{
    EntryId, FeedWriter, Replica, SignedEntry, Summary, WriterSummary, MAX_RESPONSE,
};
use network::rounds::Rounds;
use network::tier::{Access, Tier, TierError, Tiers, TransportBinding};
use proptest::prelude::*;
use std::sync::OnceLock;

const NET: [u8; 32] = [22; 32];
const MEMBER_SEEDS: [u8; 3] = [30, 31, 32];
const CIRCLE_SEEDS: [u8; 3] = [40, 41, 42];
const TRANSPORT: [u8; 32] = [77; 32];

type Item = (SignedEntry, Vec<u8>);

fn members() -> Vec<Member> {
    MEMBER_SEEDS
        .iter()
        .map(|s| Member::from_seed([*s; 32]))
        .collect()
}

fn member_keys() -> Vec<VerifyingKey> {
    members().iter().map(|m| m.public()).collect()
}

fn consortium() -> Consortium {
    Consortium::new(member_keys(), 2)
}

fn circle_keys() -> Vec<VerifyingKey> {
    CIRCLE_SEEDS
        .iter()
        .map(|s| FeedWriter::from_seed(NET, [*s; 32]).public())
        .collect()
}

fn tiers() -> Tiers {
    Tiers::new(NET, consortium(), &circle_keys()).expect("a valid tiered set")
}

fn key(m: usize, tier: Tier) -> [u8; 32] {
    match tier {
        Tier::Public => member_keys()[m].to_bytes(),
        Tier::Circle => circle_keys()[m].to_bytes(),
    }
}

fn round(epoch: u64) -> RoundId {
    RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch,
    }
}

/// Writers 0–2 are the members' public feeds (their consortium keys), 3–5 their circle feeds.
struct Net {
    writers: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    replica: Replica,
}

impl Net {
    fn new() -> Net {
        let writers = MEMBER_SEEDS
            .iter()
            .chain(&CIRCLE_SEEDS)
            .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
            .collect();
        Net {
            writers,
            logs: (0..6).map(|_| TransparencyLog::new()).collect(),
            replica: Replica::new(tiers().writers().clone()),
        }
    }

    fn publish(&mut self, m: usize, tier: Tier, object: Vec<u8>) -> EntryId {
        let w = m + if tier == Tier::Circle { 3 } else { 0 };
        let entry = self.logs[w].append(cid(&object)).clone();
        let signed = self.writers[w].sign(&entry);
        let id = signed.id();
        self.replica
            .insert(signed, object)
            .expect("a writer of the set");
        id
    }

    fn draft(&mut self, m: usize, tag: &str) {
        self.publish(m, Tier::Circle, format!("draft-{m}-{tag}").into_bytes());
    }

    fn member(&mut self, object: MemberObject) {
        self.publish(object.member(), Tier::Public, object.encode());
    }

    fn commit(&mut self, m: usize, epoch: u64) {
        let (commit, _) = members()[m].beacon_commit(round(epoch), m);
        self.member(MemberObject::Commit(commit));
    }

    fn reveal(&mut self, m: usize, epoch: u64) {
        let (_, reveal) = members()[m].beacon_commit(round(epoch), m);
        self.member(MemberObject::Reveal { epoch, reveal });
    }

    fn sign(&mut self, m: usize, cut: &Cut) {
        self.member(sign_cut(&members()[m], m, NET, &consortium(), cut));
    }

    /// The next cut as its proposer builds it, signed by every member.
    fn cut(&mut self, prev: Option<&Cut>, epoch: u64, closes: bool) -> Cut {
        let cut = Cut::next(&self.replica, prev, epoch, closes);
        for m in 0..3 {
            self.sign(m, &cut);
        }
        cut
    }

    /// A second branch of member `m`'s circle feed, `n` entries long; its head's mark.
    fn fork(&mut self, m: usize, n: usize) -> Mark {
        let mut log = TransparencyLog::new();
        let mut head = [0; 32];
        for i in 0..n {
            let object = format!("fork-{m}-{i}").into_bytes();
            let signed = self.writers[m + 3].sign(&log.append(cid(&object)).clone());
            head = signed.entry.hash;
            self.replica
                .insert(signed, object)
                .expect("a writer of the set");
        }
        Mark {
            writer: key(m, Tier::Circle),
            len: n as u64,
            head,
        }
    }
}

/// Two epochs over five cuts: member 2 withholds epoch 0's reveal, so cuts 2 and 3 exclude it.
fn history() -> (Net, Vec<Cut>) {
    let mut net = Net::new();
    for m in 0..3 {
        net.draft(m, "0");
        net.commit(m, 0);
    }
    let c0 = net.cut(None, 0, true);
    net.reveal(0, 0);
    net.reveal(1, 0);
    net.draft(2, "1");
    let c1 = net.cut(Some(&c0), 1, false);
    net.draft(0, "2");
    net.draft(1, "2");
    for m in 0..3 {
        net.commit(m, 1);
    }
    let c2 = net.cut(Some(&c1), 1, false);
    let c3 = net.cut(Some(&c2), 1, true);
    for m in 0..3 {
        net.reveal(m, 1);
    }
    net.draft(2, "3");
    let c4 = net.cut(Some(&c3), 2, false);
    (net, vec![c0, c1, c2, c3, c4])
}

fn items() -> &'static [Item] {
    static ITEMS: OnceLock<Vec<Item>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        let (net, _) = history();
        let ids: Vec<EntryId> = net.replica.ids().collect();
        ids.iter()
            .map(|id| {
                let (e, o) = net.replica.get(id).expect("listed");
                (e.clone(), o.to_vec())
            })
            .collect()
    })
}

fn is_circle(writer: &[u8; 32]) -> bool {
    tiers().tier(writer) == Some(Tier::Circle)
}

/// One pull round: `to` asks `from`, which answers a reader with `access`.
fn pull(to: &mut Replica, from: &Replica, tiers: &Tiers, access: Access) -> usize {
    let served = tiers.serve(from, access);
    let want = to.want(&served.have_for(&to.summary()));
    if want.is_empty() {
        return 0;
    }
    let got = served.entries_for(&want, MAX_RESPONSE);
    let n = got.len();
    for (e, o) in got {
        to.insert(e, o).expect("a served entry is valid");
    }
    n
}

fn sync(to: &mut Replica, from: &Replica, tiers: &Tiers, access: Access) {
    while pull(to, from, tiers, access) > 0 {}
}

/// The member's path, as the ledger and `p2p::member::Collector` take it: every chain checked.
fn member_follow(replica: &Replica, tiers: &Tiers) -> (Vec<Cut>, Rounds) {
    let c = tiers.consortium();
    let (mut cuts, mut rounds) = (Vec::<Cut>::new(), Rounds::new());
    loop {
        let number = cuts.len() as u64;
        let ready = collect_with(replica, c, NET, number, &|cut| rounds.excluded(cut));
        let Collected::Ready(cut, ..) = ready else {
            break;
        };
        let order = added(replica, cuts.last(), &cut).expect("a member holds every entry");
        for id in order {
            let object = replica.get(&id).and_then(|(_, b)| MemberObject::decode(b));
            let own = object
                .filter(|o| c.keys().get(o.member()).map(|k| k.to_bytes()) == Some(id.writer));
            if let Some(o) = own {
                let _ = rounds.count(c, NET, &o, &cut, cuts.last());
            }
        }
        rounds.finish(c, NET, &cut, cuts.last());
        cuts.push(cut);
    }
    (cuts, rounds)
}

/// The public reader's path: cuts collected from the public tier and applied until one waits.
fn observer_follow(replica: &Replica, tiers: &Tiers) -> (Observer, Vec<(Cut, Observed)>) {
    let mut observer = Observer::new();
    let mut seen = Vec::new();
    loop {
        let number = observer.last().map_or(0, |l| l.number + 1);
        let ready = collect_with(replica, tiers.consortium(), NET, number, &|cut| {
            observer.excluded(cut)
        });
        let Collected::Ready(cut, view, sigs) = ready else {
            break;
        };
        let Ok(applied) = observer.apply_in_view(replica, tiers, &cut, view, &sigs) else {
            break;
        };
        seen.push((cut, applied));
    }
    (observer, seen)
}

fn public_reader(net: &Net, tiers: &Tiers) -> Replica {
    let mut reader = Replica::new(tiers.writers().clone());
    sync(&mut reader, &net.replica, tiers, Access::PUBLIC);
    reader
}

/// AT-NET-22: Summary, Have, Entries and a targeted Want give a public reader no circle entry.
#[test]
fn at_net_22_a_public_reader_gets_no_circle_entry() {
    let (net, cuts) = history();
    let t = tiers();
    let served = t.serve(&net.replica, Access::PUBLIC);
    let public: Vec<EntryId> = net
        .replica
        .ids()
        .filter(|id| !is_circle(&id.writer))
        .collect();
    let circle: Vec<EntryId> = net
        .replica
        .ids()
        .filter(|id| is_circle(&id.writer))
        .collect();
    assert!(!circle.is_empty());
    let summary = served.summary();
    assert_eq!(summary.0.len(), 3);
    assert!(summary.0.iter().all(|w| !is_circle(&w.writer)));
    assert_eq!(served.have_for(&Summary::default()), public);
    let claims = Summary(
        circle_keys()
            .iter()
            .map(|k| WriterSummary {
                writer: k.to_bytes(),
                count: 0,
                digest: [0; 32],
                feed_len: 0,
                head: [0; 32],
            })
            .collect(),
    );
    assert!(served
        .have_for(&claims)
        .iter()
        .all(|id| !is_circle(&id.writer)));
    assert!(served.entries_for(&circle, MAX_RESPONSE).is_empty());
    let last = cuts.last().expect("five cuts");
    let heads: Vec<EntryId> = last
        .marks
        .iter()
        .filter(|m| is_circle(&m.writer))
        .map(|m| EntryId {
            writer: m.writer,
            seq: m.len - 1,
            hash: m.head,
        })
        .collect();
    assert_eq!(
        heads.len(),
        3,
        "each member's circle feed stays marked in the cut"
    );
    assert!(heads.iter().all(|id| net.replica.contains(id)));
    assert!(served.entries_for(&heads, MAX_RESPONSE).is_empty());
    let mixed: Vec<EntryId> = heads.iter().chain(&public[..2]).copied().collect();
    let got: Vec<EntryId> = served
        .entries_for(&mixed, MAX_RESPONSE)
        .iter()
        .map(|(e, _)| e.id())
        .collect();
    assert_eq!(got, public[..2]);
    let reader = public_reader(&net, &t);
    assert_eq!(reader.ids().collect::<Vec<_>>(), public);
}

/// AT-NET-22: a reader with a verified binding converges on both tiers.
#[test]
fn at_net_22_a_bound_reader_converges_on_both_tiers() {
    let (net, _) = history();
    let t = tiers();
    let access = t.access(&TRANSPORT, Some(&t.bind(&members()[2], 2, TRANSPORT)));
    assert_eq!(access.member(), Some(2));
    assert_eq!(
        t.serve(&net.replica, access).summary(),
        net.replica.summary()
    );
    let mut reader = Replica::new(t.writers().clone());
    sync(&mut reader, &net.replica, &t, access);
    assert_eq!(reader.digest(), net.replica.digest());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// AT-NET-22: over any split, a bound reader reaches the union, a public one its public part.
    #[test]
    fn at_net_22_pull_rounds_respect_the_tiers(split in proptest::collection::vec(0u8..3, 64)) {
        let t = tiers();
        let access = t.access(&TRANSPORT, Some(&t.bind(&members()[0], 0, TRANSPORT)));
        let (mut a, mut b) = (Replica::new(t.writers().clone()), Replica::new(t.writers().clone()));
        for ((e, o), side) in items().iter().zip(&split) {
            if *side != 1 {
                a.insert(e.clone(), o.clone()).unwrap();
            }
            if *side != 0 {
                b.insert(e.clone(), o.clone()).unwrap();
            }
        }
        let (mut bound, mut public) = (Replica::new(t.writers().clone()), Replica::new(t.writers().clone()));
        while pull(&mut bound, &a, &t, access) + pull(&mut bound, &b, &t, access) > 0 {}
        while pull(&mut public, &a, &t, Access::PUBLIC) + pull(&mut public, &b, &t, Access::PUBLIC) > 0 {}
        let union: Vec<EntryId> = items().iter().take(split.len()).map(|(e, _)| e.id()).collect();
        let mut union = union;
        union.sort();
        prop_assert_eq!(bound.ids().collect::<Vec<_>>(), union.clone());
        let public_part: Vec<EntryId> = union.into_iter().filter(|id| !is_circle(&id.writer)).collect();
        prop_assert_eq!(public.ids().collect::<Vec<_>>(), public_part);
    }
}

/// AT-NET-22: observer and member agree on cuts, beacons and exclusions; circle marks are attested.
#[test]
fn at_net_22_an_observer_agrees_with_a_member() {
    let (net, cuts) = history();
    let t = tiers();
    let reader = public_reader(&net, &t);
    assert!(reader.ids().all(|id| !is_circle(&id.writer)));
    let (member_cuts, rounds) = member_follow(&net.replica, &t);
    let (observer, seen) = observer_follow(&reader, &t);
    assert_eq!(member_cuts, cuts);
    let digests: Vec<[u8; 32]> = seen.iter().map(|(c, _)| c.digest()).collect();
    assert_eq!(digests, cuts.iter().map(Cut::digest).collect::<Vec<_>>());
    for epoch in 0..2 {
        assert!(rounds.outcome(epoch).and_then(|o| o.value()).is_some());
        assert_eq!(observer.beacon(epoch), rounds.outcome(epoch));
    }
    assert_eq!(rounds.outcome(0).map(|o| o.withheld()), Some(&[2][..]));
    for cut in &cuts {
        assert_eq!(observer.excluded(cut), rounds.excluded(cut));
    }
    assert_eq!(rounds.excluded(&cuts[2]), vec![2]);
    for (k, (cut, o)) in seen.iter().enumerate() {
        let prev = k.checked_sub(1).map(|i| &cuts[i]);
        let full = added(&net.replica, prev, cut).expect("a member holds every entry");
        let public: Vec<EntryId> = full
            .into_iter()
            .filter(|id| !is_circle(&id.writer))
            .collect();
        assert_eq!(o.public, public);
        let circle: Vec<Mark> = cut
            .marks
            .iter()
            .filter(|m| is_circle(&m.writer))
            .copied()
            .collect();
        assert_eq!(o.attested, circle);
    }
}

/// AT-NET-22: only a member's own binding for this network, member set, index and key grants it.
#[test]
fn at_net_22_a_binding_grants_the_circle_only_as_signed() {
    let t = tiers();
    let ms = members();
    let good = t.bind(&ms[1], 1, TRANSPORT);
    assert_eq!(t.access(&TRANSPORT, Some(&good)).member(), Some(1));
    assert_eq!(t.access(&TRANSPORT, None), Access::PUBLIC);
    assert_eq!(t.access(&[78; 32], Some(&good)), Access::PUBLIC);
    let pair: Vec<VerifyingKey> = member_keys()[..2].to_vec();
    let smaller = Tiers::new(NET, Consortium::new(pair, 2), &circle_keys()[..2]).unwrap();
    let elsewhere = Tiers::new([23; 32], consortium(), &circle_keys()).unwrap();
    let mut bad = vec![
        TransportBinding { member: 0, ..good },
        TransportBinding { member: 2, ..good },
        TransportBinding { member: 3, ..good },
        TransportBinding {
            transport: [78; 32],
            ..good
        },
        t.bind(&ms[0], 1, TRANSPORT),
        t.bind(&Member::from_seed([99; 32]), 1, TRANSPORT),
        t.bind(&Member::from_seed([CIRCLE_SEEDS[1]; 32]), 1, TRANSPORT),
        smaller.bind(&ms[1], 1, TRANSPORT),
        elsewhere.bind(&ms[1], 1, TRANSPORT),
    ];
    let bytes = good.signature.to_bytes();
    for i in 0..64 {
        let mut flipped = bytes;
        flipped[i] ^= 1;
        bad.push(TransportBinding {
            signature: Signature::from_bytes(&flipped),
            ..good
        });
    }
    for b in &bad {
        assert_eq!(t.access(&TRANSPORT, Some(b)), Access::PUBLIC, "{b:?}");
        assert_eq!(t.access(&b.transport, Some(b)).member(), None, "{b:?}");
    }
}

/// AT-NET-22: a tiered set needs exactly one circle key per member, distinct and no member key.
#[test]
fn at_net_22_ambiguous_tiers_are_refused() {
    let (mk, ck) = (member_keys(), circle_keys());
    let other = FeedWriter::from_seed(NET, [43; 32]).public();
    let refused = |circle: &[VerifyingKey]| Tiers::new(NET, consortium(), circle).err();
    assert_eq!(refused(&ck[..2]), Some(TierError::Count));
    assert_eq!(refused(&[]), Some(TierError::Count));
    assert_eq!(
        refused(&[ck[0], ck[1], ck[2], other]),
        Some(TierError::Count)
    );
    let repeated = |k: VerifyingKey| Some(TierError::Repeated { key: k.to_bytes() });
    assert_eq!(refused(&[mk[0], ck[1], ck[2]]), repeated(mk[0]));
    assert_eq!(refused(&[ck[0], mk[0], ck[2]]), repeated(mk[0]));
    assert_eq!(refused(&[ck[0], ck[0], ck[2]]), repeated(ck[0]));
    assert_eq!(refused(&[ck[0], ck[1], ck[0]]), repeated(ck[0]));
    let t = tiers();
    for m in 0..3 {
        assert_eq!(t.place(&key(m, Tier::Public)), Some((m, Tier::Public)));
        assert_eq!(t.place(&key(m, Tier::Circle)), Some((m, Tier::Circle)));
        assert!(t.writers().contains(&mk[m]) && t.writers().contains(&ck[m]));
    }
    assert_eq!(t.place(&other.to_bytes()), None);
    assert!(!t.writers().contains(&other));
    let bound = t.access(&TRANSPORT, Some(&t.bind(&members()[0], 0, TRANSPORT)));
    assert!(t.readable(&ck[0].to_bytes(), bound) && !t.readable(&ck[0].to_bytes(), Access::PUBLIC));
    assert!(!t.readable(&other.to_bytes(), bound));
}

/// AT-NET-22: a member's objects on its circle feed count for nothing.
#[test]
fn at_net_22_member_objects_on_a_circle_feed_do_not_count() {
    let mut net = Net::new();
    let t = tiers();
    let c = t.consortium();
    for m in 0..3 {
        let (commit, _) = members()[m].beacon_commit(round(0), m);
        net.publish(m, Tier::Circle, MemberObject::Commit(commit).encode());
    }
    let c0 = Cut::next(&net.replica, None, 0, true);
    for m in 0..3 {
        let signature = sign_cut(&members()[m], m, NET, c, &c0);
        net.publish(m, Tier::Circle, signature.encode());
    }
    assert_eq!(member_objects(&net.replica, c).count(), 0);
    assert_eq!(collect(&net.replica, c, NET, 0), Collected::Pending);
    for m in 0..3 {
        net.sign(m, &c0);
        net.reveal(m, 0);
    }
    let c1 = net.cut(Some(&c0), 1, false);
    let (cuts, rounds) = member_follow(&net.replica, &t);
    assert_eq!(cuts, vec![c0, c1]);
    let outcome = rounds
        .outcome(0)
        .expect("the round ends after the closing cut");
    assert_eq!(outcome.value(), None);
    assert!(outcome.revealed().is_empty());
}

/// AT-NET-22: the observer refuses malformed or regressing cuts, unknown writers, missing data.
#[test]
fn at_net_22_the_observer_keeps_the_checks_it_can_make() {
    let (net, cuts) = history();
    let t = tiers();
    let reader = public_reader(&net, &t);
    let (c3, c4) = (&cuts[3], &cuts[4]);
    assert!(observed(&reader, &t, Some(c3), c4).is_ok());
    let circle = key(0, Tier::Circle);
    let old = *c3
        .marks
        .iter()
        .find(|m| m.writer == circle)
        .expect("marked");
    assert!(old.len >= 2);
    let with = |f: &dyn Fn(&mut Cut)| {
        let mut c = c4.clone();
        f(&mut c);
        observed(&reader, &t, Some(c3), &c)
    };
    let set = |c: &mut Cut, mark: Mark| {
        let at = c
            .marks
            .iter()
            .position(|m| m.writer == mark.writer)
            .unwrap();
        c.marks[at] = mark;
    };
    assert_eq!(with(&|c| c.marks.reverse()), Err(CutError::Malformed));
    assert_eq!(with(&|c| c.marks[0].len = 0), Err(CutError::Malformed));
    assert_eq!(
        with(&|c| c.number = 6),
        Err(CutError::NotNext { expected: 4 })
    );
    assert_eq!(with(&|c| c.epoch = 1), Err(CutError::Epoch));
    let retracts = Err(CutError::Retracts { writer: circle });
    assert_eq!(with(&|c| c.marks.retain(|m| m.writer != circle)), retracts);
    assert_eq!(
        with(&|c| set(
            c,
            Mark {
                len: old.len - 1,
                ..old
            }
        )),
        retracts
    );
    assert_eq!(
        with(&|c| set(
            c,
            Mark {
                head: [0xAB; 32],
                ..old
            }
        )),
        retracts
    );
    assert_eq!(
        with(&|c| set(c, old)).map(|o| o.attested.contains(&old)),
        Ok(true)
    );
    let stranger = FeedWriter::from_seed(NET, [99; 32]).public().to_bytes();
    let unknown = with(&|c| {
        c.marks.push(Mark {
            writer: stranger,
            len: 1,
            head: [1; 32],
        });
        c.marks.sort_by_key(|m| m.writer);
    });
    assert_eq!(unknown, Err(CutError::Unknown { writer: stranger }));
    let public = key(0, Tier::Public);
    let mark = *c4
        .marks
        .iter()
        .find(|m| m.writer == public)
        .expect("marked");
    let missing = EntryId {
        writer: public,
        seq: mark.len - 1,
        hash: mark.head,
    };
    let mut partial = Replica::new(t.writers().clone());
    for id in reader.ids().filter(|id| *id != missing) {
        let (e, o) = reader.get(&id).unwrap();
        partial.insert(e.clone(), o.to_vec()).unwrap();
    }
    assert_eq!(
        observed(&partial, &t, Some(c3), c4),
        Err(CutError::Missing { writer: public })
    );
    let (mut observer, _) = observer_follow(&partial, &t);
    assert_eq!(
        observer.last(),
        Some(c3),
        "a missing public entry stops the observer"
    );
    let (e, o) = reader.get(&missing).unwrap();
    partial.insert(e.clone(), o.to_vec()).unwrap();
    let Collected::Ready(cut, view, sigs) = collect(&partial, t.consortium(), NET, 4) else {
        panic!("cut 4 is signed");
    };
    assert!(observer
        .apply_in_view(&partial, &t, &cut, view, &sigs)
        .is_ok());
}

/// AT-NET-22: the member's path refuses a circle chain the observer can only take as attested.
#[test]
fn at_net_22_only_the_members_check_the_circle_chains() {
    let (mut net, cuts) = history();
    let t = tiers();
    let reader = public_reader(&net, &t);
    let c4 = &cuts[4];
    let circle = key(0, Tier::Circle);
    let old = *c4
        .marks
        .iter()
        .find(|m| m.writer == circle)
        .expect("marked");
    let fork = net.fork(0, old.len as usize + 1);
    let unheld = Mark {
        len: old.len + 1,
        head: [0xCD; 32],
        ..old
    };
    for (mark, refusal) in [
        (fork, CutError::Retracts { writer: circle }),
        (unheld, CutError::Missing { writer: circle }),
    ] {
        let mut next = Cut::next(&net.replica, Some(c4), 2, false);
        let at = next.marks.iter().position(|m| m.writer == circle).unwrap();
        next.marks[at] = mark;
        assert_eq!(added(&net.replica, Some(c4), &next), Err(refusal));
        let seen = observed(&reader, &t, Some(c4), &next).expect("attested, not checked");
        assert!(seen.attested.contains(&mark));
    }
}

/// AT-NET-22: the observer counts no withholder's signature and stays put on an unsigned cut.
#[test]
fn at_net_22_the_observer_checks_the_signatures() {
    let (net, cuts) = history();
    let t = tiers();
    let reader = public_reader(&net, &t);
    let mut observer = Observer::new();
    for (k, cut) in cuts[..2].iter().enumerate() {
        let Collected::Ready(_, view, sigs) = collect(&reader, t.consortium(), NET, k as u64)
        else {
            panic!("cut {k} is signed");
        };
        observer
            .apply_in_view(&reader, &t, cut, view, &sigs)
            .unwrap();
    }
    let Collected::Ready(_, view, sigs) = collect(&reader, t.consortium(), NET, 2) else {
        panic!("cut 2 is signed");
    };
    let only = |who: &[usize]| -> Vec<(usize, Signature)> {
        sigs.iter()
            .filter(|(m, _)| who.contains(m))
            .copied()
            .collect()
    };
    assert_eq!(observer.excluded(&cuts[2]), vec![2]);
    for who in [&[0][..], &[0, 2][..], &[1, 2][..]] {
        let refused = observer.apply_in_view(&reader, &t, &cuts[2], view, &only(who));
        assert_eq!(refused, Err(ObserveError::Unsigned), "signed by {who:?}");
        assert_eq!(observer.last(), Some(&cuts[1]));
    }
    assert!(observer
        .apply_in_view(&reader, &t, &cuts[2], view, &only(&[0, 1]))
        .is_ok());
    assert_eq!(observer.last(), Some(&cuts[2]));
}
