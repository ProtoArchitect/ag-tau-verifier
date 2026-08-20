//! Witness and block-integrity verification logic.
//!
//! Both functions are pure — they consume only their arguments and produce
//! a `bool`/`Result`. No global state, no I/O.

use crate::errors::VerifierError;
use crate::format::{
    is_supported_block_header, Parser, ALPHA_EXTENSION_LEN, ALPHA_HEAD_MAGIC, TDX_BLOCK_VERSION_V03,
    TDX_BLOCK_VERSION_V04,
};
use crate::hash::hash_256;
use crate::math::{HardwareRegister256, Q32_32};
use crate::witness::SpiralWitness;

/// Compute one accumulator transition: `new_state` = H(domain || depth || rotate(prev, fib) || H(leaf)).
#[inline]
fn step(domain: &[u8; 32], depth: u64, fib_curr: u32, prev_state: &[u8; 32], leaf: &[u8]) -> [u8; 32] {
    let leaf_hash = hash_256(leaf);
    let momentum = Q32_32::GOLDEN_ANGLE.mul_scalar(fib_curr);
    // SAFETY: `(frac × 256) >> 32` lies in [0, 256); fits in u32.
    #[allow(clippy::cast_possible_truncation)]
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

/// Verify a 76-byte inclusion witness.
///
/// `data` is the original leaf (typically a compressed transaction).
/// Returns `true` iff the witness is valid for this leaf under `domain`.
#[must_use]
pub fn verify_inclusion(domain: [u8; 32], data: &[u8], witness: &SpiralWitness) -> bool {
    let computed = step(
        &domain,
        witness.sequence_depth,
        witness.fib_curr,
        &witness.pre_state,
        data,
    );
    computed == witness.post_state
}

/// Verify per-block integrity: re-run the accumulator over the block's
/// embedded compressed-tx stream and compare against the stored TSC root
/// in the footer.
///
/// Cross-block `prev_root` chaining is verified separately by
/// [`verify_block_chain`]; this function proves only that the block's
/// transactions, in their stored order, produce the recorded root.
///
/// # Errors
/// Returns a [`VerifierError`] variant for invalid magic, unsupported
/// version, truncation, or trailing bytes after the footer.
pub fn verify_block_integrity(block_bytes: &[u8], domain: [u8; 32]) -> Result<bool, VerifierError> {
    let mut p = Parser::new(block_bytes);

    let magic = p.read_array_4()?;
    let version = p.read_u8()?;
    if !is_supported_block_header(&magic, version) {
        if magic[0..3] != *b"TDX" {
            return Err(VerifierError::InvalidMagic);
        }
        return Err(VerifierError::UnsupportedVersion(version));
    }

    let _block_number = p.read_u64_be()?;
    let tx_count = p.read_u32_be()? as usize;

    if version == TDX_BLOCK_VERSION_V03 {
        let _ctx_mode = p.read_u8()?;
    }
    let ctx_len = p.read_u32_be()? as usize;

    let mut current = domain;
    let mut depth: u64 = 0;
    let mut fib_prev: u32 = 1;
    let mut fib_curr: u32 = 1;

    if version == TDX_BLOCK_VERSION_V04 {
        let payload = p.read_slice(ctx_len)?;
        depth += 1;
        current = step(&domain, depth, fib_curr, &current, payload);
    } else {
        // The context block (registry snapshot + dictionaries) is accumulated as
        // the FIRST leaf, exactly as BlockCompressor does. Skipping it here — as
        // this verifier used to — would let an attacker rewrite address mappings
        // or compression tables without disturbing the TSC root. Each leaf
        // advances both depth and the Fibonacci pair, so the context leaf must be
        // folded in with the same stepping the transactions use.
        let ctx_bytes = p.read_slice(ctx_len)?;
        if ctx_len > 0 {
            depth += 1;
            current = step(&domain, depth, fib_curr, &current, ctx_bytes);
            let next_fib = fib_prev.wrapping_add(fib_curr);
            fib_prev = fib_curr;
            fib_curr = next_fib;
        }
        for _ in 0..tx_count {
            let tx_len = p.read_u32_be()? as usize;
            let tx_bytes = p.read_slice(tx_len)?;
            depth += 1;
            current = step(&domain, depth, fib_curr, &current, tx_bytes);
            let next_fib = fib_prev.wrapping_add(fib_curr);
            fib_prev = fib_curr;
            fib_curr = next_fib;
        }
    }

    let stored_tsc = p.read_array_32()?;
    let _prev_root = p.read_array_32()?;
    consume_optional_alpha_extension(&mut p)?;
    Ok(current == stored_tsc)
}

/// Verify a contiguous chain of TDX blocks: intra-block integrity of every
/// block AND cross-block continuity (each block's `prev_root` equals the
/// previous block's TSC root).
///
/// `genesis_prev_root` is the expected `prev_root` of the first block
/// (typically `[0u8; 32]`). Returns `Ok(true)` iff every block is internally
/// valid and the `prev_root` chain is unbroken; `Ok(false)` on any integrity
/// or continuity mismatch.
///
/// # Errors
/// Returns a [`VerifierError`] for any malformed block (magic, version,
/// truncation, or trailing bytes).
pub fn verify_block_chain(
    blocks: &[&[u8]],
    domain: [u8; 32],
    genesis_prev_root: [u8; 32],
) -> Result<bool, VerifierError> {
    let mut expected_prev = genesis_prev_root;
    for block_bytes in blocks {
        let mut p = Parser::new(block_bytes);
        let magic = p.read_array_4()?;
        let version = p.read_u8()?;
        if !is_supported_block_header(&magic, version) {
            if magic[0..3] != *b"TDX" {
                return Err(VerifierError::InvalidMagic);
            }
            return Err(VerifierError::UnsupportedVersion(version));
        }
        let _block_number = p.read_u64_be()?;
        let tx_count = p.read_u32_be()? as usize;
        if version == TDX_BLOCK_VERSION_V03 {
            let _ctx_mode = p.read_u8()?;
        }
        let ctx_len = p.read_u32_be()? as usize;

        let mut current = domain;
        let mut depth: u64 = 0;
        let mut fib_prev: u32 = 1;
        let mut fib_curr: u32 = 1;
        if version == TDX_BLOCK_VERSION_V04 {
            let payload = p.read_slice(ctx_len)?;
            depth += 1;
            current = step(&domain, depth, fib_curr, &current, payload);
        } else {
            // Same context-binding as verify_block_integrity: the registry /
            // dictionary block is the first accumulated leaf, so it is covered
            // by the TSC root and cannot be rewritten unnoticed.
            let ctx_bytes = p.read_slice(ctx_len)?;
            if ctx_len > 0 {
                depth += 1;
                current = step(&domain, depth, fib_curr, &current, ctx_bytes);
                let next_fib = fib_prev.wrapping_add(fib_curr);
                fib_prev = fib_curr;
                fib_curr = next_fib;
            }
            for _ in 0..tx_count {
                let tx_len = p.read_u32_be()? as usize;
                let tx_bytes = p.read_slice(tx_len)?;
                depth += 1;
                current = step(&domain, depth, fib_curr, &current, tx_bytes);
                let next_fib = fib_prev.wrapping_add(fib_curr);
                fib_prev = fib_curr;
                fib_curr = next_fib;
            }
        }

        let stored_tsc = p.read_array_32()?;
        let prev_root = p.read_array_32()?;
        consume_optional_alpha_extension(&mut p)?;
        if current != stored_tsc || prev_root != expected_prev {
            return Ok(false);
        }
        expected_prev = stored_tsc;
    }
    Ok(true)
}

fn consume_optional_alpha_extension(p: &mut Parser<'_>) -> Result<(), VerifierError> {
    if p.is_at_end() {
        return Ok(());
    }
    if p.remaining_len() != ALPHA_EXTENSION_LEN {
        return Err(VerifierError::TrailingBytes);
    }
    let magic = p.read_array_4()?;
    if magic != ALPHA_HEAD_MAGIC {
        return Err(VerifierError::TrailingBytes);
    }
    p.advance(ALPHA_EXTENSION_LEN - 4)?;
    Ok(())
}
