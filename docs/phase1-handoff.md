# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14`; the official review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- HEAD: the commit that carries this note, on top of `164fff6`
  (`docs(17): A1 pilot batch contract (C3) and missing outcomes (C2)`).
- Uncommitted, the owner's: `.gitignore` (it ignores `/characterization*/` and `/smoke*/`).
  Preserve it; do not restore, commit or clean it, nor the ignored result directories it covers.
- That commit holds the revision of `17` after Astra's review of `164fff6`, committed at the
  owner's request and not yet design-reviewed: `crates/protocol/tests/a1_batch_composition.rs`,
  `docs/15-phase1-review.md`, `docs/16-a1-incentive-design.md`,
  `docs/17-a1-pilot-batch-design.md`, `docs/README.md` and this note.

## 2. Roles

Claude Code implements and verifies. Astra decides the design and reviews each commit. The
owner commits when asked and pushes; the agent session has no GitHub credentials. No approval
is inferred from green tests: a finding closes only when Astra's review says so, recorded in
`15`.

## 3. Findings (from `15`; nothing beyond it)

- **Closed within their records' scope:** A4 (`fb8a3d4`), A5 (`afc84d0`), A6 (`f5ce98b7`),
  A7 (`cabdae4`).
- **A1: open.** Diagnosis confirmed. Approved: the beacon and count analysis of `16` §§6–7,
  the band baselines (the dossier's condition C5, `protocol::panel_scores`) on `e8fdbe7`, and
  the properness proof as a result conditional on C6 and the theorem's other hypotheses, not
  as a guarantee of the protocol. Not approved: the overall proposal of `16`; `17` on
  `164fff6` and its group audit. `17` is revised, pending design review; no batching choice
  is approved.
- **A11: open, new.** A stage 1 that did not converge composes into a screen rejection and an
  observed outcome 0 on the entering path, and into `Measured { passed: false }` (outcome 0 at
  `π = ε`, counted by `FalseNegatives`) on the explored path. Confirmed by Astra on `164fff6`;
  severity **high** (set by Astra: a numerical failure can become a rejection and a negative
  reputational outcome; nothing is said of its frequency in real use). Not fixed yet. A4's
  closure is not reopened.
- **Open, no progress recorded in this work:** A2, A3, A8, A9, A10 and the register's C5 and
  C6 (author average, decay time). The register's C5 is not the dossier's condition C5.

## 4. What to read

1. `CLAUDE.md` and `docs/CLAUDE.md` (invariants, comment budget, workflow).
2. `docs/15-phase1-review.md`: the A1 and A11 rows, the correction records of A4–A7.
3. `docs/16-a1-incentive-design.md` (authoritative for A1's theorem, conditions C1–C6 and H0,
   the beacon model) and `docs/17-a1-pilot-batch-design.md` (the batch analysis, revised).
4. Tests: `crates/protocol/tests/a1_exploration_information.rs`,
   `crates/protocol/tests/a1_batch_composition.rs`, `crates/protocol/tests/panel_scores.rs`.

## 5. Astra's review of `164fff6`, and what the revision did

| Review point | Revision |
|---|---|
| Stage-1 non-convergence turned into outcome 0 is a contract defect | recorded as `15` A11; the screening test now also asserts the explored path; no production change |
| The screening test builds `MaxIters` by hand | stated as an API composition (E-api) in `17` §2, apart from the real non-convergence on simulated pilots in `13` §8.7.5 |
| The batch comparison must pin `Dif` against `NoDif` | `assert_eq!((campaign, alone), (Recheck::Dif, Recheck::NoDif))`; run passed |
| `17` wrongly required equal outcome laws across the paths | `17` §3: conditional agreement on each path's event, the argument with conditional expectations, Astra's counterexample; C6 kept distinct |
| Exclusion of dynamic batching and fixed references not demonstrated | `17` §4: B and dynamic batching "not retained, not excluded", with what an exclusion would need |
| Costs must separate reuse from a double pilot | `17` §4.1: validation, audit, overlap, total, increment, capacity; Astra's numbers verified; answers, anchors, fits and retries apart; equal count is not equal information |
| Prefixed groups activated by entry or draw missing | `17` §4, alternative D: identity, reference conditions, API effects, observed share, correlation |
| Group audit not approved; no capacity decision for the owner | `17` §6 recommends no policy and asks the owner nothing |

## 6. Evidence

- **Claude Code's Rust runs.** On `e8fdbe7`: `cargo test -p protocol` (233 passed),
  `cargo test -p scoring` (182 passed, 2 ignored), `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, the same with `--features calibration`,
  `python3 scripts/comment_budget.py`. On `164fff6` (tests and docs only): the
  `a1_batch_composition` tests (one by default, one with `calibration`, 69.5 s),
  `a1_exploration_information` (8) and `panel_scores` (7), fmt, both clippy runs and the
  budget. On this revision (one test file changed): `cargo test -p protocol --features
  calibration --test a1_batch_composition` (2 passed, 65.97 s; `Dif` and `NoDif` as asserted),
  the same test file without the feature (1 passed), `cargo fmt --all -- --check`,
  `cargo clippy -p protocol --all-targets -- -D warnings` with and without `calibration`, the
  comment budget. The full suites were not re-run for `164fff6` or for this revision.
- **Calculations.** Rational arithmetic (Python `fractions`, a scratch script, not in the
  repository); every formula is in `17` §§3–5, so each number can be recomputed by hand.
- **Astra.** Reads diffs, contracts and tests and recalculates by hand or with rational
  arithmetic; it does not re-run the Rust tests.
- Never run in this work: full characterization, smoke, mutation campaigns, golden
  cross-target matrix.

## 7. Next task

1. Astra reviews the revision's diff. Announced checks: the conditional agreement of `17` §3
   and the IPW proof of alternative D; the hypotheses behind the formulas on correlation,
   variance and reuse; the provenance of the historical convergence rates (unchanged stage-1
   code is not enough if the engine or the experimental design changed: `17` §2 and `15` A11
   now state what changed since `ec34fa4`); the new assertions on the screening's two paths.
2. Then, the next code correction is expected to be A11: keep a stage 1 that did not converge
   indeterminate through the decision boundary, without introducing a retry policy with it.
   It starts with a test that fails on the current code. Wait for the mandate.

## 8. Constraints

No change to thresholds, events or serialization, golden outputs or historical results. No
full characterization, smoke or extended campaign. No commit or push unless the owner asks.
Anything not verifiable from the repository: "da verificare".
