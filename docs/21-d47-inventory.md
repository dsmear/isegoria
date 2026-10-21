# 21 — D47: inventory of the affected material

**Historical inventory at D47.** The classifications and path map below record the
reorganization, not pending implementation instructions. D49–D53 subsequently defined
the separate answer-review path in [05](05-question-lifecycle.md#the-answer-review-01-d49d52).
Current task and finding states belong in [10](10-roadmap.md) and [15](15-phase1-review.md).

| | |
|---|---|
| **Purpose** | What [`01` D47](01-decisions.md) affects in the repository, file by file: what is kept, what the next implementation must adapt, what is archived and why, and what stays in place as a dependency although it belongs to the previous design. |
| **Baseline** | `docs/phase1-review-alignment` at `38780af`, the last tip Astra verified. The moves are in `c158670` (renames only) and the commit that added this inventory. |
| **Method** | Each file was classified from its content and its consumers, not from its directory: the workspace manifests, CI (`.github/workflows/ci.yml`), the comment budget, Rust `use` and `include_bytes!` paths, Python imports and file paths, and Markdown links. |
| **Archive** | [`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md): its index, path map and reproduction notes. |
| **Scope** | Classification only. It decides no formula, threshold or policy, and does not say how the new mechanism will use what is kept. |

The *previous design* below is the one D47 sets aside as the reviewers' main contribution: a
reviewer reveals a probability that the item passes the pilot; the same value is the bridging
rating and a forecast scored against pilot outcomes, golden items and explored rejections, which
sets the reviewer's weight.

## 1. Kept and reusable

Valid as they are under D47. Kept documents are realigned where they describe the reviewers.

| Material | Why it is kept |
|---|---|
| `README.md`, `ARCHITECTURE.md`, `docs/README.md`, `docs/CLAUDE.md`, `docs/00`, `01`, `02`, `05`, `06`, `07`, `10`, `13`, `14`, `15`, `99` | The active specification and registers; realigned to D47, with the previous design's sections marked as current code, not intended design. |
| `docs/08` | A dated audit with remediation entries; its historical entries keep their scope, and a reading note points to D47. |
| `docs/03`, `docs/04`, `docs/11`, `docs/12`, `docs/18` | Not about the reviewers' contribution: identity, the network (Phase 2), the mutation-testing runs and the panic audit (historical runs keep their scope), A3's closed correction of the engine's ability histogram. |
| `docs/phase1-handoff.md` | Rewritten for D47; the previous note is archived. |
| `scoring::{irt, dif, latent, dtf, glm, optim, fmath, validation}` | Level B's engine; D47 leaves its relation to the answers open, and its contracts stand. |
| `scoring::bridging` | The fit reads any rating in `[0, 1]`; what it reads under D47 is open (D47, Level A). |
| `scoring::collusion` | D39's detector on residuals; how it coexists with D47 is open, its code is not affected. |
| `scoring::reputation::{AuthorPrior, author_score, author_score_closings, proposal_rate}` | The author score `C` (C5, C6); D47 changes the reviewers, not the authors. |
| `identity`, `network`, `p2p` | Not affected; no Phase 2 work in this reorganization. |
| `protocol::{admission, appeal, author_time, authorship, blueprint, contested, exposure, governance, lottery, offline_log, pilot, randomness, revalidation}` | Their contracts do not read the reviewer's forecast. The appeal's trigger depends on the bridging gate, whose input under D47 is open. |
| `protocol::deposit` | The deposit stays; its mandatory `primary_source` is one of D47's open questions, and the item bytes carry no structured key yet (`10` T68's structured draft). |
| `characterization` (T24/T25 harness: `generate`, `grid`, `run`, `record`, `runner`, `stats`, `summary`, `main.rs` and their tests) | Characterizes the current detectors and gates (`docs/13`). |
| `sim/{bridging_irt_dif, latent_dif_and_capacity, export_fixtures, oracle_bridging, oracle_mixture, latent_shape_dimension, d3_selection_counts, d3_domain_limits}.py`, `sim/requirements.txt` | Fixtures, differential oracles and the A3 and D3 calculations of current code; CI runs `export_fixtures.py` and the two oracles. |
| `verification/reports/t24/`, `t25/` | Historical characterization of the current detectors and gates; pre-D43 DIF measurements stay historical evidence, not validation. |
| Every test not listed in §2 | It checks code D47 does not affect. |

## 2. To adapt in the next implementation

Current code that implements the previous design. It keeps its tests and its documented
behaviour until the implementation of D47's mechanism adapts, replaces or retires it; nothing
here is changed by this reorganization.

| Code | Role in the previous design | What D47 changes |
|---|---|---|
| `protocol::review` (`commit`, `reveal`, `submit_review`) | Commit-reveal of a probability `prob ∈ [0, 1]` | Reviewers answer the item: the committed value |
| `protocol::lifecycle` (`Event::{Commit, Reveal}`, `Event::Explore`, `State::{Explored, Measured}`), `protocol::events` (the `Reveal` encoding) | The reveal's payload; exploration's path to the pilot | The payload; exploration is suspended |
| `protocol::orchestrator` (`Judgment`, `review_round`, `extra_round`, `expanded_ratings`, `weighted_ratings`, `bridging_weights`, `epoch_weight_cap`, `ReviewerStanding`, `affine_first_ratings`, `affine_supplementary_ratings`) | Reveals become ratings, weighted by forecast skill | Level A's input and the weights are open |
| `protocol::gate` | Reads the bridging scores of those ratings | Follows the Level A decision |
| `protocol::probation` (`SkillTrack`, `status`, `effective_review_weight`, `FounderSet`, `N_PROBATION`) | Probation, CUSUM and weight on forecast scores (D34, D36) | Reputation update still to define |
| `protocol::honeypot::reviewer_skills` (`inject` is kept) | Scores forecasts on golden items | Golden items' role is open |
| `protocol::exploration` | Explored rejections and inverse-probability scoring (D35) | Suspended |
| `protocol::panel_scores` | Band items' forecast baselines; no production caller | Suspended |
| `protocol::cohort_scores`, `protocol::cohort_snapshot` | B-b's scorer and its snapshot; no production caller | Suspended; the snapshot is also a C6 dependency (§4) |
| `protocol::affine_weights` | A12's frozen weights; no operational adoption | Suspended |
| `protocol::results::ResultRecord::{ReviewerScore, ReviewerUnobserved}` and their application in `node` and `ledger` | Per-reviewer forecast scores in an epoch's results | The reliability record still to define |
| `scoring::reputation::{EvaluatorParams, difference_score, loo_baseline, loo_scores, mean_score, inverse_probability_mean, odds_weight, brier_skill_score, crowd_baseline, base_rate_baseline, Cusum, CusumParams, weight_cap, capped_weight, dasgupta_ghosh}` | The forecast score, its baselines, weights, cap and detector; peer prediction (§C.3) | Reliability and its weight still to define |

**Tests whose subject is the forecast path**, to adapt or retire with its code: in `scoring`,
`evaluator_score.rs`, `exploration_weights.rs`, and the evaluator parts of `level_c.rs`,
`hand_computed.rs`, `properties.rs` and `adversarial.rs` (the author and anti-collusion parts are
kept); in `protocol`, `a1_exploration_information.rs`, `absent_baseline.rs`, `affine_weights.rs`,
`bridging_participation.rs`, `change_detector.rs`, `cohort_scores.rs`, `cohort_snapshot.rs`,
`exploration.rs`, `founder_reset.rs`, `panel_scores.rs`, `s2_synthetic_check.rs` and `tests/s2/`.

**Tests that build reviews from a probability** but check something else — the lifecycle, the
orchestrator, the gate and band, replay, the pilot's explored path, the author's records — follow
the reveal's new payload when it changes: `lifecycle.rs`, `lifecycle_model.rs`,
`lifecycle_replay.rs`, `orchestrator.rs`, `orchestrator_driver.rs`, `orchestrator_model.rs`,
`end_to_end.rs`, `inv12_commit_binding.rs`, `supplementary_redecision.rs`, `reviewer_floor.rs`,
`panel_diversification.rs`, `results_replay.rs`, `indeterminate_screen.rs`,
`indeterminate_pilot2.rs`, `appeal_stake.rs`, `contested_facts.rs`, `author_inputs.rs`,
`author_records.rs`, `author_time_replay.rs`.

**D48 (owner, 2026-10-21)** settles part of the right-hand column for the first implementation
([`01` D48](01-decisions.md)): every reviewer weighs the same, so the forecast weights, probation
and cap have no role in it; ordinary reviews update no reviewer reputation, and the reliability
record, golden items' and peer prediction's role in evaluating reviewers are deferred with that
evaluation; the committed value is an answer, which may be that no alternative is correct. What
happens to Level A's input, the gate and the draw follows the previous filters' role, still to
decide. The code and its tests are unchanged.

**D49–D52 (owner, 2026-10-21)** approve the contract of the *answer review*, implemented as `10`
T87 (`protocol::{answer, answer_rule, answer_author, answer_path, answer_events, answer_node}`,
`scoring::reputation::author_score_weighted`, `tests/answer_review.rs`) beside the code above
rather than by changing it: a uniform draw of 12 from the pool, an
answer committed and revealed, an outcome by consensus, the author's reputation with D50's
inputs and D51's quotas. In that path bridging, the gate, the pilot, IRT, DIF, the source check,
the forecast weights, probation, golden items and exploration have no role (D48, D52); the code in
this section keeps its tests and behaviour for the existing path, and its adaptation or
retirement there is not decided.

**Specification sections** describing the previous design as current behaviour, to be replaced
when D47's mechanism is specified: `02` §§C.2–C.4, `05` [4], §Golden items and §Cold start, `14`
Level C, `08`'s AT-REP rows, `99` Part 4.

## 3. Archived

Moved to [`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md), mirroring
the original paths. Code comments that cite `docs/16`, `docs/17`, `docs/19` or `docs/20` resolve
through redirect pages left at those paths.

| Original path | Destination | Motivation | Dependencies |
|---|---|---|---|
| `docs/16-a1-incentive-design.md` | `archive/phase1-forecast-pilot/docs/16-a1-incentive-design.md` | A1's exploration and incentive analysis and A12's guarantee and closure: the forecast score's dossier | Documents current code kept in §2 (`exploration`, `panel_scores`, `affine_weights`, `scoring::reputation`) and its tests; cited by `01` D33/D35, `02` §C.2, `08`, `10`, `14`, `15`; the archived A12 tools |
| `docs/17-a1-pilot-batch-design.md` | `archive/phase1-forecast-pilot/docs/17-a1-pilot-batch-design.md` | A1's pilot batches, B-b, O1–O3 and A1's conditional closure: the forecast's target and observation design | `cohort_scores`, `cohort_snapshot`, `panel_scores` and their tests; `a1_batch_composition.rs`; A11's record in `15`; cited by the paper's `065-revisions.tex` |
| `docs/19-a2-dtf-composition-design.md` | `archive/phase1-forecast-pilot/docs/19-a2-dtf-composition-design.md` | A2's dossier: the diagnosis, contract, proofs and studies of the previous design's whole-test requirement. **A2 is not resolved** (`15` A2) | `scoring::dtf`, `scoring::latent::latent_dif_candidates`, `protocol::contested` and their tests `dtf_composition.rs`, `a2_dtf_composition.rs`; the A2 harness in `characterization` (§4); the archived A2 tools and results; cited by `01` D44–D47, `15` |
| `docs/20-s2-synthetic-check.md` | `archive/phase1-forecast-pilot/docs/20-s2-synthetic-check.md` | The S2 check of B-b's scorer on synthetic events | Its test code stays in `crates/protocol/tests/` (§4); `17` §§8–14 |
| `docs/phase1-handoff.md` | `archive/phase1-forecast-pilot/docs/phase1-handoff.md` | The working note of the previous design's correction work, up to `38780af` | None; a new note replaces it |
| `sim/a12_offline.py`, `sim/test_a12_offline.py`, `sim/fixtures/a12_complete.json`, `sim/fixtures/a12_incomplete.json` | `archive/phase1-forecast-pilot/sim/…` | A12's exact offline reference | Imported by the exporter and its test, moved with it |
| `sim/export_a12_affine.py`, `sim/test_export_a12_affine.py`, `sim/fixtures/a12_affine_nyms.json` | `archive/phase1-forecast-pilot/sim/…` | A12's frozen binary64 exporter | Generated `crates/protocol/tests/fixtures/a12_affine/`, which stays with its Rust consumer (§4); the test's path to those tables updated |
| `sim/a12_weight_counterexample.py` | `archive/phase1-forecast-pilot/sim/…` | A12's finite counterexample (`16` §7.1) | None |
| `sim/dtf_composition.py` | `archive/phase1-forecast-pilot/sim/…` | A2's diagnosis figures (`19` §§1–4; `15`'s A2 reproduction) | None at run time |
| `sim/a2_information_check.py`, `sim/a2_information_check.json`, `sim/test_a2_information_check.py` | `archive/phase1-forecast-pilot/sim/…` | A2's information check and its committed result (`19` §§13.5, 13.8) | None at run time |
| `sim/a2_probe.py`, `sim/a2_probe_reference.json`, `sim/test_a2_probe.py`, `sim/a2_tradeoff.py`, `sim/test_a2_tradeoff.py`, `sim/a2_size.py`, `sim/test_a2_size.py` | `archive/phase1-forecast-pilot/sim/…` | A2's oracle and readings of the probe's runs (`19` §§13.11–13.31) | Read the run directories of the A2 harness (§4); `test_a2_probe.py` reads the archived `a2-candidates-2e2572c` report, which keeps its relative path |
| `verification/reports/a2-{probe-9511ce1, candidates-2e2572c, dtf-check-821f52d, ind-cal-7bbfddb, ind-val-185ec93, tradeoff-ccb8f05, size-b8c9dce, ind-cal6-2a01b81, ind-val6-6d57cac}/` | `archive/phase1-forecast-pilot/verification/reports/…` | The nine A2 runs' records, checks and readings | Their candidate commits; the owner's untracked run directories; the archived oracle |
| The A2 and A12 sections of `sim/README.md` | `archive/phase1-forecast-pilot/sim/README.md` | Documentation of the scripts above, moved verbatim | `sim/README.md` keeps the rest and a pointer |

## 4. Still needed as dependencies, kept in place

Tied to the previous design, but a move would change behaviour, break a current test or CI, or
split a mixed document. These are **residual separations**: the separation is not complete.

| Material | Why it stays |
|---|---|
| The A2 probe harness in `crates/characterization`: `src/probe.rs`, `src/probe_jobs.rs`, `src/probe_calibration.rs`, `src/bin/a2-probe.rs`, `tests/{a2_probe, a2_probe_jobs, a2_calibration, latent_candidates}.rs`, the `libc` dependency | Separable at compile time, but not without a functional change: `a2-probe run` records its candidate from `HEAD:crates` and refuses uncommitted changes under `crates/`. Outside `crates/`, that guard would stop covering the harness's own code. A move needs the guard changed first, by a decision of its own. |
| `scoring::latent::latent_dif_candidates` | Written for A2's candidates phase and used only by the harness and its test, but part of the engine crate's API. |
| `protocol::cohort_snapshot` | B-b's snapshot, but `crates/protocol/tests/author_records.rs` checks C6's author records through it: part of C6's closure evidence (`825cabc`). |
| `crates/protocol/tests/fixtures/a12_affine/` | Golden tables made by the archived exporter, read by `crates/protocol/tests/affine_weights.rs`, which tests current code. |
| `crates/protocol/tests/{s2_synthetic_check.rs, s2/, a1_batch_composition.rs, a1_exploration_information.rs, absent_baseline.rs, a2_dtf_composition.rs}`, `crates/scoring/tests/dtf_composition.rs` | Their dossiers are archived, but they exercise current code (ledger, lifecycle, gates, pilot, DTF) as integration tests of their crates; they stay with it (§2 for those on the forecast path). |
| `paper/` | Mixed: it analyses Levels A and B, which the code still implements, and the forecast score (§5, Section 7's D33–D36). `paper/scripts/common.py` is imported by `sim/oracle_bridging.py` and `sim/oracle_mixture.py`, which CI runs through `crates/scoring/tests/differential_oracle.rs`; its scripts write the data its LaTeX reads. Moving it would make an active test depend on the archive or split its reproducibility. |
| `docs/16`, `docs/17`, `docs/19`, `docs/20` | Redirect pages to the archive, kept for the code comments that cite those numbers. |

## 5. Not settled here

D48 settled part of D47's open points for the first implementation, and D49–D52 the rest for the
answer review: the group, quorum and absences, thresholds and outcomes, "don't know", the source
at deposit (D49), the author's reputation (D50) and quotas (D51), the previous filters' role, A2
and Phase 1's completion (D52). What D49–D52 leave open is in the handoff (`phase1-handoff.md`
§4). This inventory says where the material is; whether the existing path's code is later
adapted or retired is not decided here.
