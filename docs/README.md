# A distributed network for producing and validating quiz questions

A network whose nodes — pseudonymous people — **propose** questions, **judge**
other people's questions, and **answer** quizzes to trial them. It exists to
produce banks of questions (true/false or multiple-choice items) in settings where
**no authority is credibly impartial**: control over the questions is distributed
across all nodes instead of belonging to an institution.

Reference use case: civic questions for a possible weighted vote, where you do not
want the government to control the questions. Less contested, more immediate use
cases: professional certifications independent of training bodies, collaborative
fact-checking, licensing exams removed from the profession that has a stake in them.

## The idea in one line

The quality of a question is **not decided by voting** (whoever holds the majority
would control the questions). It is decided by two filters in a row (the existing mechanism;
[`01` D48](01-decisions.md) rectifies this for the new one, below):

1. **Opinion filter** — aggregated peer review *across the bridge* (bridging): what
   matters is where the approval comes from, not how much of it there is. A question
   approved only by the largest camp is discarded.
2. **Evidence filter** — the question is field-trialled on a sample of respondents;
   psychometric statistics (IRT) and bias analysis (DIF) give the final verdict.
   Here nobody's opinion is needed.

**Phase 1 direction ([`01` D47](01-decisions.md), 2026-10-21).** In review, the author states
the key and reviewers answer the question, free to consult any resource. For the first
implementation [D48](01-decisions.md) decides: reviewers drawn by lot from the reviewer pool,
with equal weights; ordinary reviews do not update their reputation, and the evaluation of their
reliability over time that D47 asked for is deferred; the consensus of their answers is the
review's operational criterion, which rectifies the principle above without making consensus a
guarantee of truth; a reviewer may answer that no alternative is correct; a question is never
corrected retroactively, nor its key replaced automatically. D48 records the risks it accepts
(shared errors, different competences under equal weights, superficial reviews, coordination).
Still to decide: the group's size, quorum and absences, thresholds and outcomes, a possible
"don't know", the author's reputation, the source at deposit, the role of the two filters, A2
and Phase 1's completion criteria ([handoff](phase1-handoff.md) §4). The code still implements
the previous design, in which reviewers forecast whether the item passes the pilot.

The anonymity of the nodes is the non-negotiable base. The uniqueness of a person is
guaranteed by an external enrollment layer (national eID / CIE / SPID / others),
**cryptographically severed** from what the node does.

## Document structure

| File | Contents |
|---|---|
| [`CLAUDE.md`](CLAUDE.md) | Instructions for implementing with Claude Code: work order, invariants not to violate, stack |
| [`00-overview.md`](00-overview.md) | Two-level conceptual architecture, lifecycle of a question |
| [`01-decisions.md`](01-decisions.md) | Every decision taken and every alternative rejected, with rationale (ADR) |
| [`02-scoring-engine.md`](02-scoring-engine.md) | **The mathematical core**: bridging, IRT, DIF, reputation, anti-collusion |
| [`03-identity-enrollment.md`](03-identity-enrollment.md) | Anonymous multi-source enrollment, threshold credentials, role pseudonyms |
| [`04-storage-network.md`](04-storage-network.md) | P2P layer, consortium, erasure coding, anchoring, node types |
| [`05-question-lifecycle.md`](05-question-lifecycle.md) | Step-by-step protocol, appeal to evidence, lottery, honeypot, cold start |
| [`06-threat-model.md`](06-threat-model.md) | Attack surface and countermeasures; what the system does NOT solve |
| [`07-verification-and-assurance.md`](07-verification-and-assurance.md) | Verification, assurance and falsification methodology |
| [`08-formal-specification.md`](08-formal-specification.md) | Independent audit: what is implemented, tested and still open |
| [`10-roadmap.md`](10-roadmap.md) | **Development plan by priority** — mathematics, then P2P network, then the rest (task ids `T#`) |
| [`11-mutation-testing.md`](11-mutation-testing.md) | Mutation-testing report: how much the tests verify, and every accepted survivor |
| [`12-panic-audit.md`](12-panic-audit.md) | Panic audit and fuzzing report: every `unwrap`/`expect`/`assert` classified, every crash on hostile input fixed |
| [`13-characterization.md`](13-characterization.md) | T24 and T25's supplement: the simulation studies that characterize the detectors and gates, the harness that runs them, and their results |
| [`14-parameter-register.md`](14-parameter-register.md) | Every threshold's reason, calibration procedure, sensitivity, failure and location (`07` §13) |
| [`15-phase1-review.md`](15-phase1-review.md) | Current review findings, evidence limits and correction handoff; distinguishes implemented behavior from unresolved guarantees |
| [`18-a3-latent-shape-design.md`](18-a3-latent-shape-design.md) | A3 diagnosis and its correction P1 (closed on `16e862c`): the latent ability histogram's parametrization, the moment penalty, the dimension and the BIC count |
| [`21-d47-inventory.md`](21-d47-inventory.md) | What `01` D47 affects, file by file: kept, to adapt in the next implementation, archived (with motivation and dependencies), still needed as dependencies |
| [`phase1-handoff.md`](phase1-handoff.md) | Working note for resuming Phase 1 work between sessions; not normative |
| [`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md) | **Archived under D47:** the previous design's dossiers — `16` (A1 and A12), `17` (A1's pilot batches, B-b, O1–O3), `19` (A2; **not resolved**), `20` (S2) — the previous handoff, A12's and A2's tools and A2's results, with an index and a path map. `16`, `17`, `19` and `20` here are redirect pages |
| [`99-glossary.md`](99-glossary.md) | Every concept explained from scratch, from the problem to the formula |
| [`sim/`](../sim/) | Historical research prototypes and scoped fixture/differential oracles; see its README for current vs retired paths (A12's and A2's tools are archived) |
| [`paper/`](../paper/) | Working paper: formal statement and analysis of the scoring mechanism, with proofs, reproducible experiments and open problems — a dated snapshot whose evaluator score is the previous design's (D47); kept in place because CI's differential oracles import its scripts |

## Development priorities

The reference implementation exists (see Status below); what remains is ordered in
[`10-roadmap.md`](10-roadmap.md) in three phases:

1. **Mathematics** — D47 changed Phase 1's direction before its implementation, and D48 settled
   part of its first implementation: the next work is to take the decisions D48 leaves open,
   specify the mechanism by which reviewers answer the item, then implement it
   ([`21`](21-d47-inventory.md) lists what to adapt). The previous
   design is implemented and its work suspended; read [`15`](15-phase1-review.md) for the A–E
   review findings and D47's disposition (A2 open), [`10`](10-roadmap.md) for task status and
   [`14`](14-parameter-register.md) for parameter procedures. D32–D41 are implemented; they are
   not an unstarted feature list.
2. **P2P network** (`04`) — persistence, transport and replication, a grind-free
   randomness beacon, live anchoring: a single-organization testnet.
3. **The rest** — the protocol boundary (`05`), distributed identity and the external
   cryptographic review (`03`), statistical privacy (`06`), and the real-world pilots
   that fix the empirical parameters.

The original build order (offline engine first, then identity, network, protocol) is
done as a reference implementation; the closed pilot and the opening to independent
operators come after the three phases.

## Status

A reference implementation exists as a Cargo workspace (`crates/{scoring,identity,
network,protocol,p2p,characterization}`); see the top-level `README.md` and `ARCHITECTURE.md` for the
current state, and `docs/08-formal-specification.md` for an independent audit of what
is implemented, tested, and still open. Read its dated audit and remediation log with
[`15`](15-phase1-review.md). The previous design's dossiers, tools and results are kept, outside
the active specification, in [`archive/phase1-forecast-pilot/`](../archive/phase1-forecast-pilot/README.md). Current Rust golden outputs pin bits; Python oracles
check the particular numerical contracts their tests name, within tolerance.

## License

EUPL-1.2 (see `LICENSE`).
