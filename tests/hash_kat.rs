//! Known-answer tests for `tau_verifier::hash::hash_256`.
//!
//! Vectors are FIPS 180-4 Appendix B + NIST CAVS short messages. Verifies
//! that the verifier's SHA-256 wrapper produces bit-identical output to
//! the reference standard, so that any downstream Merkle/accumulator
//! computation chains on the same bytes as the encoder.
//!
//! These are the most important sanity tests in the public verifier:
//! if `hash_256` ever drifted, every inclusion proof would silently fail.

use tau_verifier::hash::hash_256;

/// Hex-decode a string into a 32-byte SHA-256 digest. Strict: panics on
/// odd length or non-hex characters — only used inside #[test] code.
fn hex32(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64, "expected 64 hex chars for 32-byte digest");
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        let hi = u8::from_str_radix(&s[2 * i..=2 * i], 16).expect("hex hi");
        let lo = u8::from_str_radix(&s[2 * i + 1..=2 * i + 1], 16).expect("hex lo");
        *byte = (hi << 4) | lo;
    }
    out
}

#[test]
fn empty_input_kat() {
    // SHA-256("") — FIPS 180-4 reference value.
    let expected = hex32("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(hash_256(b""), expected);
}

#[test]
fn abc_kat() {
    // SHA-256("abc") — FIPS 180-4 Appendix B.1, the canonical 24-bit test vector.
    let expected = hex32("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(hash_256(b"abc"), expected);
}

#[test]
fn long_string_kat() {
    // SHA-256 of 56-byte ASCII string spanning two 64-byte blocks once
    // length+padding are added — exercises the multi-block path.
    let expected = hex32("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    assert_eq!(
        hash_256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        expected
    );
}

#[test]
fn deterministic_repeated_call() {
    // Same input must produce identical output across N invocations.
    let payload = b"tau-verifier reproducibility check 2026";
    let first = hash_256(payload);
    for _ in 0..32 {
        assert_eq!(hash_256(payload), first);
    }
}

#[test]
fn distinct_inputs_distinct_digests() {
    // Avalanche sanity: a single-byte change must produce a different digest.
    let a = hash_256(b"tdx");
    let b = hash_256(b"tdy");
    assert_ne!(a, b);
}

#[test]
fn output_length_is_32_bytes() {
    // Trivial structural check — guards against accidental signature drift
    // away from `[u8; 32]`.
    let h = hash_256(b"length-check");
    assert_eq!(h.len(), 32);
}
