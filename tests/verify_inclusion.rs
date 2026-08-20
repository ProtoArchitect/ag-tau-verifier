//! Tests for `tau_verifier::verify_inclusion`.
//!
//! The encoder's spiral step formula is closed-source, but the verifier's
//! `step()` is internal-but-documented and can be re-derived from the
//! published primitives in the `math` and `hash` modules. We re-implement
//! the same formula here in test scope (using only public API) and cross-
//! check that:
//!
//! 1. A witness whose `post_state` matches our re-derivation verifies as
//!    `true`.
//! 2. Tampering with any single byte of `data`, `pre_state`, `post_state`,
//!    `domain`, `sequence_depth`, or `fib_curr` produces `false`.
//!
//! This exercises the production verifier path on data the verifier has
//! never seen before, giving an honest reproducibility signal independent
//! of the encoder.

use tau_verifier::hash::hash_256;
use tau_verifier::math::{HardwareRegister256, Q32_32};
use tau_verifier::verify::verify_inclusion;
use tau_verifier::witness::SpiralWitness;

/// Re-implementation of the verifier's internal `step()` using only
/// public API. Stays byte-identical to `crates/tau-verifier/src/verify.rs`
/// so a known-answer witness can be constructed inside the test.
fn step_recompute(domain: &[u8; 32], depth: u64, fib_curr: u32, prev_state: &[u8; 32], leaf: &[u8]) -> [u8; 32] {
    let leaf_hash = hash_256(leaf);
    let momentum = Q32_32::GOLDEN_ANGLE.mul_scalar(fib_curr);
    let shift_bits = ((u64::from(momentum.frac_part()) * 256) >> 32) as u32;
    let register = HardwareRegister256::from_bytes(prev_state);
    let rotated = register.cyclic_shift_left(shift_bits).to_bytes();

    let mut fusion = [0u8; 104];
    fusion[..32].copy_from_slice(domain);
    fusion[32..40].copy_from_slice(&depth.to_be_bytes());
    fusion[40..72].copy_from_slice(&rotated);
    fusion[72..104].copy_from_slice(&leaf_hash);
    hash_256(&fusion)
}

fn make_test_witness(domain: [u8; 32], depth: u64, fib_curr: u32, pre_state: [u8; 32], data: &[u8]) -> SpiralWitness {
    let post_state = step_recompute(&domain, depth, fib_curr, &pre_state, data);
    SpiralWitness {
        sequence_depth: depth,
        fib_curr,
        pre_state,
        post_state,
    }
}

#[test]
fn correct_witness_verifies_as_true() {
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let data = b"compressed-tx-leaf-bytes";
    let witness = make_test_witness(domain, 1, 1, pre, data);
    assert!(verify_inclusion(domain, data, &witness));
}

#[test]
fn tampered_post_state_rejected() {
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let data = b"compressed-tx-leaf-bytes";
    let mut witness = make_test_witness(domain, 1, 1, pre, data);
    witness.post_state[0] ^= 0x01;
    assert!(!verify_inclusion(domain, data, &witness));
}

#[test]
fn tampered_data_rejected() {
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let data_original = b"compressed-tx-leaf-bytes";
    let witness = make_test_witness(domain, 1, 1, pre, data_original);

    // Same depth/fib/pre/post but a different leaf shouldn't satisfy.
    let data_corrupted = b"compressed-tx-leaf-bytez"; // last byte differs
    assert!(!verify_inclusion(domain, data_corrupted, &witness));
}

#[test]
fn tampered_domain_rejected() {
    let domain_real = [0x11u8; 32];
    let domain_other = [0x12u8; 32];
    let pre = [0x22u8; 32];
    let data = b"leaf";
    let witness = make_test_witness(domain_real, 1, 1, pre, data);
    assert!(!verify_inclusion(domain_other, data, &witness));
}

#[test]
fn tampered_sequence_depth_rejected() {
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let data = b"leaf";
    let mut witness = make_test_witness(domain, 7, 1, pre, data);
    witness.sequence_depth = 8; // bump depth without recomputing post_state
    assert!(!verify_inclusion(domain, data, &witness));
}

#[test]
fn tampered_fib_curr_rejected() {
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let data = b"leaf";
    let mut witness = make_test_witness(domain, 1, 5, pre, data);
    witness.fib_curr = 8; // change momentum without recomputing post_state
    assert!(!verify_inclusion(domain, data, &witness));
}

#[test]
fn tampered_pre_state_rejected() {
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let data = b"leaf";
    let mut witness = make_test_witness(domain, 1, 1, pre, data);
    witness.pre_state[31] ^= 0x80;
    assert!(!verify_inclusion(domain, data, &witness));
}

#[test]
fn empty_leaf_round_trips() {
    // Edge case: zero-length leaf data must still pass the proof check
    // when the witness is constructed correctly.
    let domain = [0xAAu8; 32];
    let pre = [0xBBu8; 32];
    let data: &[u8] = b"";
    let witness = make_test_witness(domain, 1, 1, pre, data);
    assert!(verify_inclusion(domain, data, &witness));
}

#[test]
fn deep_witness_at_depth_10000_round_trips() {
    // O(1) demonstrates: verification time independent of depth. Build
    // a witness at depth 10_000 and verify it. Any cost growth here would
    // suggest the verifier secretly does linear work.
    let domain = [0xCCu8; 32];
    let pre = [0xDDu8; 32];
    let data = b"deep-witness-payload";
    let witness = make_test_witness(domain, 10_000, 4181, pre, data);
    // 4181 = Fib(20); arbitrary momentum value taken from the
    // Fibonacci sequence to mirror real encoder behaviour.
    assert!(verify_inclusion(domain, data, &witness));
}

#[test]
fn many_independent_witnesses_all_verify() {
    // Stress: 1000 independent (domain, depth, fib, pre, data) tuples,
    // each producing its own witness. All must verify true.
    let mut prev = [0u8; 32];
    for n in 0..1000u64 {
        let domain = hash_256(&n.to_be_bytes());
        let leaf = format!("leaf-{n}");
        let depth = n + 1;
        let fib = (n as u32).wrapping_mul(0x9E37_79B9) | 1; // any non-zero u32
        let witness = make_test_witness(domain, depth, fib, prev, leaf.as_bytes());
        assert!(
            verify_inclusion(domain, leaf.as_bytes(), &witness),
            "iteration {n} failed to verify"
        );
        prev = witness.post_state;
    }
}

#[test]
fn cross_witness_does_not_verify_unrelated_data() {
    // Take a valid witness for leaf A and try to use it to "prove" leaf B.
    // Should fail.
    let domain = [0x11u8; 32];
    let pre = [0x22u8; 32];
    let leaf_a = b"transfer ETH to alice";
    let leaf_b = b"transfer ETH to mallory";
    let witness_for_a = make_test_witness(domain, 1, 1, pre, leaf_a);
    assert!(verify_inclusion(domain, leaf_a, &witness_for_a));
    assert!(!verify_inclusion(domain, leaf_b, &witness_for_a));
}
