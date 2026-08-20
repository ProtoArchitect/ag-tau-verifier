//! SHA-256 wrapper matching the encoder's `TDXCore::hash_256`.

use sha2::{Digest, Sha256};

/// SHA-256 of `data` as a 32-byte array.
#[inline]
#[must_use]
pub fn hash_256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    let r = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&r);
    out
}
