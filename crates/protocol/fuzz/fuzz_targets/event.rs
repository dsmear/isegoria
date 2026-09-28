//! A node's events from untrusted bytes (T73, `docs/04` §Events and replay): decoding never
//! panics, a decoded event re-encodes to the same bytes, and applying results never panics.
#![no_main]

use libfuzzer_sys::fuzz_target;
use protocol::events::NodeEvent;
use protocol::results::ResultsState;

fuzz_target!(|bytes: &[u8]| {
    if let Some(event) = NodeEvent::decode(bytes) {
        assert_eq!(event.encode(), bytes, "a decoded event re-encodes differently");
        if let NodeEvent::Results(results) = &event {
            let _ = ResultsState::default().apply(results);
        }
    }
});
