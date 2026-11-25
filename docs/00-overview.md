# Overview

Isegoria is a research prototype for building a collection of quiz questions that people
propose and review together, with rules anyone can re-check. Its working core is the
**answer review**: an author proposes a question, randomly drawn reviewers answer it, and
a fixed rule on their answers decides whether the question enters the usable set.

It does not yet compose or administer a quiz, measure respondents' competence, or run as a
service operated by independent parties. The review runs offline on one computer (Phase 1,
closed) and is being brought onto a network of nodes (Phase 2, in progress).

## How the answer review works

1. An author deposits a question, its context, the alternatives and a **key**: the
   alternative the author states is correct. The key is a reference for counting answers,
   not a fact established by the program. No source is required at deposit.
2. After the deposit period (an *epoch*) closes, the program draws 12 distinct reviewers
   from the registered pool. Joining the pool means being available; every reviewer has
   equal weight.
3. Reviewers see the question without the key. They may choose an alternative, say that
   none is correct, or abstain with "don't know". Each first commits to an answer in sealed
   form, then reveals it, so nobody can copy an answer already shown.
4. The round closes explicitly. At least 9 usable answers are needed. With that quorum,
   agreement with the key by at least three quarters accepts the question; disagreement by
   at least three quarters discards it; otherwise the answers are divided. "None is correct"
   counts as disagreement; abstentions and absences count in neither the quorum nor the share.
5. Only accepted questions enter the usable set. The outcome counts towards the author's
   reputation from the next epoch, and that reputation sets how many questions the author
   may propose.

The program never corrects the key or opens a new round: a correction is a new question.
If the pool fixed when the deposit epoch closes has fewer than 12 reviewers, the question
stays unavailable for good, its proposal slot spent and no reputation recorded. The full
contract, with counts, quotas, duplicates and closure, is the
[lifecycle specification](05-question-lifecycle.md#the-answer-review-01-d49d52).

## People and code

People judge factual correctness, clarity and relevance to the stated context. The
Internet, documents and AI are part of the open environment: reviewers do not report their
sources, and the system does not detect, infer or assess their use of outside tools.

The code authenticates each action under a role pseudonym, draws reviewers from the fixed
pool, checks commitments and state transitions, counts answers, records outcomes and
computes reputation and quotas. It neither evaluates sources nor checks facts. Whether the
pool suits the questions is a human premise; a random draw does not establish expertise.

The author's reputation summarizes review outcomes under the adopted rule; it is not a
measure of truth or competence. Reviewers gain no reputation or weight for agreeing with
the key or with the majority; how to evaluate reviewers over time is deferred.

## Principles and limits

- **Consensus is the working criterion, not a guarantee of truth.** Shared mistakes,
  unequal expertise, superficial reviews and coordinated answers are accepted risks.
  Divided answers do not prove a question ambiguous; too few answers is not a rejection.
- **No money at stake, no profiling.** Taking part requires no token or deposit, and no
  personal or demographic attribute is collected. Author and reviewer roles use separate
  pseudonyms; the author's reputation governs quotas, not influence as a reviewer.
  Uniqueness and anonymity end to end depend on the identity layer's assumptions and
  limits, not on the review alone ([identity](03-identity-enrollment.md)).
- **Reproducible computation.** The recorded inputs determine the result. Whoever supplies
  the random input must not choose it to obtain a desired panel. The offline state and log
  hold the key in clear: hiding it from reviewers does not protect it from anyone who reads
  them ([threat model](06-threat-model.md)).
- **Assessment goals remain goals.** The project seeks factually correct answers, assisted
  ones included, and the ability to weigh claims against their evidence. The answer review
  neither measures nor validates those abilities: a correct answer does not show unaided
  knowledge, and accepting each question does not make a whole quiz balanced.

The owner's decisions of October 2026 set these principles. They replaced the original ban
on deciding by votes with the review's consensus rule, and removed the earlier bridging and
empirical-validation filters from the review's acceptance conditions
([decisions](01-decisions.md), from D44).

## The network (Phase 2)

The goal is a small experimental network whose nodes are run by **independent operators**,
each holding its own keys, on which the answer review runs from deposit to the author's
next quota. A defined group of members agrees on the order of events by co-signing it;
anyone else reaches them through relays that forward sealed messages they cannot read.
This is not a public blockchain and involves no money.

The members' nodes, the participants' tool and the committee that issues credentials are
built. The trial has three gates: one operator on one machine; separate machines and
networks with faults; independent operators. **Only the first has passed** (11 November
2026). Gate 2 follows the fuzzing of the code that reads other machines' messages; the
four machines' runbook is written but not run. The trial makes no production claim: the
members are assumed to follow the rules, the shared randomness can still be biased or
stalled by a member who withholds, and members read the review's private data. The
[roadmap](10-roadmap.md#phase-2--p2p-network) holds the work order and the gates, the
[trial records](24-milestone2-trial.md) what each run showed.

## Future direction

The owner reaffirmed on 25 October 2026 that future versions should seek to apply two
principles of the earlier design:

- **Agreement across different perspectives (bridging).** Recognize support that crosses
  differences in viewpoint and reduce the dominance of the largest group. Agreement among
  people who often disagree may say more than agreement within one bloc. It still does not
  establish factual truth.
- **Empirical validation (psychometrics).** Study actual responses to learn what questions
  and whole quizzes measure, how consistently, and where they distort. A question everyone
  answers correctly says little about differences in performance; a set of individually
  acceptable questions can still give an unbalanced picture of a subject.

These are long-term goals. The retained code, studies and open findings are starting
points; their earlier results do not show that the methods work with the answer review.
A proposal must allow the open use of the Internet, documents and AI, keep anonymity
without personal or demographic profiles, and tell observed performance from unaided
knowledge. Adopting one needs a defined role in the process, evidence and calibration, and
an explicit decision on any changed rule. This direction changes neither the current
contract nor its closure, and approves no algorithm, schedule or extra acceptance gate.

## Completion and what was retained

Phase 1 is closed only for the offline answer review; the
[review register](15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
records its checks and exclusions, and the [handoff](handoff.md) the work outside it. The
numerical parameters are provisional and uncalibrated, and the decoder of the review's
events has had only short fuzzing runs ([panic audit](12-panic-audit.md) §4).

The earlier forecast-and-pilot design stays implemented with its own contracts
([scoring](02-scoring-engine.md), [lifecycle](05-question-lifecycle.md)). Its findings and
simulations keep their original scope; its whole-test bias guarantee remains unresolved
there and is not needed by the answer review. Its [paper](../archive/phase1-forecast-pilot/paper/README.md)
is archived, and the [inventory](21-d47-inventory.md) says what was kept and what archived.

**References.** The answer review: decisions D45–D53, task T87. The network: D54–D68,
tasks T79, T19, T88 and T89. The codes are explained in the [glossary](99-glossary.md#project-codes).
