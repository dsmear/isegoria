//! The nullifier proof's wire format v1 (`docs/04` §Events and replay, `docs/12` F10, T73):
//! only a proof's own encoding decodes, so one proof has one byte string.

use identity::nullifier::NullifierProof;

/// AT-PRO-10: a proof in another encoding the library would accept (a repeated entry inside
/// the BBS+ proof, found by fuzzing) is refused.
#[test]
fn at_pro_10_a_non_canonical_proof_encoding_is_refused() {
    let bytes = include_bytes!("fixtures/noncanonical_proof.bin");
    assert!(NullifierProof::decode(bytes).is_none());
}
