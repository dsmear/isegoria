# Phase 1 work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions. The
specification is in `docs/01`–`docs/14` and `docs/21`; the review register is
[`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree, they win. The
previous note, which carried the forecast-pilot design's correction work up to `38780af`, is
archived unchanged except for its link targets:
[`archive/phase1-forecast-pilot/docs/phase1-handoff.md`](../archive/phase1-forecast-pilot/docs/phase1-handoff.md).
Its histories, runs and evidence stay there.

## 1. Where things stand (2026-10-21)

**Owner decision — [`01` D47](01-decisions.md).** The author writes the question in its context and
states the correct answer; reviewers answer it, free to consult any resource; they do not state or
record the sources they consult, and the system does not detect, infer or assess their use of
outside tools; the mathematics is to process the answers and evaluate the reviewers' reliability
over time, with a possible reputational weight (deferred by D48, below). Forecasting the pilot
outcome is no longer the reviewers' main contribution. D45 (open environment) and D46 (the
assessment's two priorities) remain the references for conditions and objectives.

**Owner decisions for the first implementation — [`01` D48](01-decisions.md).** They modify D47
where stated; D47 stays readable as the earlier decision.
- Reviewers are drawn by lot from the reviewer pool; no new selection by specialist competence.
  That the pool suits the context is a premise, not a property the draw demonstrates.
- Every reviewer weighs the same. Ordinary reviews do not update the reviewers' reputation; the
  evaluation of their reliability over time is deferred. Agreement with the majority is not
  rewarded.
- The consensus of the answers is the review's operational criterion. This rectifies the earlier
  absolute ban on deciding quality by voting (D1, invariant #2) without making consensus a
  guarantee of truth.
- A reviewer may answer that none of the alternatives is correct, which stays distinct from
  choosing an alternative other than the author's.
- A question is not corrected retroactively: its author submits a new question. The correct
  answer is never replaced automatically.
- The Internet, documents and AI are freely usable; tools are not detected and the reviewers'
  sources are not recorded.

**Accepted risks (D48):** shared errors, different competences under equal weights, superficial
reviews, coordination. The draw eliminates none of them. An evaluation of the reviewers may
become necessary later; it would need a credible reference and a distinct decision. **The
author's reputation** stays separate: rewards for acceptance and penalties for contestation or
rejection were discussed, but formula, size and conditions are not defined, and nothing of it is
an executable rule; the implemented author score is unchanged.

**Still incomplete:** the contract cannot be implemented until the decisions in §4 are taken by
the owner and reviewed by Astra. Agreement with the key or the majority is not demonstrated
reliability, and a balanced split of correct and wrong answers is a signal to study, not a rule.
The exclusion of a Schelling point (D6) stands. Do not fill the gaps with a formula or a policy
of your own.

**What the code does.** Nothing implements D47 or D48. A reviewer commits and reveals a probability
that the item passes the pilot; it is the bridging rating and the forecast that
`scoring::reputation`, `protocol::{honeypot, exploration, probation, panel_scores, cohort_scores,
affine_weights}` score and weigh (`02` §§C.2–C.4, the previous design). [`21`](21-d47-inventory.md)
§2 lists what the implementation will have to adapt.

**The reorganization (on `docs/phase1-review-alignment`):** `9cbb446` records D47; `c158670` moves
the previous design's dossiers (`16`, `17`, `19`, `20`, the previous handoff), A12's and A2's tools
and the nine A2 reports to
[`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md), renames only;
`76f12f5` adds the archive's index and path map, the inventory `21`, the redirect pages and the link
targets; `990fe9d` and `72237c5` realign the specification and the registers; `47a1698` the front
pages, the index, this note and the agent instructions; `17524b0` and `0b4f4ff` rectify them after
Astra's reviews. D48 is recorded on top of them, documentation only. No result was regenerated; no
fit, benchmark or campaign was run.

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

## 4. Decisions still needed to implement

From [`01` D48](01-decisions.md); none is to be settled in code or by a default of one's own.

1. **The drawn group:** its size, the quorum, and what absences (a drawn reviewer who does not
   answer) do. `10` T58 decided, for the previous design's panels and not implemented, a
   replacement from the same stratum, a floor of seven reveals, and a no-show losing vote weight
   and being suspended from the draw; with D48's equal weights and no reputation update from
   ordinary reviews, whether any of it carries over is part of this decision.
2. **Thresholds and outcomes:** the consensus a review needs, and what happens to the item in
   each case, among them a consensus on an alternative other than the author's and a consensus
   that no alternative is correct.
3. **"Don't know":** whether reviewers may give it, and how it counts.
4. **The author's reputation:** whether and how acceptance, contestation or rejection move it —
   formula, size, conditions, and what counts as a contestation. The implemented author score
   counts an item in D38's contested pool as a pass (D27).
5. **The source at deposit:** whether the author keeps attaching a mandatory primary source, and
   whether the contested-fact path keeps its source check (`02` §B.5, D38, `10` T68).
6. **The previous filters:** whether and how Level A (bridging, the draw stratified on its axis,
   the band, the appeal), Level B (the pilot, IRT, DIF, contested facts, re-validation), D39's
   coordination detector and D40's panel constraint keep a role.
7. **A2 and Phase 1's completion criteria:** whether the new design needs a whole-test guarantee
   and which, and what completes Phase 1 once the mechanism is implemented (`10` §1.5's
   qualification, `15`).

## 5. Open work

- **The new mechanism:** the decisions in §4, then its specification and implementation as a
  separate assignment, adapting `21` §2. New task ids come with the specification (`10`
  Priorities).
- **A2 (high), open:** the old requirement's disposition and the guarantees still to decide for
  the new design (`15`, A2's D47 entry). Certifier, other obligations and experiments suspended.
- **Suspended with the previous design:** A1's O1/O2 premises and T1, A12's candidate and
  procedures, A10's residual, exploration, B-b, S2, the forecast path's Level C calibration.
- **Unchanged:** `08` PRIV-004.1 (an inclusion proof's confidentiality, remediation open); B1–B3;
  sample floors, burden and runtime budgets (B4–B5, C3, D2, D4); T83's operating point (B7, D1,
  D6, E); A4 (d); DIF-007's metric deviation, AT-DIF-08's adversarial batch, the optimizer's
  status on the inputs already reported; C2, C4; T25's Level A and B steps; T82's re-run.
- **Phase 2** runs on its own branch (`feat/phase2-runtime`) and is not touched here.

## 6. Essential reading to resume

`CLAUDE.md` and `docs/CLAUDE.md`; `docs/01` D44–D48; `docs/21`; `docs/15` (How to read, the D47
disposition, A2's current state); `docs/02` (its Reading status, Levels A and B, §C.1, and
§§C.2–C.4 as the previous design's code); `docs/05`; `docs/10` Priorities; the archive's index
for the previous design's material. Code: `crates/protocol/src/{review,lifecycle,orchestrator,
probation,honeypot,exploration,results}.rs`, `crates/scoring/src/{reputation,bridging}.rs`.
