# identity fuzz targets (T44, T89)

`cargo fuzz` targets for the entry points of `identity` that take untrusted input. They
live outside the main workspace because libFuzzer needs a nightly toolchain; the same
entry points are covered on every push by `tests/hostile_input.rs` and the in-crate
property tests (stable, proptest). The audit they belong to is recorded in
`docs/12-panic-audit.md`.

One target needs code that is private in normal builds: cargo-fuzz compiles with
`--cfg fuzzing`, which exposes `ThresholdOprfOracle::fuzz_label_with_quorum`.
`nullifier_proof` uses the public wire format, `NullifierProof::{encode, decode}` (T73). The
ceremony targets share `common/ceremony.rs`, which runs one whole ceremony (four members,
threshold 3) when a target starts; the stable tests of the same entry points are
`credential`'s and `credential::ceremony`'s unit tests.

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `oprf_quorum` | the threshold OPRF with an arbitrary committee shape, anchor and claimed quorum | a label comes back exactly for a quorum of at least `t` distinct members of the committee, and equals the label of every other valid quorum |
| `enrollment` | `EnrollmentRegistry::enroll` of arbitrary codice-fiscale strings through the reference, VOPRF and threshold oracles | the same person via the other source is a duplicate |
| `voprf_wire` | the RFC 9497 messages decoded from arbitrary bytes: a blinded element at the server, an evaluation and a proof at the client | no decoded evaluation verifies without the server key |
| `nullifier_proof` | `nullifier::verify` of a proof decoded (wire format v1, T73) from arbitrary bytes, or from a genuine proof with a window overwritten | only the untouched genuine proof verifies, for its own role and context; a decoded proof re-encodes to the same bytes |
| `credential_wire` (T89) | `AnonymousCredential`, `IssuanceRequest`, `BlindSignature`, `PendingIssuance` decoded from arbitrary bytes or a genuine one with a window overwritten | one encoding each; only the genuine credential verifies, the genuine request is issued, the genuine reply and pending state finalize to a credential that verifies |
| `ceremony_flight` (T89) | `ceremony::Party::advance` at every flight of every member, one sender's payload arbitrary or genuine with a window overwritten, or an extra sender | the genuine flight is taken; an extra sender is refused as `Unexpected` by name; a state reached re-decodes as itself |
| `ceremony_state` (T89) | `ceremony::Party::decode` of arbitrary bytes or a genuine state at any flight with a window overwritten | one encoding |
| `ceremony_issuance` (T89) | `SlotRequest::decode` answered by every prepared member, `SignatureShare::decode` combined with the other genuine shares | one encoding each; only a genuine request is answered, by a signer, with its recorded share; the genuine shares combine; a combined credential verifies |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/identity
cargo +nightly fuzz run oprf_quorum -- -max_total_time=600
# anchors past the RFC 9497 input limit need longer inputs:
cargo +nightly fuzz run enrollment -- -max_len=70000 -max_total_time=600
```

A crash is saved under `fuzz/artifacts/<target>/`; replay it with
`cargo +nightly fuzz run <target> fuzz/artifacts/<target>/<file>`, and turn it into a
regression test before fixing it. `corpus/`, `artifacts/` and `target/` are git-ignored.
