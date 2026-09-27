//! A node's events from untrusted bytes (T73, `docs/04` §Events and replay): decoding never
//! panics, and an event that decodes re-encodes to the same bytes, so its CID is canonical.
#![no_main]

use libfuzzer_sys::fuzz_target;
use protocol::events::NodeEvent;

fuzz_target!(|bytes: &[u8]| {
    if let Some(event) = NodeEvent::decode(bytes) {
        assert_eq!(event.encode(), bytes, "a decoded event re-encodes differently");
    }
});
