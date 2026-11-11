# Isegoria — Panic audit and fuzzing report (T44)

| | |
|---|---|
| **Task** | T44 in `docs/10-roadmap.md`: fuzz every byte decoder, classify every `unwrap`/`expect`/`assert` in `src/`, and turn the ones that external input can reach into errors |
| **Audited** | master `6c61263`, 2026-09-24; the inventory re-checked after rebasing onto T41 (`dd7a6b6`): unchanged in `protocol` and `scoring`; and after merging T48 (`57bdca1`): one new internal-invariant site in `scoring` (§2.3) |
| **Changed** | `crates/network`, `crates/identity`; `crates/scoring` (`bridging`) with T62, 2026-09-24 |
| **Classified only** | `crates/protocol`, and `crates/scoring` outside `bridging`: being changed in parallel at the time (T41, the scoring half of T42, T48), so their sites are recorded here and not touched; the shape preconditions of the other `scoring` entry points were probed with T62 (§2.3) and are T46's |
| **Status of the "done when"** | no panic on arbitrary bytes: met for every decoder of `network` and `identity` (§3, §4); classification recorded: §2 |

## 1. What was looked for, and the classes

Two things. First, every `unwrap()`, `expect(..)`, `assert*!`, `panic!`, `unreachable!`
outside `#[cfg(test)]`. Second — found by fuzzing, or by reading the decoders once the
fuzzer pointed at them — every other way hostile input can stop the process: an
arithmetic overflow (a panic in debug builds), an allocation sized from the input, an
out-of-range index, and unbounded work inside a dependency that parses untrusted bytes.

Each site is in one of four classes:

| Class | Meaning | Treatment |
|---|---|---|
| **Internal invariant** | holds by construction: a failure is a bug in this code | kept; the message states the invariant |
| **Configuration** | a parameter the operator picks at setup (a committee's shape, the layout of one's own erasure code) | kept, documented as `# Panics` |
| **Caller precondition** | the shape of an argument the calling code builds, not one another party sends | kept; listed for T46 (validated boundary types) |
| **External input** | reachable with bytes or values another party controls | turned into an error, or the function made total |

"External" is judged against the protocol, not today's in-process wiring: the anchor
comes from an identity document, receipts, shards, checkpoints and signatures come from
other nodes, and a quorum's answers come from committee members.

## 2. Inventory

35 sites outside `#[cfg(test)]` at `6c61263` (86 counting test modules): identity 28,
network 4, protocol 2, scoring 1. After this audit: 30, none of them reachable from
external input. T48, merged afterwards, adds one internal-invariant site in `scoring`
(§2.3), for 31; T63 (2026-09-26) two configuration sites in `network` (§2.2), for 33; T13
(2026-09-27) three internal-invariant sites in `network::store` (§2.2), for 36; T73 four
more — `NullifierProof::encode` (§2.1) and `codec::Reader::fixed` (§2.2) — for 40; its second step one in `protocol::events` (§2.3), for 41; its third four in
`protocol::results` (§2.3), for 45; T18 one in `network::replica` (§2.2) and two in
`p2p` (§2.2), for 48; T74 one in `protocol::ledger` (§2.3), for 49; T76 (2026-10-07) records
in `network::rounds` (§2.2) the two the ledger's beacon round carried since T74 step 2,
moved there, for 51. Sites are named by function; line numbers drift.

### 2.1 `identity`

| Where | Site | Class | Status |
|---|---|---|---|
| `credential::Credential::request_issuance` | `commit_to_messages(..).expect` | internal: one committed message at constant index 0 < 2 | kept |
| same | `sc.response(..).expect` | internal: two witnesses, two blindings | kept |
| `credential::pok_challenge` | 3 × `serialize_compressed(..).unwrap()` | **external**: the issuer computes it over a request it received (`verify_request`). Writing to a `Vec` cannot fail, but a verification path should not rest on that | **made fallible**; the issuer answers `InvalidProofOfKnowledge`, the holder side keeps one `expect` on its own data |
| `credential::ThresholdIssuer::new` | `assert!(1 <= t <= n)` | configuration | kept, `# Panics` |
| same | `deal_random_secret(..).expect` | internal: follows from the assert | kept |
| `credential::ThresholdIssuer::threshold_sign` | `comm_zeros[..].get(&i).unwrap()` | internal **today**: the committee runs in one process, and every member's zero-sharing commitments cover all the others | kept; **becomes external** when members are remote (DKG/transport, future): must then be an error |
| `credential::setup_base_ot` | 6 × `.unwrap()` on the base-OT rounds | internal **today**: trusted in-process setup | kept; same caveat as above |
| `enrollment::VoprfOracle::new` | `new_from_seed(..).expect` | internal: a 32-byte seed with a fixed info string always derives a key (failure has negligible probability) | kept |
| `enrollment::VoprfOracle::label` | `blind(..).expect`, `finalize(..).expect` | **external**: the anchor's bytes come from the enrollment adapter. RFC 9497 caps an input at `u16::MAX` bytes, and a longer anchor made `finalize` fail — a panic, under a message claiming the proof failed | **fixed (F8)**: the oracle is total; both `expect`s are now internal and say why |
| `nullifier::context_generator` | 2 × `.expect` on hash-to-curve | internal: constant domain tag and curve configuration | kept |
| `nullifier::NullifierProof::encode` (T73) | 3 × `serialize_compressed(..).unwrap()` | internal: writing the proof's own points and BBS+ proof to a `Vec`; `decode` calls it on a proof that already decoded | kept |
| `nullifier::NullifierProof::id` | `serialize_compressed(..).unwrap()` | internal: writing a verified point to a `Vec` | kept |
| `nullifier::challenge` | `contribute(..).expect` and 3 × `serialize_compressed(..).unwrap()` | **external**: `verify` computes it over a proof it received | **made fallible (F9)**; `verify` answers `false`, `prove` keeps one `expect` on its own proof |
| `nullifier::prove` | `PoKOfSignatureG1Protocol::init(..).expect`, `gen_proof(..).expect` | internal: the holder's own credential, and `IssuerPublic` is opaque and always carries two message generators | kept |
| `oprf::ThresholdOprfOracle::new` | `assert!(1 <= t <= n)` | configuration | kept, `# Panics` |
| `oprf::ThresholdOprfOracle::label` | `label_with_quorum(..).expect` | internal: the first `t` members are the oracle's own honest shares | kept |

### 2.2 `network`

| Where | Site | Class | Status |
|---|---|---|---|
| `anchoring::serialize` | `to_writer(..).expect` | internal: a timestamp this module built, written to a `Vec` | kept |
| `consortium::Consortium::new` (T63) | `assert!(1 <= threshold <= n)`, `assert_eq!` on the count of distinct keys | configuration: the operator sets up the consortium's member keys and threshold; `t = 0` accepted a checkpoint with no signature, `t > n` never verified | kept, `# Panics` |
| `erasure::encode` | `ReedSolomon::new(..).expect` | configuration: the encoder picks the layout of its own data | kept, `# Panics` |
| same | `r.encode(..).expect` | internal: equal-length shards built just above | kept |
| `log::TransparencyLog::append` | `last().unwrap()` | internal: an entry was just pushed | kept |
| `codec::Reader::fixed` (T73) | `try_into().expect` | internal: `take(N)` returned exactly `N` bytes | kept |
| `replica::Replica::entries_for` (T18) | index `self.entries[id]` | internal: `id` was kept only if `contains(id)` | kept |
| `rounds::Rounds::count`, `Rounds::finish` (T74, moved by T76) | `replay(..).expect` on the counted commits, `.expect` on finishing the round | internal: only commits and reveals the round accepted are kept, so replaying them in the same order succeeds; a round with commits closes and finishes, with no reveal too. It needs the same consortium and network id at every call: the ledger holds its own, `observer::Observer` refuses other tiers before counting (`ObserveError::Context`, T77), and `p2p::member::Collector`, fixed to one consortium and network by `Collector::new` or its first update, refuses another (`OtherContext`, T77) | kept |
| `p2p::Handle::ask` (T18) | 2 × `.expect` on the command channel and its answer | internal: the node's task runs while its handle lives (dropping the handle aborts it) and answers every command | kept |
| `store::DurableLog::open` (T13) | 3 × `try_into().expect` on a record's fields | internal: slices of fixed width inside a `chunks_exact(104)` record; the file's bytes decide only whether the record chains | kept |

### 2.3 `protocol` and `scoring` (classified only)

| Where | Site | Class | Status |
|---|---|---|---|
| `protocol::randomness::Beacon::seed` | `d[..8].try_into().expect` | internal: SHA-256 yields 32 bytes | — |
| `protocol::review::assign_reviewers` | `stratum.choose(..).unwrap()` | internal: every stratum is non-empty (`lo < n`, `hi >= lo + 1`) | — |
| `protocol::ledger::Ledger::apply` (T74) | `replica.get(..).expect` | internal: `cut::added` names only entries the replica holds | kept |
| `protocol::events::read_nyms` (T73) | `try_into().expect` | internal: a `chunks_exact(32)` chunk of a field whose length was checked to be a multiple of 32 | kept |
| `protocol::results::read_floats` (T73) | `try_into().expect` | internal: a `chunks_exact(8)` chunk | kept |
| `protocol::results::read_record` (T73) | 2 × `try_into().expect` | internal: the two parts of a `chunks_exact(40)` chunk, 32 and 8 bytes | kept |
| `protocol::results::ResultsState::apply` (T73) | `.expect("an escrow has a history")` | internal: an open escrow is recorded only with its author's history | kept |
| `scoring::bridging::Ratings::with_weights` | `assert_eq!(weights.len(), self.n)` | was a caller precondition; **gone** (T62): `Ratings::validate` reports the mismatch as `RatingsError::WeightCount` at `fit`/`bridge_scores` | RESOLVED (T62) |
| `scoring::bridging::fit` (T48) | `best.expect("at least one start")` | internal: the loop runs `n_starts.max(1)` times, and the first start always sets `best` | — |
| `scoring::bridging` objective and gradient | slice indexing by `Obs { u, j }` | external input: `Ratings` has public fields, and an observation with `u ≥ n` or `j ≥ m` panicked on the bounds check (third review, 2026-09-24). **RESOLVED** (T62): `Ratings::validate` runs first in `fit` and `bridge_scores`, which return `Result<_, RatingsError>` — out-of-range index, wrong weight count, non-finite rating, non-finite or negative weight, duplicate `(u, j)` pair; `scoring/tests/malformed_ratings.rs` pins each case and a property over arbitrary `Ratings`; `scoring/fuzz/bridging` (§4) | RESOLVED (T62) |

Not an `unwrap`, but noted while reading: `scoring::bridging::Ratings::from_dense` indexes
`mask[u][j]` and `r[u][j]` with the width of row 0, so a ragged or short matrix panics.
A caller precondition today; T46's `ValidatedRatings` makes it unrepresentable. The
public entry points of `protocol` were not fuzzed here (§6).

**Shape preconditions of the other `scoring` entry points (probed with T62, 2026-09-24).**
Each slice- or matrix-taking entry point was called with mismatched lengths, ragged
matrices and empty input under `catch_unwind` (a scratch test, not committed). Their
inputs are built by the protocol crate from its own data, so a mismatch is a caller
precondition of the same kind as `from_dense`'s, recorded here and left to T46's validated
types (a sample with one row per admitted respondent, a square correlation matrix):

| Entry point | Mismatched input | Effect |
|---|---|---|
| `irt::point_biserial` | `item.len() ≠ total.len()` | panic (index) |
| `irt::fit_2pl_item` | empty input | panic; a shorter `theta` or `responses` is zip-truncated |
| `dif::logistic_dif` (calibration) | `theta` shorter than `item`; empty input | panic |
| `dif::mantel_haenszel` (calibration) | `theta` shorter than `item` | panic; `n_strata = 0` and empty input tolerated |
| `dif::mixture_dif` | `theta` shorter than the rows of `x` | panic; ragged `x`, `k` past the row length, `k = 0` and empty input tolerated |
| `collusion::correlation_matrix` | ragged `judgments` | panic |
| `collusion::discount_weights` | fewer `cluster_ids` than `weights` | panic; more are ignored |
| `reputation::crowd_baseline` | ragged `predictions` | panic; a weight vector of another length is zip-truncated |
| `irt::theta_from_anchors`, `collusion::cluster_by_correlation`, `reputation::{brier_skill_score, author_score}` | ragged, non-square or mismatched input | tolerated (zip truncation or a defined degenerate value) |

## 3. External-input crashes found, and fixes

Each was reproduced first (stable toolchain, debug build, the way `cargo test` runs),
then fixed, then pinned by a regression test.

| # | Input | Effect | Where the fault is | Fix | Regression test |
|---|---|---|---|---|---|
| F1 | an `.ots` varint of more than ten bytes (e.g. as the version) | **panic** in debug builds: `attempt to shift left with overflow` | `opentimestamps` 0.2.0 `ser.rs::read_uint` | bounded pre-scan in `anchoring` refuses varints longer than 9 bytes before the library parses | `anchoring::tests::an_overlong_varint_is_refused` |
| F2 | an unknown attestation declaring 2⁶² bytes | **process abort**: `memory allocation of 4611686018427387904 bytes failed` (uncatchable) | library `read_fixed_bytes(len)` allocates before reading | pre-scan requires the declared length to fit in the proof | `…::an_attestation_longer_than_the_proof_is_refused` |
| F3 | a chain of `Hexlify` operations | message grows ×2 per input byte: 20 bytes of input → 32 MiB in 0.7 s; ~30 → out of memory | library executes operations while parsing, with no cap on a result | pre-scan caps every operation's result at 4096 bytes — the limit of python-opentimestamps, the reference implementation | `…::a_message_longer_than_4096_bytes_is_refused` |
| F4 | many fork branches over a large message | each branch clones the message and every step stores its output: memory hundreds of times the input, even with results capped | library design | pre-scan charges every clone and stored output against a 1 MiB budget | `…::fork_amplification_is_bounded` |
| F5 | an erasure layout with `data_shards + parity_shards` overflowing | **panic** in debug builds inside `ReedSolomon::new` | `reed-solomon-erasure` 6.0.0 sums the counts unchecked | `erasure::check_layout` validates the claimed counts first → `RecoverError::InvalidLayout` | `shard_authentication.rs::an_impossible_layout_is_refused_not_sized` |
| F6 | an `orig_len` of `usize::MAX` | **panic**: `capacity overflow` (smaller huge values: abort) | `erasure::reconstruct` sized its buffer from the claim | the claim must fit in the recovered shards → `InvalidLayout`; nothing is allocated from it before that | same test |
| F7 | `merkle_proof` with an index ≥ the leaf count | **panic** (index out of bounds), or for some indices a silent proof of no leaf | `merkle::merkle_proof` | returns `Option`; `None` for an out-of-range index | `integrity.rs::merkle_inclusion_proof_verifies` |
| F8 | an anchor longer than 65535 bytes | **panic** in `VoprfOracle::label` | RFC 9497's input limit, reached through an `expect` | a longer anchor is hashed to 64 bytes first and its label carries its own tag, so the oracle is total and cannot collide a long anchor with a short one; anchors up to the limit get the same label as before | `voprf_oracle.rs::an_anchor_beyond_the_rfc_9497_limit_still_gets_a_stable_label` |
| F9 | a received nullifier proof or issuance request | would panic if serializing the transcript ever failed (it cannot today) | `nullifier::challenge`, `credential::pok_challenge` | fallible; the verifier answers `false` / `InvalidProofOfKnowledge` | covered by the no-panic properties (§5) |
| F10 | a nullifier proof whose BBS+ part repeats an entry (found by `protocol/event`, T73) | no panic: the library decodes it to the same proof, so one proof — and one event — had several encodings and CIDs | `bbs_plus` 0.25.0 deserialization accepts a repeated entry | `NullifierProof::decode` keeps only a proof's own encoding (it re-encodes and compares) | `identity/tests/proof_encoding.rs`, `protocol/tests/node_replay.rs::at_pro_10_a_non_canonical_encoding_is_refused` (fixtures from the crash) |
| F11 | a previous cut with a mark of length 0 (found by `network/cut`, T74) | arithmetic overflow, a panic in debug builds | `cut::added` took `len - 1` on the previous cut's mark, assuming it well formed | `checked_sub`: such a cut is refused as `Retracts` | `cuts.rs` `at_net_15_a_malformed_previous_cut_is_refused` |

**The OTS pre-scan.** `anchoring::within_bounds` walks the same grammar as the library's
parser — header, digest, step tree, attestations, no trailing bytes — without executing
anything, and tracks the message length the library would build at each step. It is
deliberately a byte-for-byte mirror: refusing is always safe (such a proof is `Invalid`),
and a proof it accepts makes the library read exactly the same bytes, with its depth
(256, the library's own limit), every operation's result (≤ 4096 bytes), every declared
length (inside the proof), the total memory it materializes (≤ 1 MiB) and the proof itself
(≤ 64 KiB) bounded. The two real proofs from the library's own test vectors
(`network/tests/fixtures/ots/`, including a full Bitcoin-attested one) pass it. Earlier
audits (docs/08 NET-008) took the library's recursion limit as the bound; it bounds depth
only, not lengths, allocations or work. `opentimestamps` 0.2.0 is the latest release;
F1–F4 are worth reporting upstream.

## 4. Fuzz targets

Twenty-eight `cargo fuzz` targets, in `crates/{network,identity,scoring,protocol,p2p,ceremony}/fuzz/`
(outside the workspace: libFuzzer needs nightly); the eight marked *new* have had only their
first smoke runs (2026-11-11, below). Each
README says how to run them. Besides "no panic, no abort, bounded memory", each asserts a property
of its entry point:

| Target | Entry point | Also asserted |
|---|---|---|
| `network/ots_verify` | `OtsAnchor::verify` on arbitrary receipt bytes (AT-NET-07) | — |
| `network/erasure` | `reconstruct`, `reconstruct_verified`: hostile shards and layouts; genuine encodings with losses and corruptions | a genuine encoding recovers exactly when `data_shards` authentic shards survive, to the original bytes |
| `network/checkpoint` | `CheckpointClient::ingest` / `ingest_with_log` over sequences of honest and forged checkpoints | the trusted height never decreases; only `Accepted` changes the trusted checkpoint; acceptance needs a threshold of distinct member signatures |
| `network/merkle` | `merkle_proof`, `verify_proof` | a proof exists exactly for an in-range leaf, and verifies |
| `protocol/event` (T73) | `NodeEvent::decode` on arbitrary bytes, seeded with genuine events | a decoded event re-encodes to the same bytes (it found F10) |
| `network/cut` (T74) | `Cut::decode`, `SignedEntry::decode`, `MemberObject::decode` on arbitrary bytes; `added` on arbitrary previous and next cuts, with epochs, over a replica with forked and partial feeds; `Cut::next` | one encoding each; `added` names only held entries, each once, and keeps the epoch rule; a proposal extends an applied cut (it found F11) |
| `network/replica` (T18) | `Message::decode` on arbitrary bytes; replicas fed honest, forked and outsider entries and raw Entries messages, pulling from each other within any cap | a decoded message re-encodes to the same bytes; a wanted entry the peer holds is sent; every entry a replica holds is valid; equivocations verify; two pulls each way converge |
| `network/store` (T13) | `DurableLog::open`, `ObjectStore::open` on arbitrary file bytes, with and without a genuine header | an opened log verifies and takes an append that survives a reopen; a stored object reads back under its CID after a reopen |
| `identity/oprf_quorum` | the threshold OPRF with any committee shape, anchor and claimed quorum | a label exactly for `≥ t` distinct committee members, equal for every valid quorum |
| `identity/enrollment` | `EnrollmentRegistry::enroll` through the reference, VOPRF and threshold oracles | the same person via the other source is a duplicate |
| `identity/voprf_wire` | RFC 9497 messages decoded from arbitrary bytes, at the server and at the client | no decoded evaluation verifies without the server key |
| `identity/nullifier_proof` | `nullifier::verify` of proofs decoded from arbitrary bytes or spliced into a genuine one | only the untouched genuine proof verifies, for its own role and context |
| `scoring/bridging` (T62) | `bridging::fit`, `bridging::bridge_scores` on a `Ratings` assembled field by field: indices past `n`/`m`, any rating, any number of weights of any value | an error exactly when `Ratings::validate` refuses the input; a validated input always fits |
| `protocol/answer_event` (T89) | `AnswerEvent::decode` on arbitrary bytes or a genuine join, deposit or reveal with a window overwritten, then `AnswerPath::apply` to a path holding one reviewer | one encoding |
| `protocol/genesis` (T89) | `Genesis::decode`, then `AnswerConfig::decode` and `IssuerPublic::decode` on its parts, as `node::genesis::load` reads them; arbitrary bytes or a genuine document with a window overwritten | one encoding each; a decoded document yields its tiered set under its id |
| `p2p/bind` (T89) | `Bind::decode` (`/isegoria/bind/1`, with `TransportBinding`) on arbitrary bytes or a genuine hello with a window overwritten, then `Tiers::access`; `transport_key` of arbitrary peer ids | one encoding; only the genuine binding, from its own transport key, grants the circle |
| `p2p/key_config` (T89) | `KeyAnnouncement::decode`, `verify`, `hpke_key` on arbitrary bytes or a genuine signed configuration with a window overwritten | one encoding; only the genuine configuration verifies; `seal` succeeds exactly under a verified one |
| `p2p/client_reply` (T89) | `Carried::decode` and `Sealed::open` on arbitrary relay replies, on any plaintext sealed in the request's context, and on every gateway answer | one encoding; a reply not sealed for the request never opens; a receipt names only the member's circle feed; an answer opens as the gateway sealed it |
| `p2p/gateway` (T89) | the gateway's decapsulation of arbitrary relayed requests and its reading of arbitrary inner requests | an event sealed by `seal` opens and reads back as itself, and as another network's request elsewhere |
| `p2p/serve` (T89) | object and shard requests through `ObjectCodec`, `ShardCodec` and the node's servers | a request reads exactly when well formed; only a held public object is served; an empty store serves nothing |
| `protocol/answer_gateway` (T89, *new*) | `answer_gateway::admit` of arbitrary bytes or a genuine join, deposit, commit or reveal with a window overwritten | a genuine event is admitted; a join, deposit or commit only as a genuine one; never a member's or derived event |
| `protocol/answer_read` (T89, *new*) | `ReadRequest::decode`, `Reply::decode` and `answer_read::answer` over an empty ledger, on arbitrary bytes or a genuine request or reply with a window overwritten | one encoding each; a genuine request is answered; an answer only for a genuine request's kind, response key and proof, opening under that key to a reply |
| `protocol/answer_seal` (T89, *new*) | `ResponseKey::open` of arbitrary bytes or a genuine seal with a window overwritten; `seal` to an arbitrary response key | only the untouched genuine seal opens, to its reply; a seal is its reply plus `OVERHEAD` |
| `identity/credential_wire` (T89, *new*) | `AnonymousCredential`, `IssuanceRequest`, `BlindSignature`, `PendingIssuance` decoded from arbitrary bytes or a genuine one with a window overwritten | one encoding each; only the genuine credential verifies, the genuine request is issued, the genuine reply and pending state finalize |
| `identity/ceremony_flight` (T89, *new*) | `credential::ceremony::Party::advance` at every flight of every member of one ceremony (4 members, `t = 3`), one sender's payload arbitrary or genuine with a window overwritten, or an extra sender | the genuine flight is taken; an extra sender is refused by name; a state reached re-decodes as itself |
| `identity/ceremony_state` (T89, *new*) | `Party::decode` of arbitrary bytes or a genuine state at any flight with a window overwritten | one encoding |
| `identity/ceremony_issuance` (T89, *new*) | `SlotRequest::decode` answered by every prepared member; `SignatureShare::decode` combined with the genuine shares | one encoding each; only a genuine request is answered, by a signer, with its recorded share; the genuine shares combine |
| `ceremony/message` (T89, *new*) | `ceremony::message::open` by its recipient of arbitrary bytes or a genuine message with a window overwritten | a genuine message opens to its header and payload; no other bytes open |

Two crates expose private code under `--cfg fuzzing` (set by cargo-fuzz); normal builds do
not contain it: `ThresholdOprfOracle::fuzz_label_with_quorum`, and `p2p::fuzz` (the gateway's
opening and reading of a request, an answer's Binary HTTP, the object and shard servers).
`identity/nullifier_proof` used a test encoding under the same flag until T73, and now
fuzzes the wire format v1 (`NullifierProof::{encode, decode}`).

**T89 part 1: the decoders a trial node or a participant's client accepts from another
machine** (`01` D62), drawn from the code on `feat/phase2-runtime` `7cef417`
(2026-11-10), then on `8e88939` (2026-11-11: T79's increment 3 so far, and T19). A trial node is
a tiered node (`04` §A node process), member or relay, run as `04` §The trial's procedure says,
archiving outside it:

| Decoder | Path | Read from another machine by | Target |
|---|---|---|---|
| sync messages: Summary, Have, Want, Entries (with `SignedEntry`) | `network::replica::Message::decode` | every node: `/isegoria/sync/1` requests and replies, gossip announcements | `network/replica` |
| members' objects: cut signatures, beacon commits and reveals, view changes; cuts | `network::cut::{MemberObject, Cut}::decode` | every node collecting cuts from replicated entries | `network/cut` |
| the binding exchange, and the peer's transport key | `p2p::tiered::Bind::decode` (`network::tier::TransportBinding::decode`), `tiered::transport_key` | every tiered node, on each connection | `p2p/bind` |
| signed key configurations | `p2p::blind::KeyAnnouncement::{decode, verify}` (`ohttp::KeyConfig::decode`) | each operator's node TOML (`consortium.hpke`, made by `--announce` on the member's machine); the client | `p2p/key_config` |
| a relay's reply, and the sealed answer in it | `blind::Carried::decode`, `blind::Sealed::open` (OHTTP decapsulation, `Answer::from_http`) | the participant's client | `p2p/client_reply` |
| a relayed request, and its inner request | `blind::open_request` (OHTTP decapsulation), `blind::read_request` (Binary HTTP) | a member's gateway | `p2p/gateway` |
| object and shard requests | `discovery::serve`, `archive::Store::serve`, through `ObjectCodec` and `ShardCodec` | a tiered node, unless `discovery_and_archive = false` turns both protocols and Kademlia off (`04` §A node process, built per `01` D64), as a trial node can; the archive store is empty in the trial | `p2p/serve` |
| the genesis document, its answer configuration and issuer key | `network::genesis::Genesis::decode`, `AnswerConfig::decode`, `IssuerPublic::decode` (`node::genesis::load`) | every node and `--genesis-digest` | `protocol/genesis` |
| answer-review events, and the nullifier proof in them | `protocol::answer_events::AnswerEvent::decode`, `NullifierProof::decode` | a member's node, applying each collected cut (the hook, `node::answer`) | `protocol/answer_event`, `identity/nullifier_proof` |
| a person's event, before it is logged | `protocol::answer_gateway::admit` (`AnswerEvent::decode`, the proof's check) | a member's gateway, once the node passes it as the gateway's admission: on `8e88939` the gateway admits nothing (`node::blind_plan`) | `protocol/answer_gateway` (*new*) |
| read requests, the replies, and their inner seal | `protocol::answer_read::{ReadRequest, Reply}::decode`, `answer_read::answer`; `answer_seal::seal` to the request's key, `ResponseKey::open` | a member's gateway on `…/read`, and the client, once both are wired: on `8e88939` neither calls them | `protocol/answer_read`, `protocol/answer_seal` (*new*) |
| the credential's private encodings | `identity::credential::{IssuanceRequest, BlindSignature, PendingIssuance, AnonymousCredential}::decode` | the test issuer (a request), the person (its reply), each by files from the other's machine (`isegoria-test-issuer`, `05` §Distributed integration); the pending state and the credential are the person's own files | `identity/credential_wire` (*new*) |
| ceremony messages, and the payloads in them | `ceremony::message::open` (the header's decoding, the seal, the signature), then `credential::ceremony::Party::advance` (each payload one strict arkworks object) | each committee member's `isegoria-ceremony step`, from the other members' files (`03` T19-P3, P4) | `ceremony/message`, `identity/ceremony_flight` (*new*) |
| a member's ceremony state | `credential::ceremony::Party::decode` | the member's own state file, written by the tool itself | `identity/ceremony_state` (*new*) |
| slot requests and signature shares | `credential::ceremony::{SlotRequest, SignatureShare}::decode` | a signer's `isegoria-ceremony answer` (a participant's request); the participant (each signer's share), whose side is not built | `identity/ceremony_issuance` (*new*) |

The genesis is read from a file, but it counts: one operator assembles it and the others
decode it (`--genesis-digest`) before confirming it, the digest proving no origin (step 2 of
the procedure), so its bytes come from another machine. `--genesis-assemble`'s TOML input
is one operator's, its keys copied from what the others printed; it ends in the same
`Genesis::decode`, and has no target of its own. A relay reads only a request's
first byte (the key id) and returns the gateway's reply unchanged, so it decodes nothing.
The ceremony's procedure TOML has no target: `03` T19-P4's tool takes only messages from other
machines; the procedure is each operator's input, its keys copied from what the others printed, as
`--genesis-assemble`'s, and a procedure differing from the others' changes the opening digest and
the id, so their messages are refused. The ceremony tool's own state file (`State::decode`, with
`Procedure::decode` in it) and a signer's `answered/<slot>` records are its own files, written by
itself; only the `Party` inside the state has a target. The ceremony tool and the test issuer are
neither a trial node nor the client; they are listed because their files cross machines.

Not covered, and why: libp2p's own decoders (Noise, Yamux, multistream-select, gossipsub's and,
with the protocols on, Kademlia's protobuf messages) are a dependency's, reached by every node; part 1 targets none
of them, and they are outside D62's perimeter (`01` D64). The object and shard *replies* (`discovery::checked`, `archive::checked`) are read
only by `discovery::fetch`, `archive::fetch` and archive maintenance, which no trial node
runs; the archive files of the offline tools (`SignedManifest`, `Segment`, the certificates)
are outside the trial (`04` §Archives and holders in the trial). The node's own files are
written by itself (`network/store` covers its log and object store). **Still to cover**: the
hook's application of collected cuts to `AnswerLedger` (`AnswerLedger::apply_in_view` over
hostile replicated entries; its decoders have targets, the application has none); the gateway's
wiring of admission and of `…/read` (the node's admission closure calling `admit`, and in `p2p`
a Binary HTTP path for reads reaching `answer_read::answer`); and `isegoria-client`'s files and
readings (its credential, request and share files, the shares' combination, the read replies);
each once the work in progress builds it.
The targets marked *new* above had their first smoke runs on 2026-11-11, authorized by the owner
(`01` D62, D67), recorded below.

**Runs recorded for this audit** (nightly `cargo-fuzz` 0.13.2, libFuzzer with
AddressSanitizer, 4 cores, 2 GiB RSS limit):

| Target | 15 min (executions) | 3 min on the committed code (executions) | Crashes |
|---|---:|---:|---:|
| `network/ots_verify` (seeded with the two fixture proofs) | 10 426 667 | 2 753 350 | 0 |
| `network/erasure` | 1 603 278 | 352 562 | 0 |
| `network/checkpoint` | 307 814 | 47 438 | 0 |
| `network/merkle` | 12 924 232 | 2 396 733 | 0 |
| `identity/oprf_quorum` | 1 245 236 | 234 811 | 0 |
| `identity/enrollment` (`-max_len=70000`) | 77 016 | 16 198 | 0 |
| `identity/voprf_wire` | 5 529 289 | 1 224 479 | 0 |
| `identity/nullifier_proof` | 91 962 | 18 663 | 0 |
| `network/store` (T13, 2026-09-27; 5 min during the work, 3 min on the committed code) | 309 898 | 110 450 | 0 |
| `protocol/event` (T73, 2026-09-27; 4 min unseeded, then 5 min seeded after the F10 fix) | 2 276 305 | 488 280 | 1 (F10, fixed) |
| `identity/nullifier_proof` (T73: the wire format, 4 min) | — | 17 462 | 0 |
| `protocol/event` (T73 step 2: lifecycle steps, seeded, 5 min) | — | 700 821 | 0 |
| `protocol/event` (T73 step 3: epoch results, seeded, 5 min) | — | 1 264 857 | 0 |
| `protocol/event` (T73 step 3: decoded results also applied, 5 min) | — | 1 657 170 | 0 |
| `network/replica` (T18, 2026-09-27, 5 min) | — | 195 796 | 0 |
| `network/cut` (T74, 2026-09-27; 5 min, then 5 min after the F11 fix) | 5 min to F11 | 66 374 | 1 (F11, fixed) |
| `network/cut` (T74 step 2: members' objects and proposals, 5 min) | — | 40 445 | 0 |

**T89 part 1's runs** (2026-11-10) are short smoke runs, the only kind the owner authorized on
this machine: `-max_total_time=60` per target, `rustc 1.102.0-nightly (ac8310ba6 2026-11-09)`,
`cargo-fuzz` 0.13.2, libFuzzer with AddressSanitizer, one process per target, 2 GiB RSS limit,
from an empty corpus, on a machine shared with other builds. They show no panic on the inputs
they reached and do not stand for a campaign (`01` D62); their counts are not comparable with
the table above.

| Target | Code | Duration | Executions | Crashes |
|---|---|---:|---:|---:|
| `p2p/bind` | `2c1bd3b` | 60 s | 534 600 | 0 |
| `p2p/key_config` | `2c1bd3b` | 60 s | 129 049 | 0 |
| `p2p/client_reply` | `2c1bd3b` | 60 s | 55 477 | 0 |
| `p2p/gateway` | `2c1bd3b` | 60 s | 115 865 | 0 |
| `p2p/serve` | `2c1bd3b` | 60 s | 1 358 840 | 0 |
| `protocol/answer_event` | `2c1bd3b` | 60 s | 7 692 | 0 |
| `protocol/genesis` | `2c1bd3b` | 60 s | 10 825 | 0 |
| `protocol/genesis` (from that run's corpus) | `7cef417` | 60 s | 11 047 | 0 |
| `network/cut` (its decoders changed with T76) | `2c1bd3b` | 60 s | 24 099 | 0 |
| `network/replica` (likewise) | `2c1bd3b` | 60 s | 64 055 | 0 |

Between `2c1bd3b` and `7cef417` only `network::genesis`, `node` and a test module of `identity`
changed, so only `protocol/genesis` reaches changed code. `protocol/answer_event` and
`protocol/genesis` run slowly, each execution verifying a proof or decompressing a key:
libFuzzer's length limit reached only 6 and 33 bytes, so they exercised mostly the splices.

**Increment 3's and T19's runs** (2026-11-11) are smoke runs of the same kind (`01` D67):
`-max_total_time=60` per target, the same toolchain, `cargo-fuzz` and sanitizer, from an empty
corpus, on the combined code of `10` T79's increment 3 and T19's tool. No target crashed.

| Target | Code | Duration | Executions | Crashes |
|---|---|---:|---:|---:|
| `protocol/answer_gateway` | `1e9d2f1` | 60 s | 8 875 | 0 |
| `protocol/answer_read` | `1e9d2f1` | 60 s | 8 217 | 0 |
| `protocol/answer_seal` | `1e9d2f1` | 60 s | 74 880 | 0 |
| `identity/credential_wire` | `1e9d2f1` | 60 s | 12 807 | 0 |
| `identity/ceremony_flight` | `1e9d2f1` | 60 s | 1 353 | 0 |
| `identity/ceremony_state` | `1e9d2f1` | 60 s | 10 754 | 0 |
| `identity/ceremony_issuance` | `1e9d2f1` | 60 s | 3 608 | 0 |
| `ceremony/message` | `1e9d2f1` | 60 s | 241 212 | 0 |

`identity/ceremony_flight` and `identity/ceremony_issuance` run one ceremony at start and a
library flight or proof per execution, so their counts are low; a campaign belongs on the
owner's machine.

`scoring/bridging` was added with T62 (2026-09-24) and has not been run yet: that session
had no nightly toolchain. `scoring/tests/malformed_ratings.rs` covers the same entry points
on every push (§5).

The 15-minute runs predate two final edits — `pok_challenge` made fallible, and the
`merkle` harness's input turned into a named struct — so every target was run again on
the committed code. Peak memory was about 1 GB at most (`ots_verify`, ASan included).
The corpora are not committed; `ots_verify` restarts from the fixtures.

**Is the harness able to find these bugs?** With the pre-scan disabled, `ots_verify`
reached F2 (an out-of-memory `calloc` of ~40 GB) after 29 executions, starting from the
two fixture proofs. The only failures while the targets were written (60-second smoke
runs) were harness bugs, not findings: `erasure` counted a shard as tampered after two
flips of one byte had restored it, and `checkpoint` expected `ingest_with_log` to compare
the very first checkpoint with the log (it trusts it on first use, as specified; later
ones are checked).

## 5. Regression net on every push

The fuzz targets run on demand; CI runs the stable property tests that cover the same
entry points:

- `network/tests/durable_store.rs` (T13): arbitrary bytes after a genuine log prefix and
  after an object header (256 cases each), a crash at every byte of a log, damage in every
  field of every earlier record.
- `network/tests/hostile_input.rs`: arbitrary bytes and edited genuine proofs through
  `verify` (4096 cases each: with the pre-scan disabled, most runs abort); hostile shard
  sets and layouts; recovery exactly when enough authentic shards survive; forged
  checkpoint signatures and indices; Merkle proofs for any index.
- `identity/tests/hostile_input.rs`: arbitrary codice-fiscale strings (realistic, arbitrary
  Unicode, whitespace, and past the RFC 9497 limit) through every oracle; the RFC 9497
  messages decoded from arbitrary bytes.
- `identity::nullifier::proptests::hostile_proof_bytes_never_verify` and the T42
  byte-flip properties in `credential` and `nullifier`.
- T89's decoders: `p2p/tests/tiered_nodes.rs`, `p2p/tests/blind_relays.rs`,
  `network/tests/genesis.rs`, `protocol/tests/answer_review.rs`,
  `protocol/tests/answer_config_encoding.rs` and `protocol/tests/answer_participants.rs`; the
  credential's encodings and the ceremony: `identity`'s `credential` and `credential::ceremony`
  unit tests, `ceremony/tests/formats.rs` and `test-issuer/tests/`.
- One unit test per finding (§3).
- `scoring/tests/malformed_ratings.rs` (T62): a `Ratings` with an out-of-range index, a
  wrong weight count, a non-finite rating, a non-finite or negative weight or a duplicate
  pair returns its `RatingsError` from `fit` and `bridge_scores`; and 192 arbitrary
  `Ratings` (indices past `n`/`m`, NaN and infinite values, negative weights, any weight
  count) either fit or return an error, exactly as `validate` decides — never a panic.

## 6. Left open

- **`scoring` and `protocol`**: `bridging` now validates its input (T62, §2.3, §4). The
  other `scoring` entry points keep shape preconditions on their slices and matrices
  (the table in §2.3), and the `protocol` entry points were not fuzzed: both are T46's,
  whose validated types make the mismatches unrepresentable.
- **Network codecs**: the node's own files are the first (T13, `network/store`, fuzzed).
  The node's events (T73, `protocol/event`) and nullifier proofs (`identity/nullifier_proof`)
  followed, then the replication messages (T18, `network/replica`); §4's T89 list holds
  what a trial node reads and what T79's increments will add. T74 will add wire
  formats for checkpoints, signatures, receipts, shards and OPRF partials; each needs
  a fuzz target of the same kind, and the `credential` sites marked "becomes external"
  above turn into errors when the committee goes remote.
- **Fuzzing in CI**: the targets are not built by CI (they need nightly). A scheduled job
  running each for a few minutes would keep them from rotting.
- **Noticed, not panics**: `Consortium::new` accepted a threshold of 0, which accepts any
  checkpoint with no signature — resolved by T63: it panics unless `1 ≤ t ≤ n` with
  distinct keys (§2.2), and `verify` checks the member-set hash; `ingest_with_log` trusts the first
  checkpoint on first use, so a local log that does not extend it is reported only at the
  next checkpoint: as `LocalLogDiverged` once it is long enough to show the divergence, as
  `LogBehind` before that (T43).
