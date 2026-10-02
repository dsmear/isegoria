# Architecture

How the design in [`docs/`](docs/) maps onto the code. This document is the bridge
between the conceptual specification and the Rust implementation. Read
[`README.md`](README.md) first for the plain-language overview.

## Contents

- [Principles](#principles)
- [Workspace and dependency graph](#workspace-and-dependency-graph)
- [`scoring` — the deterministic engine](#scoring--the-deterministic-engine)
- [`identity` — anonymous enrollment](#identity--anonymous-enrollment)
- [`network` — tamper-evident storage](#network--tamper-evident-storage)
- [`protocol` — lifecycle orchestration](#protocol--lifecycle-orchestration)
- [`p2p` — the libp2p transport](#p2p--the-libp2p-transport)
- [`characterization` — the characterization harness](#characterization--the-characterization-harness)
- [Invariants and where they are enforced](#invariants-and-where-they-are-enforced)
- [Reproducibility](#reproducibility)
- [Testing strategy](#testing-strategy)
- [Plug points: real vs. placeholder](#plug-points-real-vs-placeholder)
- [Future work](#future-work)

## Principles

Four rules shape every crate:

1. **The scoring engine runs offline.** `scoring` has no dependency on `identity`,
   `network`, or any I/O — the compiler enforces it. It is the piece that must be
   independently re-runnable to catch a dishonest signer.
2. **Determinism is a security property, not an optimization.** Given identical
   input, the engine produces identical output, bit-for-bit (pinned toolchain,
   seeded RNGs, fixed iteration order, one pure-Rust implementation of the
   transcendental functions). See [Reproducibility](#reproducibility).
3. **Prefer mature crypto; roll our own only with a high bar.** Heavy primitives
   enter behind traits; production wires them to mature, audited libraries. Bespoke
   cryptography is allowed when it genuinely serves the design (no suitable library,
   or a needed variant such as a threshold scheme built on a vetted single-party one),
   kept small, built on audited building blocks, and pinned down with known-answer and
   property tests. It is a considered exception, not the default.
4. **The docs are the source of truth for the logic.** Code comments are minimal
   and point back to the relevant `docs/` section; they do not restate the maths.
   `scripts/comment_budget.py` enforces the budget (`docs/CLAUDE.md`, "Code style and
   docs") in CI and after every edit made through Claude Code.

## Workspace and dependency graph

```
protocol ──► scoring
        ├──► identity
        └──► network

characterization ──► protocol, scoring, identity   (the characterization harness: a tool, not part of a node)
p2p              ──► network             (the libp2p transport for replication, T18)

scoring   (no internal deps; only rand, rand_chacha)
identity  (sha2, voprf, curve25519-dalek, bbs_plus, schnorr_pok, arkworks,
           oblivious_transfer_protocols, secret_sharing_and_dkg, dock_crypto_utils)
network   (sha2, ed25519-dalek, reed-solomon-erasure, opentimestamps)
p2p       (libp2p 0.56, tokio, futures, async-trait)
```

`scoring` sits at the bottom on purpose. `protocol` is the only crate that composes
the other three.

## `scoring` — the deterministic engine

The mathematical core (`docs/02`). Python scripts in `sim/` supply historical fixtures
and scoped differential oracles; several retain retired models. Current contracts are
specified in `docs/02` and exercised by the corresponding Rust tests. Open differences
between contracts, guarantees and implementation are tracked in
[`docs/15`](docs/15-phase1-review.md).

| Module | Spec | Key items |
|---|---|---|
| `bridging` | §A | `Ratings` (`with_weights`, `with_axis` — the reviewer floor's mask, T39), `BridgingParams`, `Fit`, `fit`, `bridge_scores`, `side_balanced` (sides from the participants: axis and a positive weight, `docs/15` A7), `two_means` (the exact cut, each side at least `side_floor`, D42), `coverage` (the ratings of an item's less-rated side, D42) |
| `irt` | §B.1–B.2, B.4 | `theta_from_anchors`, `kr20` (anchor reliability, D37), `point_biserial`, `fit_2pl_item`, `A_MIN`, `R_PBIS_MIN`, `KR20_MIN` |
| `dif` | §B.3 | `logistic_dif`, `mantel_haenszel` (`EtsClass`), `mixture_dif` (the proxy-θ model, retired from the production path by T54, kept for the fixtures; `MixtureDif::differential` is a diagnostic, D37), `BETA2_MAX`, `MIXTURE_DIF_MAX` |
| `latent` | §B.1, §B.3 (D37, D25, D43) | `latent_dif`, `latent_dif_with`, `LatentParams`, `LatentDif::flags`, `Ability`, `Format`, `Formats`, `BadFormats`, `FLOOR_PRIOR_WEIGHT` — the target model: the anchors inside the likelihood, θ integrated on a grid, classes by BIC, an analytic gradient from the EM artificial data (T54); each column's guessing floor set by its declared format, fitted under a prior (T25); the classes' ability a shared histogram, standardized, estimated with the rest (T82), or held normal (`estimate_shape`, the pilot's screen) |
| `dtf` | §B.7 (D38) | `ClassCurves` (`new`, `with_floors`, `with_ability`, `of` a latent fit, `dtf`), `DTF_MAX` — the unsigned DTF of a set of items at the worst pair of counted classes, over the batch's ability density, the fit's histogram (T55, T82) |
| `validation` | §B.4 | `purify_theta` (iterative purification to a fixed point) |
| `reputation` | §C | `author_score`, `loo_scores`, `mean_score`, `inverse_probability_mean` (D35), `odds_weight` (D33), `Cusum` (D34), `weight_cap`, `brier_skill_score` (sim oracle only), `dasgupta_ghosh` |
| `collusion` | §Anti-collusion | `ResidualHistory`, `coordination_clusters`, `permutation_p_value`, `CoordinationParams` (residual-correlation detection, D39/T56); `sublinear_group_weight`, `discount_weights` (analysis only, D40); `correlation_matrix`, `cluster_by_correlation` (the retired raw rule, kept as reference) |
| `glm` (private) | — | shared maximum-likelihood logistic regression |
| `optim` (private) | §A.5 | in-house L-BFGS + numerical gradient |

Notes on non-obvious choices:

- **`bridge_scores` warm-starts each bootstrap from the full fit.** The objective is
  non-convex (the bilinear term `⟨f_u, f_j⟩`), so independent random inits would let
  some subsamples land in a different minimum and pollute the bootstrap-min with
  optimizer noise rather than sampling variability.
- **L-BFGS is in-house** (`optim`) rather than a dependency, to keep full control
  over floating-point determinism.
- **The mixture detector uses a numerical gradient**, matching SciPy's gradient-free
  L-BFGS in `sim/latent_dif_and_capacity.py`; its δ is initialized non-zero to break
  the class symmetry.

## `identity` — anonymous enrollment

The state authenticates but does not issue (`docs/03`).

| Module | Spec | Key items |
|---|---|---|
| `nym` | §M3 | `Role`, `Nym`, `derive_nym` = `H(secret, role)` (lightweight address) |
| `nullifier` | §M3 | `NullifierProof` (`encode`, `decode`: the wire format v1, canonical, T73), `prove`, `verify` (ZK nullifier bound to the BBS+ credential) |
| `ratelimit` | §Cost of proposing | `rln_token`, `within_quota`, `SlotLedger` |
| `enrollment` | §M1 | `IdentityDocument` (+ `Cie`, `Spid`), `UniquenessOracle` (`VoprfOracle` real + `ReferenceOracle` test-only), `EnrollmentRegistry` |
| `oprf` | §M1 | `ThresholdOprfOracle` (Shamir + DLEQ), `KeyShare`, `PublicShare`, `PartialEval`, `DleqProof` |
| `credential` | §M2 | `Credential`, `Issuer`, `ThresholdIssuer`, `IssuerPublic`, `IssuanceRequest`, `AnonymousCredential` |
| `hash` (private) | — | domain-separated SHA-256 |

**Real:** role pseudonyms (deterministic → not rotatable → no whitewashing; distinct
per role → unlinkable) with, on top, a Semaphore-style **ZK nullifier** (`nullifier`):
`N = x·H_role` plus a proof that binds it, in zero knowledge, to a valid BBS+
credential over the same secret `x` — a bespoke sigma-protocol composition (the BBS+
proof of knowledge sharing its message blinding with the nullifier's Schnorr proof
under one Fiat–Shamir challenge), so no circom/Groth16 stack is needed. Also real:
rate-limiting tokens (reuse collides and is detected), the uniqueness label, and
credential issuance. The label has two real backends: a
single-server **VOPRF** (`VoprfOracle`, RFC 9497 via `voprf` — oblivious and
verifiable) and a real **threshold** t-of-n OPRF (`oprf::ThresholdOprfOracle`) that
closes the single-holder gap — the key is Shamir-shared, each member proves its
partial evaluation with a Chaum–Pedersen **DLEQ**, and any `t` Lagrange-combine to the
label, so `t-1` members cannot compute it and none can brute-force the codice-fiscale
space alone. It is a bespoke DH-OPRF on the vetted `curve25519-dalek` group (a tested
exception, see the crypto-rule note). Credential issuance is a real **BBS+** blind
signature (via `bbs_plus`, BLS12-381): the holder commits to its secret and proves
knowledge of it (`schnorr_pok`), the issuer verifies that proof and blind-signs
`(secret, label)` learning only the label, and the holder unblinds a verifiable
signature. This too has both a single-issuer backend (`Issuer`) and a real **threshold**
t-of-n one (`ThresholdIssuer`): the signing key is Shamir-shared and a signature is
produced by the DKLS-based MPC of `bbs_plus::threshold` (DKG + base OT + a
multiplication phase), so `t-1` members cannot sign; the aggregate is an ordinary BBS+
signature, so the holder's request and unblinding are unchanged.
**Still modeled:** a real distributed key-generation ceremony and network transport for
both threshold committees (here trusted-dealer keygen + an in-process committee running
every protocol message locally); and selective-disclosure *presentation* of the
credential (the `PoKOfSignature` reveal, tied to the M3 nullifier).

## `network` — tamper-evident storage

Integrity without permissionless consensus (`docs/04`).

| Module | Spec | Key items |
|---|---|---|
| `cid` | §Content-addressed storage | `Cid`, `cid` |
| `merkle` | §Merkle tree | `leaf_hash`, `merkle_root`, `merkle_proof`, `verify_proof` |
| `log` | §Signed append-only logs | `TransparencyLog` (hash-chained; `verify` detects tampering; `checkpoint` + `verify_extends` prove consistency/truncation against a signed prior head, T14) |
| `consortium` | §The consortium as backbone | `Member` (ed25519), `Checkpoint` (net-id + member-set bound, T15), `Consortium::new` (`1 ≤ t ≤ n` distinct keys, T63), `Consortium::verify` (t-of-n over its own member set, T63), `verify_excluding` (the beacon's withholders, T37), `CheckpointClient` (monotonic-height, equivocation, T15) |
| `beacon` | §The epoch's beacon (D41) | `BeaconRound` (`open`, `commit`, `close_commits`, `close_deposits`, `reveal`, `finish`), `Member::beacon_commit`, `BeaconCommit`, `BeaconReveal`, `BeaconOutcome` (`value`, `revealed`, `withheld`, `record`), `RoundError` — commit-reveal among the members, in process (T37) |
| `codec` | §Events and replay | `Writer`, `Reader`, `DecodeError` — our encoding: little-endian integers, length-prefixed fields checked before sizing (T73) |
| `store` | §A node's own disk | `DurableLog`, `ObjectStore` (content-addressed), `Recovery`, `StoreError`, `MAX_OBJECT` — append-only files synced before acknowledging; a torn tail cut, other damage refused (T13) |
| `replica` | §Replication between nodes | `WriterSet`, `FeedWriter`, `SignedEntry`, `EntryId`, `Replica` (`insert`, `feed`, `equivocations`, `digest`, `summary`, `have_for`, `want`, `entries_for`), `Equivocation`, `Message`, `DurableReplica` (on disk, T74) — the grow-only set of writers' signed entries and the pull sync's messages (T18) |
| `cut` | §Cuts | `Cut` (`of`, `next`, `digest`, `checkpoint`, `encode`, `decode`), `Mark`, `added`, `CutError`, `MemberObject`, `collect`, `Collected`, `proposer`, `sign_cut`, `should_sign`, `member_objects` — the consortium-signed marks that fix which entries count and their order; the members' objects that sign them and run the beacon (T74) |
| `anchoring` | §Anchoring | `Anchor` trait, `OtsAnchor`, `Receipt`, `AnchorState` |
| `erasure` | §Durability | `encode`, `reconstruct`, `reconstruct_verified` (real Reed–Solomon; per-shard manifest, corrupt-shard authentication before decode, T16) |

**Real:** content addressing, Merkle trees, the hash-chained append-only log,
ed25519 consortium checkpoints, the commit-reveal beacon round (in process), erasure
coding, and the anchoring proof format —
`OtsAnchor` builds, serialises and verifies real **OpenTimestamps** `.ots` proofs (via
`opentimestamps`): `verify` runs the actual OTS walk (`Op::execute` over the step tree)
and checks a Bitcoin attestation against a block Merkle root. **Still modeled** for
anchoring: the live network parts — POSTing to a calendar server and reading block
roots from a Bitcoin node/SPV; here an injected block source stands in and
`OtsAnchor::upgrade` models the calendar's confirm-and-upgrade with one hashing step.
**Replication** of the signed set is real (`replica`, over libp2p in `p2p`, T18). **Not
yet implemented:** the protocol state as a function of the replicated set and its merge
rules is `protocol::ledger` over signed cuts, and the members' cuts and beacon run between
nodes (T74); a silent proposer's replacement is T76, the DHT T75.

## `protocol` — lifecycle orchestration

Composes the three layers into the question lifecycle (`docs/05`). Deterministic
steps are seeded for reproducibility.

| Module | Stage | Key items | Uses |
|---|---|---|---|
| `events` | §Events and replay (`docs/04`) | `NodeEvent` (deposit, reviewer admitted, respondent admitted, lifecycle step, epoch results), `encode`, `decode` (T73) | `network::codec`, `identity::nullifier` |
| `node` | §Events and replay (`docs/04`) | `Node::{open, submit, state}`, `NodeState::{apply, item, results}`, `Outcome`, `NodeError`, `Rejection` — the protocol state rebuilt by replaying the durable log (T73) | `events`, `results`, `deposit`, `review`, `pilot`, `network::store` |
| `ledger` | §Cuts (`docs/04`), PROTO-015 | `Ledger` (`apply`, `state`, `last`, `refused`, `beacon`), `Refusal`, `CutReport`, `LedgerError` — the protocol state from signed cuts over the replicated set, the first of conflicting events winning; the epoch's beacon on cuts (T74) | `node`, `events`, `network::cut`, `network::replica` |
| `results` | §Events and replay (`docs/04`), PRIV-004 | `EpochResults`, `ResultRecord`, `ResultsState`, `ResultsRejected`; `rating_leaf`, `answer_leaf`, `inputs_root`, `inclusion_proof`, `verify_inclusion` — an epoch's engine outputs as one event, bound to the Merkle root of its inputs (T73) | `reputation`, `appeal`, `exposure`, `contested`, `network::merkle` |
| `admission` | INV-9/ID-008 | `admit`, `NullifierSet` (T6); `QuotaLedger` — per-credential proposal quota (T11) | `identity::nullifier`, `identity::ratelimit` |
| `blueprint` | [8]/L2 | `Blueprint`, `quotas`, `coverage_deviation`, `assemble_test` | — |
| `contested` | [7b] (D38) | `ContestedPool` (`record`, `remove`, `dtf`, `draw`, `draw_from_beacon`), `NoBalancedDraw`, `RecordError` — contested facts by the fit that last measured them, drawn into a test only in selections whose DTF bound (the sum of per-fit DTFs) is within `DTF_MAX`; the draw exact and seeded from the beacon (T55) | `randomness`, `scoring::dtf` |
| `deposit` | [2] | `Draft`, `deposit`, `deposit_with_identity` (identity-gated) | `admission`, `identity`, `network::{log,cid}` |
| `exposure` | [9] | `ExposureLedger`, `should_retire`, `Template` (its `id`, D43), `least_exposed_variant` | `network::cid` |
| `randomness` | INV-10 | `Beacon::{from_outcome, seed}` — every draw seeds from the epoch's commit-reveal beacon (D41, T37; checkpoint-derived until then, T8) | `network::beacon` |
| `lottery` | [3] | `admit`, `admit_from_beacon` (beacon-seeded; the deposits read as a set, in content-id order, T37) | `randomness` |
| `review` | [4] | `Reviewer`, `assign_reviewers`, `commit`, `reveal`, `submit_review` (identity-gated), `assign_extra_from_beacon`, `K_EXTRA` (the band's extra panel, T60); `assign_diverse`, `assign_diverse_from_beacon`, `assign_extra_diverse_from_beacon` (at most one member of a coordination cluster per panel, D40/T57) | `admission`, `identity`, `network::cid` |
| `gate` | [5]/[5b] | `GateOutcome`, `bridging_gate`, `supplementary_review` (D26 re-decision, T10/T30/T59), `MIN_COVERAGE` (an item one side never rated goes to review, D42) | `scoring::bridging` |
| `appeal` | [5b] | `AuthorHistory::{record, reputation, covers_stake, file_appeal, settle}`, `appeal_floor`, `STAKE_QUALITY` — the stake as a pseudo-observation inside `C_a` (D27, T61) | `scoring::reputation` |
| `pilot` | [6]/[7] | `stage1_screen`, `stage1_fit` and `stage1_verdicts` (`r_pbis`, then the one-class target model on the items it keeps, the ability held normal: `a`, `\|b\|`, the floor over chance — D25/T25 step 2; each item `Screening::{Pass, Fail, Indeterminate}`, indeterminate for every item of a fit that did not converge — `docs/15` A11), `stage2_dif`; batch/sample gates `screen`, `dif_batch`, `admit_dif_batch`, `admit_anchors` (KR-20 floor, D37/T53), `admit_templates` with `Templates` (one template per batch, D43/T82) (INV-8, T9) | `scoring::irt`, `scoring::dif` |
| `honeypot` | Golden items | `inject`, `reviewer_skill`, `HONEYPOT_RATE` | `scoring::reputation` |
| `exploration` | §C.2 exploration (D35) | `explore`, `explore_from_beacon`, `EXPLORATION_RATE`, `Observation`, `Scored`, `outcome_of`, `record_outcome`, `FalseNegatives` — the beacon's draw of gate rejections piloted for measurement only, their outcomes into the reviewers' tracks at weight `1/ε`, the gate's false-negative rate (T52) | `lifecycle`, `probation`, `randomness`, `scoring::reputation` |
| `governance` | Meta-level | `stratified_sortition` (a function of the candidate set, T72), `change_approved` | — |
| `probation` | Cold start / P2 | `status`, `review_weight`, `effective_review_weight` (of a `ReviewerStanding`: the capped odds weight, D33; the founder seed only with no alarm on record, `docs/15` A6), `SkillTrack` (the mean, the count and the CUSUM; an alarm → probation, a founder's too, D34; `record_observed` at `1/π`, `record_unobserved`, `reference` — the inverse-probability mean over every reviewed item, the detector on the unweighted scores, D35/T52), `FounderSet`, `N_PROBATION` (30, D36) | `identity::nym`, `scoring::reputation` |
| `revalidation` | [8] | `revalidate_pool` (multi-axis), `revalidate_batch_latent` (the gated production entry: items, respondents, a format and a template per column, anchor reliability — T9/T65/T25/T82/T53 — then the target model, T54), `latent_batch` (the admitted fit itself, whose curves the contested pool records, T55), `target_flags` and `target_rechecks` (per item `Recheck`: DIF, none detected, or indeterminate when the fit did not converge, `docs/15` A4), `revalidate_pool_latent`/`latent_flags` (the retired proxy path, fixtures only), `items_to_retire` | `scoring::latent`, `scoring::dif`, `exposure` |
| `lifecycle` | §9.1 | `State`, `Event`, `step`, `deposit`, `K_MIN` — rejects every invalid transition (T12); `Event::Resolve` re-decides the band (T10/T30); `SupplementaryReview` carries the extra round, `AssignExtraReviewers` then commit-reveal, `K_EXTRA_MAX` (T60); `Event::Explore` takes a gate rejection to `Explored` and `Measured`, never the pool (D35, T52); `State::Contested` — a DIF item with a verified source, from `Pilot2` or the pool, re-validated into either pool (D38, T55) | `gate`, `review`, `exposure`, `identity::nym` |
| `orchestrator` | Epoch glue | `bridging_weights`, `weighted_ratings` (prior-epoch `w_u` → the fit, T5), `epoch_weight_cap` (`3 × median` over the weights that count, D33), `run_item`, `ItemVerdicts` (drives the epoch through `step`, T12), `ExtraRound`, `extra_round`, `expanded_ratings` (the band's second round and the re-decision's ratings, T60), `settle_appeal` (the escrow on the pilot's conclusion, T61; any other state hands it back, `docs/15` A11 (f)), `ItemVerdicts::dif` (stage 2's `Recheck`, indeterminate kept pending, `docs/15` A4), `ItemVerdicts::explored` (the beacon's exploration draw walked to `Measured`, T52), `ItemVerdicts::source_verified` (a DIF failure with a verified source is `Contested`, and settles an appeal as promoted, T55), `axis_mask`, `N_MIN_REVIEWS` (the reviewer floor of the axis, T39) | `lifecycle`, `appeal`, `probation`, `scoring::bridging` |
| `panel_scores` | §C.2 band baselines (`docs/16` C5) | `Forecast`, `first_panel_baselines` (the other first panelists), `extra_round_baseline` (the first panel, fixed before the extra round), `item_scores`; `None` where no weight remains, never the reviewer's own forecast or 0 | `scoring::reputation` |

Each module's doc comment names the attack the stage neutralizes (brigading,
information cascades, queue explosion, the true-but-divisive false negative, block
voting).

## `p2p` — the libp2p transport

Replication between nodes over libp2p (`docs/04` §Replication between nodes): `Handle`
(`spawn`, `listen`, `dial`, `publish`, `insert`, `replica`, `peers`), `Config`, `Own` — a
node's task drives a swarm with gossipsub (announcements of new entries) and
request-response (`/isegoria/sync/1`: Summary→Have, Want→Entries). A writer's own feed is
its `network::store` log, signed again on restart; given a directory, the node keeps its
replica on disk (`DurableReplica`, T74). `member`: `MemberRole`, `collected` — a consortium
member node's duties on a timer: commit, reveal, propose cut `k` when `k mod n` is its
index, co-sign (T74). Only `network` is a dependency: the
engine and the protocol state stay free of I/O and of an async runtime (T18). The sync
answers any peer, so whoever connects reads the whole set (T77); a node is started only by
a test (T78), and a member's duties are the cuts and the beacon, not an epoch's work (T79).

## `characterization` — the characterization harness

Not part of a node: nothing depends on it. It runs the seeded simulation studies of
[`docs/13`](docs/13-characterization.md) on the production estimators and gates — in
parallel, resumable after an interruption, every run reproducible from its seed on any
machine — and summarizes them with intervals and the threshold tables T25 reads. T24's
full run, 54,412 runs, is stated in `docs/13` §7 and its tables are in
`verification/reports/t24/`; T25's supplement (`docs/13` §8, `--study t25`), 19,700 runs,
measures the model with the guessing floor and the band's extra round, stated in §8.7 and
tabled in `verification/reports/t25/`.

| Module | Role |
|---|---|
| `grid` | the studies — T24's ten and the supplement's six (`T24`, `SUPPLEMENT`) — their cells (`Cell::key`, `Cell::parse`), replicates, each run's seed and its draws' (`draw_seed`) |
| `generate` | the populations: latent-DIF batches (the paper's `dif_generate`, extended: a floor per column, a skewed ability, templates), the Level A mirror design and its probes for the extra round, the reference fixture |
| `run` | one run: `latent_dif` on the design's declared formats read by `revalidation::target_flags`, the gates recorded as `admitted`; `bridge_scores` read by `gate::bridging_gate`, the extra round drawn by `review::assign_reviewers` and re-decided by `gate::supplementary_review`; `ClassCurves` fitted and true, with their floors |
| `record` | a run's CSV record — a floor study's adds the fitted floors — and the store that appends to it and resumes |
| `runner` | worker threads, progress and ETA, `errors.log` |
| `stats`, `summary` | Wilson and design-effect intervals, quantiles; `summary.md` and the CSV tables |

## Invariants and where they are enforced

The invariants from [`docs/CLAUDE.md`](docs/CLAUDE.md):

| # | Invariant | Where |
|---|---|---|
| 1 | Anonymity is the base; no demographic attributes in the deployment path | Anonymous DIF uses `scoring::latent`; observed-group routines are calibration-only (`docs/02` §B.3) |
| 2 | Quality is never decided by majority vote | `scoring::bridging` + `scoring::latent`; `protocol::gate` has no vote count |
| 3 | No money as stake | Bond is reputation (`protocol::deposit`, `scoring::reputation`) |
| 4 | Two reputation scores, never combined | `scoring::reputation` (`author_score` vs `loo_scores`/`odds_weight`); separate role nyms in `identity::nym` |
| 5 | One role, one deterministic non-rotatable pseudonym | `identity::nym::derive_nym` |
| 6 | The state authenticates, does not issue | `identity::enrollment` (`UniquenessOracle`) separate from `credential::BlindIssuer` |
| 7 | Scoring is deterministic and reproducible | `scoring` (pinned toolchain, seeded RNG); `tests/reproducibility.rs` |
| 8 | Empirical validation happens in batches | `protocol::revalidation::latent_batch` and the batch admission gates; historical single-item tests show low detection in their regime, not universal non-identifiability (`docs/15` A8) |

## Reproducibility

Invariant #7 is load-bearing: reproducible computation is what unmasks a dishonest
signer. Measures:

- Pinned toolchain (`rust-toolchain.toml`) and a release profile with
  `codegen-units = 1` and no fast-math.
- Explicitly seeded RNGs (`rand_chacha::ChaCha8Rng`) with a fixed consumption order.
- In-house L-BFGS with a fixed iteration/summation order.
- The transcendental functions (`exp`, `ln`, `ln_1p`, `cos`, `pow`) come from the
  pure-Rust `libm` crate through `scoring::fmath`, not from the platform's libm, whose
  last bits differ between glibc, musl, Apple and Microsoft (AT-BR-04).
- `crates/scoring/tests/reproducibility.rs` asserts `fit`, `bridge_scores`, and
  `mixture_dif` are **bit-for-bit** identical across runs (`f64::to_bits`), and CI
  checks the golden bits (`golden.rs`) on linux-gnu (dev and release), linux-musl,
  macOS-aarch64 and Windows-MSVC (`.github/workflows/ci.yml`, job `golden`).

Note: bit-for-bit equality holds **within** the Rust engine — across platforms and
build profiles since AT-BR-04 — not between Python and Rust: SciPy and the in-house
optimizer differ. Each Python oracle checks a named model and numerical contract
within tolerance; retired fixture models do not specify the current latent re-check.

## Testing strategy

Nine kinds of test (the per-crate counts change often; `cargo test --workspace` reports them):

1. **Oracle acceptance tests** run the Rust engine on the *same dataset* as the
   Python sims (exported by `sim/export_fixtures.py` into
   `crates/scoring/tests/fixtures/`) and check it reproduces their numbers — e.g.
   bridging recovers the latent axis at |corr| ≈ 0.99; the retired evaluator BSS still
   matches the sim exactly (follows-the-crowd −1.33, expert 0.95), and the difference
   score that replaced it (D33) ranks the same profiles the same way.
2. **Property tests** encode the design guarantees: cross-source duplicate
   enrollment is rejected; a tampered log entry breaks `verify`; a k-of-n checkpoint
   needs k valid signatures; erasure recovers from any k of n; a 500-node cartel's
   influence ≈ 22 independents; a lone biased item is not detected in its historical fixture.
   These scenario checks do not prove general statistical identifiability claims.
   A `proptest` suite (`network/tests/properties.rs`, `protocol/tests/properties.rs`)
   fuzzes these over arbitrary inputs: Merkle inclusion + root sensitivity, erasure
   recovery from any survivor set, log append/verify, blueprint apportionment, lottery,
   sortition. **Model-based** suites (T43: `protocol/tests/{lifecycle,orchestrator}_model.rs`,
   `network/tests/checkpoint_model.rs`) drive the three state machines with random event
   sequences, checking every step against a small reference model and their invariants.
3. **Reproducibility tests** assert determinism (above).
4. **End-to-end integration** (`protocol/tests/end_to_end.rs`) walks the ten civic
   items of the oracle fixtures through all four crates in one epoch and asserts each
   item is stopped at the right stage (ESM by DIF not review; the non-discriminating
   item by the pilot screen; a true-but-divisive item recovered via the appeal). The
   bridging uncertainty band is resolved by the **D26 re-decision**
   (`gate::supplementary_review`, T10/T30): re-run the bridging fit and decide the
   side-balanced score against the plain threshold τ — a bridging decision over the
   latent axis, not a weighted vote — and a polarized item that fails it keeps the
   appeal channel (D26 amendment, T59). Since T60 the re-decision fits the first panel's
   ratings plus a second round of reviewers drawn outside it (`Event::AssignExtraReviewers`,
   `orchestrator::{extra_round, expanded_ratings}`).
   `supplementary_redecision.rs` pins the improvement: the partisan fixture items 08/09
   are **not** passed (the retired weighted-mean tie-break carried them), while a genuine
   near-threshold item is. The √k anti-collusion stays covered at the scoring layer
   (`anti_collusion.rs`) and in the T5 bridging weights.
5. **Adversarial scenarios** (`*/tests/adversarial.rs`) compose mechanisms against
   the threat model: a 400-node cartel is detected and √k-discounted below an honest
   majority (the engine's measure; on the protocol path a cluster is kept apart on
   panels, D40), and a jittered cartel among two honest camps is found on the residuals
   of the bridging fit with no honest pair flagged (`coordination.rs`, D39); a long con
   is caught by the change detector on its per-item scores and
   sent back to probation, and any weight is capped;
   whitewashing fails because the role pseudonym is deterministic and re-enrollment
   is refused.
6. **Golden outputs** (`scoring/tests/golden.rs`): every value `fit`, `bridge_scores`
   and `mixture_dif` return on the fixtures is pinned bit-for-bit in
   `fixtures/golden_bits.txt`, so a silent change fails even inside an oracle
   tolerance; hand-computed values (`hand_computed.rs`, `exact_outcomes.rs`) pin the
   formulas behind the range and ordering checks.
7. **Mutation testing** (`cargo-mutants`, run on demand, not in CI): measures what the
   suite verifies rather than executes. Every surviving mutant is killed or justified
   as equivalent in `docs/11-mutation-testing.md`.
8. **`#[ignore]` guards**, run on demand: `fixture_drift` regenerates the oracle
   fixtures from the Python sims and diffs them against the committed ones (catches
   sim/fixture drift; needs numpy/scipy); `power` is a Monte-Carlo check of the
   historical no-floor sample-size scenario (latent-class DIF at N≈1500 vs 3000),
   not the current guessing-aware power requirement (`docs/02` §B.6).
9. **Characterization** (T24, `docs/13`): the studies run on demand through
   `crates/characterization`; its own tests pin the grid to `docs/13` §4 and §8, the generators
   to the paper's populations (the KR-20 table), the records to their tasks whatever the
   number of workers, the resumption after a torn line, the production verdict
   (`revalidate_batch_latent`) and the summary's statistics on hand-built records.

Run them:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

## Plug points: real vs. placeholder

| Concern | Status | Production backend |
|---|---|---|
| Bridging, IRT, DIF, reputation, anti-collusion | **Real** | — |
| Role pseudonyms; one credential per label; proposal rate limit | **Real** — deterministic role nyms; `IssuanceRegistry` (one credential per label, T11); `QuotaLedger` per-credential proposal quota keyed on the proven id (T11) | cryptographic-grade RLN (ZK `slot < quota`, reuse reveals key) — T20 |
| ZK nullifier (pseudonym ⇐ valid credential) | **Real** (BBS+-bound sigma protocol), wired into the protocol boundary (T6): `admission` verifies it and the deposit/review/respond entry points key on its proven `id`, with an action-context binding (draft cid and epoch) against replay, and a duplicate cid refused before the quota is charged (T64) | External review of the bespoke composition; cryptographic-grade enrollment/quota (T20/T11) |
| Content addressing, Merkle, transparency log, checkpoints, erasure | **Real** | — |
| Uniqueness label | **Real** (single-server VOPRF RFC 9497; **threshold** t-of-n OPRF, Shamir + DLEQ) | Real DKG ceremony + network transport for the committee |
| Credential issuance | **Real** (BBS+ blind; single-issuer **and** threshold t-of-n MPC) | Real DKG ceremony + network transport; selective-disclosure presentation |
| Public-chain anchoring | **Real** (OpenTimestamps proof format + verification) | Live calendar POST + Bitcoin node/SPV block source |
| Gossip transport, replication of the signed set | **Real** — libp2p gossipsub + request-response, a grow-only set with equivocation evidence (T18) | — |
| Protocol state from the replicated set, merge rules; cuts and beacon between nodes | **Real** — signed cuts fix the order, the first of conflicting events wins; members propose in turn, co-sign, commit and reveal on their feeds (T74) | a silent proposer's replacement (T76) |
| DHT | Documented, not implemented (T75) | libp2p Kademlia |
| Who may read the replicated set | **None** — a node serves its whole set to any peer that connects, the drafts under review and the reveals included, against D17 (T77) | the mechanism the owner picks for T77: closed replication, confidential objects, or the voting patterns off the log |
| A node process | **None** — nodes are started by tests (`Handle::spawn`) (T78) | a binary started from a configuration file |
| An epoch run between nodes | **None** — the draws, reviews, engine run and results are driven by tests (`end_to_end.rs::run_epoch`) (T79) | a member's epoch duties, specified first |
| A writer's recovery | **None** — a damaged log does not start; lost files mean an equivocation at the next publish (T80) | the feed rebuilt from the replicas before writing |

Reference implementations are clearly marked and provide **no** security; they exist
to make the pipeline testable end-to-end.

## Future work

[`docs/10`](docs/10-roadmap.md) is the task-status source. Start Phase 1 corrections
from [`docs/15`](docs/15-phase1-review.md); complete parameter procedures in
[`docs/14`](docs/14-parameter-register.md) after the relevant guarantees and model
choices are settled. The target re-check and stage-1 screen already support guessing
floors; 3PL is not merely a proposed future refinement.

For runtime and deployment work, use the plug-point table above and `docs/10` §2–§3.
Persistence (T13), replication (T18) and signed cuts (T74) are implemented; external
review and real-world calibration remain separate gates. Optional research ideas
such as Bayesian Truth Serum are not prerequisites for closing Phase 1.
