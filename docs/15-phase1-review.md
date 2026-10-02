# Phase 1 — review status and correction handoff

| | |
|---|---|
| **Baseline** | `master` at [`64a4c530781e65709b834ec6e0b3dc4f74cbcfd6`](https://github.com/dsmear/isegoria/tree/64a4c530781e65709b834ec6e0b3dc4f74cbcfd6), reviewed on 2026-10-01/02. |
| **Purpose** | Track the independent A–E review's open claims and acceptance criteria without treating either the code or the review as infallible. Stable review IDs are preserved below. |
| **Status** | Opened as documentation alignment: no scoring formula, threshold, protocol transition, fixture or measurement changed at the baseline. Later corrections are recorded in their finding's entry, which separates implementation and verification from design review; no finding is marked fixed before that review. |
| **Evidence** | L: source/report read; D: independent algebra or static deduction; C: Python calculation performed during the review; P: compilation and tests reported passing by the owner. Rust tests were not run by this reviewer: `cargo` was unavailable. P is not a claim that the post-D43 characterization has run. |

The five review answers concern correctness (A), statistics (B), consistency (C),
direction (D), and completion prospects (E). This is their repository handoff, not
a replacement for the dated audits or an external certification under T26.
Line references below are to the **baseline commit**, since documentation edits move
lines. Code symbols identify the implementation to re-check on a newer branch.

## How to read the documentation

- [`CLAUDE.md`](CLAUDE.md) holds the invariants; [`01`](01-decisions.md) records decisions.
  A decision's implementation status does not establish every mathematical claim made
  in its rationale. Design changes need an explicit decision amendment.
- [`02`](02-scoring-engine.md) states the current mechanism and identifies disputed
  interpretations. A mismatch with code remains a finding until resolved; it is not
  made correct by copying the code into the specification.
- [`08`](08-formal-specification.md) contains a dated audit plus subsequent remediation
  entries. Read those entries with this register; a historical RESOLVED label is scoped
  to the repair and evidence recorded there.
- [`10`](10-roadmap.md) owns task status; [`14`](14-parameter-register.md) owns calibration
  procedures. This register adds review findings, not new task IDs or deadlines.
- [`13`](13-characterization.md) and the report READMEs own measurement provenance.
  Pre-D43 DIF/DTF/floor results do not characterize D43. Bridging results have a
  separate provenance. A code sample floor is not a power guarantee.
- [`sim/`](../sim/README.md) contains research prototypes and scoped test oracles.
  The paper is a dated snapshot. Neither is an executable specification of every
  current production-path component.

## Open correctness findings

All entries below remain **open for implementation/design review** unless their entry says
otherwise (A4 and A5 are closed). The evidence column says what was checked, not that a proposed
correction has been validated.

| ID / severity | Finding and baseline source | Evidence and completion criterion |
|---|---|---|
| **A1 — critical for the incentive claim** | The epoch beacon determines assignment and exploration before reports. Domain separation does not hide the exploration outcome. `protocol/src/{randomness.rs:1–37,review.rs:28–41,exploration.rs:12–20}`; paper §7, `065-revisions.tex:207–224`. | L/D/C. Define the information available at reporting and prove IPW properness under that information. Test an adaptive reporting strategy, not only a forecast fixed before the draw. This is a counterexample to the simplified incentive argument, not a demonstrated attack on the full bridging fit. |
| **A2 — high** | Per-fit DTF uses each fit's own distribution/classes; summing it is not by itself a bound for a common target population. Unflagged active items need not contribute zero. `02:560–598`; `scoring/src/dtf.rs:104–154,194–224`; `protocol/src/contested.rs:126–142`. | L/D/C. Specify the target measure, linking assumptions and whole-test contribution, then prove the bound and separate estimation error. A sampling-error margin alone does not close this. |
| **A3 — high** | The histogram's claimed gauge can affect standardized shape. Its penalty and `Q−3` parameter count require justification. `02:318–331`; `scoring/src/latent.rs:205–207,241–262,818–859`. | L/D/C. Specify genuine constraints or regularization, the objective, model dimension and selection criterion consistently. The same two-parameter counting offset across histogram candidates does not alone alter their BIC ordering. |
| **A4 — high** | An unconverged latent fit produces false flags; `revalidate_batch_latent` discards fit status, while a false `emerging_dif` can restore ActivePool. `scoring/src/latent.rs:145–152`; `protocol/src/revalidation.rs:74–90`; `protocol/src/lifecycle.rs:580–590`. | L/D, static path, no observed failure frequency. Preserve an indeterminate outcome through the decision boundary; verify it cannot certify absence of DIF. **Closed — design review approved on commit `fb8a3d4`** ([record](#a4-correction-record)). |
| **A5 — high** | `expanded_ratings` adds a newcomer's row and weight without extending `axis`; the next fit can return `AxisCount`. `protocol/src/orchestrator.rs:205–227`; `scoring/src/bridging.rs:72–86,255–257`. | L/D, static. Verify newcomer → expanded ratings → fit → supplementary verdict, including the eligibility of the new row. The existing newcomer test at `protocol/tests/supplementary_redecision.rs:335–344` stops before the fit. **Closed — design review approved on commit `afc84d0`** ([record](#a5-correction-record)). |
| **A6 — high** | After a CUSUM reset, `status(true, 0)` restores founder weight 1, whereas §C.4 promises weight 0 until new outcomes. `protocol/src/probation.rs:20–36,94–104,152–166`; `02:800–803`. | L/D, static. Decide whether initial founder privilege survives a disciplinary reset; implement and test that explicit policy. Keep the mismatch visible until then. **Confirmed; implemented and verified, design review pending** ([record](#a6-correction-record)). |
| **A7 — medium** | Participation differs between fitting, side averaging and bootstrap: zero-weight axis rows can affect the score; bootstrap can reuse positive weights of off-axis rows. `protocol/src/orchestrator.rs:49–62`; `scoring/src/bridging.rs:261–294,515–568,650–675`. | L/D/C. Define consistent participation semantics for fitting, partition, means, coverage and bootstrap. The off-axis/positive-weight case is an API-contract counterexample, not a demonstrated production mapping. |
| **A8 — medium** | Low power for one biased item is presented as structural non-identifiability. `paper/sections/04-level-b.tex:62–85,138–142`; `08:522–526`. | L/D. Separate distinguishability from the null, parameter identifiability and finite-sample power. Preserve observed simulation failures without turning them into a general impossibility theorem. |
| **A9 — medium** | Exact camp-mean reproduction and positive regularization need a non-degenerate stationary example for the majoritarian-leak proposition. `paper/sections/03-level-a.tex:154–203`. | L/D in the complete, uniformly weighted case. Exhibit compatible hypotheses or state a controlled approximation; no impossibility claim was established for every missingness/weight pattern. |
| **A10 — low** | With no other positive weight, the leave-one-out fallback uses the reviewer's own report, giving zero for every forecast. `scoring/src/reputation.rs:56–85`. | L/D. Scope strict properness to a report-independent baseline; treat the fallback as uninformative or change it through an explicit decision. |
| **C5 — medium** | The implemented author average is not the posterior mean of the Beta–Beta hierarchy formerly stated in §C.1. `02:648–662`; `scoring/src/reputation.rs:14–33`; `paper/sections/05-level-c.tex:8–16`. | L/D/C. Decide between the existing regularized index and a specified inferential model; also define the continuous quality input. The documentation now describes the implemented formula without claiming that posterior derivation. |
| **C6 — medium** | `exp(−age/18)` uses an exponential decay time, not an 18-month half-life. `02:656–662,902`; `scoring/src/reputation.rs:14–29`. | L/D/C. Current behavior is documented as a decay time, with half-life `T ln 2`. A policy change to an 18-month half-life remains a separate decision and code change. |

Paths beginning `scoring/` or `protocol/` in the table are relative to `crates/`;
numbered document shorthand is relative to `docs/`. Paper paths are repository-relative.

### A4 correction record

| | |
|---|---|
| **State** | **Closed — design review approved on commit `fb8a3d4`**, with no blocking issue. Implemented and verified by the implementation agent. |
| **Design review** | Astra read the diff, the contracts and the tests; it did not re-run the Rust tests. The runs are the implementation agent's, recorded under Verification. |
| **Path at the baseline** | Implemented links: `LatentDif::flags` gave `false` for every item of an unconverged selected fit, the value of a converged fit with no item over the cut; `target_flags` and `revalidate_batch_latent` returned that `Vec<bool>` without the status; `Event::Revalidate { emerging_dif: false, .. }` moved `Contested` to `ActivePool`. No production code composes the re-check's result into `Event::Revalidate` (the node replays logged events; the runtime that would emit them is not built), and `ItemHealth::emerging_dif` is fed only by the calibration Variant 1. The defect is an API contract a caller following it inherits, shown by a test, not an observed incident. |
| **Reproduction** | On `afc84d0`: `latent_dif` at the production settings on 30 respondents, 10 anchors and 8 open trial items (seed 0) ends at `MaxIters` with one class; `target_flags` gave eight `false`, and `step(Contested, Revalidate { emerging_dif: false, source_verified })` returned `Ok(ActivePool)` for both source values. |
| **Contract and representation** | `LatentDif::flags` returns `scoring::latent::DifFlags`: `Evaluated(Vec<bool>)` from a converged selected fit (all `false` with one class), `Indeterminate(Convergence)` otherwise. `revalidation::target_rechecks` turns it into one `Recheck { Dif, NoDif, Indeterminate }` per trial item, and `revalidate_batch_latent` returns `Result<Vec<Recheck>, PilotError>`, so a refused batch stays an error. `Event::Revalidate { dif: Recheck, source_verified }`: `Indeterminate` keeps `ActivePool` or `Contested` whatever `source_verified`; `NoDif` and `Dif` move items as before. No retry, scheduler or new pool state. |
| **Verification** | `protocol/tests/indeterminate_recheck.rs`. The unconverged fit reads `Indeterminate`, every item `Indeterminate`; through `target_rechecks` into `step`, `Contested` and `ActivePool` stay where they are for both source values, and `ExposureLimit` still retires them. A converged one-class fit reads `NoDif` and sends both pools to `ActivePool`; a converged two-class verdict whose gaps the test sets gives `Dif`: `Contested` with a verified source, `Retired(EmergingDif)` without. The same small batch through `revalidate_batch_latent` is `NotEnoughRespondents`. The second test fails when the lifecycle, the adapter or `flags` reads non-convergence as no DIF (each variant checked). The fit is real but below the gate's floors: no admitted batch that fails to converge is known (four pathological admitted batches at N = 3,000 converged), and the gates keep their own tests. `characterization/tests/production.rs` checks that the harness and production agree on an admitted batch; it imposes neither convergence nor DIF, so it is no guarantee of a real positive. `lifecycle_replay.rs` pins each reading's byte and refuses 3; the lifecycle model suite generates all three readings. Run by the implementation agent: `cargo test -p protocol` (208 passed), `-p scoring` (175 passed, 2 ignored), `-p characterization` (54 passed); with `calibration`, `contested_facts` (11 passed) and `latent_target_model` (5 passed: every null fit converged, so the stricter assertions hold); fmt, clippy with and without `calibration`, the comment budget. |
| **Effects** | Public API: `LatentDif::flags` and `target_flags` return `DifFlags`; `target_rechecks` and `Recheck` are new; `revalidate_batch_latent` returns `Vec<Recheck>`; `Event::Revalidate`'s field is `dif: Recheck`. Serialization: the field keeps its byte, 0 no DIF and 1 DIF as the boolean was, 2 indeterminate (`04` §Events and replay); the event version is unchanged, earlier logs replay unchanged, and an earlier decoder refuses byte 2. Characterization records are unchanged: an unconverged fit is still recorded with no flag beside `converged` (`run::recorded_flags`). Scoring tests that asserted "no flag" now assert a converged verdict with no flag. |
| **Left open** | (a) Pilot stage 2: a caller composing `Pilot2Batch { passed }` from the re-check must decide what `Indeterminate` means; the lifecycle has no indeterminate pilot outcome, outside this correction. (b) `ItemHealth::emerging_dif` stays a boolean; `false` is what an indeterminate reading means for retirement (no DIF retirement), but it is not typed. (c) The retired proxy path (`latent_flags`, `revalidate_pool_latent`, fixtures only) still gives `false` for an unconverged fit. (d) `status` is the selected candidate's: a converged one-class fit reads `Evaluated` even when every mixture candidate failed to converge, a search question of B1–B3. All four remain open after A4's closure. |

### A6 correction record

| | |
|---|---|
| **State** | **Confirmed.** Implemented and verified by the implementation agent; **design review pending** (Astra). A6 stays open until that review. |
| **Path at the baseline** | Implemented links: an alarm in `SkillTrack::record_observed` restarts the stretch and keeps the alarm count; `probation::status(true, 0)` is `Founder`; `SkillTrack::{status, weight, standing}` pass only `is_founder` and the stretch's counts, so `bridging_weights`, `weighted_ratings`, `epoch_weight_cap` and `expanded_ratings` gave an alarmed founder weight 1 and counted it in the cap. No production code builds standings from tracks yet (the epoch driver is not built; tests compose `SkillTrack::standing` with the orchestrator): a contract defect shown by a test, not an observed incident. Tracks are rebuilt by replaying epoch results (`results.rs`), so the alarm count is replay state, not a stored field. |
| **Reproduction** | On `fb8a3d4`: 30 scores of 0.5, then 0.2, then −1.0 make the detector fire (`Alarm { count: 1, scored: 31 }`); the founder's track then read `Founder`, weight 1, bridging weight `[1.0]`, epoch cap 3 (counted at 1). |
| **Policy** | Decided for this correction: the founder weight serves the bootstrap. A founder with no alarm keeps today's path; after any alarm founders and non-founders are on probation at weight 0 until `N_PROBATION` new observed outcomes, then at the current skill's odds weight with shrinkage and cap, never at the founder seed. Earlier outcomes and reviews, and unobserved reviews, do not shorten it. Thresholds, the CUSUM, its reference, IPW and the skill formula are unchanged. Recorded in `01` D34, `02` §C.4, `05` §Cold start, `08` §9.2. |
| **Representation** | `ReviewerStanding` carries `alarms`, the count the track already keeps (`SkillTrack::standing` fills it); the founder weight needs `is_founder && alarms == 0`, so the membership `is_founder`, which `axis_mask` reads, is unchanged. `probation::status` and `effective_review_weight` take the standing, so no caller can weigh a reviewer without its alarm history; `SkillTrack::{status, weight}` go through `standing`, and `bridging_weights`, `epoch_weight_cap` and `expanded_ratings` through the same two functions. No new flag or counter; no default for `alarms`. |
| **Verification** | `protocol/tests/founder_reset.rs`, every alarm the detector's own on a deterministic sequence: a founder seeds at 1 and then weighs its capped skill; after an alarm founder and non-founder are at 0; still 0 after 29 new outcomes, at 30 the skill weight (cap 3 binding on a skill of 0.5; 1.175 uncapped on 0.02); 60 unobserved reviews do not end the probation; a second alarm sends the recovered founder back to 0; alarm → standing → `bridging_weights`, `weighted_ratings` and an extra-round row of `expanded_ratings` give the track's own weights `[0, 0, 3, 1]`; the cap leaves an alarmed founder out (∞ alone, `3 × w` beside an established reviewer) and `axis_mask` keeps it on the axis. The tests fail when the status ignores alarms, the standing drops them, the cap counts an alarmed founder or the bridging weight rebuilds the seed (each checked). Run by the implementation agent: `cargo test -p protocol` (215 passed), `-p scoring` (175 passed, 2 ignored), the eight affected protocol test files with `calibration` (74 passed); fmt, clippy with and without `calibration`, the comment budget. |
| **Effects** | Public API: `ReviewerStanding` has a field `alarms` (literal constructions must state it); `probation::status(&ReviewerStanding)` and `probation::effective_review_weight(&ReviewerStanding, w_max)` replace the boolean forms. No persistence or replay format changes: alarms are rebuilt by replaying the scored outcomes. |
| **Left open** | An alarmed founder stays on the axis at weight 0, and an alarmed non-founder, whose stretch restarts its review count, leaves the axis until `n_min` new reviews: both are A7's participation semantics, unchanged here. |

### A5 correction record

| | |
|---|---|
| **State** | **Closed — design review approved on commit `afc84d0`**, with no blocking issue. Implemented and verified by the implementation agent. |
| **Design review** | Astra read the diff, the contracts and the tests and recomputed the weight against the cap; it did not re-run the Rust tests. The runs are the implementation agent's, recorded under Verification. Non-blocking, not prerequisites and not implemented here: an explicit `w_max` contract on `expanded_ratings` and a test with a binding cap. |
| **Reproduction** | On `d0a0808` (code identical to the baseline): the band item of `supplementary_redecision.rs` (200 reviewers, a first panel of nine at `τ + 0.02`) with an extra round of four reviewers who have no row in the epoch's ratings. `gate::supplementary_review` on the output of `expanded_ratings` returned `Err(AxisCount { expected: 204, found: 200 })`, and the re-decision closure inside `run_item` failed with the same error: `expanded_ratings` extended `weights` but not `axis`, which `Ratings::validate` refuses. |
| **Eligibility policy** | Taken from the existing contracts, not chosen anew. Axis membership belongs to the reviewer's standing — founder, or at least `n_min` reviews on record (`02` §A.4, `orchestrator::axis_mask`) — and the weight is its review weight (`bridging_weights`, D33/D36); having no row this epoch changes neither. `expanded_ratings` now takes the newcomer's `ReviewerStanding` and the epoch's `w_max` in place of a bare weight, and builds the row with the two functions `weighted_ratings` uses. A caller-supplied weight plus axis flag was not adopted: it admits pairs no standing produces. |
| **Verification** | `supplementary_redecision.rs::an_extra_reviewer_without_a_row_is_re_decided_on_its_standing`. Rows for a probationer, a reviewer past the floor but in probation, a founder and an established reviewer equal those `weighted_ratings` builds (`axis = [false, true, true, true]`). Through `run_item`, four established reviewers without rows reject the item when they disapprove and pass it when they approve; four probationers leave it in the pool either way, with `S_j` bit-identical. The test fails on the defect (`AxisCount` at the fit), with new rows forced off the axis (the established reviewers' disapproval no longer rejects) and forced on (the axis check). Run by the implementation agent: `cargo test -p protocol` (204 passed), `cargo test -p scoring` (175 passed, 2 ignored), fmt, clippy with and without `calibration`, the comment budget. |
| **Left open** | A reviewer past the floor but in probation sits on the axis at weight 0 and enters the side averages without its rating counting: A7's participation semantics, unchanged here. An extra round drawn only from zero-weight reviewers adds no weighted evidence, so the re-decision rests on the first panel's ratings alone against the plain `τ`, the effect PROTO-008 described; `review::assign_extra_from_beacon` does not consider weight. Whether the extra draw should account for weight is a distinct design question. Both remain open after A5's closure. |

### Small reproductions to preserve during correction

- **A1:** use the test's simplified gate, `q=0.4`, baseline `0.7`, gate `0.5`,
  exploration `0.05`. Expected difference score is `(b−q)²−(p−q)²`.
  Truthful reporting gives `0.09`; reporting `0.4` on explored slots and `0.5`
  otherwise gives `0.05×(0.09/0.05)+0.95×0.08=0.166`.
  Source: `crates/scoring/tests/exploration_weights.rs:8–30,37–75` (L/D/C).
- **A2:** equal classes with normal unit ability, `a=1.25`, `c=0.2`,
  difficulties `−0.45,+0.45`: their gap `0.9` is below the flag cut `1.0`,
  while numerical integration gives unsigned DTF `0.1698619472`, above `0.10`.
  This refutes the zero-contribution claim even within one fit (D/C).
- **A3:** symmetric weights `(0.25,0.5,0.25)` and `(0.125,0.75,0.125)` on
  nodes `−1,0,1` both standardize to mean 0 and variance 1, but have fourth
  moments 2 and 4. Standardization does not remove every shape change (D/C).
- **C5:** for `q=0.8`, age 0, prior `Beta(2,3)`, and hierarchy parameter
  `κ=10`, integrating the stated hierarchy gives posterior mean `0.6682708888`;
  the implemented average is `(2+0.8)/6=0.4666666667`. `κ=10` is a
  counterexample value, not an implemented project parameter (D/C).

## Statistical and operational work still open

| Review IDs | Question and source | Completion evidence needed |
|---|---|---|
| **B1–B3** | Useful identification of `c`, sensitivity to its prior, anchor invariance, BIC under misspecification, and selection after penalized fitting. `02` §B.1/§B.3; `scoring/src/latent.rs:13–15,806–859,955–1019`. | Decision-level sensitivity, candidate/search stability and comparison with simpler alternatives and relevant misspecified populations. Do not equate fitted classes with real social groups. |
| **B4–B5, C3** | Current sample floors are not power guarantees; screening and KR-20 can reject informative batches or retain guessable items. `13` §8.7.1/§8.7.5/§8.8; `protocol/src/pilot.rs:284–318`. | Post-change results by format, gate admission, convergence, conditional and joint error rates, and indeterminate outcomes. The stage-1 filter has a stated purpose consistent with its information at that sample size. |
| **B6, D5** | Exact side splitting is not validation of the social meaning of the sides or confidence coverage; panel size, uncertainty band and extra round interact. `14` entries `d`, `k`; `13` §8.7.4. | Gate error/cost measurements for the declared domain; a second axis requires a score definition, not only a different dimension setting. |
| **B7, D1, D6, E** | T83 has no chosen operating point; model changes can repeatedly invalidate calibration. `10` T83; `14` “How to read an entry”; `13` §8.8. | Error and resource criteria fixed before final threshold selection; a frozen candidate and confirmation beyond the cases used to choose it. Phase 1 may retain provisional values with defined T27 procedures. |
| **D2, D4** | Respondent burden and current runtime need their own budgets. `02` §B.6; `characterization/src/generate.rs:82–89,116–143`; `runner.rs:117–119`. | Count anchors and unsuccessful batches; record latency and hardware for the actual candidate. `fit_seconds` is mean harness-run elapsed time, not an isolated optimizer benchmark. |
| **D3** | Contested selection enumerates subsets within each fit. `protocol/src/contested.rs:75–104,148–169,212–234`. | Define a sustainable maximum domain or revise selection while preserving its specified distribution. With `M` members and at most `n` slots the current traversal visits `Σ(r=0..min(M,n)) binom(M,r)` subsets (D/C). |

## Documentation disposition

| Review IDs | What this documentation pass does | What it does not close |
|---|---|---|
| **C1** | Identifies current specifications, historical simulations and scoped oracles. | It does not port retired simulations to the current model. |
| **C2** | Labels report versions and removes promises that every later model reproduces old DIF records. | It does not replace historical measurements or claim a post-D43 full run. |
| **C3** | Replaces conflicting sample/throughput promises with current code floors and measurement provenance. | It does not choose new sample sizes or approve the present gates. |
| **C4** | Points disputed guarantee closures to A1–A7 above. | It does not repair implementation or erase the audit's historical evidence. |
| **C5–C6** | Describes the author formula and actual time-decay convention accurately. | It does not choose a new model, continuous quality definition or decay policy. |
| **C7** | Names the current and retired entry points in the architecture and handoff. | Stale Rust doc comments in `revalidation.rs:22–25` and `pilot.rs:356–358` remain for the code pass; no `.rs` files are changed here. |
| **C8** | States T81's ten-file scope explicitly. | T81 stays complete in that scope; it is not a proof of every composed guarantee. |

## Working agreement for implementation and design review

1. The implementation agent checks each finding against its branch and records
   **confirmed / partly confirmed / rejected / unresolved**, with evidence. Do not
   close a finding merely because an existing test passes, or implement a review
   suggestion before checking its premises.
2. Work in small coherent changes. Reproduce a code defect with a failing test,
   repair it, and test the full affected decision path. For a mathematical claim,
   supply a derivation or counterexample as well as any numerical regression.
3. The design reviewer checks the property, assumptions, patch and documentation.
   Routine fixes proceed under the owner's correction mandate. Escalate only choices
   that change an invariant, a substantive guarantee, accepted errors or resource
   commitments; present a concrete recommendation and consequences together.
4. Keep implemented behavior, intended behavior and open mismatch distinguishable.
   Update the relevant sections and this register with each accepted correction.
   Preserve task IDs, historical run records and the scope of completed T81 work.
5. Settle changes to the estimator before final calibration. Use targeted checks and
   the smoke grid during development. Full characterization remains an explicit
   owner-run/owner-authorized campaign, with its candidate commit recorded.
6. A Phase 1 completion claim requires proportionate evidence for every critical
   claim (`08` §16.1), plus the stated procedures for provisional parameters.
   T26 must review the final candidate; T27 remains the empirical pilot dependency.

**First correction handoff:** verify A1–A3's guarantee premises and A4–A7's code paths.
A5, the local input-contract defect, was the first bounded implementation
([correction record](#a5-correction-record)); it and A4 ([record](#a4-correction-record))
are closed after design review. A6's correction awaits design review
([record](#a6-correction-record)). T83 is needed before final calibration, not before
investigating these defects.
