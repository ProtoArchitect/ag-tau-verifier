//! Wire-format constants pinned to the published TDX block specification.
//!
//! These are not "tests" in the behavioural sense — they are *contracts*
//! that lock the on-the-wire bytes. If any of these constants is ever
//! changed without a wire-format version bump, this test fails loudly.

use tau_verifier::{TDX_BLOCK_VERSION, TDX_BLOCK_VERSION_V01, TDX_MAGIC, TDX_MAGIC_V01};

#[test]
fn tdx_magic_is_exact_published_bytes() {
    // Shipping: "TDX\x02" — 0x54 0x44 0x58 0x02.
    assert_eq!(TDX_MAGIC, [0x54, 0x44, 0x58, 0x02]);
    assert_eq!(&TDX_MAGIC[0..3], b"TDX");
    assert_eq!(TDX_MAGIC[3], TDX_BLOCK_VERSION);
}

#[test]
fn tdx_legacy_magic_pinned() {
    assert_eq!(TDX_MAGIC_V01, [0x54, 0x44, 0x58, 0x01]);
    assert_eq!(TDX_BLOCK_VERSION_V01, 0x01);
}

#[test]
fn tdx_block_version_is_two() {
    assert_eq!(TDX_BLOCK_VERSION, 0x02);
}

#[test]
fn magic_length_is_four() {
    // Block header always begins with exactly 4 magic bytes.
    assert_eq!(TDX_MAGIC.len(), 4);
}
