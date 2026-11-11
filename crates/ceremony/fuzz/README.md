# ceremony fuzz targets (T89)

`cargo fuzz` targets for the bytes `isegoria-ceremony` reads from another member's machine. They
live outside the main workspace because libFuzzer needs a nightly toolchain;
`tests/formats.rs` covers the same entry point on every push. The payloads inside a message, the
member's state, slot requests and shares are `identity`'s, fuzzed in `crates/identity/fuzz/`. The
audit they belong to is recorded in `docs/12-panic-audit.md`.

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `message` | `message::open` by its recipient of arbitrary bytes or a genuine message with a window overwritten | a genuine message opens to its header and payload; no other bytes open |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/ceremony
cargo +nightly fuzz run message -- -max_total_time=600
```

A crash is saved under `fuzz/artifacts/<target>/`; replay it with
`cargo +nightly fuzz run <target> fuzz/artifacts/<target>/<file>`, and turn it into a
regression test before fixing it. `corpus/`, `artifacts/` and `target/` are git-ignored.
