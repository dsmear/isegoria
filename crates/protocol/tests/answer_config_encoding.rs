//! The answer review's configuration in the genesis (`docs/04` T88-P1, `docs/05` T79-P1,
//! AT-NET-40): the offline record's encoding, decoded strictly to a valid configuration.

use proptest::collection::vec;
use proptest::prelude::*;
use protocol::answer_author::QuotaBands;
use protocol::answer_path::AnswerConfig;
use protocol::answer_rule::ConsensusRule;
use protocol::author_time::AuthorConfig;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The twelve words of an encoding, each little-endian.
fn words(values: [u64; 12]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn default_words() -> [u64; 12] {
    let f = |x: f64| x.to_bits();
    [12, 9, 3, 4, f(0.25), f(0.30), f(0.70), 1, 2, 3, 2, 45]
}

// D49–D51's configuration has an oracle's known encoding and decodes back (AT-NET-40).
#[test]
fn the_known_encoding() {
    let bytes = AnswerConfig::default().encode();
    assert_eq!(
        hex(&bytes),
        concat!(
            "0c00000000000000090000000000000003000000000000000400000000000000",
            "000000000000d03f333333333333d33f666666666666e63f0100000000000000",
            "0200000000000000030000000000000002000000000000002d00000000000000"
        )
    );
    assert_eq!(bytes, words(default_words()));
    assert_eq!(AnswerConfig::decode(&bytes), Some(AnswerConfig::default()));
}

// Prefixes, trailing bytes and every invalid or non-canonical value are refused (AT-NET-40).
#[test]
fn what_decoding_refuses() {
    let good = AnswerConfig::default().encode();
    for n in 0..good.len() {
        assert_eq!(AnswerConfig::decode(&good[..n]), None, "prefix {n}");
    }
    assert_eq!(AnswerConfig::decode(&[good, vec![0]].concat()), None);
    let f = |x: f64| x.to_bits();
    let bad: [(usize, u64); 16] = [
        (0, 0),
        (0, 1 << 32),
        (1, 0),
        (1, 13),
        (2, 2),
        (2, 5),
        (4, f(0.0)),
        (4, f(1.5)),
        (4, f(f64::NAN)),
        (5, f(-0.0)),
        (5, f(0.8)),
        (5, f(-0.1)),
        (6, f(1.01)),
        (7, 1 << 32),
        (10, 1 << 32),
        (11, 0),
    ];
    for (at, value) in bad {
        let mut w = default_words();
        w[at] = value;
        assert_eq!(
            AnswerConfig::decode(&words(w)),
            None,
            "word {at} = {value:#x}"
        );
    }
    let mut zero = default_words();
    zero[5] = f(0.0);
    zero[6] = f(-0.0);
    assert_eq!(AnswerConfig::decode(&words(zero)), None, "negative zero");
    zero[6] = f(0.0);
    assert!(AnswerConfig::decode(&words(zero)).is_some());
}

prop_compose! {
    /// A valid configuration over the whole range each part accepts.
    fn arb_config()(
        group in 1u32..200,
        quorum in any::<prop::sample::Index>(),
        den in 1u32..200,
        num in any::<prop::sample::Index>(),
        weight in 1u64..=1000,
        bands in (0u64..=1000, 0u64..=1000),
        quotas in any::<[u32; 3]>(),
        new_author in any::<u32>(),
        half_life in 1u64..,
    ) -> AnswerConfig {
        let num = den / 2 + 1 + num.index((den - den / 2) as usize) as u32;
        let rule = ConsensusRule::new(group, quorum.index(group as usize) as u32 + 1, num, den);
        let (low, high) = (bands.0.min(bands.1) as f64 / 1000.0, bands.0.max(bands.1) as f64 / 1000.0);
        AnswerConfig {
            rule: rule.unwrap(),
            divided_weight: weight as f64 / 1000.0,
            quotas: QuotaBands::new(low, high, quotas, new_author).unwrap(),
            author: AuthorConfig::new(half_life).unwrap(),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Any valid configuration round-trips through its 96-byte encoding (AT-NET-40).
    #[test]
    fn any_configuration_round_trips(config in arb_config()) {
        let bytes = config.encode();
        prop_assert_eq!(bytes.len(), 96);
        prop_assert_eq!(AnswerConfig::decode(&bytes), Some(config));
    }

    // Any byte string decodes only as its own canonical encoding (AT-NET-40).
    #[test]
    fn only_canonical_bytes_decode(
        config in arb_config(),
        at in any::<prop::sample::Index>(),
        value in any::<u8>(),
        noise in vec(any::<u8>(), 0..120),
    ) {
        let mut bytes = config.encode();
        let i = at.index(bytes.len());
        bytes[i] ^= value;
        for candidate in [bytes, noise] {
            if let Some(decoded) = AnswerConfig::decode(&candidate) {
                prop_assert_eq!(decoded.encode(), candidate);
            }
        }
    }
}
