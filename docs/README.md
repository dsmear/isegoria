# Documentation

Start with the [overview](00-overview.md): what the project does, its principles and where
it stands. The [glossary](99-glossary.md) explains the terms and the project's codes
(decisions `D…`, tasks `T…`, findings `A…`, tests `AT-…`). To resume work, read the
[handoff](handoff.md).

Each subject has one authoritative document; the others link to it. Task status is only in
the roadmap, gate records only in the trial records, review findings only in the review
register.

## The current contract and work

| Document | What it holds |
|---|---|
| [00 — Overview](00-overview.md) | The project in plain language: how the review works, principles, limits, state |
| [01 — Decisions](01-decisions.md) | The owner's decisions, with an index of which are current and which were replaced |
| [05 — Lifecycle](05-question-lifecycle.md) | The answer review's contract, its integration between nodes, then the earlier forecast-and-pilot path |
| [02 — Scoring](02-scoring-engine.md) | The author index's mathematics (§C.1); the earlier engine and its scoped guarantees |
| [06 — Threat model](06-threat-model.md) | Current protections, assumptions and accepted risks |
| [14 — Parameters](14-parameter-register.md) | Provisional values, calibration status and procedures |
| [15 — Review register](15-phase1-review.md) | Review findings, their evidence and the scope of each closure |
| [10 — Roadmap](10-roadmap.md) | Task status, the work order and the milestones' gates |
| [24 — Milestone 2 trial](24-milestone2-trial.md) | The record of each trial gate run against a pinned revision |
| [23 — Gate 2 runbook](23-gate2-runbook.md) | What each of the owner's four machines does in gate 2; written, not run |
| [Handoff](handoff.md) | Where things stand and what to read to resume |
| [Agent instructions](CLAUDE.md) | Invariants and working rules |
| [Glossary](99-glossary.md) | The project's codes and where each is defined; terms of current and earlier mechanisms |

## Components and assurance

| Document | What it holds |
|---|---|
| [Architecture](../ARCHITECTURE.md) | Which crate implements what, and the test strategy |
| [03 — Identity](03-identity-enrollment.md) | Enrollment and pseudonyms; the trial's credential-issuing committee |
| [04 — Storage and network](04-storage-network.md) | Signed logs, replication, cuts, the beacon, relays, the node process, the operators' contract and the trial's procedure |
| [25 — Public archive](25-public-archive.md) | The public history in shards (frozen, outside Milestone 2) |
| [07 — Assurance](07-verification-and-assurance.md) | The verification methodology |
| [08 — Audit](08-formal-specification.md) | The formal audit, its claims, tests (§12) and claim/evidence matrix (§15), with later remediation |
| [11 — Mutation testing](11-mutation-testing.md) | Mutation-testing runs and their scope |
| [12 — Panic audit](12-panic-audit.md) | Panic classes and fuzz targets; Phase 2's targets have had short runs only |

## Earlier design and research

- [13 — Characterization](13-characterization.md): the earlier engine's studies, inputs,
  versions and measurements.
- [18 — A3](18-a3-latent-shape-design.md): the closed correction of the latent ability histogram.
- [21 — Inventory](21-d47-inventory.md): what the change to the answer review kept, adapted
  and archived.
- [22 — Feasibility survey](22-phase1-feasibility-survey.md): what the literature says of the
  archived paper's objectives and of the answer review's margin; informative, not reviewed.
- [Archive](../archive/phase1-forecast-pilot/README.md): retired dossiers, tools and results;
  documents 16, 17, 19 and 20 here only redirect to it, for the code comments that cite them.
- [Simulations](../sim/README.md): historical prototypes and scoped numerical oracles.
- [Paper](../archive/phase1-forecast-pilot/paper/README.md): the archived analysis of the
  earlier design, with its sources, data and scripts; not a validation of the answer review.

Historical results keep their hypotheses and scope. They are not measurements of the answer
review, and their open findings are not closed by its closure.
