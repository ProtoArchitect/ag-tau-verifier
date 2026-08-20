//! Verifier error type.

/// Errors returned by the public verifier.
///
/// All variants describe *malformed input* — missing magic, truncation,
/// unsupported version, etc. A failed integrity check is never an error;
/// it is reported as `Ok(false)` from [`crate::verify_block_integrity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierError {
    /// Block bytes too short / truncated at field boundary.
    Truncated,
    /// First four bytes don't match `"TDX\x01"`.
    InvalidMagic,
    /// Version byte is unknown to this verifier build.
    UnsupportedVersion(u8),
    /// Unexpected trailing bytes after the block footer.
    TrailingBytes,
    /// Compressed transaction stream too short for a semantic header.
    EmptyTx,
}

impl core::fmt::Display for VerifierError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Truncated => write!(f, "block truncated"),
            Self::InvalidMagic => write!(f, "invalid TDX magic"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported TDX block version: {v}"),
            Self::TrailingBytes => write!(f, "trailing bytes after block footer"),
            Self::EmptyTx => write!(f, "empty compressed transaction"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerifierError {}
