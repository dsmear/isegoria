//! Our binary encoding (`docs/04` §Events and replay): little-endian integers and
//! length-prefixed fields, each length checked against the input before any allocation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError;

#[derive(Default)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.bytes.push(v);
        self
    }

    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.fixed(&v.to_le_bytes())
    }

    /// Bytes of a width both sides know, with no length.
    pub fn fixed(&mut self, bytes: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(bytes);
        self
    }

    /// A variable field: its length, then its bytes.
    pub fn field(&mut self, bytes: &[u8]) -> &mut Self {
        self.u64(bytes.len() as u64).fixed(bytes)
    }

    pub fn finish(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Reader { bytes }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if n > self.bytes.len() {
            return Err(DecodeError);
        }
        let (head, rest) = self.bytes.split_at(n);
        self.bytes = rest;
        Ok(head)
    }

    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    pub fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    pub fn fixed<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        Ok(self.take(N)?.try_into().expect("take returns N bytes"))
    }

    pub fn field(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.u64()?;
        self.take(usize::try_from(len).map_err(|_| DecodeError)?)
    }

    /// `Ok` only if every byte was read.
    pub fn finish(self) -> Result<(), DecodeError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(DecodeError)
        }
    }
}
