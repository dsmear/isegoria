# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14`; the official review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- HEAD: the commit that carries this note, on top of `16e862c` (`fix(scoring): the BIC counts
  the latent histogram's Q−1 logits; its moment penalty is no gauge (A3, P1)`).
- Uncommitted, the owner's: `.gitignore` (it ignores `/characterization*/` and `/smoke*/`).
  Preserve it; do not restore, commit or clean it, nor the ignored result directories it covers.
- That commit holds A3's closure record and A2's diagnosis (§6), made at the owner's request
  before Astra's review; the push is the owner's (the session has no GitHub credentials).

## 2. Roles

Claude Code implements and verifies. Astra decides the design and reviews each commit. The
owner commits when asked and pushes; the agent session has no GitHub credentials. No approval
is inferred from green tests: a finding closes only when Astra's review says so, recorded in
`15`.

## 3. Findings (from `15`; nothing beyond it)

- **Closed within their records' scope:** A3 (`16e862c`: the moment penalty's interpretation
  and the nominal count), A4 (`fb8a3d4`), A5 (`afc84d0`), A6 (`f5ce98b7`), A7 (`cabdae4`), A11
  (`dd842d6`), A4's residual (a) and the premature settlement (`af17eb3`).
- **A3's closure:** Astra read the diff, code, tests and contracts, decoded and recalculated the
  six golden BICs in Python (`+2 ln 1500` within rounding, `bic_gain` unchanged on the two
  fixtures); no Rust run, no re-run of Claude's scratch captures. Two non-blocking rectifications
  made: `18` §8's margins are unchanged at the precision reported, those towards the per-class
  candidates move by `7.28·10⁻¹²` in their bits; `15`'s A3 record now states that the ignored A3
  test re-executed the two diagnostic fits (assertions passed, numbers not captured).
- **A2: open — diagnosis and design pending review** (`docs/19`). Nothing implemented.
- **Open:** A1; A4's (b)–(d); B1–B3 (A3's residues); C2, C4; the full runtime; A8, A9, A10 and
  the register's C5 and C6, with no progress in this work.

## 4. What to read

1. `CLAUDE.md` and `docs/CLAUDE.md` (invariants, comment budget, workflow).
2. `docs/15-phase1-review.md`: the A2 row.
3. `docs/19-a2-dtf-composition-design.md`; `docs/02` §B.7; `docs/01` D38;
   `crates/scoring/src/dtf.rs`; `crates/protocol/src/contested.rs`.
4. Evidence: `crates/scoring/tests/dtf_composition.rs`, `crates/protocol/tests/a2_dtf_composition.rs`,
   `sim/dtf_composition.py`.

## 5. A2 in short (details in `19`)

- `ContestedPool::dtf` sums per-fit DTFs of a test's contested facts, each over its batch's
  fitted population and counted classes (share ≥ 5%, renormalized); active items never enter;
  `draw` runs only in tests, and no code composes a full test.
- A sufficient proposition bounds a test's DTF by the per-fit terms without matching class
  labels, under H1 (one population measure for every fit), H2 (groups represented by each fit's
  counted classes; the envelope when their composition varies with ability) and H3 (every item
  covered). The current APIs guarantee none of them.
- Evidence: hand-built curves through the real code (the measure moves one item's DTF from
  0.094 to 0.344; the pool admits two facts at cost 0.069 whose common-population DTF is 0.416;
  an item under the flag cut reaches 0.170 alone); a real fit with known truth (the golden open
  batch selects one class: fitted DTF 0, true 0.782); a rational example where the envelope (1)
  exceeds the per-pair maximum (¾). No frequency measured.
- Directions: R1 (describe the cost honestly; no guarantee) now; R3 (a test-level fit with an
  uncertainty bound) to realize the guarantee, with one owner trade-off (respondents per form);
  R2 (co-measured facts) only removes the cross-fit sum.

## 6. Files

- **A3 closure:** `docs/15` (A3 row and record: closure, review, rectified verification text),
  `docs/18` (status, §8 margins), `docs/08` header, `docs/README.md`.
- **A2:** new `docs/19`; `docs/15` (A2 row, closing paragraph); `docs/README.md`; new tests
  `crates/scoring/tests/dtf_composition.rs` (three: measure, item under the cut, one-class real
  fit) and `crates/protocol/tests/a2_dtf_composition.rs` (the pool's sum); new
  `sim/dtf_composition.py` with its entry in `sim/README.md`. No production line changed.

## 7. Evidence

- **Claude Code's runs on this work:** `cargo test -p scoring --test dtf --test dtf_composition`
  (10 and 3 passed), `cargo test -p protocol --test contested_facts --test a2_dtf_composition`
  (10 and 1 passed), `python3 sim/dtf_composition.py`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` with and without `--features
  calibration`, `python3 scripts/comment_budget.py`. A2 changes no production code, so the full
  suites were not re-run.
- **Astra.** Reads diffs, contracts and tests and recalculates by hand or with rational
  arithmetic; it does not re-run the Rust tests.
- **Note:** build scratch worktrees with a separate `CARGO_TARGET_DIR`.
- Never run in this work: full characterization, smoke, mutation or calibration campaigns.

## 8. Next task

Astra's review of A2's diagnosis and of the recommended R1/R3; implementation only after it, then
the agreed decision synthesis (which mechanism to complete, which guarantees to pursue) before
further structural corrections. Points Astra announced it will check (preliminary, not a review):
the proposition's hypotheses — mixture weights independent of ability and coherent across the
subset's items for the per-pair maximum, and H3 as a partition of the test among the summed terms;
that a test-level fit leaves representation, misspecification and estimation open, and that a
confidence bound on the envelope needs its own justification (a fitted zero certifies nothing);
that a batch per form must be compared with a reusable common calibration, more conservative
bounds and a reduced declared guarantee before any cost is put to the owner.

## 9. Constraints

No change to thresholds, golden outputs or historical results without a decided correction. No
full characterization, smoke or extended campaign. No commit or push unless the owner asks.
Anything not verifiable from the repository: "da verificare".
