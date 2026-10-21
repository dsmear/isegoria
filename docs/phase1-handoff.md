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
author's reputation** stays separate; D48 left its rewards and penalties undefined, and D50
defines them for the answer review (below); the existing path's author score is unchanged.

**The answer review's contract — [`01` D49–D52](01-decisions.md), approved by the owner.** The
decisions §4 listed are taken for the offline path of the first implementation:
- *D49, the review.* The author supplies question, context, alternatives and key, with no source
  and no source check. 12 distinct reviewers are drawn at random from the pool, with equal
  weights and no reputation update. A reviewer answers an alternative, "none is correct" or "don't
  know"; the key is not shown before the commitment, and commit-reveal stays. At least 9 usable
  answers (alternatives and "none"); abstentions and absences are outside the denominator and the
  quorum, with no penalty and no replacement. The round closes by an explicit event; no deadline,
  early closure or retry. A pool too small for 12 gives an explicit unavailability. Outcome, with
  `V` usable and `C` coinciding with the key: `V < 9` insufficient participation; `4C ≥ 3V`
  accepted; `4(V − C) ≥ 3V` discarded; divided answers otherwise. Only accepted questions are
  usable; no new rounds.
- *D50, the author's reputation.* §C.1's index and aging with adapted inputs: accepted 1 at
  weight 1, discarded 0 at weight 1, divided 0 at weight 1/4, insufficient participation no
  observation; the weight multiplies the time weight in both sums; prior 0.40, `H = 45`. Outcome
  and contribution recorded together, once per question; counted from the next epoch.
- *D51, the quotas.* Fixed once per epoch: 1 below 0.30, 2 below 0.70, 3 from 0.70; a new author
  2. A deposit consumes a slot; an invalid or duplicate request does not; no refund, no
  accumulation, no change within the epoch. Logical epochs closed explicitly.
- *D52, the perimeter.* Consensus is the operational criterion; bridging, the pilot, IRT, DIF
  and the source check are not conditions of acceptance; the coordination detector changes
  nothing automatically. A2 stays unresolved in its scope, and its whole-test guarantee is not
  required for the answer review; no fairness of the whole quiz is guaranteed. The parameters are
  provisional and uncalibrated. Phase 1 is not complete before Astra's final review.

Agreement with the key or the majority is not demonstrated reliability; divided answers are not
demonstrated ambiguity. The exclusion of a Schelling point (D6) stands.

**What the code does.** The answer review is implemented offline as `10` T87, beside the
existing path, which it does not change: `scoring::reputation::author_score_weighted` (`3d88b9b`),
`protocol::{answer, answer_rule}` (`9427158`), `protocol::answer_author` (`6908992`),
`protocol::{answer_path, answer_events, answer_node}` (`f230edf`); its technical choices are in
`05` §The answer review. *Verified (executed, E):* AT-ANS-01–13 in `tests/answer_review.rs`,
`scoring/tests/author_weighted.rs` and the modules' unit tests; the existing tests of the shared
code (`inv10_beacon_seed`, `draw_order`, `node_replay`, `level_c`); fmt, workspace clippy with and
without `calibration`, the comment budget. *Not run:* the full suites (they start fits), fits,
benchmarks, campaigns, fuzzing. *Not reviewed:* Astra's review is pending. A reviewer of the
existing path commits and reveals a probability that the item passes the pilot; it is the
bridging rating and the forecast that `scoring::reputation`, `protocol::{honeypot, exploration,
probation, panel_scores, cohort_scores, affine_weights}` score and weigh (`02` §§C.2–C.4).
[`21`](21-d47-inventory.md) §2 lists that code.

**The reorganization (on `docs/phase1-review-alignment`):** `9cbb446` records D47; `c158670` moves
the previous design's dossiers (`16`, `17`, `19`, `20`, the previous handoff), A12's and A2's tools
and the nine A2 reports to
[`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md), renames only;
`76f12f5` adds the archive's index and path map, the inventory `21`, the redirect pages and the link
targets; `990fe9d` and `72237c5` realign the specification and the registers; `47a1698` the front
pages, the index, this note and the agent instructions; `17524b0` and `0b4f4ff` rectify them after
Astra's reviews. D48 (`9afdb09`) and D49–D52 (`45e2b5b`) are recorded on top of them, and T87
implements the answer review after them. No result was regenerated; no fit, benchmark or
campaign was run.

**Residual separations** (`21` §4): the A2 harness stays in `crates/characterization` because
`a2-probe run` guards its provenance on `crates/`; `scoring::latent::latent_dif_candidates`;
`protocol::cohort_snapshot`, which C6's evidence reads; the A12 golden tables in
`crates/protocol/tests/fixtures/`; the S2 and A1 test code, which exercises current code; the
paper, whose `scripts/common.py` feeds CI's differential oracles.

**Phase 1 is not complete.** A2 is open and not resolved by the archiving or by D52, which only
makes its whole-test guarantee unnecessary for the answer review
([`15`](15-phase1-review.md#a2-current-state-design-decision-deferred), D47 and D52 entries).

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

## 4. Decisions taken, and what the contract leaves open

D48's list of decisions still needed is settled for the answer review by D49–D52 (§1): the drawn
group, quorum and absences (D49; `10` T58's no-show rule is not carried over); thresholds and
outcomes (D49); "don't know" (D49, an abstention); the author's reputation (D50) and quotas
(D51); the source at deposit (D49: none, no check); the previous filters (D52: not conditions of
acceptance); A2 (D52: not required for the answer review, still unresolved) and Phase 1's
completion (D52: not before Astra's final review).

Not settled by the contract, and not to be settled in code with a policy of one's own:
- a policy of new rounds, or what follows a round with insufficient participation or divided
  answers beyond its record (D49 excludes new rounds from this implementation);
- what the usable set feeds (quiz assembly, exposure, retirement, re-validation) — the contract
  stops at the set;
- a later evaluation of the reviewers (D48), and any calibration of the provisional parameters
  (D52, `14`).

Choices T87 made where the contract is silent, to confirm or correct in review (`05` §The
answer review, *Implementation*):
- *A pool too small is final for that question.* The draw reads the pool as it was when the
  deposit's epoch closed and that epoch's beacon value, so that nobody can steer it by joining or
  by timing; with fewer than 12 the round is `Unavailable` for good — no redraw (D49 excludes
  retries), the author's slot spent (no refund, D51) and no observation. Whether such a question
  deserves another treatment is a product choice.
- *A question is drawn only after its deposit epoch closes*, since the beacon value is recorded at
  the closure: a round runs at the earliest in the epoch after the deposit.
- *The pool* is the set of reviewers who joined under their `Judge` pseudonym; nobody leaves it,
  and nothing records availability per epoch. "Available" reads as "joined by the closure".
- *Well-formedness:* non-empty context and text, 2–256 alternatives, none empty, no two equal, the
  key one of them. A duplicate is the same content id, key included: a question that changes only
  its key is a new question.
- *Commit-reveal* has an explicit end of commits before the reveals and the closure; a reviewer who
  commits and does not reveal is an absence.
- *The usable set* takes an accepted question at its closure; a new author is one with no
  observation counting in the epoch; epochs are numbered from 0, one closing each.
- *Not addressed offline:* the key is in clear in the log, so its confidentiality from reviewers
  in a distributed log is not addressed; the beacon value is the caller's input, its formation
  outside the offline path; the deposit's admission lottery (D10) is not part of the path.

## 5. Open work

- **T87, the answer review offline** (`10`): implemented with its targeted tests (above); Astra's
  review of the commits from `45e2b5b` to the branch's tip, and of the choices above, is next.
  Phase 1 is not declared complete before it (D52).
- **A2 (high), open:** unresolved within its scope; not required for the answer review (D52).
  Certifier, other obligations and experiments suspended.
- **Suspended with the previous design:** A1's O1/O2 premises and T1, A12's candidate and
  procedures, A10's residual, exploration, B-b, S2, the forecast path's Level C calibration.
- **Unchanged:** `08` PRIV-004.1 (an inclusion proof's confidentiality, remediation open); B1–B3;
  sample floors, burden and runtime budgets (B4–B5, C3, D2, D4); T83's operating point (B7, D1,
  D6, E); A4 (d); DIF-007's metric deviation, AT-DIF-08's adversarial batch, the optimizer's
  status on the inputs already reported; C2, C4; T25's Level A and B steps; T82's re-run.
- **Phase 2** runs on its own branch (`feat/phase2-runtime`) and is not touched here.

## 6. Essential reading to resume

`CLAUDE.md` and `docs/CLAUDE.md`; `docs/01` D44–D52; `docs/21`; `docs/15` (How to read, the D47
disposition, A2's current state); `docs/02` (its Reading status, Levels A and B, §C.1, and
§§C.2–C.4 as the previous design's code); `docs/05`; `docs/10` Priorities; the archive's index
for the previous design's material. Code: `crates/protocol/src/{review,lifecycle,orchestrator,
probation,honeypot,exploration,results}.rs`, `crates/scoring/src/{reputation,bridging}.rs`.
