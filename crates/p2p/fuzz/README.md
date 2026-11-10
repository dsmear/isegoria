# p2p fuzz targets (T89)

`cargo fuzz` targets for the decoders of `p2p` that read bytes from other machines. They live
outside the main workspace because libFuzzer needs a nightly toolchain; `tests/tiered_nodes.rs`
and `tests/blind_relays.rs` cover the same entry points on every push. The audit they belong
to, with the list of decoders and their runs, is recorded in `docs/12-panic-audit.md` §4.

Some readers are private in normal builds: cargo-fuzz compiles with `--cfg fuzzing`, which
exposes `p2p::fuzz` (the gateway's opening and reading of a request, the Binary HTTP of an
answer, and the object and shard servers).

| Target | Entry point | Asserted besides "no panic, no abort" |
|---|---|---|
| `bind` | `Bind::decode` (`/isegoria/bind/1`) on arbitrary bytes or a genuine hello with a window overwritten, then `Tiers::access`; `transport_key` of arbitrary peer ids | one encoding; only the genuine binding, from its own transport key, grants the circle |
| `key_config` | `KeyAnnouncement::decode` and `verify` on arbitrary bytes or a genuine configuration with a window overwritten | one encoding; only the genuine configuration verifies; `seal` succeeds exactly under a verified one |
| `client_reply` | `Carried::decode` and `Sealed::open` on arbitrary relay replies, on any plaintext sealed in the request's context, and on every gateway answer | one encoding; a reply not sealed for the request never opens; a receipt names only the member's circle feed; an answer opens as it was sealed |
| `gateway` | the gateway's decapsulation of arbitrary relayed requests and its reading of arbitrary inner requests | an event sealed by `seal` opens and reads back as itself, and as another network's request elsewhere |
| `serve` | object and shard requests through `ObjectCodec`, `ShardCodec` and the node's servers | a request reads exactly when well formed; only a held public object is served; an empty store serves nothing |

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
cd crates/p2p
cargo +nightly fuzz run bind -- -max_total_time=600
```

A crash is saved under `fuzz/artifacts/<target>/`; replay it with
`cargo +nightly fuzz run <target> fuzz/artifacts/<target>/<file>`, and turn it into a
regression test in `tests/` before fixing it. `corpus/`, `artifacts/` and `target/` are
git-ignored.
