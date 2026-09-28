//! Our binary encoding (`docs/04` §Events and replay, `docs/08` NET-012, T73): values
//! round-trip; a short, overlong or trailing input is refused, never sized from.

use network::codec::{DecodeError, Reader, Writer};
use proptest::prelude::*;

/// AT-NET-12: integers, fixed and variable fields round-trip, in order.
#[test]
fn at_net_12_values_round_trip() {
    let bytes = Writer::new()
        .u8(7)
        .u64(u64::MAX - 1)
        .fixed(&[9; 32])
        .field(b"variable")
        .field(b"")
        .finish();
    assert_eq!(bytes.len(), 1 + 8 + 32 + (8 + 8) + 8);
    assert_eq!(
        &bytes[..9],
        &[7, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    );
    assert_eq!(&bytes[41..49], &8u64.to_le_bytes());
    let mut r = Reader::new(&bytes);
    assert_eq!(r.u8(), Ok(7));
    assert_eq!(r.u64(), Ok(u64::MAX - 1));
    assert_eq!(r.fixed::<32>(), Ok([9; 32]));
    assert_eq!(r.field(), Ok(&b"variable"[..]));
    assert_eq!(r.field(), Ok(&b""[..]));
    assert_eq!(r.finish(), Ok(()));
}

/// AT-NET-12: every read past the end fails, and trailing bytes fail `finish`.
#[test]
fn at_net_12_short_and_trailing_inputs_are_refused() {
    assert_eq!(Reader::new(&[]).u8(), Err(DecodeError));
    assert_eq!(Reader::new(&[1; 7]).u64(), Err(DecodeError));
    assert_eq!(Reader::new(&[1; 31]).fixed::<32>(), Err(DecodeError));
    assert_eq!(Reader::new(&[1; 7]).field(), Err(DecodeError));
    let mut longer = 5u64.to_le_bytes().to_vec();
    longer.extend_from_slice(b"four");
    assert_eq!(Reader::new(&longer).field(), Err(DecodeError));
    let mut exact = 4u64.to_le_bytes().to_vec();
    exact.extend_from_slice(b"four");
    let mut r = Reader::new(&exact);
    assert_eq!(r.field(), Ok(&b"four"[..]));
    assert_eq!(r.finish(), Ok(()));
    let mut r = Reader::new(&[1, 2]);
    assert_eq!(r.u8(), Ok(1));
    assert_eq!(r.finish(), Err(DecodeError));
    let mut huge = u64::MAX.to_le_bytes().to_vec();
    huge.push(0);
    assert_eq!(
        Reader::new(&huge).field(),
        Err(DecodeError),
        "never sized from the claim"
    );
}

proptest! {
    /// AT-NET-12: arbitrary bytes never panic any sequence of reads.
    #[test]
    fn at_net_12_hostile_bytes_never_panic(
        bytes in prop::collection::vec(any::<u8>(), 0..64),
        ops in prop::collection::vec(0u8..4, 0..8),
    ) {
        let mut r = Reader::new(&bytes);
        for op in ops {
            let _ = match op {
                0 => r.u8().map(|_| ()),
                1 => r.u64().map(|_| ()),
                2 => r.fixed::<32>().map(|_| ()),
                _ => r.field().map(|_| ()),
            };
        }
        let _ = r.finish();
    }

    /// AT-NET-12: any sequence of fields written reads back identically.
    #[test]
    fn at_net_12_any_fields_round_trip(fields in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..40), 0..6)) {
        let mut w = Writer::new();
        for f in &fields {
            w.field(f);
        }
        let bytes = w.finish();
        let mut r = Reader::new(&bytes);
        for f in &fields {
            prop_assert_eq!(r.field(), Ok(&f[..]));
        }
        prop_assert_eq!(r.finish(), Ok(()));
    }
}
