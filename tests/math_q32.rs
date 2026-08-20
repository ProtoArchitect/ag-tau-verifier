//! `Q32_32` fixed-point arithmetic + `HardwareRegister256` cyclic-shift tests.
//!
//! These primitives drive the spiral accumulator's per-step rotation. If
//! any of them drifts byte-for-byte from the encoder side, every block's
//! TSC root would diverge.

use tau_verifier::math::{HardwareRegister256, Q32_32};

#[test]
fn golden_angle_constant_is_canonical() {
    // GOLDEN_ANGLE = 0.6180339887... × 2^32 = 2_654_435_769.
    // This is the conjugate of φ scaled into the Q32.32 fractional part.
    assert_eq!(Q32_32::GOLDEN_ANGLE.0, 2_654_435_769);
    // Integer part of conjugate-φ is 0 (since 0 < 1/φ < 1).
    assert_eq!(Q32_32::GOLDEN_ANGLE.int_part(), 0);
    assert_eq!(Q32_32::GOLDEN_ANGLE.frac_part(), 2_654_435_769);
}

#[test]
fn q32_decomposition_round_trips() {
    // For any raw u64 value, int_part << 32 | frac_part must reconstruct it
    // (modulo wrap), since they are simply the high and low 32-bit halves.
    let raw = 0x1234_5678_9ABC_DEF0u64;
    let q = Q32_32(raw);
    assert_eq!(q.int_part(), 0x1234_5678);
    assert_eq!(q.frac_part(), 0x9ABC_DEF0);
    let reconstructed = (u64::from(q.int_part()) << 32) | u64::from(q.frac_part());
    assert_eq!(reconstructed, raw);
}

#[test]
fn mul_scalar_zero_yields_zero() {
    assert_eq!(Q32_32::GOLDEN_ANGLE.mul_scalar(0), Q32_32(0));
    assert_eq!(Q32_32(0xFFFF_FFFF_FFFF_FFFF).mul_scalar(0), Q32_32(0));
}

#[test]
fn mul_scalar_one_is_identity() {
    let q = Q32_32::GOLDEN_ANGLE;
    assert_eq!(q.mul_scalar(1), q);
}

#[test]
fn mul_scalar_two_doubles_value() {
    // Q32_32(0x0000_0000_8000_0000) = 0.5 in Q32.32.
    // Multiplying by 2 should yield 1.0 = 0x0000_0001_0000_0000.
    let half = Q32_32(0x0000_0000_8000_0000);
    let one = half.mul_scalar(2);
    assert_eq!(one, Q32_32(0x0000_0001_0000_0000));
}

#[test]
fn register_round_trip_preserves_bytes() {
    // For any 32-byte buffer, from_bytes followed by to_bytes is the identity.
    let buf: [u8; 32] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x10, 0x32,
        0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
    ];
    let r = HardwareRegister256::from_bytes(&buf);
    assert_eq!(r.to_bytes(), buf);
}

#[test]
fn cyclic_shift_zero_is_identity() {
    let buf: [u8; 32] = [0x55; 32];
    let r = HardwareRegister256::from_bytes(&buf);
    assert_eq!(r.cyclic_shift_left(0).to_bytes(), buf);
}

#[test]
fn cyclic_shift_full_rotation_is_identity() {
    // 256-bit register, shift by 256 bits = no change.
    let mut buf = [0u8; 32];
    for (i, b) in buf.iter_mut().enumerate() {
        *b = i as u8;
    }
    let r = HardwareRegister256::from_bytes(&buf);
    assert_eq!(r.cyclic_shift_left(256).to_bytes(), buf);
    // Modular: shift by 512 also identity.
    assert_eq!(r.cyclic_shift_left(512).to_bytes(), buf);
}

#[test]
fn cyclic_shift_128_swaps_halves() {
    // Bits 0..128 occupy limb 0 (low half on little-endian), 128..256 limb 1.
    // A 128-bit cyclic left shift swaps the two limbs.
    let mut buf = [0u8; 32];
    buf[0..16].copy_from_slice(&[0xAA; 16]); // low limb
    buf[16..32].copy_from_slice(&[0xBB; 16]); // high limb
    let r = HardwareRegister256::from_bytes(&buf);
    let rotated = r.cyclic_shift_left(128).to_bytes();
    let mut expected = [0u8; 32];
    expected[0..16].copy_from_slice(&[0xBB; 16]);
    expected[16..32].copy_from_slice(&[0xAA; 16]);
    assert_eq!(rotated, expected);
}

#[test]
fn cyclic_shift_modulo_256() {
    // shift by 257 ≡ shift by 1 (mod 256).
    let buf: [u8; 32] = [0xC3; 32];
    let r = HardwareRegister256::from_bytes(&buf);
    assert_eq!(r.cyclic_shift_left(257).to_bytes(), r.cyclic_shift_left(1).to_bytes());
}

#[test]
fn cyclic_shift_left_then_right_recovers_input() {
    // Conceptual reversibility: shifting left by N and then left by 256 - N
    // returns to the original (since 256-rotation = identity).
    let buf: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11,
        0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    ];
    let r = HardwareRegister256::from_bytes(&buf);
    for n in [1u32, 7, 63, 64, 100, 127, 128, 129, 200, 255] {
        let rotated = r.cyclic_shift_left(n);
        let recovered = rotated.cyclic_shift_left(256 - n);
        assert_eq!(recovered.to_bytes(), buf, "failed at n={n}");
    }
}
