//! TDX block format constants + bounds-checking parser.
//!
//! ## Block layout (TDX wire v0.02 — shipping)
//! ```text
//! [magic 4B = "TDX\x02"][version 1B = 0x02][block_number 8B BE][tx_count 4B BE]
//! [sovereign_context_len 4B BE][sovereign_context N B]
//! for each tx: [tx_len 4B BE][compressed_tx N B]
//! [tsc_root 32B][prev_root 32B]
//! ```
//!
//! v0.01 blocks (`TDX\x01` / registry snapshot) remain verifiable.

use crate::errors::VerifierError;

/// Magic bytes for the shipping TDX block stream: `b"TDX\x02"`.
pub const TDX_MAGIC: [u8; 4] = [0x54, 0x44, 0x58, 0x02];

/// Legacy v0.01 block magic.
pub const TDX_MAGIC_V01: [u8; 4] = [0x54, 0x44, 0x58, 0x01];

/// v0.03 block magic (delta sovereign context headers).
pub const TDX_MAGIC_V03: [u8; 4] = [0x54, 0x44, 0x58, 0x03];

/// v0.04 block magic (M2 columnar field-plane body).
pub const TDX_MAGIC_V04: [u8; 4] = [0x54, 0x44, 0x58, 0x04];

/// Current wire format version (encoder default).
pub const TDX_BLOCK_VERSION: u8 = 0x02;

/// Delta-context block wire format version.
pub const TDX_BLOCK_VERSION_V03: u8 = 0x03;

/// M2 columnar block wire format version.
pub const TDX_BLOCK_VERSION_V04: u8 = 0x04;

/// Optional `AlphaHead` extension magic after TSC footer (`b"AHD\x01"`).
pub const ALPHA_HEAD_MAGIC: [u8; 4] = [0x41, 0x48, 0x44, 0x01];

/// Encoded `AlphaHead` payload (weyl + logic seal + shield seal).
pub const ALPHA_HEAD_BYTES: usize = 72;

/// Full optional extension: magic + head.
pub const ALPHA_EXTENSION_LEN: usize = 4 + ALPHA_HEAD_BYTES;

/// Legacy wire format version.
pub const TDX_BLOCK_VERSION_V01: u8 = 0x01;

/// Returns `true` when `magic` and `version` form a recognised TDX block header.
#[must_use]
pub fn is_supported_block_header(magic: &[u8; 4], version: u8) -> bool {
    magic[0..3] == *b"TDX"
        && magic[3] == version
        && (version == TDX_BLOCK_VERSION_V01
            || version == TDX_BLOCK_VERSION
            || version == TDX_BLOCK_VERSION_V03
            || version == TDX_BLOCK_VERSION_V04)
}

/// Bounds-checking byte parser. All reads return an error on truncation;
/// no panic paths.
pub(crate) struct Parser<'a> {
    pub data: &'a [u8],
    pub offset: usize,
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn ensure(&self, n: usize) -> Result<(), VerifierError> {
        if self.offset + n > self.data.len() {
            Err(VerifierError::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn read_slice(&mut self, n: usize) -> Result<&'a [u8], VerifierError> {
        self.ensure(n)?;
        let s = &self.data[self.offset..self.offset + n];
        self.offset += n;
        Ok(s)
    }

    pub fn advance(&mut self, n: usize) -> Result<(), VerifierError> {
        self.ensure(n)?;
        self.offset += n;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, VerifierError> {
        self.ensure(1)?;
        let v = self.data[self.offset];
        self.offset += 1;
        Ok(v)
    }

    pub fn read_u32_be(&mut self) -> Result<u32, VerifierError> {
        let s = self.read_slice(4)?;
        Ok(u32::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_u64_be(&mut self) -> Result<u64, VerifierError> {
        let s = self.read_slice(8)?;
        Ok(u64::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_array_4(&mut self) -> Result<[u8; 4], VerifierError> {
        let s = self.read_slice(4)?;
        Ok(s.try_into().unwrap())
    }

    pub fn read_array_32(&mut self) -> Result<[u8; 32], VerifierError> {
        let s = self.read_slice(32)?;
        Ok(s.try_into().unwrap())
    }

    pub fn is_at_end(&self) -> bool {
        self.offset == self.data.len()
    }

    pub fn remaining_len(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }
}
