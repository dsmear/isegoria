# protocol fuzz targets (T73, T89)

`cargo fuzz` targets for the decoders of `protocol` and its answer-review ledger on cuts. They live outside the main workspace
because libFuzzer needs a nightly toolchain; `tests/node_replay.rs`, `tests/answer_review.rs`, `tests/answer_on_cuts_model.rs`,
`tests/answer_config_encoding.rs`, `tests/answer_participants.rs` and `network/tests/genesis.rs`
cover the same entry points on every push. The audit they belong to is recorded in `docs/12-panic-audit.md`.

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `event` | `NodeEvent::decode` on arbitrary bytes | a decoded event re-encodes to the same bytes, so an event has one CID |
| `answer_event` | `AnswerEvent::decode` on arbitrary bytes or a genuine join, deposit or reveal with a window overwritten, then `AnswerPath::apply` | one encoding; applying never panics |
| `genesis` | `Genesis::decode`, then `AnswerConfig::decode` and `IssuerPublic::decode` on its parts, as a node loads its genesis; arbitrary bytes or a genuine document with a window overwritten | one encoding each; a decoded document yields its tiered set under its id |
| `answer_gateway` | `answer_gateway::admit` of arbitrary bytes or a genuine join, deposit, commit or reveal with a window overwritten | a genuine event is admitted; a join, deposit or commit only as a genuine one; never a member's or derived event |
| `answer_read` | `ReadRequest::decode`, `Reply::decode` and a member's `answer_read::answer` over an empty ledger, on arbitrary bytes or a genuine request or reply with a window overwritten | one encoding each; a genuine request is answered; an answer only for a request with a genuine one's kind, response key and proof, opening under that key to a reply |
| `answer_seal` | `ResponseKey::open` of arbitrary bytes or a genuine seal with a window overwritten; `seal` to an arbitrary response key | only the untouched genuine seal opens, to its reply; a seal is its reply's length plus `OVERHEAD` |
| `answer_cuts` | `AnswerLedger::apply_in_view` over four members' feeds of genuine, altered and arbitrary objects, a second branch of each feed included, and cuts proposed by `Cut::next`, altered, moved to another chain a feed holds, and signed by any members in the view or the next | a cut is refused as out of sequence, unsigned, by the members' checks or by `added`, in that order, changing nothing; an unaltered proposal `t` members signed applies; an applied cut reports the entries `added` names and its seal; a member object counts only on its member's public feed, an event only on a circle feed, never a derived one; every refusal is recorded; the applied cuts given again to a fresh ledger reach the same digest, state and refusals |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/protocol
cargo +nightly fuzz run event -- -max_total_time=600
```

`answer_cuts` reads its input's first byte as how far one question's genuine review has gone
(none of it, or up to its closure) before the operations the other bytes spell; an execution
verifies each proof it applies twice, the replay included, so it runs slowly.

A genuine event as a seed reaches the proof decoder sooner: write one with
`NodeEvent::encode` into `fuzz/corpus/event/`.
