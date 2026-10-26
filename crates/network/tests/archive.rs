use ed25519_dalek::Signature;
use network::archive::{
    BuiltArchive, Context, Error, Layout, Manifest, Segment, ShardStatus, SignedManifest, Snapshot,
    MAX_ENCODED_SHARDS, MAX_MANIFEST, MAX_SEGMENT,
};
use network::cid::cid;
use network::codec::Writer;
use network::consortium::{Consortium, Member};
use network::cut::{Cut, Mark};
use network::erasure::{self, shard_hash};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica};
use network::state::{Origin, RootStatement, Shape, ShapeEntry};
use network::tier::Tiers;
use proptest::prelude::*;
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::collections::BTreeSet;

const NETWORK: [u8; 32] = [81; 32];
const LINEAGE: [u8; 32] = [82; 32];
const PUBLIC: &[u8] = b"public positive control: archive this";
const PRIVATE: &[u8] = b"private sentinel: never archive this";

struct Fixture {
    members: Vec<Member>,
    tiers: Tiers,
    replica: Replica,
    previous: Option<Cut>,
    target: Cut,
}

impl Fixture {
    fn new(delta: bool, closes: bool) -> Self {
        let members: Vec<_> = (1..=4).map(|n| Member::from_seed([n; 32])).collect();
        let public = FeedWriter::from_seed(NETWORK, [1; 32]);
        let circles: Vec<_> = (11..=14)
            .map(|n| FeedWriter::from_seed(NETWORK, [n; 32]))
            .collect();
        let consortium = Consortium::new(members.iter().map(Member::public).collect(), 3);
        let tiers = Tiers::new(
            NETWORK,
            consortium,
            &circles.iter().map(FeedWriter::public).collect::<Vec<_>>(),
        )
        .unwrap();
        let mut replica = Replica::new(tiers.writers().clone());
        let mut log = TransparencyLog::new();
        let first = public.sign(log.append(cid(b"trusted prefix is absent from replica")));
        let previous = delta.then(|| Cut {
            number: 7,
            epoch: 2,
            closes: false,
            marks: vec![Mark {
                writer: first.writer,
                len: 1,
                head: first.entry.hash,
            }],
        });
        let entry = if delta {
            public.sign(log.append(cid(PUBLIC)))
        } else {
            let mut fresh = TransparencyLog::new();
            public.sign(fresh.append(cid(PUBLIC)))
        };
        replica.insert(entry.clone(), PUBLIC.to_vec()).unwrap();
        let mut private_log = TransparencyLog::new();
        let private = circles[0].sign(private_log.append(cid(PRIVATE)));
        replica.insert(private.clone(), PRIVATE.to_vec()).unwrap();
        let mut marks = vec![
            Mark {
                writer: entry.writer,
                len: entry.entry.seq + 1,
                head: entry.entry.hash,
            },
            Mark {
                writer: private.writer,
                len: 1,
                head: private.entry.hash,
            },
        ];
        marks.sort_by_key(|m| m.writer);
        let target = Cut {
            number: if delta { 8 } else { 0 },
            epoch: if delta { 2 } else { 0 },
            closes,
            marks,
        };
        Self {
            members,
            tiers,
            replica,
            previous,
            target,
        }
    }

    fn context(&self) -> Context<'_> {
        Context {
            tiers: &self.tiers,
            lineage: LINEAGE,
            previous: self.previous.as_ref(),
            target: &self.target,
            view: 2,
            excluded: &[],
        }
    }

    fn snapshot(&self) -> Snapshot {
        let shape = Shape::new(vec![
            ShapeEntry {
                position: [1; 32],
                leaf_hash: [31; 32],
                origin: Origin::Live,
            },
            ShapeEntry {
                position: [2; 32],
                leaf_hash: [32; 32],
                origin: Origin::Frozen {
                    network_id: [91; 32],
                    epoch: 1,
                },
            },
        ])
        .unwrap();
        let statement = RootStatement {
            network_id: NETWORK,
            lineage: LINEAGE,
            member_set_hash: self.tiers.consortium().member_set_hash(),
            epoch: self.target.epoch,
            cut_number: self.target.number,
            cut_digest: self.target.digest(),
            inputs_root: [33; 32],
            state_root: shape.root(),
            shape_digest: shape.digest(),
        };
        Snapshot { statement, shape }
    }

    fn segment(&self) -> Segment {
        let ctx = self.context();
        let cp =
            self.target
                .checkpoint_in(ctx.view, NETWORK, self.tiers.consortium().member_set_hash());
        let signatures = self
            .members
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect();
        Segment::build(
            &ctx,
            &self.replica,
            signatures,
            self.target.closes.then(|| self.snapshot()),
        )
        .unwrap()
    }

    fn sign(&self, manifest: Manifest) -> SignedManifest {
        let cp = manifest.checkpoint();
        let signatures = self
            .members
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect();
        SignedManifest {
            manifest,
            signatures,
        }
    }
}

fn archive(f: &Fixture, k: usize, m: usize) -> BuiltArchive {
    BuiltArchive::build(
        &f.context(),
        &f.segment(),
        Layout {
            data_shards: k,
            parity_shards: m,
        },
    )
    .unwrap()
}

fn raw_manifest(
    template: &Manifest,
    bytes: &[u8],
    orig_len: usize,
    hashes: &[[u8; 32]],
) -> Manifest {
    let layout = template.layout();
    let mut w = Writer::new();
    w.u8(1)
        .fixed(&template.network_id())
        .fixed(&template.lineage())
        .fixed(&template.member_set_hash());
    match template.previous_digest() {
        None => {
            w.u8(0);
        }
        Some(d) => {
            w.u8(1).fixed(&d);
        }
    }
    w.u64(template.target_number())
        .fixed(&template.target_digest())
        .u64(template.view())
        .fixed(&cid(bytes).0)
        .u64(orig_len as u64)
        .u64(layout.data_shards as u64)
        .u64(layout.parity_shards as u64)
        .u64(hashes.len() as u64);
    for h in hashes {
        w.fixed(h);
    }
    Manifest::decode(&w.finish()).unwrap()
}

fn forge(f: &Fixture, template: &Manifest, bytes: &[u8]) -> (SignedManifest, Vec<Option<Vec<u8>>>) {
    let layout = template.layout();
    let encoded = erasure::encode(bytes, layout.data_shards, layout.parity_shards);
    let manifest = raw_manifest(template, bytes, bytes.len(), &encoded.manifest);
    (
        f.sign(manifest),
        encoded.shards.into_iter().map(Some).collect(),
    )
}

// AT-NET-26: public deltas omit circle payloads and recover without any trusted prefix replica.
#[test]
fn public_delta_privacy_and_nonzero_previous() {
    let f = Fixture::new(true, false);
    let segment = f.segment();
    let bytes = segment.encode().unwrap();
    assert!(bytes.windows(PUBLIC.len()).any(|w| w == PUBLIC));
    assert!(!bytes.windows(PRIVATE.len()).any(|w| w == PRIVATE));
    assert_eq!(segment.entries.len(), 1);
    assert_eq!(segment.entries[0].0.entry.seq, 1);
    let built = archive(&f, 3, 2);
    let verified = f.sign(built.manifest).verify(&f.context()).unwrap();
    let available = built
        .shards
        .into_iter()
        .enumerate()
        .map(|(i, s)| (i < 3).then_some(s))
        .collect::<Vec<_>>();
    assert_eq!(
        verified.recover(&f.context(), &available).unwrap().segment,
        segment
    );
}

// AT-NET-26: live and frozen commitments survive every k-of-n subset for both layouts.
#[test]
fn every_subset_recovers_closing_snapshot() {
    let f = Fixture::new(true, true);
    for (k, m) in [(3, 2), (2, 3)] {
        let built = archive(&f, k, m);
        let signed = f.sign(built.manifest);
        let verified = signed.verify(&f.context()).unwrap();
        for mask in 0u32..(1 << (k + m)) {
            if mask.count_ones() as usize != k {
                continue;
            }
            let available: Vec<_> = built
                .shards
                .iter()
                .enumerate()
                .map(|(i, s)| (mask & (1 << i) != 0).then(|| s.clone()))
                .collect();
            let recovered = verified.recover(&f.context(), &available).unwrap();
            assert_eq!(recovered.segment, f.segment());
            assert_eq!(recovered.repairs.len(), m);
            for (i, bytes) in recovered.repairs {
                assert_eq!(bytes, built.shards[i]);
            }
        }
    }
}

// AT-NET-26: contexts, exclusions, duplicate signers and both signature thresholds are enforced.
#[test]
fn trust_context_and_thresholds() {
    let f = Fixture::new(false, false);
    let built = archive(&f, 3, 2);
    let signed = f.sign(built.manifest);
    let verified = signed.verify(&f.context()).unwrap();
    let shards: Vec<_> = built.shards.into_iter().map(Some).collect();
    let roster: Vec<_> = (0..8).map(|n| [n; 32]).collect();
    let ctx = f.context();
    for wrong in [
        Context {
            lineage: [9; 32],
            ..ctx
        },
        Context { view: 3, ..ctx },
        Context {
            excluded: &[0],
            ..ctx
        },
    ] {
        assert!(signed.verify(&wrong).is_err());
        assert!(verified.audit(&wrong, &shards).is_err());
        assert!(verified.recover(&wrong, &shards).is_err());
        assert!(verified.placement(&wrong, &roster).is_err());
    }
    for different_network in [false, true] {
        let others: Vec<_> = [1, 2, 3, 5]
            .into_iter()
            .map(|n| Member::from_seed([n; 32]))
            .collect();
        let consortium = if different_network {
            Consortium::new(f.members.iter().map(Member::public).collect(), 3)
        } else {
            Consortium::new(others.iter().map(Member::public).collect(), 3)
        };
        let circles: Vec<_> = (11..=14)
            .map(|n| FeedWriter::from_seed(NETWORK, [n; 32]).public())
            .collect();
        let tiers = Tiers::new(
            if different_network { [90; 32] } else { NETWORK },
            consortium,
            &circles,
        )
        .unwrap();
        let wrong = Context {
            tiers: &tiers,
            ..ctx
        };
        assert!(matches!(signed.verify(&wrong), Err(Error::Context)));
        assert!(matches!(
            verified.audit(&wrong, &shards),
            Err(Error::Context)
        ));
        assert!(matches!(
            verified.recover(&wrong, &shards),
            Err(Error::Context)
        ));
        assert!(matches!(
            verified.placement(&wrong, &roster),
            Err(Error::Context)
        ));
    }
    let mut target = f.target.clone();
    target.epoch += 1;
    assert!(signed
        .verify(&Context {
            target: &target,
            ..ctx
        })
        .is_err());
    let mut short = signed.clone();
    short.signatures.pop();
    assert!(short.verify(&ctx).is_err());
    let mut duplicate = signed.clone();
    duplicate.signatures[1] = duplicate.signatures[0];
    assert!(duplicate.verify(&ctx).is_err());
    let mut bad = signed.clone();
    bad.signatures[0].1 = Signature::from_bytes(&[0; 64]);
    assert!(bad.verify(&ctx).is_err());
    let mut segment = f.segment();
    segment.signatures.pop();
    assert_eq!(segment.verify(&ctx), Err(Error::Signatures));
    let mut segment = f.segment();
    segment.signatures[1] = segment.signatures[0];
    assert!(segment.encode().is_err());
}

// AT-NET-26: audit separates loss and corruption, recovery repairs only bad slots, and k-1 fails.
#[test]
fn audit_and_repair() {
    let f = Fixture::new(false, false);
    let built = archive(&f, 3, 2);
    let verified = f.sign(built.manifest).verify(&f.context()).unwrap();
    let mut available: Vec<_> = built.shards.iter().cloned().map(Some).collect();
    available[0] = None;
    available[4].as_mut().unwrap()[0] ^= 1;
    assert_eq!(
        verified.audit(&f.context(), &available).unwrap(),
        vec![
            ShardStatus::Missing,
            ShardStatus::Valid,
            ShardStatus::Valid,
            ShardStatus::Valid,
            ShardStatus::Corrupt
        ]
    );
    let recovered = verified.recover(&f.context(), &available).unwrap();
    assert_eq!(
        recovered.repairs,
        vec![(0, built.shards[0].clone()), (4, built.shards[4].clone())]
    );
    available[1] = None;
    assert!(matches!(
        verified.recover(&f.context(), &available),
        Err(Error::TooFewShards)
    ));
    available[1] = Some(vec![0; built.shards[1].len() + 1]);
    assert_eq!(
        verified.audit(&f.context(), &available).unwrap()[1],
        ShardStatus::Corrupt
    );
    assert!(verified.audit(&f.context(), &available[..4]).is_err());
}

// AT-NET-26: a threshold manifest and authentic shards cannot bypass semantic segment validation.
#[test]
fn signed_semantically_invalid_segments_still_fail_recovery() {
    let f = Fixture::new(false, false);
    let built = archive(&f, 3, 2);
    for kind in 0..3 {
        let mut segment = f.segment();
        match kind {
            0 => {
                segment.signatures.pop();
            }
            1 => {
                segment.entries.clear();
            }
            _ => {
                let private = f
                    .replica
                    .ids()
                    .find(|id| f.tiers.tier(&id.writer) == Some(network::tier::Tier::Circle))
                    .unwrap();
                let (e, o) = f.replica.get(&private).unwrap();
                segment.entries.push((e.clone(), o.to_vec()));
                segment.entries.sort_by_key(|(e, _)| e.id());
            }
        }
        let (signed, shards) = forge(&f, &built.manifest, &segment.encode().unwrap());
        let verified = signed.verify(&f.context()).unwrap();
        assert!(verified
            .audit(&f.context(), &shards)
            .unwrap()
            .iter()
            .all(|s| *s == ShardStatus::Valid));
        assert!(
            matches!(verified.recover(&f.context(), &shards), Err(error) if error == if kind == 0 { Error::Signatures } else { Error::Delta })
        );
    }
}

// AT-NET-26: absent parity commitments, nonzero padding, and altered length cannot be hidden.
#[test]
fn parity_padding_and_length_commitments() {
    let f = Fixture::new(false, false);
    let built = archive(&f, 3, 2);
    let bytes = f.segment().encode().unwrap();
    let mut hashes = built.manifest.shard_hashes().to_vec();
    hashes[4] = [7; 32];
    let inconsistent = raw_manifest(&built.manifest, &bytes, bytes.len(), &hashes);
    let verified = f.sign(inconsistent).verify(&f.context()).unwrap();
    let available: Vec<_> = built
        .shards
        .iter()
        .enumerate()
        .map(|(i, s)| (i < 3).then(|| s.clone()))
        .collect();
    assert!(matches!(
        verified.recover(&f.context(), &available),
        Err(Error::Commitment)
    ));
    let altered = raw_manifest(
        &built.manifest,
        &bytes,
        bytes.len() - 1,
        built.manifest.shard_hashes(),
    );
    let verified = f.sign(altered).verify(&f.context()).unwrap();
    assert!(verified.recover(&f.context(), &available).is_err());
    let k = (2..=8).find(|k| bytes.len() % k != 0).unwrap();
    let built = archive(&f, k, 2);
    let mut shards = built.shards.clone();
    let width = shards[0].len();
    let last = bytes.len() / width;
    let offset = bytes.len() % width;
    shards[last][offset] = 99;
    ReedSolomon::new(k, 2).unwrap().encode(&mut shards).unwrap();
    let hashes: Vec<_> = shards.iter().map(|s| shard_hash(s)).collect();
    let padded = raw_manifest(&built.manifest, &bytes, bytes.len(), &hashes);
    let verified = f.sign(padded).verify(&f.context()).unwrap();
    let available: Vec<_> = shards.into_iter().map(Some).collect();
    assert!(verified
        .audit(&f.context(), &available)
        .unwrap()
        .iter()
        .all(|s| *s == ShardStatus::Valid));
    assert!(matches!(
        verified.recover(&f.context(), &available),
        Err(Error::Commitment)
    ));
}

// AT-NET-26: snapshot context, presence, root and shape digest are checked independently.
#[test]
fn snapshot_contract() {
    let f = Fixture::new(false, true);
    let ctx = f.context();
    let mut s = f.segment();
    s.snapshot = None;
    assert_eq!(s.verify(&ctx), Err(Error::Snapshot));
    for field in 0..9 {
        let mut s = f.segment();
        let r = &mut s.snapshot.as_mut().unwrap().statement;
        match field {
            0 => r.network_id[0] ^= 1,
            1 => r.lineage[0] ^= 1,
            2 => r.member_set_hash[0] ^= 1,
            3 => r.epoch += 1,
            4 => r.cut_number += 1,
            5 => r.cut_digest[0] ^= 1,
            6 => r.state_root[0] ^= 1,
            7 => r.shape_digest[0] ^= 1,
            _ => {
                s.snapshot.as_mut().unwrap().shape = Shape::new(vec![]).unwrap();
            }
        }
        assert_eq!(s.verify(&ctx), Err(Error::Snapshot));
    }
    let f = Fixture::new(false, false);
    let mut s = f.segment();
    s.snapshot = Some(f.snapshot());
    assert_eq!(s.verify(&f.context()), Err(Error::Snapshot));
}

// AT-NET-26: cut succession, epochs, unknown writers, retained marks and circle heads are checked.
#[test]
fn cut_contract() {
    let f = Fixture::new(true, false);
    let build = |ctx: &Context<'_>| {
        let cp =
            ctx.target
                .checkpoint_in(ctx.view, NETWORK, f.tiers.consortium().member_set_hash());
        let sigs = f
            .members
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect();
        Segment::build(ctx, &f.replica, sigs, None)
    };
    for kind in 0..8 {
        let mut target = f.target.clone();
        let circle = target
            .marks
            .iter()
            .position(|m| f.tiers.tier(&m.writer) == Some(network::tier::Tier::Circle))
            .unwrap();
        match kind {
            0 => target.number += 1,
            1 => target.epoch -= 1,
            2 => {
                target.marks.remove(
                    target
                        .marks
                        .iter()
                        .position(|m| f.tiers.tier(&m.writer) == Some(network::tier::Tier::Public))
                        .unwrap(),
                );
            }
            3 => target.marks[circle].len = 0,
            4 => {
                target.marks[circle].writer = [0; 32];
                target.marks.sort_by_key(|m| m.writer);
            }
            5 => target.marks.reverse(),
            6 => {
                let mark = target.marks[0];
                target.marks.insert(0, mark);
            }
            _ => {
                let public = target
                    .marks
                    .iter_mut()
                    .find(|m| f.tiers.tier(&m.writer) == Some(network::tier::Tier::Public))
                    .unwrap();
                public.len = 1;
                public.head = [7; 32];
            }
        }
        assert_eq!(
            build(&Context {
                target: &target,
                ..f.context()
            }),
            Err(Error::Cut)
        );
    }
    let mut previous = f.previous.clone().unwrap();
    previous.closes = true;
    assert_eq!(
        build(&Context {
            previous: Some(&previous),
            ..f.context()
        }),
        Err(Error::Cut)
    );
    previous = f.previous.clone().unwrap();
    previous.number = u64::MAX;
    assert_eq!(
        build(&Context {
            previous: Some(&previous),
            ..f.context()
        }),
        Err(Error::Cut)
    );
    previous = f.previous.clone().unwrap();
    previous.marks[0].head[0] ^= 1;
    assert_eq!(
        build(&Context {
            previous: Some(&previous),
            ..f.context()
        }),
        Err(Error::Delta)
    );
    let mut circle = *f
        .target
        .marks
        .iter()
        .find(|m| f.tiers.tier(&m.writer) == Some(network::tier::Tier::Circle))
        .unwrap();
    circle.len = 7;
    circle.head = [43; 32];
    previous = f.previous.clone().unwrap();
    previous.marks.push(circle);
    previous.marks.sort_by_key(|m| m.writer);
    let mut target = f.target.clone();
    let at = target
        .marks
        .iter()
        .position(|m| m.writer == circle.writer)
        .unwrap();
    target.marks[at].len = 9;
    target.marks[at].head = [44; 32];
    assert!(build(&Context {
        previous: Some(&previous),
        target: &target,
        ..f.context()
    })
    .is_ok());
    target.marks[at].len = 6;
    assert_eq!(
        build(&Context {
            previous: Some(&previous),
            target: &target,
            ..f.context()
        }),
        Err(Error::Cut)
    );
    target.marks[at].len = 7;
    assert_eq!(
        build(&Context {
            previous: Some(&previous),
            target: &target,
            ..f.context()
        }),
        Err(Error::Cut)
    );
}

// AT-NET-26: payloads, signatures, links and surplus entries are checked.
#[test]
fn exact_authenticated_delta() {
    let f = Fixture::new(true, false);
    for kind in 0..6 {
        let mut s = f.segment();
        match kind {
            0 => s.entries[0].1[0] ^= 1,
            1 => s.entries[0].0.signature[0] ^= 1,
            2 => s.entries[0].0.entry.prev[0] ^= 1,
            3 => s.entries[0].0.entry.hash[0] ^= 1,
            4 => s.entries[0].0.entry.seq += 1,
            _ => s.entries.push(s.entries[0].clone()),
        }
        assert!(s.verify(&f.context()).is_err());
    }
    let mut cached = f.segment();
    cached.entries[0].0.entry.hash[0] ^= 1;
    assert_eq!(cached.encode(), Err(Error::Codec));
    let missing = Replica::new(f.tiers.writers().clone());
    assert!(Segment::build(&f.context(), &missing, vec![], None).is_err());
}

// AT-NET-26: deterministic placement is independent of roster order and assigns distinct holders.
#[test]
fn deterministic_placement() {
    let f = Fixture::new(false, false);
    let built = archive(&f, 3, 2);
    let verified = f.sign(built.manifest).verify(&f.context()).unwrap();
    let mut roster: Vec<_> = (0..9).map(|n| [n; 32]).collect();
    let placement = verified.placement(&f.context(), &roster).unwrap();
    roster.reverse();
    assert_eq!(
        verified.placement(&f.context(), &roster).unwrap(),
        placement
    );
    assert_eq!(placement.iter().collect::<BTreeSet<_>>().len(), 5);
    assert!(verified.placement(&f.context(), &roster[..4]).is_err());
    roster.push(roster[0]);
    assert!(verified.placement(&f.context(), &roster).is_err());
}

// AT-NET-26: codecs reject trailing bytes, bad options, unsorted records and oversized envelopes.
#[test]
fn strict_codecs_and_envelope_cap() {
    for delta in [false, true] {
        for closes in [false, true] {
            let f = Fixture::new(delta, closes);
            let s = f.segment();
            let bytes = s.encode().unwrap();
            assert_eq!(Segment::decode(&bytes).unwrap(), s);
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(Segment::decode(&trailing).is_err());
            let mut option = bytes.clone();
            option[97] = 2;
            assert!(Segment::decode(&option).is_err());
            for i in 0..bytes.len() {
                assert!(Segment::decode(&bytes[..i]).is_err());
            }
            let built = archive(&f, 3, 2);
            let signed = f.sign(built.manifest);
            let manifest = signed.manifest.encode().unwrap();
            assert_eq!(Manifest::decode(&manifest).unwrap(), signed.manifest);
            let envelope = signed.encode().unwrap();
            assert_eq!(SignedManifest::decode(&envelope).unwrap(), signed);
            let mut trailing = envelope;
            trailing.push(0);
            assert!(SignedManifest::decode(&trailing).is_err());
            let mut option = manifest;
            option[97] = 2;
            assert!(Manifest::decode(&option).is_err());
            let mut huge = signed.clone();
            huge.signatures = (0..1000)
                .map(|i| (i, Signature::from_bytes(&[0; 64])))
                .collect();
            assert_eq!(huge.encode(), Err(Error::Bounds));
        }
    }
    assert!(Segment::decode(&vec![0; MAX_SEGMENT + 1]).is_err());
    assert!(SignedManifest::decode(&vec![0; MAX_MANIFEST + 1]).is_err());
}

// AT-NET-26: layout, shard, expansion and full-cut budgets fail before encoding oversized inputs.
#[test]
fn resource_caps() {
    let f = Fixture::new(false, false);
    let s = f.segment();
    for (k, m) in [(0, 1), (1, 0), (255, 2), (usize::MAX, 2)] {
        assert!(BuiltArchive::build(
            &f.context(),
            &s,
            Layout {
                data_shards: k,
                parity_shards: m
            }
        )
        .is_err());
    }
    let mut large = s.clone();
    large.entries[0].1 = vec![0; (16 << 20) + 1];
    assert_eq!(large.encode(), Err(Error::Bounds));
    let mut large = s.clone();
    large.entries[0].1 = vec![0; 16 << 20];
    large.entries.push(large.entries[0].clone());
    assert_eq!(large.encode(), Err(Error::Bounds));
    let mut whole_cut = s;
    whole_cut.target.marks = vec![whole_cut.target.marks[0]; MAX_SEGMENT / 72];
    assert_eq!(whole_cut.encode(), Err(Error::Bounds));
    let mut f = Fixture::new(false, false);
    for (size, k, m) in [(8 << 20, 1, 8), (16 << 20, 1, 1)] {
        let payload = vec![17; size];
        let writer = FeedWriter::from_seed(NETWORK, [1; 32]);
        let mut log = TransparencyLog::new();
        let entry = writer.sign(log.append(cid(&payload)));
        let mark = f
            .target
            .marks
            .iter_mut()
            .find(|m| m.writer == entry.writer)
            .unwrap();
        mark.head = entry.entry.hash;
        f.replica = Replica::new(f.tiers.writers().clone());
        f.replica.insert(entry, payload).unwrap();
        let segment = f.segment();
        assert!(matches!(
            BuiltArchive::build(
                &f.context(),
                &segment,
                Layout {
                    data_shards: k,
                    parity_shards: m
                }
            ),
            Err(Error::Bounds)
        ));
    }
    assert_eq!(MAX_ENCODED_SHARDS, 64 << 20);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]
    // AT-NET-26: hostile bytes reject or round-trip canonically.
    #[test]
    fn decoder_properties(bytes in prop::collection::vec(any::<u8>(), 0..8192)) {
        if let Ok(s) = Segment::decode(&bytes) { prop_assert_eq!(s.encode().unwrap(), bytes.clone()); }
        if let Ok(m) = Manifest::decode(&bytes) { prop_assert_eq!(m.encode().unwrap(), bytes.clone()); }
        if let Ok(s) = SignedManifest::decode(&bytes) { prop_assert_eq!(s.encode().unwrap(), bytes); }
    }

    // AT-NET-26: structured canonical codecs round-trip with varied layouts.
    #[test]
    fn structured_codec_properties(delta in any::<bool>(), closes in any::<bool>(), k in 1usize..9, m in 1usize..9) {
        let f = Fixture::new(delta, closes);
        let s = f.segment();
        prop_assert_eq!(Segment::decode(&s.encode().unwrap()).unwrap(), s);
        let built = archive(&f, k, m);
        let signed = f.sign(built.manifest);
        prop_assert_eq!(SignedManifest::decode(&signed.encode().unwrap()).unwrap(), signed);
    }
}
