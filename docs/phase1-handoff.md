# Phase 1 work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14` and `docs/21`; the review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win. The
previous note, which carried the forecast-pilot design's correction work up to `38780af`, is
archived unchanged except for its link targets:
[`archive/phase1-forecast-pilot/docs/phase1-handoff.md`](../archive/phase1-forecast-pilot/docs/phase1-handoff.md).
Its histories, runs and evidence stay there.

## 1. Where things stand (2026-10-21)

**Owner decision — [`01` D47](01-decisions.md).** The author writes the question in its context
and states the correct answer; reviewers answer it, free to consult any resource; the system does
not detect, infer or assess the use of outside tools and does not require the sources consulted
to be stated or recorded; the mathematics is to process the answers and evaluate the reviewers'
reliability over time, with a possible reputational weight. Forecasting the pilot outcome is no
longer the reviewers' main contribution. D45 (open environment) and D46 (the assessment's two
priorities) remain the references for conditions and objectives.

**Still to define, by the owner and Astra, before any implementation:** the aggregation formula,
the reputation update, the treatment of a contested key, thresholds, outcomes and guarantees;
how Level A (bridging, band, appeal) and Level B (pilot, DIF, contested facts) read the answers;
the role of golden items; whether the deposit keeps its mandatory source and the contested-fact
path its source check; whether the new design needs a whole-test guarantee (A2). A balanced split
of correct and wrong answers is a signal to study, not a rule; agreement with the key or the
majority is not demonstrated reliability. Invariant #2 and the exclusion of a Schelling point
(D6) bind the choice. Do not fill these gaps with a formula or a policy of your own.

**What the code does.** Nothing implements D47. A reviewer commits and reveals a probability that
the item passes the pilot; it is the bridging rating and the forecast that `scoring::reputation`,
`protocol::{honeypot, exploration, probation, panel_scores, cohort_scores, affine_weights}`
score and weigh (`02` §§C.2–C.4, the previous design). [`21`](21-d47-inventory.md) §2 lists what
the implementation will have to adapt.

**The reorganization (local commits, not pushed):** `9cbb446` records D47; `c158670` moves the
previous design's dossiers (`16`, `17`, `19`, `20`, the previous handoff), A12's and A2's tools
and the nine A2 reports to [`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md),
renames only; `76f12f5` adds the archive's index and path map, the inventory `21`, the redirect
pages and the link targets; `990fe9d` and `72237c5` realign the specification and the
registers; the front pages, the index, this note and the agent instructions follow. No result
was regenerated; no fit, benchmark or campaign was run.

**Residual separations** (`21` §4): the A2 harness stays in `crates/characterization` because
`a2-probe run` guards its provenance on `crates/`; `scoring::latent::latent_dif_candidates`;
`protocol::cohort_snapshot`, which C6's evidence reads; the A12 golden tables in
`crates/protocol/tests/fixtures/`; the S2 and A1 test code, which exercises current code; the
paper, whose `scripts/common.py` feeds CI's differential oracles.

**Phase 1 is not complete.** A2 is open and not resolved by the archiving
([`15`](15-phase1-review.md#a2-current-state-design-decision-deferred), D47 entry).

## 2. Roles and working agreement

- **Claude Code** implements, verifies (tests, calculations, runs) and commits when the owner
  asks; it does not push. **The owner** orchestrates the sessions, publishes and decides genuine
  product trade-offs. **Astra** decides the design and reviews each commit, reading diffs and
  recalculating; it does not re-run Rust.
- A finding closes only on Astra's recorded review (`15`), never on green tests. One finding per
  intervention; a code defect starts with a test that fails on the baseline.
- Evidence labels: L read, D proved, C calculated, E executed; say whether a fit is real and
  whether a composition is the API or a runtime (no epoch driver exists); never present a
  counterexample as a frequency.
- Long runs, fits and campaigns need the owner's explicit authorization. Repository rules
  (`docs/CLAUDE.md`): the comment budget (`scripts/comment_budget.py`, also on the archive's
  `sim/`), docs updated with the code, English only. Build scratch worktrees with a separate
  `CARGO_TARGET_DIR`.

## 3. Closed findings, within their perimeters

Records in [`15`](15-phase1-review.md); the previous note's §3 summarizes each scope. A3
(`16e862c`), A4 (`fb8a3d4`; (a) `af17eb3`; (b), (c) `52f3822`), A5 (`afc84d0`), A6
(`f5ce98b7`), A7 (`cabdae4`), A8–A9 (bounded claim rectification), A10 (`115f538`), A11
(`dd842d6`; (f) `af17eb3`), C5 (`f76d5dd`), C6 (`825cabc`), D3 (`9df086f`), the Variant 1 fit
states (`18e1b5f`, `6274270`); A1 conditionally on `1bd865c`, A12 conditionally on `15b846c`.
A1, A12 and A10 concern the previous design's forecast score and certify nothing of D47's
mechanism; A10's residual is suspended (`15`, D47 disposition).

## 4. Open work

- **D47's mechanism:** its specification (above), then its implementation as a separate
  assignment, adapting `21` §2. New task ids come with the specification (`10` Priorities).
- **A2 (high), open:** the old requirement's disposition and the guarantees still to decide for
  the new design (`15`, A2's D47 entry). Certifier, other obligations and experiments suspended.
- **Suspended with the previous design:** A1's O1/O2 premises and T1, A12's candidate and
  procedures, A10's residual, exploration, B-b, S2, the forecast path's Level C calibration.
- **Unchanged:** `08` PRIV-004.1 (an inclusion proof's confidentiality, remediation open); B1–B3;
  sample floors, burden and runtime budgets (B4–B5, C3, D2, D4); T83's operating point (B7, D1,
  D6, E); A4 (d); DIF-007's metric deviation, AT-DIF-08's adversarial batch, the optimizer's
  status on the inputs already reported; C2, C4; T25's Level A and B steps; T82's re-run.
- **Phase 2** runs on its own branch (`feat/phase2-runtime`) and is not touched here.

## 5. Essential reading to resume

`CLAUDE.md` and `docs/CLAUDE.md`; `docs/01` D44–D47; `docs/21`; `docs/15` (How to read, the D47
disposition, A2's current state); `docs/02` (its Reading status, Levels A and B, §C.1, and
§§C.2–C.4 as the previous design's code); `docs/05`; `docs/10` Priorities; the archive's index
for the previous design's material. Code: `crates/protocol/src/{review,lifecycle,orchestrator,
probation,honeypot,exploration,results}.rs`, `crates/scoring/src/{reputation,bridging}.rs`.
