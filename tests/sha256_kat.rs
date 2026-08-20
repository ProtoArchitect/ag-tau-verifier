// Copyright (c) 2026 Mikhail Kostan. All rights reserved.
//! SHA-256 KAT pinning for `tdx_verifier::hash::hash_256`.
//!
//! These vectors lock the verifier's hash primitive to FIPS-180-4 SHA-256.
//! Drift here = wire-format break for every existing TSC root.
//!
//! Conceptual link: AGF-0214 (Spiral Accumulator) describes the math of
//! the hash chain in atlas terms; AGF-0064 (splitmix64 mock) is **not**
//! a substitute — it's a non-cryptographic PRNG, never SHA-256.

use tau_verifier::hash::hash_256;

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn sha256_empty_string_kat() {
    // FIPS 180-4 reference vector: SHA-256("") =
    //   e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let h = hash_256(b"");
    assert_eq!(
        hex(&h),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn sha256_abc_kat() {
    // FIPS 180-4 reference vector: SHA-256("abc") =
    //   ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    let h = hash_256(b"abc");
    assert_eq!(
        hex(&h),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn sha256_long_message_kat() {
    // FIPS 180-4 reference vector: SHA-256 of the 56-byte
    // "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq" =
    //   248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1
    let h = hash_256(
        b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
    assert_eq!(
        hex(&h),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn hash_256_deterministic() {
    let a = hash_256(b"deterministic-test");
    let b = hash_256(b"deterministic-test");
    assert_eq!(a, b);
}

#[test]
fn hash_256_avalanche() {
    // Single-bit flip changes most output bits — basic SHA-256 sanity.
    let a = hash_256(b"abc");
    let b = hash_256(b"abd");
    let mut diff_bits = 0u32;
    for i in 0..32 {
        diff_bits += (a[i] ^ b[i]).count_ones();
    }
    // SHA-256 avalanche typically flips ~50% of the 256 output bits.
    assert!(
        diff_bits > 80,
        "avalanche too weak: only {} bits differ",
        diff_bits
    );
}
