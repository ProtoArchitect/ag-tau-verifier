//! Zero-allocation metadata peek over a single compressed transaction.
//!
//! Reads only the 1-byte semantic header — never the body, never v/r/s.

use crate::errors::VerifierError;

/// Coarse summary derived solely from the semantic header bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxSummary {
    /// `bit 7` — payload is handled by a semantic plugin (typically ERC20).
    pub is_erc20: bool,
    /// `bit 6` — value is *not* zero (i.e. non-zero ETH transfer).
    pub has_value: bool,
    /// `bit 5` — gas limit is exactly 21,000.
    pub is_standard_gas: bool,
    /// `bit 0` — destination address is present (not contract creation).
    pub has_to_address: bool,
}

/// Decode a single compressed transaction's semantic header.
/// O(1), never touches the body.
///
/// # Errors
/// Returns [`VerifierError::EmptyTx`] if `compressed_tx` is empty.
pub fn peek_metadata(compressed_tx: &[u8]) -> Result<TxSummary, VerifierError> {
    if compressed_tx.is_empty() {
        return Err(VerifierError::EmptyTx);
    }
    let h = compressed_tx[0];
    Ok(TxSummary {
        is_erc20: (h & 0b1000_0000) != 0,
        has_value: (h & 0b0100_0000) == 0,
        is_standard_gas: (h & 0b0010_0000) != 0,
        has_to_address: (h & 0b0000_0001) != 0,
    })
}
