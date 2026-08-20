//! Tests for `tau_verifier::peek::peek_metadata`.
//!
//! `peek_metadata` is the asymmetric-superpower entry point: a 1-byte read
//! that answers "what kind of tx is this?" without ever decompressing the
//! body. These tests fix the wire-level bit layout of the semantic header
//! so that any encoder change (or any reader change) is caught immediately.
//!
//! ## Header bit layout (TDX wire v0.1)
//! ```text
//! bit 7 — is_erc20         (semantic plugin handles ERC20 transfer)
//! bit 6 — has_value == 0   (note: stored INVERTED — bit set = zero value)
//! bit 5 — is_standard_gas  (gas limit == 21_000)
//! bit 0 — has_to_address   (destination present, not contract creation)
//! bits 1..5 — reserved
//! ```

use tau_verifier::peek::{peek_metadata, TxSummary};
use tau_verifier::VerifierError;

#[test]
fn empty_input_returns_empty_tx_error() {
    assert_eq!(peek_metadata(&[]), Err(VerifierError::EmptyTx));
}

#[test]
fn all_zero_header_decodes_as_baseline_tx() {
    // header = 0b0000_0000:
    //   is_erc20         = false (bit 7 clear)
    //   has_value        = true  (bit 6 clear → value != 0)
    //   is_standard_gas  = false (bit 5 clear)
    //   has_to_address   = false (bit 0 clear → contract creation)
    let summary = peek_metadata(&[0b0000_0000]).expect("non-empty");
    assert_eq!(
        summary,
        TxSummary {
            is_erc20: false,
            has_value: true,
            is_standard_gas: false,
            has_to_address: false,
        }
    );
}

#[test]
fn typical_eth_transfer_header() {
    // Standard ETH transfer: has_value=true, is_standard_gas=true,
    // has_to_address=true, not ERC20.
    // header = 0b0010_0001
    let summary = peek_metadata(&[0b0010_0001]).expect("non-empty");
    assert!(!summary.is_erc20);
    assert!(summary.has_value);
    assert!(summary.is_standard_gas);
    assert!(summary.has_to_address);
}

#[test]
fn erc20_transfer_header() {
    // ERC20 transfer: is_erc20=true, has_value=false (no ETH moved),
    // not standard gas, has_to_address=true (token contract).
    // header = 0b1100_0001
    let summary = peek_metadata(&[0b1100_0001]).expect("non-empty");
    assert!(summary.is_erc20);
    assert!(!summary.has_value);
    assert!(!summary.is_standard_gas);
    assert!(summary.has_to_address);
}

#[test]
fn contract_creation_header() {
    // Contract creation: no `to`, has value (the deployer pays gas),
    // not ERC20, not standard gas.
    // header = 0b0000_0000  (already covered above)
    // But specifically with arbitrary trailing body.
    let mut tx = vec![0b0000_0000];
    tx.extend_from_slice(b"deploy bytecode payload here");
    let summary = peek_metadata(&tx).expect("non-empty");
    assert!(!summary.has_to_address);
}

#[test]
fn body_bytes_are_ignored() {
    // peek_metadata should be O(1) — only the first byte determines the
    // result. Trailing body bytes (signature, calldata, etc.) must not
    // affect the decoded summary.
    let h: u8 = 0b1010_0001;
    let single = peek_metadata(&[h]).expect("non-empty");
    let with_payload: Vec<u8> = std::iter::once(h).chain(0u8..255).collect();
    let with_long_payload: Vec<u8> = std::iter::once(h).chain((0u8..=255).cycle().take(10_000)).collect();
    assert_eq!(peek_metadata(&with_payload).expect("ok"), single);
    assert_eq!(peek_metadata(&with_long_payload).expect("ok"), single);
}

#[test]
fn all_combinations_of_meaningful_bits() {
    // Every (is_erc20, has_value, is_standard_gas, has_to_address)
    // 16-tuple is reachable from a single header byte. Verify exact decode
    // for all 16 combinations.
    for combo in 0..16u8 {
        let is_erc20 = (combo & 0b1000) != 0;
        let value_zero = (combo & 0b0100) != 0;
        let std_gas = (combo & 0b0010) != 0;
        let has_to = (combo & 0b0001) != 0;

        let mut h: u8 = 0;
        if is_erc20 {
            h |= 0b1000_0000;
        }
        if value_zero {
            h |= 0b0100_0000;
        }
        if std_gas {
            h |= 0b0010_0000;
        }
        if has_to {
            h |= 0b0000_0001;
        }

        let s = peek_metadata(&[h]).expect("non-empty");
        assert_eq!(s.is_erc20, is_erc20, "is_erc20 wrong at combo={combo:04b}");
        // peek decodes has_value as the *inverse* of bit 6.
        assert_eq!(s.has_value, !value_zero, "has_value wrong at combo={combo:04b}");
        assert_eq!(s.is_standard_gas, std_gas, "is_standard_gas wrong at combo={combo:04b}");
        assert_eq!(s.has_to_address, has_to, "has_to_address wrong at combo={combo:04b}");
    }
}

#[test]
fn reserved_bits_are_ignored() {
    // Bits 1..5 (mask 0b0001_1110) are reserved. Setting them must not
    // change any TxSummary field.
    let baseline = peek_metadata(&[0u8]).expect("ok");
    for b in 1u8..32 {
        // covers reserved bits 1..5
        let with_reserved = baseline_sets_only_reserved(b);
        let s = peek_metadata(&[with_reserved]).expect("ok");
        assert_eq!(s, baseline, "reserved bits 0b{b:08b} affected output");
    }
}

/// Build a header byte that sets ONLY reserved bits (1..5), leaving
/// `is_erc20` / `has_value` / `is_standard_gas` / `has_to_address` all clear.
fn baseline_sets_only_reserved(b: u8) -> u8 {
    // Reserved mask is bits 1..5 inclusive: 0b0001_1110.
    // Multiply input by 2 to skip bit 0, then clip into 5 lowest bits
    // (equivalent to placing `b` into bit positions 1..5).
    (b << 1) & 0b0001_1110
}
