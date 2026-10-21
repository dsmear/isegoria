# Documentation

Start with the [overview](00-overview.md). The active Phase 1 contract is
[05 — The answer review](05-question-lifecycle.md#the-answer-review-01-d49d52), based on
D49–D53. Its [closure record](15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
limits completion to the offline path. Older mechanisms remain documented for their code
and evidence; they are not additional acceptance gates for this path.

## Current contract and work

| Document | Responsibility |
|---|---|
| [00 — Overview](00-overview.md) | Plain-language explanation and principles |
| [01 — Decisions](01-decisions.md) | Owner decisions and their chronology; D45–D53 govern the current direction |
| [05 — Lifecycle](05-question-lifecycle.md) | Answer-review contract first, separate forecast-and-pilot contract afterwards |
| [02 — Scoring](02-scoring-engine.md) | Author-index mathematics (§C.1); retained earlier engine and its scoped guarantees |
| [06 — Threat model](06-threat-model.md) | Current protections, assumptions and accepted risks |
| [14 — Parameters](14-parameter-register.md) | Provisional values, calibration status and procedures by mechanism |
| [15 — Review](15-phase1-review.md) | Findings, evidence and closure scopes |
| [10 — Roadmap](10-roadmap.md) | Task status; completion is qualified by perimeter |
| [Handoff](phase1-handoff.md) | Resume work without duplicating the specification |
| [Agent instructions](CLAUDE.md) | Invariants and working rules |
| [Glossary](99-glossary.md) | Terms used by current and earlier mechanisms |

## Components and assurance

| Document | Responsibility |
|---|---|
| [Architecture](../ARCHITECTURE.md) | Design-to-code map and test strategy |
| [03 — Identity](03-identity-enrollment.md) | Enrollment and pseudonym requirements, assumptions and gaps |
| [04 — Storage and network](04-storage-network.md) | Separate network design; outside the offline closure |
| [07 — Assurance](07-verification-and-assurance.md) | Verification methodology |
| [08 — Audit](08-formal-specification.md) | Dated audit and remediation records, including the added answer-review matrix |
| [11 — Mutation testing](11-mutation-testing.md) | Scoped test-quality evidence |
| [12 — Panic audit](12-panic-audit.md) | Scoped panic and fuzzing evidence; not a fuzzing claim for the new decoder |

## Earlier design and research

- [13 — Characterization](13-characterization.md): study inputs, versions and measurements.
- [18 — A3](18-a3-latent-shape-design.md): the closed latent-histogram correction.
- [21 — D47 inventory](21-d47-inventory.md): retained material, dependencies and archive map.
- [Archive](../archive/phase1-forecast-pilot/README.md): retired dossiers, tools and results;
  documents 16, 17, 19 and 20 in this directory are redirects.
- [Simulations](../sim/README.md): historical prototypes and scoped numerical oracles.
- [Paper](../paper/README.md): analysis of the earlier forecast-and-pilot design,
  with distinct PDF and source versions; not a validation of the answer review.

Historical results retain their hypotheses and scope. They are not measurements of the
new mechanism, and their open findings are not closed by T87.
