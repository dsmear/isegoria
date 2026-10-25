# Overview

Isegoria's current Phase 1 product is an **offline process for reviewing quiz
questions**, ending with a set of accepted questions. It does not yet compose or
administer a quiz, measure respondents' competence, or operate as a distributed service.

## How it works

1. An author deposits a question, context, alternatives and a **key**: the alternative
   the author states is correct. The key is a reference for counting answers, not a fact
   established by the program. No source is required at deposit.
2. After the deposit epoch closes, the program draws 12 distinct reviewers from the
   registered pool. Joining means being available; reviewers have equal weight.
3. Reviewers see the question without the key. They may choose an alternative, say
   that none is correct, or abstain with "don't know". They commit before revealing.
4. The round closes explicitly. At least 9 usable answers are needed. With that quorum,
   agreement with the key by at least three quarters accepts the question; disagreement
   by at least three quarters discards it; otherwise the answers are divided. "None"
   counts as disagreement; abstentions and absences count in neither quorum nor denominator.
5. Only accepted questions enter the usable set. The recorded outcome contributes to
   the author's reputation from the next epoch, governing their proposal quota.

There is no automatic correction of the key or new review round: a correction requires
a new question. With fewer than 12 reviewers in the pool fixed at the deposit epoch's
closure, the question stays unavailable, its slot spent and no reputation observation made.
The [lifecycle specification](05-question-lifecycle.md#the-answer-review-01-d49d52)
defines the full contract, including counts, quotas, duplicates and closure.

## People and code

People judge factual correctness, clarity and relevance to the stated context. Internet,
documents and AI are part of the open environment: reviewers do not report their sources,
and the system does not detect, infer or assess their use of outside tools (D45, D47–D49).

The code authenticates actions under role pseudonyms, draws reviewers from the fixed pool,
checks commitments and state transitions, counts answers, records outcomes and computes
reputation and quotas. It neither evaluates sources nor performs automatic fact-checking.
Pool suitability is a human premise; random selection does not establish expertise.

The author's reputation summarizes review outcomes under the adopted rule. It is not
a direct measure of truth or competence. Reviewers receive no reputation update or weight
for agreeing with the key or the majority; their future evaluation remains deferred.

## Principles and limits

- **Consensus is the operational criterion, not a truth guarantee.** Shared mistakes,
  unequal expertise, superficial reviews and coordination are accepted risks (D48).
  Divided answers do not prove ambiguity; insufficient participation is not rejection.
- **No financial stake and no personal-attribute profiling.** Author and reviewer roles
  use separate pseudonyms; author reputation governs quotas, not reviewer influence.
  End-to-end uniqueness and anonymity depend on the identity layer's assumptions and
  limits, not on the answer-review algorithm alone ([identity](03-identity-enrollment.md)).
- **Reproducible computation.** Recorded inputs determine the result. The caller must
  supply the random input without choosing it to obtain a desired panel. The offline
  state and log contain the key in clear; the reviewer view does not protect against
  someone who reads them ([threat model](06-threat-model.md)).
- **Assessment goals remain goals.** D46 seeks factually correct answers, including
  assisted answers, and the ability to weigh claims against their evidence. This path
  neither measures nor validates those abilities. Correctness alone establishes no
  unaided knowledge; item acceptance establishes no balanced coverage of a whole quiz.

These are the current principles after the owner's revisions in
[D44–D53](01-decisions.md). D48 changed the original ban on deciding by votes;
D52 excludes bridging and empirical validation from this path's acceptance conditions.
These choices govern the current path; the future direction is described below.

## Future direction

The owner reaffirmed on 2026-10-25 that future versions should seek to apply two
principles from the earlier design:

- **Agreement across different perspectives (bridging).** Investigate how to recognize
  support that crosses differences in viewpoint and reduce the dominance of the largest
  group. For example, agreement among people who often disagree may be more informative
  than agreement within a single bloc. It still does not establish factual truth.
- **Empirical validation (psychometrics).** Study actual responses to understand what
  questions and whole quizzes measure, how consistently they do so, and where they may
  introduce distortions. For example, a question everyone answers correctly offers
  little information about differences in performance; a set of individually acceptable
  questions can still give an unbalanced picture of a subject.

These are long-term research and design goals. The retained code, studies and open
findings are starting points for evaluating and adapting methods to the current design.
Their earlier results do not establish that they work with the new review process.

Any proposal must account for the open use of Internet, documents and AI, preserve
anonymity without collecting personal or demographic profiles, and distinguish observed
performance from unaided knowledge. Adoption requires a defined role in the process,
appropriate evidence and calibration, and an explicit decision on any changed rules.

This direction leaves the current answer-review contract and its closure unchanged.
It approves no particular algorithm, integration schedule or additional acceptance gate.

## Completion and retained work

Phase 1 and T87 are closed only within the offline answer-review perimeter. The
[review register](15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
records the checks and exclusions; the [handoff](phase1-handoff.md) identifies work outside it.
Parameters remain provisional and uncalibrated, and the decoder has not been fuzzed.

The earlier forecast-and-pilot path remains implemented with its own contracts in
[02](02-scoring-engine.md) and [05](05-question-lifecycle.md). Its findings and simulations
retain their original scopes. A2 remains unresolved there and is not required for the
answer review. The [paper](../paper/README.md) is historical; the
[D47 inventory](21-d47-inventory.md) explains what was retained and archived.
