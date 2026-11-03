//! The issuer's public encoding (`docs/05` §Distributed integration, credentials before T19,
//! AT-ID-09): one byte string per key, decoded strictly, under which credentials verify.

use identity::credential::{Credential, Issuer, IssuerPublic, ThresholdIssuer};
use identity::enrollment::Label;
use proptest::collection::vec;
use proptest::prelude::*;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

const KNOWN: &str = concat!(
    "0191daeecc18a50d9e64e2502c5efe3046323d66013438981e60d399f302b0c2bf",
    "e41eb0decd2ac0b1b7c5ddc253c241a705c60e7cce2c82faa6009045b752916f",
    "ebeff8531c04a8fede6e1e8822c8134fdf250df1351282ac44368067f145fc63",
);

// The test issuer's key has a known encoding: version, compressed G2 point (AT-ID-09).
#[test]
fn the_known_encoding() {
    let bytes = Issuer::new([1; 32]).public().encode();
    assert_eq!(bytes.len(), 97);
    assert_eq!(bytes[0], 1);
    assert_eq!(
        bytes[1] & 0xc0,
        0x80,
        "compressed, not the point at infinity"
    );
    assert_eq!(hex(&bytes), KNOWN);
}

// A decoded key re-encodes identically and verifies exactly its issuer's credentials (AT-ID-09).
#[test]
fn credentials_verify_under_the_decoded_key() {
    let issuer = Issuer::new([2; 32]);
    let decoded = IssuerPublic::decode(&issuer.public().encode()).expect("its own encoding");
    assert_eq!(decoded.encode(), issuer.public().encode());
    let holder = Credential::from_secret([3; 32]);
    let (request, pending) = holder.request_issuance(&Label([4; 32]), &decoded);
    let credential = pending.finalize(issuer.issue(&request).unwrap());
    assert!(credential.verify(&decoded));
    let other = IssuerPublic::decode(&Issuer::new([5; 32]).public().encode()).unwrap();
    assert!(!credential.verify(&other));
    let committee = ThresholdIssuer::new([6; 32], 2, 2).public();
    let again = IssuerPublic::decode(&committee.encode()).unwrap();
    assert_eq!(again.encode(), committee.encode());
}

// Prefixes, trailing bytes, other versions, the identity and non-canonical points fail (AT-ID-09).
#[test]
fn what_decoding_refuses() {
    let good = Issuer::new([1; 32]).public().encode();
    for n in 0..good.len() {
        assert!(IssuerPublic::decode(&good[..n]).is_none(), "prefix {n}");
    }
    assert!(IssuerPublic::decode(&[good.clone(), vec![0]].concat()).is_none());
    for version in [0, 2, 0xff] {
        let mut bytes = good.clone();
        bytes[0] = version;
        assert!(IssuerPublic::decode(&bytes).is_none(), "version {version}");
    }
    let mut identity = vec![1, 0xc0];
    identity.resize(97, 0);
    assert!(IssuerPublic::decode(&identity).is_none());
    for flags in [0x00, 0x20, 0xe0] {
        let mut bytes = good.clone();
        bytes[1] = (bytes[1] & 0x1f) | flags;
        assert!(IssuerPublic::decode(&bytes).is_none(), "flags {flags:#04x}");
    }
    let mut over = vec![1, 0x9f];
    over.resize(97, 0xff);
    assert!(IssuerPublic::decode(&over).is_none());
}

// A point on the curve but outside G2 does not decode, though it is well formed (AT-ID-09).
#[test]
fn a_point_outside_g2_is_refused() {
    use ark_bls12_381::{Fq, Fq2, G2Affine};
    use ark_serialize::CanonicalSerialize;
    let outside = (1u64..)
        .filter_map(|i| {
            G2Affine::get_point_from_x_unchecked(Fq2::new(Fq::from(i), Fq::from(0u64)), true)
        })
        .find(|p| !p.is_in_correct_subgroup_assuming_on_curve())
        .unwrap();
    assert!(outside.is_on_curve());
    let mut bytes = vec![1];
    outside.serialize_compressed(&mut bytes).unwrap();
    assert_eq!(bytes.len(), 97);
    assert!(IssuerPublic::decode(&bytes).is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // A changed byte, an inserted or removed one decode only as their own encoding (AT-ID-09).
    #[test]
    fn only_canonical_bytes_decode(
        at in any::<prop::sample::Index>(),
        value in any::<u8>(),
        edit in 0usize..3,
    ) {
        let mut bytes = Issuer::new([1; 32]).public().encode();
        let i = at.index(bytes.len());
        match edit {
            0 => bytes[i] ^= value | 1,
            1 => bytes.insert(i, value),
            _ => { bytes.remove(i); }
        }
        if let Some(key) = IssuerPublic::decode(&bytes) {
            prop_assert_eq!(key.encode(), bytes);
        }
    }

    // Arbitrary version-1 strings of the length decode only as their own encoding (AT-ID-09).
    #[test]
    fn arbitrary_bytes_are_canonical_or_refused(tail in vec(any::<u8>(), 96)) {
        let bytes = [vec![1], tail].concat();
        if let Some(key) = IssuerPublic::decode(&bytes) {
            prop_assert_eq!(key.encode(), bytes);
        }
    }
}
