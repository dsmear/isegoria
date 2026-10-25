# Isegoria

**A project for building a shared collection of quiz questions through peer review.**

Imagine a quiz about how public institutions work. Someone must write its questions
and say which answers are correct. How can other people check those choices, and
how can the review process itself be checked?

Isegoria explores a process with explicit rules and results that can be recomputed
from the recorded inputs. Its longer-term goal is a distributed network, with
participation under pseudonyms and no money staked to propose or review questions.
Today, its completed core is an offline prototype for reviewing questions.

## In plain words

1. **Someone proposes a question.** The author supplies the context, answer options
   and the option they consider correct. This proposed answer is called the *key*.
2. **Other people answer it.** The program randomly draws reviewers from a registered
   pool. Their question view omits the key. They can consult the Internet, documents
   or AI, choose an option, say "none is correct", or answer "don't know".
3. **The program counts their answers.** Every reviewer has equal weight. Under the
   current rule, enough agreement with the key accepts the question; enough
   disagreement discards it. Divided answers and too few answers are recorded as
   separate outcomes. Only accepted questions enter the usable collection.

For example, with all 12 reviewers giving usable answers, 9 matching the key are
enough to accept the question. If only 8 match, the result is divided. "Don't know"
and missing answers do not count as usable answers. These numbers are provisional;
the [review rules](docs/05-question-lifecycle.md#the-answer-review-01-d49d52) define
the thresholds when fewer people answer.

The recorded outcomes also affect how many questions an author may propose in
future rounds. They do not give that person more weight as a reviewer, and reviewers
gain no reputation for agreeing with the majority.

**People assess the question; the software enforces the procedure.** It does not
look up facts or decide which sources to trust. Reviewers can share the same mistake,
so acceptance is not proof that an answer is true. Nor does a collection of accepted
questions, by itself, make a fair quiz or establish what its respondents know.

## Current status

**Phase 1 is complete only for this offline answer-review path (T87, D49–D53).**
The [closure record](docs/15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21)
defines its evidence and limits. Parameters are provisional and uncalibrated; the new
decoder has not been fuzzed. Consensus certifies neither truth nor the fairness of a
quiz, and does not demonstrate respondents' knowledge or ability to assess evidence.
Quiz composition, exposure, retirement, distributed operation and production are
outside this closure.

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

The Cargo workspace contains `scoring`, `identity`, `network`, `protocol`, `p2p` and
`characterization`. The answer review is implemented by `protocol::answer*` and
`scoring::reputation::author_score_weighted`. See [Architecture](ARCHITECTURE.md) for
module contracts and dependencies.

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
