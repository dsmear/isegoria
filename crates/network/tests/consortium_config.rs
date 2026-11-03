//! Consortium configuration and member-set binding (`docs/08` NET-005, `docs/10` T63):
//! `1 ≤ t ≤ n` distinct keys at construction; `verify` holds a checkpoint to its own set.

use curve25519_dalek::{EdwardsPoint, Scalar};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use network::consortium::{
    Checkpoint, CheckpointClient, CheckpointReject, CheckpointUpdate, Consortium, Member,
};

const NET: [u8; 32] = [0x5A; 32];

fn members(n: u8) -> Vec<Member> {
    (0..n).map(|i| Member::from_seed([i + 1; 32])).collect()
}

fn keys(members: &[Member]) -> Vec<VerifyingKey> {
    members.iter().map(Member::public).collect()
}

fn sign_by(members: &[Member], who: &[usize], cp: &Checkpoint) -> Vec<(usize, Signature)> {
    who.iter().map(|&i| (i, members[i].sign(cp))).collect()
}

/// AT-NET-09: a threshold of 0 is refused at construction.
#[test]
#[should_panic(expected = "1 <= threshold <= members")]
fn at_net_09_a_threshold_of_zero_is_refused() {
    Consortium::new(keys(&members(3)), 0);
}

/// AT-NET-09: a threshold above the member count is refused at construction.
#[test]
#[should_panic(expected = "1 <= threshold <= members")]
fn at_net_09_a_threshold_above_n_is_refused() {
    Consortium::new(keys(&members(3)), 4);
}

/// AT-NET-09: an empty member set is refused at construction, whatever the threshold.
#[test]
#[should_panic(expected = "1 <= threshold <= members")]
fn at_net_09_an_empty_member_set_is_refused() {
    Consortium::new(Vec::new(), 1);
}

/// AT-NET-09: a member key listed twice is refused at construction.
#[test]
#[should_panic(expected = "distinct member keys")]
fn at_net_09_duplicate_member_keys_are_refused() {
    let m = members(3);
    let mut k = keys(&m);
    k.push(m[1].public());
    Consortium::new(k, 2);
}

/// AT-NET-09: a duplicate key is refused wherever it sits, not only next to its twin.
#[test]
#[should_panic(expected = "distinct member keys")]
fn at_net_09_a_duplicate_key_apart_from_its_twin_is_refused() {
    let m = members(4);
    let k = vec![m[0].public(), m[1].public(), m[2].public(), m[0].public()];
    Consortium::new(k, 2);
}

/// AT-NET-09: every threshold in `1..=n` is accepted and verifies with exactly `t` signers.
#[test]
fn at_net_09_every_threshold_from_one_to_n_verifies_with_exactly_t_signers() {
    for n in 1..=5u8 {
        let m = members(n);
        for t in 1..=n as usize {
            let con = Consortium::new(keys(&m), t);
            let cp = Checkpoint::new(NET, con.member_set_hash(), 3, [7; 32]);
            let quorum: Vec<usize> = (0..t).collect();
            assert!(con.verify(&cp, &sign_by(&m, &quorum, &cp)), "{t} of {n}");
            assert!(
                !con.verify(&cp, &sign_by(&m, &quorum[1..], &cp)),
                "{t}-1 of {n}"
            );
        }
    }
}

/// AT-NET-09: `t` real members signing a checkpoint that declares another member set fail.
#[test]
fn at_net_09_verify_rejects_a_checkpoint_declaring_another_member_set() {
    let m = members(4);
    let con = Consortium::new(keys(&m), 3);
    let own = Checkpoint::new(NET, con.member_set_hash(), 9, [4; 32]);
    let all: Vec<usize> = (0..4).collect();
    assert!(con.verify(&own, &sign_by(&m, &all, &own)));

    let stranger = Member::from_seed([0xEE; 32]);
    let mut other_keys = keys(&m);
    other_keys.push(stranger.public());
    let other_set = Consortium::new(other_keys, 3).member_set_hash();
    let mut flipped = con.member_set_hash();
    flipped[31] ^= 1;
    for foreign in [other_set, [0; 32], flipped] {
        let cp = Checkpoint::new(NET, foreign, 9, [4; 32]);
        assert!(
            !con.verify(&cp, &sign_by(&m, &all, &cp)),
            "foreign set {foreign:?}"
        );
    }
}

/// The identity point, of small order, and `R = [s]B`, `s`: it verifies leniently on any message.
fn small_order_key_and_forgery() -> (VerifyingKey, Signature) {
    let mut identity = [0; 32];
    identity[0] = 1;
    let s = Scalar::from(7u64);
    let r = EdwardsPoint::mul_base(&s).compress().to_bytes();
    let forged: [u8; 64] = [r, s.to_bytes()].concat().try_into().unwrap();
    (
        VerifyingKey::from_bytes(&identity).unwrap(),
        Signature::from_bytes(&forged),
    )
}

/// AT-NET-09: a signature forged for a small-order member key counts toward no checkpoint.
#[test]
fn at_net_09_a_forgery_for_a_small_order_member_key_does_not_count() {
    let m = members(2);
    let (weak, forged) = small_order_key_and_forgery();
    assert!(weak.verify(b"any message", &forged).is_ok());
    let con = Consortium::new(vec![weak, m[0].public(), m[1].public()], 2);
    let cp = Checkpoint::new(NET, con.member_set_hash(), 5, [3; 32]);
    let honest = vec![(1, m[0].sign(&cp)), (2, m[1].sign(&cp))];
    assert!(con.verify(&cp, &honest));
    let sigs = vec![(0, forged), honest[0]];
    assert!(!con.verify(&cp, &sigs));
    assert!(!con.verify_excluding(&cp, &sigs, &[2]));
    let mut client = CheckpointClient::new(NET, con);
    let refused = CheckpointUpdate::Rejected(CheckpointReject::InsufficientSignatures);
    assert_eq!(client.ingest(&cp, &sigs), refused);
}
