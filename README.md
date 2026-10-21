# Isegoria

An offline prototype for reviewing quiz questions. Authors supply a question and its
answer key; randomly drawn reviewers answer it, with equal weight and free access to
Internet, documents and AI. The program applies the agreed consensus rule, records
the outcome and updates the author's proposal quota through their reputation.

**Phase 1 is complete only for this offline answer-review path (T87, D49–D53).**
The [closure record](docs/15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
defines its evidence and limits. Parameters are provisional and uncalibrated; the new
decoder has not been fuzzed. Consensus certifies neither truth nor the fairness of a
quiz, and does not establish D46's assessment objectives. Quiz composition, exposure,
retirement, distributed operation and production are outside this closure.

## Documentation

- [Overview](docs/00-overview.md): the current product, responsibilities and principles.
- [Answer-review specification](docs/05-question-lifecycle.md#the-answer-review-01-d49d52):
  the full lifecycle and implementation contract.
- [Documentation index](docs/README.md): specifications, decisions and evidence registers.
- [Architecture](ARCHITECTURE.md): the design-to-code map, including retained older modules.
- [Handoff](docs/phase1-handoff.md): where to resume work.

The earlier forecast-and-pilot mechanism remains in the repository as a separate path.
Its filters and guarantees do not transfer to the answer review. The
[historical paper](paper/README.md) analyses that earlier design; its PDF and amended
sources are different snapshots. Retired dossiers, tools and results are indexed in the
[archive](archive/phase1-forecast-pilot/README.md).

## Code architecture

The Cargo workspace contains `scoring`, `identity`, `network`, `protocol`, `p2p`, `node` and
`characterization`. The answer review is implemented by `protocol::answer*` and
`scoring::reputation::author_score_weighted`. See [Architecture](ARCHITECTURE.md) for
module contracts and dependencies.

## Phase 2 runtime

This branch also contains the node process (`isegoria-node`), writer recovery, view
changes and exclusion of beacon withholders (T78, T80, T76). The process drives cuts
and the beacon; it does not yet run the answer-review lifecycle between nodes (T79).
See [storage and runtime](docs/04-storage-network.md) and the
[Phase 2 roadmap](docs/10-roadmap.md#phase-2--p2p-network).

Replication still serves its whole set to connected peers. D54's access tiers, blind
relays and state tree are decided but not implemented (T77, T85, T86): run nodes only
among machines you control.

The target is a small experimental P2P testnet with independent operators, each
holding its own keys (D55). Single-operator runs are intermediate checks; distributed
committee setup and the operating contract remain to be built (T19, T88).
No production readiness is claimed.

## Try it

From the repository root, run the focused offline integration tests:

```sh
cargo test -p protocol --test answer_review
```

This exercises a library path, not a user interface or a deployed service. Broader
tests and characterization procedures are documented in
[Architecture](ARCHITECTURE.md#testing-strategy) and the
[verification methodology](docs/07-verification-and-assurance.md); full suites can run fits.

## License

[EUPL-1.2](LICENSE).
