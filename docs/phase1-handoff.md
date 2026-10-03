# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14`; the official review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- HEAD: the commit that carries this note, A3's P1, on top of `deb4e5e` (`docs(18): A3
  diagnosis — the latent histogram's moments are no gauge; the count's offset cancels`).
- Uncommitted, the owner's: `.gitignore` (it ignores `/characterization*/` and `/smoke*/`).
  Preserve it; do not restore, commit or clean it, nor the ignored result directories it covers.
- That commit was made at the owner's request, before Astra's review of its diff; the push is
  the owner's (the session has no GitHub credentials). Content in §6.

## 2. Roles

Claude Code implements and verifies. Astra decides the design and reviews each commit. The
owner commits when asked and pushes; the agent session has no GitHub credentials. No approval
is inferred from green tests: a finding closes only when Astra's review says so, recorded in
`15`.

## 3. Findings (from `15`; nothing beyond it)

- **Closed within their records' scope:** A4 (`fb8a3d4`), A5 (`afc84d0`), A6 (`f5ce98b7`),
  A7 (`cabdae4`), A11 (`dd842d6`), A4's residual (a) and the premature settlement (`af17eb3`).
- **A3: open — P1 implemented and verified, Astra's review pending**
  ([record](15-phase1-review.md#a3-correction-record), `docs/18` §8).
- **A3's diagnosis, reviewed by Astra on `deb4e5e`** (diff, code and tests read, mathematics
  checked, the Python script and a rounding example run; no Rust or diagnostic fit re-run).
  Approved: the dimension `Q − 1` of the ability family on the open simplex; its distinction
  from identifiability through the responses, not proved; the penalty as a regularization
  proportional to `n`; the mathematical cancellation of the common offset; P1. Not approved:
  observable identifiability, the penalty's intensity, the BIC's validity for penalized
  mixture fits.
- **Open:** A1; A4's (b)–(d); C2, C4; the full runtime; B1–B3 (A3's residues below); A2, A8,
  A9, A10 and the register's C5 and C6, with no progress in this work.

## 4. What to read

1. `CLAUDE.md` and `docs/CLAUDE.md` (invariants, comment budget, workflow).
2. `docs/15-phase1-review.md`: the A3 row and its correction record.
3. `docs/18-a3-latent-shape-design.md` (§8 for P1), `docs/02` §B.3, `docs/01` D43.
4. `crates/scoring/src/latent.rs` (`free_params`, `moment_penalty`, the A3 tests at the end).

## 5. A3's P1 in short

- `free_params` counts the histogram's `Q − 1` logits less their shift instead of `Q − 3`
  (unchanged with the shape held): a nominal count of the parametrized family, not a proof of
  the dimension the responses identify. `gauge` renamed `moment_penalty`, described as a
  regularizer growing with `n`. Nothing else in the model or the selection changes.
- On the two golden fits verified, before and after: parameters, candidates visited, model,
  convergence, flags and `bic_gain` identical in their bits; the six candidates' BICs up by
  `2 ln 1500` within −0.57 to +0.43 ulp; margins 0.523 to 50.87, far above rounding. Five pinned
  characterization records identical. Only the two golden rows of the candidates' BICs
  regenerated. No bit-level guarantee for other batches.
- Residues assigned to B1–B3: identifiability through the responses; weak information; the
  penalty's effect on fits, mixtures and verdicts; the BIC on penalized fits. No general
  conclusion on DIF verdicts from the two one-class diagnostic fits.

## 6. Files

- **Code:** `crates/scoring/src/latent.rs` (`:214–218` count, `:828–830,866` the penalty's
  name and comment, the count test `:1249–1253`, the tests' calls).
- **Golden:** `crates/scoring/tests/fixtures/golden_bits.txt` (`latent.candidates`,
  `floor.candidates`, three rows each).
- **Docs:** `02` §B.3 (implemented fit and selection, open interpretation); `01` D43 (dated
  clarification); `10` T82 (dated clarification); `08` header; `15` (A3 row, A3 correction
  record, closing paragraph); `18` (review of `deb4e5e`, §3 conditional curvatures, §4, §5,
  §6, §7, new §8); `README.md`.
- The capture tests used for the comparison were scratch files, removed; their outputs are not
  in the repository.

## 7. Evidence

- **Claude Code's runs on P1:** `cargo test -p scoring --test golden` on the baseline (pass),
  after the change (six rows moved) and after the regeneration (pass); the scratch captures
  before and after; `cargo test -p scoring` (184 passed, 3 ignored); the `latent` unit tests
  and the ignored A3 test with `--ignored`; `cargo test -p characterization --test harness
  a_record_of_each_kind_is_pinned` (pass, pins unchanged) and the scratch record capture
  (identical); `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
  warnings` with and without `--features calibration`, `python3 scripts/comment_budget.py`.
- **Not run:** protocol tests (no consumer reads an absolute BIC), full characterization,
  smoke, mutation campaigns; other batches than the golden ones.
- **Astra.** Reads diffs, contracts and tests and recalculates by hand or with rational
  arithmetic; it does not re-run the Rust tests.
- **Note:** build scratch worktrees with a separate `CARGO_TARGET_DIR`.

## 8. Next task

Astra's review of P1's implementation. A3 closes only on that review; B1–B3 keep A3's
residues.

## 9. Constraints

No change to thresholds, golden outputs or historical results without a decided correction. No
full characterization, smoke or extended campaign. No commit or push unless the owner asks.
Anything not verifiable from the repository: "da verificare".
