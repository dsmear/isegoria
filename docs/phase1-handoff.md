# Phase 1 work — handoff note

Working note, not normative. Decisions belong in [01](01-decisions.md), the contract in
[05](05-question-lifecycle.md#the-answer-review-01-d49d52), evidence in
[15](15-phase1-review.md), and task status in [10](10-roadmap.md).

## 1. Where things stand (2026-10-21)

**T87 and Phase 1 are closed only for the offline answer-review path:** deposit and quotas,
draw, commit-reveal, closure, outcome, author reputation from the next epoch and usable set.
The [closure record](15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
is the authority for review evidence and exclusions. D53 confirms pool membership as
availability, drawing after the deposit epoch closes, and permanent unavailability when
that epoch's pool contains fewer than 12 reviewers.

Parameters are provisional and uncalibrated; the new decoder has not been fuzzed.
There is no certification of truth, whole-quiz fairness or D46's assessment objectives.
The state and log contain keys in clear; the caller is trusted not to select the beacon
for a desired panel. Read the [threat model](06-threat-model.md) before making stronger claims.

## 2. Roles and working agreement

- The owner decides product trade-offs and assigns execution and publication; the
  workflow does not require a particular implementation agent.
- Findings close through a recorded review, not merely green tests. Preserve the
  distinction between reading, proving, calculating and executing, and name who ran checks.
- Long runs, fits and campaigns need explicit authorization. Documentation changes need
  source and link checks, not Rust tests. Follow [agent instructions](CLAUDE.md).
- Keep one authoritative description per subject. Remove obsolete explanations and
  duplicate summaries; link to the contract or register. Preserve unique historical
  evidence and decision records within their original scope.

## 3. Closed findings, within their perimeters

The current states and their evidence are in [15](15-phase1-review.md). Earlier closures
are neither reopened nor extended by the answer review. In particular, A1 and A12's
conditional results do not certify this mechanism; A2 remains unresolved in its own scope.

## 4. Decisions taken, and what the contract leaves open

D49–D53 govern this implementation. Use [05](05-question-lifecycle.md#the-answer-review-01-d49d52)
for the lifecycle and implementation choices, [02 §C.1](02-scoring-engine.md#c1-author-score-c_a)
for the author index, and [14](14-parameter-register.md#the-answer-review--provisional-uncalibrated)
for parameter status. Do not restore earlier filters, source requirements, reviewer weights,
or retries as if they were missing parts of the approved contract.

## 5. Open work

Outside the closure: calibration, decoder fuzzing, any later reviewer evaluation or new
round policy, quiz composition, exposure and retirement. Each requires its own assignment;
product changes require an owner decision. Network and production are outside this work.

The previous design's suspended work and remaining findings are recorded in
[15](15-phase1-review.md), [10](10-roadmap.md) and the
[D47 inventory](21-d47-inventory.md). No experiments are authorized by this note.

## 6. Essential reading to resume

Read [00](00-overview.md), [CLAUDE](CLAUDE.md), D49–D53 in [01](01-decisions.md), the
answer-review section of [05](05-question-lifecycle.md#the-answer-review-01-d49d52), and
the closure record in [15](15-phase1-review.md). Code: `crates/protocol/src/answer*.rs`,
`crates/protocol/tests/answer_review.rs` and the weighted author index in
`crates/scoring/src/reputation.rs`.
