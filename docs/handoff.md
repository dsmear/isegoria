# Handoff

A working note to resume work, not a specification. Decisions belong in
[01](01-decisions.md), contracts in [03](03-identity-enrollment.md),
[04](04-storage-network.md) and [05](05-question-lifecycle.md), task status in
[10](10-roadmap.md), gate records in [24](24-milestone2-trial.md), findings and evidence in
[15](15-phase1-review.md). The codes are explained in the [glossary](99-glossary.md#project-codes).

## 1. Where things stand (2026-11-25)

**Phase 2 — the network.** The target is a small experimental network of nodes run by
independent operators, with the answer review running on it (decision D55). The members'
nodes, the relays, the participants' client and the credential-issuing committee's tool are
built. The trial has three gates:

1. *One operator, one machine:* passed and accepted on 11 November 2026
   ([record](24-milestone2-trial.md#gate-1--controlled-processes-on-one-machine-2026-11-11)).
2. *Separate machines and networks, with faults:* not run. It starts after the fuzzing of
   every decoder that reads another machine's bytes (D62, task T89). Each such decoder has
   a fuzz target with a 60-second run and no crash ([12](12-panic-audit.md) §4); the
   roadmap does not yet record that part as done, and longer runs need the owner's
   authorization. Its rehearsal in network namespaces on one machine passed (`05` §Gate 2's
   rehearsal); the four machines' [runbook](23-gate2-runbook.md) is written, not run.
3. *Independent operators,* with the committee's issuer key: not run.

The work order, what each task still needs and the gates' conditions are in
[10 §2](10-roadmap.md#phase-2--p2p-network). Nothing here claims production readiness: the
trial assumes members follow the rules, and its beacon and recovery limits stand
([threat model](06-threat-model.md#phase-2-network-boundary)).

**Phase 1 — the offline answer review** is closed only within its offline path: deposit
and quotas, the draw, commit and reveal, closure, outcome, the author's reputation from the
next epoch and the usable set. The
[closure record](15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
is the authority for its evidence and exclusions. Decision D53 confirms pool membership as
availability, the draw after the deposit epoch closes, and a question unavailable for good
when that epoch's pool holds fewer than 12 reviewers.

Its parameters are provisional and uncalibrated, and the decoder of its events has had only
short fuzzing runs. Nothing certifies truth, the fairness of a whole quiz or the assessment
objectives of D46. The state and log hold keys in clear, and whoever supplies the beacon is
trusted not to pick it for a desired panel. Read the [threat model](06-threat-model.md)
before making a stronger claim.

## 2. Roles and working agreement

- The owner decides product trade-offs and assigns execution and publication; the workflow
  needs no particular implementation agent.
- A finding closes through a recorded review, not merely green tests. Keep reading,
  proving, calculating and executing distinct, and name who ran each check.
- Long runs, fits and campaigns need explicit authorization. Documentation changes need
  source and link checks, not Rust tests. Follow the [agent instructions](CLAUDE.md).
- Keep one authoritative description per subject, remove obsolete explanations and
  duplicate summaries, and link to the contract or register. Preserve unique historical
  evidence and decision records within their original scope.

## 3. Decisions taken, and what they leave open

D49–D53 govern the answer review. Use [05](05-question-lifecycle.md#the-answer-review-01-d49d52)
for its lifecycle and implementation choices, [02 §C.1](02-scoring-engine.md#c1-author-score-c_a)
for the author index and [14](14-parameter-register.md#the-answer-review--provisional-uncalibrated)
for the parameters' status. D54–D68 govern the network and the trial; their contracts are in
`04` §Operating contract, `05` §Distributed integration and `03` §Committee setup for the trial.

Do not restore earlier filters, source requirements, reviewer weights or retries as if they
were missing parts of the approved contract. Earlier closures are neither reopened nor
extended by the answer review: the conditional results on the earlier evaluator score (A1,
A12) do not certify it, and the earlier whole-test bias guarantee (A2) stays unresolved in
its own scope ([15](15-phase1-review.md)).

## 4. Open work

Outside the Phase 1 closure: calibration, any later reviewer evaluation or new-round policy,
quiz composition, exposure and retirement. Each needs its own assignment; a product change
needs an owner decision. Phase 2's work neither reopens that closure nor authorizes
production. The earlier design's suspended work and remaining findings are in
[15](15-phase1-review.md), [10](10-roadmap.md) and the [inventory](21-d47-inventory.md).
This note authorizes no experiment.

Before proposing reviewer weights, peer prediction, coordination discounts or a label-free
whole-test guarantee, read the [feasibility survey](22-phase1-feasibility-survey.md)
(2026-11-11, its sources re-read 2026-11-15): informative, not reviewed, it decides nothing.

## 5. What to read to resume

The [overview](00-overview.md), the [agent instructions](CLAUDE.md), the decisions D49–D53
and, for the network, D54–D68 in [01](01-decisions.md), the answer review's section of
[05](05-question-lifecycle.md#the-answer-review-01-d49d52) and the closure record in
[15](15-phase1-review.md); for the trial, [10 §2](10-roadmap.md#phase-2--p2p-network),
[24](24-milestone2-trial.md) and the [runbook](23-gate2-runbook.md).

Code: `crates/protocol/src/answer*.rs` and `crates/protocol/tests/answer_review.rs` for the
review, the weighted author index in `crates/scoring/src/reputation.rs`, and for the network
the crates `node`, `client`, `ceremony` and `trial` ([architecture](../ARCHITECTURE.md)).
