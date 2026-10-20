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
tools. A score is performance under those declared conditions; a correct answer alone does not
show unaided knowledge, and reading the score as competence, or using it to attribute weight or
authority, needs a separate justification. D45 approves no behaviour automatically and records
the tensions it raises with existing contracts, including Level B's resistance above.

A2's whole-test guarantee remains open. The next bounded evaluation work is described
in [`19` §13.10](19-a2-dtf-composition-design.md#1310-owner-direction-foundations-and-iterative-testing-2026-10-18);
no real-world performance level has been established by this decision.

## The three actions of a node

Every node (person) can take three distinct actions, each under a **separate
pseudonym not linkable to the others**:

- **Propose** — write a new question.
- **Judge** — evaluate other people's questions during review.
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
