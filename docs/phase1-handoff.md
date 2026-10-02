# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14`; the official review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- HEAD: the commit that carries this note, A4's residual (a) and the premature settlement,
  on top of `dd842d6` (`fix(protocol): an unconverged stage-1 screen is indeterminate, never
  a rejection (A11)`). Not pushed by this session (no GitHub credentials).
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
  A7 (`cabdae4`), A11 (`dd842d6`).
- **A11 closed** on `dd842d6`: Astra read diff, contracts, callers and tests, without
  re-running Rust; the approval covers the screening contract only (indeterminate preserved,
  no observed outcome, escrow handed back on `Pilot1`, historical bytes compatible).
- **A4's residual (a) and the premature settlement in `Pilot2` (A11 (f), confirmed by
  Astra): implemented and verified, design review pending** (committed on top of
  `dd842d6`; [record](15-phase1-review.md#a4-residual-a-and-premature-settlement-record)).
  A4's original correction and A11 are not reopened.
- **A1: open.** Approved so far: the beacon and count analysis of `16` §§6–7, the band
  baselines on `e8fdbe7`, the properness proof as a conditional result, and `17` on `37addca`
  as a conditional analysis. No batching policy approved for implementation.
- **Open:** C2 (termination of a pending pilot), C4 (denominator, missing reveals), A4's
  residuals (b)–(d) (the candidates' search among them), batching, the random source, the
  runtime; A2, A3, A8, A9, A10 and the register's C5 and C6, with no progress in this work.

## 4. What to read

1. `CLAUDE.md` and `docs/CLAUDE.md` (invariants, comment budget, workflow).
2. `docs/15-phase1-review.md`: the A1, A4 and A11 rows; the records of A4's residual (a),
   A11 and A4–A7.
3. `docs/16-a1-incentive-design.md` and `docs/17-a1-pilot-batch-design.md` for A1.
4. Tests: `crates/protocol/tests/indeterminate_pilot2.rs` (this work),
   `indeterminate_screen.rs` (A11), `indeterminate_recheck.rs` (A4), `lifecycle_replay.rs`.

## 5. What this work does

- **Stage 2:** `Event::Pilot2Batch { batch_size, dif: Recheck, source_verified }` and
  `ItemVerdicts::dif: Recheck`. `NoDif` and `Dif` keep the former `passed = true / false`
  transitions, the verified-source rule included; `Indeterminate` keeps `Pilot2 { appealed }`
  or `Explored { screened: true }`, whatever the source check: `Pending`, nothing recorded,
  nothing measured; `run_item` stops there. The batch floor and the phase come first.
- **Escrow:** `settle_appeal` (still `Result<(), Escrow>`) promotes on `ActivePool` and
  `Contested`, fails on `Rejected(Screen | Dif)`, and hands the escrow back untouched on every
  other state, matched explicitly (pending pilot, pre-pilot states, gate rejections,
  `Explored`, `Measured`, `Retired`).
- **Encoding:** stage 2's byte keeps the boolean's order (0 `Dif`, 1 `NoDif`, 2
  `Indeterminate`, 3+ refused), with its own codec; the re-check's codec keeps 0 `NoDif`,
  1 `Dif`. Version unchanged; an earlier decoder refuses byte 2.
- **Not done, on purpose:** no retry, scheduler, batching, threshold, candidate-selection or
  source-verification change; no missing-outcome policy; `SkillTrack`'s denominator unchanged.

## 6. Files

- **Code:** `protocol/src/lifecycle.rs` (event field, two transitions), `events.rs`
  (`stage2_byte`, `read_stage2`), `orchestrator.rs` (`ItemVerdicts::dif`, `run_item`,
  `settle_appeal`).
- **Tests:** new `indeterminate_pilot2.rs`; `lifecycle_replay.rs` (indeterminate stage 2 in
  the walks, pins, earlier bytes replayed); `lifecycle_model.rs`, `orchestrator_model.rs`
  (three readings); mechanical updates in the other callers; `end_to_end.rs` maps Variant 1's
  `Undetermined` to `Indeterminate`; `contested_facts.rs` composes `target_rechecks`.
- **Docs:** `01` D27 note, `02` §B.3, `04` §Events and replay, `05` [5b] and [6]–[7], `08`
  (header, §9.1 rows), `15` (A4 and A11 rows, A11 record, the new record), `16` (C2 support),
  `17` (§1 row 8, §5), `ARCHITECTURE.md`.

## 7. Evidence

- **Claude Code's Rust runs on this work.** The two reproduction tests failed on `dd842d6`
  before the fix. Then: `cargo test -p protocol` (244 passed), `cargo test -p scoring`
  (182 passed, 2 ignored), the sixteen touched protocol test files with `--features
  calibration` (116 passed), `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` with and without `--features calibration`, `python3
  scripts/comment_budget.py`. No characterization change; no costly fit re-run.
- **Note for later sessions:** a worktree built with the shared `CARGO_TARGET_DIR` left a
  stale `protocol` artifact (a baseline signature); `cargo clean -p protocol` cleared it. Use a
  separate target directory for scratch worktrees.
- **Earlier runs.** On `dd842d6` (A11): protocol 238, scoring 182 (2 ignored), touched files
  with `calibration` 115, characterization `production` 5, pinned records unchanged.
- **Astra.** Reads diffs, contracts and tests and recalculates by hand or with rational
  arithmetic; it does not re-run the Rust tests.
- Never run in this work: full characterization, smoke, mutation campaigns, golden
  cross-target matrix.

## 8. Next task

Astra's design review of A4's residual (a) and the premature settlement. A1 stays open, with
no batching policy approved.

## 9. Constraints

No change to thresholds or golden outputs or historical results; event or serialization
changes only as a finding's correction defines them. No full characterization, smoke or
extended campaign. No commit or push unless the owner asks. Anything not verifiable from the
repository: "da verificare".
