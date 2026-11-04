//! Domain-separated SHA-256: the length-prefixed domain, then each length-prefixed field.

use sha2::{Digest, Sha256};

pub fn tagged(domain: &str, fields: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update((domain.len() as u64).to_le_bytes());
    h.update(domain.as_bytes());
    for f in fields {
        h.update((f.len() as u64).to_le_bytes());
        h.update(f);
    }
    h.finalize().into()
}
