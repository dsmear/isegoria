# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14`; the official review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- HEAD: the commit that carries this note, on top of `af17eb3` (`fix(protocol): an
  indeterminate stage 2 keeps the pilot pending; an appeal settles only on its conclusion (A4
  residual, A11 (f))`).
- Uncommitted, the owner's: `.gitignore` (it ignores `/characterization*/` and `/smoke*/`).
  Preserve it; do not restore, commit or clean it, nor the ignored result directories it covers.
- The commit that carries this note holds A3's diagnosis (§6), made at the owner's request
  before Astra's review of the diff; not pushed by this session (no GitHub credentials).

## 2. Roles

Claude Code implements and verifies. Astra decides the design and reviews each commit. The
owner commits when asked and pushes; the agent session has no GitHub credentials. No approval
is inferred from green tests: a finding closes only when Astra's review says so, recorded in
`15`.

## 3. Findings (from `15`; nothing beyond it)

- **Closed within their records' scope:** A4 (`fb8a3d4`), A5 (`afc84d0`), A6 (`f5ce98b7`),
  A7 (`cabdae4`), A11 (`dd842d6`), and on `af17eb3` A4's residual (a) and the premature
  settlement A11 (f).
- **`af17eb3` review:** Astra read diff, contracts, callers and tests, without re-running
  Rust. Approved: an appeal settles only on the pilot's conclusive outcomes, and a caller that
  settles later must keep that outcome; stage 2's historical bytes preserved, its codec distinct
  from `Revalidate`'s. `end_to_end.rs` shows no really indeterminate case; that coverage is in the
  dedicated tests on hand-built fits.
- **A3: open — diagnosis and proposal pending design review** (`docs/18`). Nothing corrected.
- **Open:** A1; A4's (b)–(d), the candidates' search among them; C2, C4; the full runtime;
  A2, A8, A9, A10 and the register's C5 and C6, with no progress in this work.

## 4. What to read

1. `CLAUDE.md` and `docs/CLAUDE.md` (invariants, comment budget, workflow).
2. `docs/15-phase1-review.md`: the A3 row; the correction records.
3. `docs/18-a3-latent-shape-design.md` (A3), `docs/02` §B.3, `docs/01` D43,
   `crates/scoring/src/latent.rs` (`Grid::shape`, `gauge`, `free_params`, `latent_dif_with`).
4. Evidence: `sim/latent_shape_dimension.py`; the three A3 tests at the end of `latent.rs`.

## 5. A3 in short (details in `18`)

- The histogram's only exact redundancy is the logits' shift; the map from weights to the
  standardized distribution is injective with injective derivative, so the ability family has
  dimension `Q − 1 = 40`, not `Q − 3`. Whether the responses identify all 40 is not proved.
- The "gauge" `n[m² + (v − 1)²]` on the grid moments is a regularizer that grows with `n` (weight
  1 per respondent, a non-zero relative weight at any size). On two one-class fits the histograms
  are rough, the NLL curves along those moments, and the penalty holds them; removing it lowers
  the NLL by 0.17–0.21 nats and moves skewness ≤ 0.012, kurtosis ≤ 0.064, item parameters
  ≤ 0.0034. Two cases: no general conclusion on mixtures or verdicts.
- Verification and delimitation, not a new defect: the review's note that a common offset
  cancels holds in every current comparison (ordering unchanged; each candidate's absolute BIC,
  golden rows `latent.candidates` and `floor.candidates`, would shift by `2 ln n`; numerically
  only a tie within rounding could differ).
- Recommended P1: describe the penalty as an `n`-scaled regularizer, count `Q − 1`; not
  implemented. P2 (hard moment constraints) would change the statistical family.

## 6. Files

- **Tests:** three in `crates/scoring/src/latent.rs`'s test module (no production line
  changed): `a_shift_of_the_histogram_logits_is_its_only_exact_gauge`,
  `standardized_nodes_do_not_make_the_grid_moments_a_gauge`,
  `the_moment_penalty_holds_a_fitted_histogram` (`--ignored`, about 11 s).
- **Calculation:** `sim/latent_shape_dimension.py` (numpy), with its entry in `sim/README.md`.
- **Docs:** new `18`; `15` (A3 row; the `af17eb3` closures in the A4 and A11 rows and records;
  the old residual lists now state their baseline and point to the closures); `02` §B.3 (pointer);
  `08` header; `17` (status of the A11 and A4 corrections); `README.md`.

## 7. Evidence

- **Claude Code's runs on this work:** the three A3 tests (the ignored one with `--ignored`);
  `nll_and_gradient_match_the_reference` and `free_parameters_are_counted_as_specified`
  re-run; `python3 sim/latent_shape_dimension.py`; `cargo test -p scoring` (184 passed, 3
  ignored), fmt, clippy with and without `calibration`, the comment budget.
- **Astra.** Reads diffs, contracts and tests and recalculates by hand or with rational
  arithmetic; it does not re-run the Rust tests.
- **Note:** build scratch worktrees with a separate `CARGO_TARGET_DIR`.
- Never run in this work: full characterization, smoke, mutation campaigns, golden
  cross-target matrix.

## 8. Next task

Astra's design review of A3's diagnosis and of P1. Announced checks: the family's dimension
against identifiability from the responses; the penalty's terminology (it grows with `n`); the
offset's cancellation in every effective comparison (ordering, absolute BICs, numerics); the
scope of the two fits. Only after it, and if P1 is authorized, a code change (count and
comments) with the two golden rows regenerated. A3 stays open; no model change is approved.

## 9. Constraints

No change to thresholds, golden outputs or historical results without a decided correction. No
full characterization, smoke or extended campaign. No commit or push unless the owner asks.
Anything not verifiable from the repository: "da verificare".
