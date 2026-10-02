# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14`; the official review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- HEAD: the commit that carries this note, the A11 correction, on top of `37addca`
  (`docs(17): A1 batch dossier revised after review of 164fff6; A11 recorded`).
- Uncommitted, the owner's: `.gitignore` (it ignores `/characterization*/` and `/smoke*/`).
  Preserve it; do not restore, commit or clean it, nor the ignored result directories it covers.
- That commit was made at the owner's request, before Astra's review of its diff; its
  content is listed in §6.

## 2. Roles

Claude Code implements and verifies. Astra decides the design and reviews each commit. The
owner commits when asked and pushes; the agent session has no GitHub credentials. No approval
is inferred from green tests: a finding closes only when Astra's review says so, recorded in
`15`.

## 3. Findings (from `15`; nothing beyond it)

- **Closed within their records' scope:** A4 (`fb8a3d4`), A5 (`afc84d0`), A6 (`f5ce98b7`),
  A7 (`cabdae4`).
- **A1: open.** Diagnosis confirmed. Approved: the beacon and count analysis of `16` §§6–7,
  the band baselines (the dossier's condition C5, `protocol::panel_scores`) on `e8fdbe7`, the
  properness proof as a result conditional on C6 and the theorem's other hypotheses, and, on
  `37addca`, the revised mathematical argument and comparison of alternatives of `17` as a
  conditional analysis. Not approved: the overall proposal of `16`; any batching policy for
  implementation.
- **A11: high; implemented and verified, design review pending** (committed on top of
  `37addca`; [record](15-phase1-review.md#a11-correction-record)). A stage-1 fit that did
  not converge now reads `Screening::Indeterminate` for every item, and the item stays in
  `Pilot1` or `Explored`: no screen rejection, no observed outcome, no measured exploration, no
  appeal settlement. Not closed. A4's closure is not reopened.
- **Open, no progress recorded in this work:** A2, A3, A8, A9, A10 and the register's C5 and
  C6 (author average, decay time). The register's C5 is not the dossier's condition C5.

## 4. What to read

1. `CLAUDE.md` and `docs/CLAUDE.md` (invariants, comment budget, workflow).
2. `docs/15-phase1-review.md`: the A1 and A11 rows, the correction records of A11 and A4–A7.
3. `docs/16-a1-incentive-design.md` (authoritative for A1's theorem, conditions C1–C6 and H0,
   the beacon model) and `docs/17-a1-pilot-batch-design.md` (the batch analysis).
4. Tests: `crates/protocol/tests/indeterminate_screen.rs` (A11),
   `crates/protocol/tests/a1_exploration_information.rs`,
   `crates/protocol/tests/a1_batch_composition.rs`, `crates/protocol/tests/panel_scores.rs`.

## 5. Astra's review of `37addca`

- Approved as a conditional analysis: the revised argument of `17` §3 and the comparison of
  alternatives of `17` §4. No batching policy approved for implementation; A1 open.
- Evidence: reading of diff, code and tests, independent recalculations; Rust not re-run.
- The historical screen study's provenance checked by comparing the code since `ec34fa4`;
  its percentages read in the historical results, neither recalculated by Astra nor re-run;
  the study's individual records are not kept.
- Non-blocking, done in `17` §4.1: sampling by groups gives a different precision of the IPW
  score, not always a lower one (the count's variance grows when a reviewer's items share a
  draw; the score sum's can shrink: `d₁ = 1`, `d₂ = −1` gives 0); formulas under draws
  independent across groups and fixed assignments.
- Next code correction: A11, keeping the indeterminate without a retry policy (done here).

## 6. The A11 correction

- **Code:** `protocol/src/pilot.rs` (`Screening`, `stage1_verdicts`, `stage1_screen`,
  `screen`), `lifecycle.rs` (`Event::Pilot1Batch { screen }`, the two transitions),
  `events.rs` (byte 0 fail, 1 pass, 2 indeterminate), `orchestrator.rs`
  (`ItemVerdicts::screen`, `run_item`'s doc, `settle_appeal -> Result<(), Escrow>`);
  `characterization/src/run.rs` (`recorded_kept`, records unchanged), `summary.rs` (doc).
- **Tests:** new `indeterminate_screen.rs` (the former
  `a1_batch_composition.rs::an_unconverged_screen_composes_into_outcome_zero`, turned into the
  acceptance test and moved); `pilot_screen.rs`, `lifecycle_replay.rs` (indeterminate in the
  walks, bytes pinned, pre-A11 bytes replayed), `lifecycle_model.rs`, `orchestrator_model.rs`
  (three readings); mechanical updates in the other callers; `characterization/tests/production.rs`.
- **Docs:** `02` §B.2, `04` §Events and replay, `05` [6]–[7], `08` (header, §9.1 rows,
  AT-PRO-15, IRT-003), `13` §8.3, `15` (A1 row, A11 row and record), `16` (C2 support, one
  wording), `17` (the review of `37addca`, §4.1, the A11 passages), `ARCHITECTURE.md`.
- **Not done, on purpose:** no retry, scheduler, threshold, batching or missing-outcome
  policy; stage 2 untouched (A4 residual (a)); `SkillTrack`'s denominator unchanged (C4).
- **Residual defect, recorded at Astra's request (`15`, A11 record, (f)):** `settle_appeal`
  settles an appeal as failed while the item is still in `Pilot2`, turning an unconcluded
  stage 2 into a definitive settlement. Not corrected; Astra will judge whether to address it
  with A4's residual (a).

## 7. Evidence

- **Claude Code's Rust runs on the A11 correction.** The acceptance test failed on `37addca`
  (both paths) before the fix. Then: `cargo test -p protocol` (238 passed), `cargo test -p
  scoring` (182 passed, 2 ignored), the fourteen touched protocol test files with
  `--features calibration` (115 passed), `cargo test -p characterization --test production`
  (5 passed), `harness.rs::a_record_of_each_kind_is_pinned` (digests unchanged), `cargo fmt
  --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` with and without
  `--features calibration`, `python3 scripts/comment_budget.py`. Hand mutations (unconverged
  as fail, indeterminate moving either state, byte 2 as fail, settling on `Pilot1`) each fail
  a targeted test; the pre-A11 decoder's refusal of byte 2 checked on a scratch worktree of
  `37addca`, since removed. The batch comparison's costly fit was not re-run (untouched).
- **Earlier runs.** On `e8fdbe7`: protocol 233, scoring 182 (2 ignored), fmt, both clippy runs,
  the budget. On `37addca`: the `a1_batch_composition` tests (2 passed with `calibration`,
  65.97 s), fmt, clippy for `protocol`, the budget.
- **Astra.** Reads diffs, contracts and tests and recalculates by hand or with rational
  arithmetic; it does not re-run the Rust tests.
- Never run in this work: full characterization, smoke, mutation campaigns, golden
  cross-target matrix.

## 8. Next task

Astra reviews the A11 commit's diff. Announced checks: the propagation of
`Screening::Indeterminate` and the orchestrator's stop; the escrow's preservation and the
callers' handling of `settle_appeal`'s result; the historical bytes' compatibility and the
refusal of invalid encodings; the separation of characterization's `recorded_kept` from the
protocol's verdicts. Then, per its outcome, A11 closed or amended, and a decision on the
`Pilot2` settlement defect. A1 stays open, with no batching policy approved.

## 9. Constraints

No change to thresholds or golden outputs or historical results; event or serialization
changes only as a finding's correction defines them (A11: byte 2). No full characterization,
smoke or extended campaign. No commit or push unless the owner asks. Anything not verifiable
from the repository: "da verificare".
