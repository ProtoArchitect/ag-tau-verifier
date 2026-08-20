// Verifier is a pure-Rust, safe-only re-implementation of the read side
// of the TDX wire format. `forbid(unsafe_code)` is part of the audit
// surface — see docs/compliance/CRYPTO_REVIEW_SCOPE.md.
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

//! # tau-verifier
//!
//! **Public, open-source verifier** for the TDX (Topological Data eXchange)
//! semantic compression engine.
//!
//! This crate is an *independent re-implementation* of the read-side of the
//! TDX protocol: TSC witness verification, block integrity checks, and
//! metadata peek. It depends only on `sha2` and `primitive-types` — it does
//! NOT pull in `tdx-core` and contains no encoder, no plugins, no registry
//! mutation logic.
//!
//! ## Why a separate crate?
//! The encoder (`tdx-core`) is closed-source. The verifier is published so
//! that anyone can independently confirm:
//! - that a TDX-compressed block hasn't been tampered with (`verify_block_integrity`);
//! - that a transaction was included in a block via a 76-byte witness
//!   (`verify_inclusion`);
//! - that the high-level metadata of a compressed transaction is a known
//!   shape (`peek_metadata`).
//!
//! ## Equivalence guarantee
//! The companion `tdx-core` crate runs an equivalence test on every commit:
//! random witnesses and blocks produced by the encoder must verify identically
//! through both `tdx-core`'s internal verifier and this crate's public
//! verifier. Any divergence fails CI.
//!
//! ## Format reference
//! See the [`format`] module for the byte-level block layout and
//! [`witness::SpiralWitness`] for the inclusion-proof structure. Both are
//! stable as of TDX v0.1 wire format.
//!
//! [`format`]: crate::format
//!
//! © 2026 Mikhail Kostan / `AuriGlyph`. All rights reserved.

#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

pub mod errors;
pub mod format;
pub mod hash;
pub mod math;
pub mod peek;
pub mod verify;
pub mod witness;

pub use errors::VerifierError;
pub use format::{
    is_supported_block_header, TDX_BLOCK_VERSION, TDX_BLOCK_VERSION_V01, TDX_BLOCK_VERSION_V03, TDX_MAGIC,
    TDX_MAGIC_V01, TDX_MAGIC_V03,
};
pub use peek::{peek_metadata, TxSummary};
pub use verify::{verify_block_chain, verify_block_integrity, verify_inclusion};
pub use witness::SpiralWitness;
