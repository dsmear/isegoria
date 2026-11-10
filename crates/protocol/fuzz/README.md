# protocol fuzz targets (T73, T89)

`cargo fuzz` targets for the decoders of `protocol`. They live outside the main workspace
because libFuzzer needs a nightly toolchain; `tests/node_replay.rs`, `tests/answer_review.rs`,
`tests/answer_config_encoding.rs` and `network/tests/genesis.rs` cover the same entry points
on every push. The audit they belong to is recorded in `docs/12-panic-audit.md`.

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `event` | `NodeEvent::decode` on arbitrary bytes | a decoded event re-encodes to the same bytes, so an event has one CID |
| `answer_event` | `AnswerEvent::decode` on arbitrary bytes or a genuine join, deposit or reveal with a window overwritten, then `AnswerPath::apply` | one encoding; applying never panics |
| `genesis` | `Genesis::decode`, then `AnswerConfig::decode` and `IssuerPublic::decode` on its parts, as a node loads its genesis; arbitrary bytes or a genuine document with a window overwritten | one encoding each; a decoded document yields its tiered set under its id |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/protocol
cargo +nightly fuzz run event -- -max_total_time=600
```

A genuine event as a seed reaches the proof decoder sooner: write one with
`NodeEvent::encode` into `fuzz/corpus/event/`.
