//! The genesis document (`docs/04` §Operating contract, T88-P1, AT-NET-40): its one encoding,
//! strict decoding, the digest and network id it derives, and the tiered set it configures.

use ed25519_dalek::VerifyingKey;
use network::consortium::Member;
use network::genesis::{Genesis, GenesisError, GenesisMember, MAX_MEMBERS};
use network::tier::Tier;
use proptest::collection::vec;
use proptest::prelude::*;

fn key(seed: [u8; 32]) -> [u8; 32] {
    Member::from_seed(seed).public().to_bytes()
}

fn short(seed: u8) -> [u8; 32] {
    key([seed; 32])
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Two members, both signing, two relays: the known-answer document.
fn known() -> Genesis {
    let mut relays = vec![short(41), short(42)];
    relays.sort();
    Genesis {
        threshold: 2,
        members: vec![
            GenesisMember {
                consortium: short(1),
                circle: short(11),
                hpke: [0x21; 32],
            },
            GenesisMember {
                consortium: short(2),
                circle: short(12),
                hpke: [0x22; 32],
            },
        ],
        answer_config: b"config".to_vec(),
        issuer: b"issuer".to_vec(),
        relays,
    }
}

/// The schema's fields in order, written out by hand.
fn by_hand(g: &Genesis) -> Vec<u8> {
    let mut out = vec![1u8];
    out.extend((g.threshold as u64).to_le_bytes());
    out.extend((g.members.len() as u64).to_le_bytes());
    for m in &g.members {
        out.extend(m.consortium.iter().chain(&m.circle).chain(&m.hpke));
    }
    out.push(1);
    for field in [&g.answer_config, &g.issuer] {
        out.extend((field.len() as u64).to_le_bytes());
        out.extend(field);
    }
    out.extend((g.relays.len() as u64).to_le_bytes());
    out.extend(g.relays.iter().flatten());
    out
}

// The encoding is the schema's fields; digest and id are an independent oracle's (AT-NET-40).
#[test]
fn the_known_answers() {
    let g = known();
    assert_eq!(
        hex(&short(1)),
        "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c"
    );
    assert_eq!(g.encode(), by_hand(&g));
    assert_eq!(g.encode().len(), 310);
    assert_eq!(
        hex(&g.digest()),
        "bf92c2afcc9750b3bc4979604b8e1b831e15ab333e2c1c1ebffcffa56034a41b"
    );
    assert_eq!(
        hex(&g.id()),
        "93d4c9bfcbed8760eda9442c01b046ac2a7aa46fc47aeb2dffa9d16fbf6d0e93"
    );
    assert_eq!(Genesis::decode(&g.encode()), Ok(g));
}

// The tiered set is the document's members, threshold and circle keys under its id (AT-NET-40).
#[test]
fn the_tiers_it_configures() {
    let g = known();
    let tiers = g.tiers().unwrap();
    assert_eq!(tiers.network_id(), g.id());
    assert_eq!(tiers.consortium().threshold(), 2);
    let members: Vec<[u8; 32]> = tiers
        .consortium()
        .keys()
        .iter()
        .map(|k| k.to_bytes())
        .collect();
    assert_eq!(members, vec![short(1), short(2)]);
    assert_eq!(tiers.place(&short(12)), Some((1, Tier::Circle)));
    assert_eq!(tiers.place(&short(41)), None);
    let mut bad = g.clone();
    bad.threshold = 3;
    assert_eq!(bad.tiers().err(), Some(GenesisError::Threshold));
}

/// A key that is no point, and two non-canonical encodings of valid points.
fn not_canonical() -> [[u8; 32]; 3] {
    let no_point = (0u8..)
        .map(|b| [b; 32])
        .find(|k| VerifyingKey::from_bytes(k).is_err())
        .unwrap();
    let mut y_over_p = [0xff; 32];
    y_over_p[0] = 0xee;
    y_over_p[31] = 0x7f;
    let mut negative_zero = [0; 32];
    negative_zero[0] = 1;
    negative_zero[31] = 0x80;
    for k in [y_over_p, negative_zero] {
        assert!(VerifyingKey::from_bytes(&k).is_ok());
    }
    [no_point, y_over_p, negative_zero]
}

// Every structural fault, bound, key, repetition and relay order is refused (AT-NET-40).
#[test]
fn what_decoding_refuses() {
    use GenesisError::*;
    let good = known().encode();
    let refused = |g: &Genesis| Genesis::decode(&g.encode()).err();
    for n in 0..good.len() {
        assert_eq!(Genesis::decode(&good[..n]), Err(Malformed), "prefix {n}");
    }
    assert_eq!(
        Genesis::decode(&[good.clone(), vec![0]].concat()),
        Err(Malformed)
    );
    let byte = |at: usize, value: u8| {
        let mut bytes = good.clone();
        bytes[at] = value;
        Genesis::decode(&bytes).err()
    };
    assert_eq!(byte(0, 0), Some(Version));
    assert_eq!(byte(0, 2), Some(Version));
    assert_eq!(byte(17 + 2 * 96, 0), Some(Family));
    assert_eq!(byte(17 + 2 * 96, 2), Some(Family));
    let with = |f: &dyn Fn(&mut Genesis)| {
        let mut g = known();
        f(&mut g);
        refused(&g)
    };
    assert_eq!(with(&|g| g.threshold = 0), Some(Threshold));
    assert_eq!(with(&|g| g.threshold = 3), Some(Threshold));
    assert_eq!(with(&|g| g.threshold = usize::MAX), Some(Threshold));
    assert_eq!(with(&|g| g.members.clear()), Some(Members));
    let mut count = good.clone();
    count[9..17].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(Genesis::decode(&count), Err(Members));
    let mut relays = good.clone();
    let at = good.len() - 2 * 32 - 8;
    relays[at..at + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(Genesis::decode(&relays), Err(Malformed));
    for bad in not_canonical() {
        assert_eq!(with(&|g| g.members[1].consortium = bad), Some(Key));
        assert_eq!(with(&|g| g.members[0].circle = bad), Some(Key));
        assert_eq!(with(&|g| g.relays[0] = bad), Some(Key));
    }
    let mut p = [0xff; 32];
    p[0] = 0xed;
    p[31] = 0x7f;
    let mut below_p = p;
    below_p[0] = 0xeb;
    assert_eq!(with(&|g| g.members[0].hpke = p), Some(Key));
    assert_eq!(with(&|g| g.members[0].hpke = [0xff; 32]), Some(Key));
    assert_eq!(with(&|g| g.members[0].hpke = below_p), None);
    assert_eq!(
        with(&|g| g.members[1].consortium = short(1)),
        Some(Repeated)
    );
    assert_eq!(with(&|g| g.members[0].circle = short(2)), Some(Repeated));
    assert_eq!(with(&|g| g.members[1].circle = short(11)), Some(Repeated));
    assert_eq!(with(&|g| g.members[1].hpke = [0x21; 32]), Some(Repeated));
    assert_eq!(with(&|g| g.relays[1] = g.relays[0]), Some(Repeated));
    assert_eq!(with(&|g| g.relays.reverse()), Some(Order));
    assert_eq!(with(&|g| g.relays.clear()), None);
}

/// Small-order Ed25519 points, canonically encoded: the identity, order 2 and order 4.
fn small_order() -> [[u8; 32]; 3] {
    let mut identity = [0; 32];
    identity[0] = 1;
    let mut minus_one = [0xff; 32];
    minus_one[0] = 0xec;
    minus_one[31] = 0x7f;
    for k in [identity, minus_one, [0; 32]] {
        assert!(VerifyingKey::from_bytes(&k).is_ok_and(|k| k.is_weak()));
    }
    [identity, minus_one, [0; 32]]
}

/// Low-order X25519 coordinates: 0, 1, `p − 1` and the two of order 8.
fn low_order() -> [[u8; 32]; 5] {
    let unhex = |h: &str| -> [u8; 32] {
        let b: Vec<u8> = (0..32)
            .map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap())
            .collect();
        b.try_into().unwrap()
    };
    let mut one = [0; 32];
    one[0] = 1;
    let mut minus_one = [0xff; 32];
    minus_one[0] = 0xec;
    minus_one[31] = 0x7f;
    [
        [0; 32],
        one,
        minus_one,
        unhex("e0eb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b800"),
        unhex("5f9c95bca3508c24b1d0b1559c83ef5b04445cc4581c8e86d8224eddd09f1157"),
    ]
}

// Small-order keys in every role, and a key repeated across roles, are refused (AT-NET-40).
#[test]
fn weak_and_shared_keys_are_refused() {
    use GenesisError::*;
    let with = |f: &dyn Fn(&mut Genesis)| {
        let mut g = known();
        f(&mut g);
        (
            Genesis::decode(&g.encode()).err(),
            g.check().err(),
            g.tiers().err(),
        )
    };
    let all = |e| (Some(e), Some(e), Some(e));
    for weak in small_order() {
        assert_eq!(with(&|g| g.members[1].consortium = weak), all(Key));
        assert_eq!(with(&|g| g.members[0].circle = weak), all(Key));
        assert_eq!(with(&|g| g.relays = vec![weak]), all(Key));
    }
    for weak in low_order() {
        assert_eq!(with(&|g| g.members[1].hpke = weak), all(Key));
    }
    assert_eq!(with(&|g| g.relays = vec![short(1)]), all(Repeated));
    assert_eq!(with(&|g| g.relays = vec![short(12)]), all(Repeated));
    assert!(short(1)[31] < 0x80);
    assert_eq!(with(&|g| g.members[1].hpke = short(1)), all(Repeated));
}

// An in-memory document holds the member bound too: none, or one more than the bound (AT-NET-40).
#[test]
fn the_member_bound_holds_in_memory() {
    let mut g = known();
    g.members.clear();
    g.threshold = 0;
    assert_eq!(g.check(), Err(GenesisError::Members));
    assert_eq!(g.tiers().err(), Some(GenesisError::Members));
    g.members = members(MAX_MEMBERS + 1);
    g.threshold = 1;
    assert_eq!(g.check(), Err(GenesisError::Members));
    assert_eq!(g.tiers().err(), Some(GenesisError::Members));
}

/// Member `i`'s keys from two bytes of seed, distinct for every `i` and role.
fn wide(i: usize, role: u8) -> [u8; 32] {
    let mut seed = [role; 32];
    seed[..2].copy_from_slice(&(i as u16).to_le_bytes());
    key(seed)
}

/// `n` members with distinct keys, none of small order.
fn members(n: usize) -> Vec<GenesisMember> {
    (0..n)
        .map(|i| {
            let mut hpke = [0x55; 32];
            hpke[..2].copy_from_slice(&(i as u16).to_le_bytes());
            hpke[31] = 0;
            GenesisMember {
                consortium: wide(i, 1),
                circle: wide(i, 2),
                hpke,
            }
        })
        .collect()
}

// `MAX_MEMBERS` members decode and one more is refused (AT-NET-40).
#[test]
fn the_member_bound() {
    let mut g = known();
    g.members = members(MAX_MEMBERS);
    assert_eq!(Genesis::decode(&g.encode()), Ok(g.clone()));
    g.members = members(MAX_MEMBERS + 1);
    assert_eq!(Genesis::decode(&g.encode()), Err(GenesisError::Members));
}

prop_compose! {
    /// A valid document: up to five members, any threshold, opaque parts, up to three relays.
    fn arb_genesis()(
        n in 1usize..=5,
        t in any::<prop::sample::Index>(),
        base in any::<[u8; 30]>(),
        hpke in vec(any::<[u8; 32]>(), 5),
        answer_config in vec(any::<u8>(), 0..120),
        issuer in vec(any::<u8>(), 0..120),
        relays in 0usize..=3,
    ) -> Genesis {
        let seed = |i: usize, role: u8| {
            let mut s = [role; 32];
            s[..30].copy_from_slice(&base);
            s[30] = i as u8;
            key(s)
        };
        let members = (0..n)
            .map(|i| GenesisMember {
                consortium: seed(i, 1),
                circle: seed(i, 2),
                hpke: { let mut h = hpke[i]; h[31] &= 0x3f; h },
            })
            .collect();
        let mut relays: Vec<[u8; 32]> = (0..relays).map(|i| seed(i, 3)).collect();
        relays.sort();
        Genesis { threshold: t.index(n) + 1, members, answer_config, issuer, relays }
    }
}

/// `g` with one field changed to another valid value, `fresh` a key nowhere in `g`.
fn changed(g: &Genesis, field: usize, fresh: [u8; 32]) -> Option<Genesis> {
    let mut h = g.clone();
    let n = h.members.len();
    match field % 9 {
        0 if n > 1 => h.threshold = h.threshold % n + 1,
        1 => h.members[0].consortium = fresh,
        2 => h.members[n - 1].circle = fresh,
        3 => h.members[0].hpke[0] ^= 1,
        4 => h.answer_config.push(0),
        5 => h.issuer.insert(0, 7),
        6 => h.relays.push(fresh),
        7 if !h.relays.is_empty() => {
            h.relays.remove(0);
        }
        8 => h.members.swap(0, n - 1),
        _ => return None,
    }
    h.relays.sort();
    (h != *g && Genesis::decode(&h.encode()).is_ok()).then_some(h)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // Any valid document round-trips and configures tiers under its own id (AT-NET-40).
    #[test]
    fn any_document_round_trips(g in arb_genesis()) {
        let bytes = g.encode();
        prop_assert_eq!(&bytes, &by_hand(&g));
        prop_assert_eq!(Genesis::decode(&bytes), Ok(g.clone()));
        prop_assert_eq!(g.tiers().unwrap().network_id(), g.id());
    }

    // Any single-field change gives another encoding, digest and network id (AT-NET-40).
    #[test]
    fn any_change_changes_the_id(g in arb_genesis(), field in 0usize..9, seed in any::<[u8; 32]>()) {
        if let Some(h) = changed(&g, field, key(seed)) {
            prop_assert_ne!(h.encode(), g.encode());
            prop_assert_ne!(h.digest(), g.digest());
            prop_assert_ne!(h.id(), g.id());
        }
    }

    // A decoded byte string is the canonical encoding; extensions and prefixes fail (AT-NET-40).
    #[test]
    fn only_canonical_bytes_decode(
        g in arb_genesis(),
        at in any::<prop::sample::Index>(),
        value in any::<u8>(),
        edit in 0usize..4,
        tail in vec(any::<u8>(), 1..8),
    ) {
        let good = g.encode();
        let i = at.index(good.len());
        let mut bytes = good.clone();
        match edit {
            0 => bytes[i] ^= value | 1,
            1 => bytes.insert(i, value),
            2 => { bytes.remove(i); }
            _ => bytes.truncate(i),
        }
        if let Ok(h) = Genesis::decode(&bytes) {
            prop_assert_eq!(h.encode(), bytes);
        }
        prop_assert!(Genesis::decode(&[good.clone(), tail].concat()).is_err());
        prop_assert!(Genesis::decode(&good[..i]).is_err());
    }

    // Arbitrary bytes decode only as their own canonical encoding (AT-NET-40).
    #[test]
    fn arbitrary_bytes_are_canonical_or_refused(bytes in vec(any::<u8>(), 0..400)) {
        if let Ok(h) = Genesis::decode(&bytes) {
            prop_assert_eq!(h.encode(), bytes);
        }
    }
}
