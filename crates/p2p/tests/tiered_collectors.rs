//! A tiered network's collectors and bind messages, in process (`docs/04` §Transport, `docs/10`
//! T77): the public reader's cuts and conflicts, fixed contexts, one encoding per message.

use libp2p::identity::Keypair;
use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{sign_cut, Cut, MemberObject};
use network::log::TransparencyLog;
use network::observer::ObserveError;
use network::replica::{FeedWriter, Replica};
use network::tier::{Tier, Tiers};
use p2p::member::{Collector, OtherContext, PublicCollector};
use p2p::tiered::{transport_key, Bind};
use proptest::prelude::*;

const NET: [u8; 32] = [31; 32];
const MEMBER_SEEDS: [u8; 3] = [50, 51, 52];
const CIRCLE_SEEDS: [u8; 3] = [53, 54, 55];

fn members() -> Vec<Member> {
    MEMBER_SEEDS
        .iter()
        .map(|s| Member::from_seed([*s; 32]))
        .collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

fn tiers_on(net: [u8; 32], circle_seeds: [u8; 3]) -> Tiers {
    let circle = circle_seeds.map(|s| FeedWriter::from_seed(net, [s; 32]).public());
    Tiers::new(net, consortium(), &circle).expect("a valid tiered set")
}

fn tiers() -> Tiers {
    tiers_on(NET, CIRCLE_SEEDS)
}

/// The members' public feeds (0–2) and circle feeds (3–5) in one full replica.
struct Net {
    writers: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    replica: Replica,
}

impl Net {
    fn new() -> Net {
        Net {
            writers: MEMBER_SEEDS
                .iter()
                .chain(&CIRCLE_SEEDS)
                .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
                .collect(),
            logs: (0..6).map(|_| TransparencyLog::new()).collect(),
            replica: Replica::new(tiers().writers().clone()),
        }
    }

    fn publish(&mut self, w: usize, object: Vec<u8>) {
        let entry = self.logs[w].append(cid(&object)).clone();
        let signed = self.writers[w].sign(&entry);
        self.replica.insert(signed, object).unwrap();
    }

    fn member(&mut self, object: MemberObject) {
        self.publish(object.member(), object.encode());
    }

    fn draft(&mut self, m: usize, tag: &str) {
        self.publish(m + 3, format!("draft-{m}-{tag}").into_bytes());
    }

    fn commit(&mut self, m: usize, epoch: u64) {
        let round = RoundId {
            network_id: NET,
            member_set_hash: consortium().member_set_hash(),
            epoch,
        };
        let (commit, reveal) = members()[m].beacon_commit(round, m);
        self.member(MemberObject::Commit(commit));
        let _ = reveal;
    }

    fn sign(&mut self, cut: &Cut, by: &[usize]) {
        for &m in by {
            self.member(sign_cut(&members()[m], m, NET, &consortium(), cut));
        }
    }

    /// What a reader without circle access holds of this replica.
    fn public(&self) -> Replica {
        let t = tiers();
        let mut out = Replica::new(t.writers().clone());
        for id in self.replica.ids() {
            if t.tier(&id.writer) == Some(Tier::Public) {
                let (e, o) = self.replica.get(&id).unwrap();
                out.insert(e.clone(), o.to_vec()).unwrap();
            }
        }
        out
    }
}

/// Three cuts signed by members 0 and 1, circle drafts between them.
fn three_cuts(net: &mut Net) -> Vec<Cut> {
    let mut cuts: Vec<Cut> = Vec::new();
    for (k, epoch) in [0, 0, 1].into_iter().enumerate() {
        net.draft(k % 3, &k.to_string());
        net.commit(k % 3, epoch);
        let cut = Cut::next(&net.replica, cuts.last(), epoch, false);
        net.sign(&cut, &[0, 1]);
        cuts.push(cut);
    }
    cuts
}

/// AT-NET-23: a public collector reaches a member's cuts and beacons without any circle entry.
#[test]
fn at_net_23_the_public_collector_follows_the_members() {
    let mut net = Net::new();
    let cuts = three_cuts(&mut net);
    let t = tiers();
    let public = net.public();
    assert!(public.len() < net.replica.len());
    let mut member = Collector::new(&consortium(), NET);
    let mut reader = PublicCollector::new(&t);
    assert_eq!(
        member.update(&net.replica, &consortium(), NET),
        Ok(&cuts[..])
    );
    assert_eq!(reader.update(&public, &t), Ok(&cuts[..]));
    assert_eq!(reader.beacons(), member.beacons());
    assert!(!reader.conflict());
}

/// AT-NET-23: a conflict found after its number was followed keeps the cuts before it.
#[test]
fn at_net_23_a_later_conflict_stays_detectable() {
    let mut net = Net::new();
    let cuts = three_cuts(&mut net);
    let t = tiers();
    let mut member = Collector::new(&consortium(), NET);
    let mut reader = PublicCollector::new(&t);
    assert_eq!(reader.update(&net.public(), &t).map(<[Cut]>::len), Ok(3));
    assert_eq!(
        member
            .update(&net.replica, &consortium(), NET)
            .map(<[Cut]>::len),
        Ok(3)
    );
    net.draft(2, "late");
    let other = Cut::next(&net.replica, Some(&cuts[0]), 0, false);
    assert_ne!(other, cuts[1]);
    net.sign(&other, &[1, 2]);
    assert_eq!(reader.update(&net.public(), &t), Ok(&cuts[..1]));
    assert!(reader.conflict());
    assert_eq!(
        member.update(&net.replica, &consortium(), NET),
        Ok(&cuts[..1])
    );
    assert!(member.conflict());
}

/// AT-NET-23: a collector fixed to one context refuses another before changing anything.
#[test]
fn at_net_23_collectors_keep_their_context() {
    let mut net = Net::new();
    let cuts = three_cuts(&mut net);
    let t = tiers();
    let other_net = tiers_on([32; 32], CIRCLE_SEEDS);
    let other_circle = tiers_on(NET, [56, 57, 58]);
    let mut reader = PublicCollector::new(&t);
    let public = net.public();
    for other in [&other_net, &other_circle] {
        assert_eq!(
            reader.update(&public, other).err(),
            Some(ObserveError::Context)
        );
        let foreign = Replica::new(other.writers().clone());
        assert_eq!(
            reader.update(&foreign, &t).err(),
            Some(ObserveError::Replica)
        );
        assert!(reader.cuts().is_empty());
    }
    assert_eq!(reader.update(&public, &t), Ok(&cuts[..]));
    let smaller = Consortium::new(members()[..2].iter().map(|m| m.public()).collect(), 2);
    let lower = Consortium::new(members().iter().map(|m| m.public()).collect(), 1);
    let mut lazy = Collector::default();
    assert_eq!(lazy.update(&net.replica, &consortium(), NET), Ok(&cuts[..]));
    for collector in [&mut Collector::new(&consortium(), NET), &mut lazy] {
        let before = collector.cuts().to_vec();
        assert_eq!(
            collector.update(&net.replica, &consortium(), [32; 32]),
            Err(OtherContext)
        );
        assert_eq!(
            collector.update(&net.replica, &smaller, NET),
            Err(OtherContext)
        );
        assert_eq!(
            collector.update(&net.replica, &lower, NET),
            Err(OtherContext)
        );
        assert_eq!(collector.cuts(), &before[..]);
        assert_eq!(
            collector.update(&net.replica, &consortium(), NET),
            Ok(&cuts[..])
        );
    }
}

/// AT-NET-23: a peer id yields the ed25519 key it was made from, and nothing for other bytes.
#[test]
fn at_net_23_the_transport_key_is_the_peer_ids() {
    let key = Keypair::ed25519_from_bytes([9; 32]).unwrap();
    let bytes = key.public().try_into_ed25519().unwrap().to_bytes();
    assert_eq!(transport_key(&key.public().to_peer_id()), Some(bytes));
    let hashed =
        libp2p::PeerId::from_multihash(libp2p::multihash::Multihash::wrap(0x12, &[7; 32]).unwrap())
            .unwrap();
    assert_eq!(transport_key(&hashed), None);
}

proptest! {
    /// AT-NET-23: bind messages round-trip, and other bytes decode as one only if re-encoded.
    #[test]
    fn at_net_23_bind_messages_have_one_encoding(bytes in proptest::collection::vec(any::<u8>(), 0..140), granted: bool) {
        let t = tiers();
        let binding = t.bind(&members()[1], 1, [5; 32]);
        for bind in [
            Bind::Hello(None),
            Bind::Hello(Some(binding)),
            Bind::Reply { granted, binding: None },
            Bind::Reply { granted, binding: Some(binding) },
        ] {
            let encoded = bind.encode();
            prop_assert_eq!(Bind::decode(&encoded), Some(bind));
            for cut in 0..encoded.len() {
                prop_assert_eq!(Bind::decode(&encoded[..cut]), None);
            }
        }
        if let Some(bind) = Bind::decode(&bytes) {
            prop_assert_eq!(bind.encode(), bytes.clone());
        }
        if let Some(b) = network::tier::TransportBinding::decode(&bytes) {
            prop_assert_eq!(b.encode(), bytes);
        }
    }
}
