# client fuzz targets (T89)

`cargo fuzz` targets for what `isegoria-client` reads from other machines that no other crate's
targets reach. They live outside the main workspace because libFuzzer needs a nightly toolchain;
`tests/reads.rs` and `tests/events.rs` cover the same entry points on every push. The decoders
under them are fuzzed in `crates/{p2p,protocol,identity}/fuzz/`; the audit they belong to, with
the list of decoders, is recorded in `docs/12-panic-audit.md` §4. Both targets share
`common/network.rs`, the network of the client's tests.

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `read_reply` | `read::Opener::open` of a status, a reviewer's and an author's read: arbitrary sealed bytes, arbitrary or genuine replies with a window overwritten sealed to the read's key; `read::render` of what opens | a reply opens only under its own read's key, exactly when it decodes as a reply of that read's kind; it prints one line per fact |
| `listed` | `event::announcement` and `event::relay` of arbitrary text, or a member's configuration in hex or the listed relay's address with a window overwritten | a member's own configuration and the listed relay's address are taken; nothing else but a member's signed configuration with its genesis key, or an address ending in the listed relay's peer id |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/client
cargo +nightly fuzz run read_reply -- -max_total_time=600
```

A crash is saved under `fuzz/artifacts/<target>/`; replay it with
`cargo +nightly fuzz run <target> fuzz/artifacts/<target>/<file>`, and turn it into a
regression test in `tests/` before fixing it. `corpus/`, `artifacts/` and `target/` are
git-ignored.
