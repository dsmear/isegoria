# Overview

## The real problem

If the quality of a question is decided by voting, whoever controls the majority of
votes controls the questions: power shifts from the government to the largest or
best-organized faction. A naive reputation system (whoever gets more upvotes rises)
makes things worse, because it concentrates power in a cohesive group faster than
plain voting does.

The design rests on three pillars, in order of importance:

1. **Sybil resistance** — if creating identities is cheap, no reputation math has
   any value. It is the constraint that holds up all the rest; it is solved by the
   enrollment layer (`03`).
2. **Non-opinion quality signals** — question functioning needs evidence beyond
   reviewers' judgments: psychometrics on real answer data (`02`, evidence filter).
   It does not by itself establish truth, relevance or the intended competence.
3. **Bridging aggregation, not majority** — a question is accepted only if it is
   approved *across* the lines of division, not if it collects more votes than the
   opposing front (`02`, bridging).

## Two-level architecture

| | Level A — Opinion | Level B — Evidence |
|---|---|---|
| Input | Peer reviews from nodes | Real answers from samples |
| Question | "Does this question look good?" | "Does this question behave well?" |
| Attackable? | Yes, with coalitions | Very hard: requires corrupting the sample |
| Role | Upstream filter | Final verdict |

Under [`01` D47](01-decisions.md) a reviewer's contribution is an answer to the item, and the
mathematics is to evaluate the reviewers' reliability over time. Whether and how the answers
enter the opinion filter, and how they relate to Level B, is still to define: the two levels
describe the existing mechanism and are not imposed on the new one automatically. The code
still reads a forecast of the pilot outcome as the reviewer's rating.

The system's center of gravity is Level B: peer review decides *which questions are
worth field-testing*, subject to the appeal path. The empirical decision uses statistics
computed on answer data, subject to the declared model and source/admissibility rules;
it is not a majority vote or a semantic truth oracle. Review and evidence have distinct
roles, and neither establishes all of the other's requirements.

## Foundational interpretation

The owner clarified these principles on 2026-10-18; [`01` D44](01-decisions.md)
is their decision record.

- The test's context tells proponents and reviewers what is relevant. The numerical
  engine need not understand political content; the review gate remains bridging.
- Better relevant knowledge may produce a higher score. Equality of raw group scores
  is not the goal; DIF alone does not establish unfairness.
- Sourced contested facts keep their existing path. Whether to compensate differences
  in pertinent factual knowledge remains an explicit A2 decision, not an automatic
  consequence of a statistical flag.
- Item metadata and coverage choices are distinct from personal attributes, which
  remain excluded. Existing blueprint quotas do not prove semantic coverage.
- Test and refine with declared metrics, uncertainty and costs. "75%" is an expression
  of willingness to accept partial progress, not an adopted statistical threshold.
  Estimates, enforced fitted-cost limits and demonstrated guarantees remain separate.

**Open environment (2026-10-20, [`01` D45](01-decisions.md)).** Isegoria does not presume that
reviewers or respondents are isolated from the Internet, documentation, search engines or AI
tools. Answers are produced under those declared conditions, and a correct answer alone does
not show unaided knowledge; reading scores or the model's `θ` as competence, or using them to
attribute weight or authority, needs a separate justification. D45 approves no behaviour
automatically and records the tensions it raises with existing contracts, including Level B's
resistance above.

**Assessment priorities (2026-10-20, [`01` D46](01-decisions.md)).** In that environment the
assessment aims first at (A) factually correct answers, even with assistance, to questions
precise enough, and (B) telling reliable from misleading information by how a specific claim
relates to its evidence. The two stay distinct; no subscore, weight, threshold, format, rubric
or aggregate is adopted, and assisted correctness certifies no unaided understanding.

**Phase 1 direction (2026-10-21, [`01` D47](01-decisions.md)).** The author writes the question
in its context and states the correct answer; reviewers answer it, free to consult any resource;
they do not state or record the sources they consult, and the system does not detect, infer or
assess their use of outside tools. Whether the author keeps attaching a mandatory source at
deposit, and the contested-fact path its source check, is a separate question, still open. The
mathematical mechanism is to process the answers and evaluate the
reviewers' reliability over time, with a possible reputational weight. Forecasting whether an
item passes the pilot is no longer the reviewers' main contribution. Still to define: the
aggregation, the reputation update, the treatment of a contested key, thresholds, outcomes and
guarantees; a balanced split of correct and wrong answers is a signal to study, not a rule, and
agreement with the key or the majority is not demonstrated reliability. D45 and D46 remain the
references for conditions and objectives. The code still implements the previous design, whose
dossiers are archived ([`21`](21-d47-inventory.md)); D47 changes no code.

A2's whole-test guarantee remains open: D47 and the archiving of its dossier neither resolve nor
withdraw it. Its bounded studies, run under
[`19` §13.10](../archive/phase1-forecast-pilot/docs/19-a2-dtf-composition-design.md#1310-owner-direction-foundations-and-iterative-testing-2026-10-18),
keep their domains; whether the new design needs a whole-test guarantee, and which, is still to
decide. No real-world performance level has been established.

## The three actions of a node

Every node (person) can take three distinct actions, each under a **separate
pseudonym not linkable to the others**:

- **Propose** — write a new question.
- **Judge** — evaluate other people's questions during review; under D47, by answering them
  (the code still collects a forecast of the pilot outcome).
- **Answer** — act as a sample for the trial: answer quizzes in which the new
  question is mixed with already-validated ones, without the answer counting toward
  one's own score.

Propose and judge were the two roles envisaged from the start. Answering emerged as
a need of the evidence filter: someone has to answer for a question to be trialled.
It is convenient for the same nodes to do it, provided the system does not know it is
the same person.

## Lifecycle of a question

```
 [1] Draft          author fills in item + mandatory primary source
      ↓
 [2] Deposit        reputation bond (not money) + hash on the log
      ↓
 [3] Review         k reviewers assigned AT RANDOM, blind,
     (peer)         commit-reveal, nobody sees others' votes
      ↓
 [4] Bridging       S_j gate, uncertainty band, supplementary review (`02` §A.3)
      ↓                 │
      │                 └──→ appeal to evidence (if discarded for polarization)
      ↓
 [5] Pilot 1        initial screen, provisional floor ~300 respondents
                    (filter limits measured in `13` §8.7.5)
      ↓
 [6] Pilot 2        sample by format and required power: IRT + latent DIF
                    (in batches, never a single item)
      ↓                 │
      │                 └──→ contested fact: DIF, but the primary source establishes
      ↓                      the key → a separate pool, drawn only in balanced sets
 [7] Active pool    usable; periodic re-validation of the whole pool
      ↓
 [8] Retirement     for exposure, drift, obsolescence, or emerging DIF
```

Under D47 the author states the key at [1] and reviewers answer the item at [3]; whether and
how [4]–[6] read the answers, and whether [1] and [6]'s contested-fact branch keep the author's
source and its check, is still to define (`05`).

Sample floors and capacity are specified in `02` §B.6. They are not universal power
guarantees; the current model still needs post-D43 characterization. Open guarantee
and protocol-contract findings are tracked in [`15`](15-phase1-review.md).

Nobody chooses what to review (prevents brigading). Nobody sees the author (prevents
voting on the person). Nobody sees others' judgments before casting their own
(prevents information cascades).

The **[4]→appeal** branch is the most important correction to emerge from testing: a
true but politically divisive question produces the same voting pattern as one-sided
propaganda, and bridging wrongly discards it. The appeal channel lets the author send
it straight to the pilot, staking their own reputation: a zero-quality observation
enters their average when they file, and the item's real quality replaces it if the
psychometrics promote it (`01` D27). It is the only way to recover the "true but
inconvenient" category, politically the most important. Details in `05`.

## Node types (network layer)

Three infrastructural roles, not to be confused with the three actions of people:

- **Person-nodes** — the pseudonymous participants. Number: unlimited, the more the
  better (security grows with the count).
- **Signer-nodes** (consortium) — a few dozen always-on machines, heterogeneous and
  in different jurisdictions, that keep a full copy and sign the state. Number:
  limited by the coordination cost (~n²). Security comes from the diversity of who
  controls them, not from the number.
- **Light-nodes** — anyone can run one: it keeps only the slice of data it needs and
  verifies signatures without taking part in the agreement. Number: unlimited, slows
  nothing down.

Details in `04`.
