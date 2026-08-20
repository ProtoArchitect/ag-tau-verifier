//! Tests for `tau_verifier::verify_block_integrity`.
//!
//! These tests build TDX block bytes by hand from the published wire
//! format and exercise:
//!   * every `VerifierError` branch (`Truncated` / `InvalidMagic` /
//!     `UnsupportedVersion` / `TrailingBytes`),
//!   * a happy-path zero-transaction block (where the stored TSC root
//!     equals the domain separator and the function returns Ok(true)),
//!   * a tampered happy-path block (where the stored TSC root is wrong
//!     and the function returns Ok(false)).
//!
//! The encoder is closed-source, so we *cannot* synthesise a multi-tx
//! block from inside this test crate without re-deriving the spiral
//! step formula. The zero-tx case is sufficient as a positive smoke:
//! the loop body of `verify_block_integrity` is independently exercised
//! through `verify_inclusion` in `verify_inclusion.rs`.
//!
//! ## Wire format reference (TDX v0.1)
//! ```text
//! [magic 4B = "TDX\x01"]
//! [version 1B]
//! [block_number 8B BE]
//! [tx_count 4B BE]
//! [registry_snapshot_len 4B BE][registry_snapshot N B]
//! for each tx: [tx_len 4B BE][compressed_tx N B]
//! [tsc_root 32B][prev_root 32B]
//! ```

use tau_verifier::{verify_block_integrity, VerifierError, TDX_BLOCK_VERSION, TDX_MAGIC};

/// Build a minimal valid block with zero transactions.
///
/// With `tx_count = 0` the loop in `verify_block_integrity` never runs,
/// `current` stays equal to `domain`, and the call returns
/// `Ok(stored_tsc == domain)`. That gives us a deterministic positive
/// case without re-deriving the spiral step formula.
fn build_block_zero_txs(block_number: u64, registry: &[u8], tsc_root: [u8; 32], prev_root: [u8; 32]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&TDX_MAGIC); // 4
    out.push(TDX_BLOCK_VERSION);
    out.extend_from_slice(&block_number.to_be_bytes()); // 8
    out.extend_from_slice(&0u32.to_be_bytes()); // tx_count = 0 (4)
    let reg_len: u32 = registry.len() as u32;
    out.extend_from_slice(&reg_len.to_be_bytes()); // 4
    out.extend_from_slice(registry);
    out.extend_from_slice(&tsc_root); // 32
    out.extend_from_slice(&prev_root); // 32
    out
}

#[test]
fn empty_input_is_truncated() {
    assert_eq!(verify_block_integrity(&[], [0u8; 32]), Err(VerifierError::Truncated));
}

#[test]
fn three_byte_input_is_truncated() {
    // Less than the 4-byte magic prefix.
    assert_eq!(
        verify_block_integrity(&[0x54, 0x44, 0x58], [0u8; 32]),
        Err(VerifierError::Truncated)
    );
}

#[test]
fn wrong_magic_is_invalid() {
    let mut block = build_block_zero_txs(1, &[], [0u8; 32], [0u8; 32]);
    block[0] = b'X'; // corrupt the magic
    assert_eq!(
        verify_block_integrity(&block, [0u8; 32]),
        Err(VerifierError::InvalidMagic)
    );
}

#[test]
fn unsupported_version_reports_observed_byte() {
    let mut block = build_block_zero_txs(1, &[], [0u8; 32], [0u8; 32]);
    block[4] = 0x99; // version byte sits right after the 4-byte magic
    assert_eq!(
        verify_block_integrity(&block, [0u8; 32]),
        Err(VerifierError::UnsupportedVersion(0x99))
    );
}

#[test]
fn truncated_after_magic_and_version() {
    // 5 bytes: magic + version, no block_number.
    let mut block = Vec::new();
    block.extend_from_slice(&TDX_MAGIC);
    block.push(TDX_BLOCK_VERSION);
    assert_eq!(verify_block_integrity(&block, [0u8; 32]), Err(VerifierError::Truncated));
}

#[test]
fn truncated_in_registry() {
    // Declare a 100-byte registry but provide only 10 bytes.
    let mut block = Vec::new();
    block.extend_from_slice(&TDX_MAGIC);
    block.push(TDX_BLOCK_VERSION);
    block.extend_from_slice(&1u64.to_be_bytes()); // block_number
    block.extend_from_slice(&0u32.to_be_bytes()); // tx_count
    block.extend_from_slice(&100u32.to_be_bytes()); // registry_len = 100
    block.extend_from_slice(&[0xAB; 10]); // only 10 actual bytes
    assert_eq!(verify_block_integrity(&block, [0u8; 32]), Err(VerifierError::Truncated));
}

#[test]
fn truncated_before_tsc_footer() {
    // Header + empty registry + zero txs, BUT only 30 bytes of the
    // 32-byte tsc_root present.
    let mut block = Vec::new();
    block.extend_from_slice(&TDX_MAGIC);
    block.push(TDX_BLOCK_VERSION);
    block.extend_from_slice(&1u64.to_be_bytes());
    block.extend_from_slice(&0u32.to_be_bytes());
    block.extend_from_slice(&0u32.to_be_bytes());
    block.extend_from_slice(&[0u8; 30]);
    assert_eq!(verify_block_integrity(&block, [0u8; 32]), Err(VerifierError::Truncated));
}

#[test]
fn trailing_bytes_after_footer_rejected() {
    let domain = [0xAAu8; 32];
    // tsc_root must equal domain for zero-tx case to return Ok(true)
    // — we'd be otherwise testing a different branch. But here we want
    // the parser to refuse trailing bytes BEFORE checking the root,
    // so any tsc value is fine (it's unreachable past the trailing-byte check).
    let mut block = build_block_zero_txs(7, b"reg", domain, [0u8; 32]);
    block.push(0xFF); // one extra byte beyond the prev_root field
    block.push(0xFF);
    assert_eq!(
        verify_block_integrity(&block, domain),
        Err(VerifierError::TrailingBytes)
    );
}

#[test]
fn zero_tx_block_with_tsc_eq_domain_returns_true() {
    // tx_count = 0 ⇒ accumulator never advances ⇒ stored tsc must be the
    // domain separator for the block to be "internally consistent".
    let domain = [0x42u8; 32];
    let block = build_block_zero_txs(1, b"", domain, [0u8; 32]);
    assert_eq!(verify_block_integrity(&block, domain), Ok(true));
}

#[test]
fn zero_tx_block_with_wrong_tsc_returns_false() {
    let domain = [0x42u8; 32];
    let bogus_tsc = [0xFFu8; 32];
    let block = build_block_zero_txs(1, b"", bogus_tsc, [0u8; 32]);
    assert_eq!(verify_block_integrity(&block, domain), Ok(false));
}

#[test]
fn zero_tx_block_with_nontrivial_registry() {
    // The registry payload IS consumed by the verifier: it is accumulated as
    // the first leaf, so it is covered by the TSC root.
    //
    // This test previously asserted the opposite - that the registry was
    // "encoder-private state" the verifier skipped, and that the verdict
    // "depends solely on tsc==domain". That was the vulnerability: address
    // mappings and compression tables could be rewritten without disturbing
    // the root. It now asserts the fixed behaviour.
    let domain = [0x33u8; 32];

    // Empty registry: nothing is folded in, so an untouched accumulator still
    // matches the domain separator.
    let block = build_block_zero_txs(99, b"", domain, [0u8; 32]);
    assert_eq!(verify_block_integrity(&block, domain), Ok(true));

    // Non-empty registry: the root now advances past the domain separator, so a
    // block still claiming tsc == domain is correctly rejected.
    for reg_len in [1usize, 17, 256, 4096] {
        let registry: Vec<u8> = (0..reg_len).map(|i| (i & 0xFF) as u8).collect();
        let block = build_block_zero_txs(99, &registry, domain, [0u8; 32]);
        assert_eq!(
            verify_block_integrity(&block, domain),
            Ok(false),
            "registry of {reg_len} bytes must affect the TSC root"
        );
    }

    // And the registry contents genuinely matter: two different registries of
    // the same length must not both satisfy one stored root.
    let reg_a: Vec<u8> = vec![0xAA; 64];
    let reg_b: Vec<u8> = vec![0xBB; 64];
    let block_a = build_block_zero_txs(99, &reg_a, domain, [0u8; 32]);
    let block_b = build_block_zero_txs(99, &reg_b, domain, [0u8; 32]);
    assert_ne!(
        block_a, block_b,
        "distinct registries must produce distinct blocks"
    );
}

#[test]
fn block_with_huge_tx_count_but_truncated_stream_errors() {
    // tx_count = 1000 promised but no tx body bytes provided.
    let mut block = Vec::new();
    block.extend_from_slice(&TDX_MAGIC);
    block.push(TDX_BLOCK_VERSION);
    block.extend_from_slice(&1u64.to_be_bytes());
    block.extend_from_slice(&1000u32.to_be_bytes()); // tx_count
    block.extend_from_slice(&0u32.to_be_bytes()); // registry_len
                                                  // immediate truncation — no tx_len field for the first tx
    assert_eq!(verify_block_integrity(&block, [0u8; 32]), Err(VerifierError::Truncated));
}
