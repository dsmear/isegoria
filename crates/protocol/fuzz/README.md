# protocol fuzz targets (T73)

`cargo fuzz` targets for the decoders of `protocol`. They live outside the main workspace
because libFuzzer needs a nightly toolchain; `tests/node_replay.rs` covers the same entry
point on every push. The audit they belong to is recorded in `docs/12-panic-audit.md`.

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `event` | `NodeEvent::decode` on arbitrary bytes | a decoded event re-encodes to the same bytes, so an event has one CID |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/protocol
cargo +nightly fuzz run event -- -max_total_time=600
```

A genuine event as a seed reaches the proof decoder sooner: write one with
`NodeEvent::encode` into `fuzz/corpus/event/`.
