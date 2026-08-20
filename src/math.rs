//! Q32.32 fixed-point arithmetic + 256-bit hardware register.
//!
//! Independent re-implementation of the encoder's math primitives, kept
//! byte-for-byte identical so verification produces the same TSC root.

/// Q32.32 fixed-point: high 32 bits = integer part, low 32 bits = fractional.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Q32_32(pub u64);

impl Q32_32 {
    /// Conjugate of the golden ratio scaled by 2^32: `0.6180339887... * 2^32`.
    pub const GOLDEN_ANGLE: Q32_32 = Q32_32(2_654_435_769);

    /// Integer part.
    #[inline(always)]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn int_part(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Fractional part.
    #[inline(always)]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn frac_part(self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    /// Multiply Q32.32 by a u32 scalar.
    #[inline(always)]
    #[must_use]
    pub fn mul_scalar(self, scalar: u32) -> Q32_32 {
        let f_mul = u64::from(self.frac_part()) * u64::from(scalar);
        let i_mul = u64::from(self.int_part()) * u64::from(scalar);
        let carry = f_mul >> 32;
        // SAFETY: masking to 32 bits guarantees the value fits in u32.
        #[allow(clippy::cast_possible_truncation)]
        let new_frac = (f_mul & 0xFFFF_FFFF) as u32;
        #[allow(clippy::cast_possible_truncation)]
        let new_int = (i_mul.wrapping_add(carry) & 0xFFFF_FFFF) as u32;
        Q32_32((u64::from(new_int) << 32) | u64::from(new_frac))
    }
}

/// 256-bit register simulated as a pair of u128s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareRegister256(pub [u128; 2]);

impl HardwareRegister256 {
    /// Build from a 32-byte big-endian buffer (LE limb encoding internally,
    /// matching the encoder).
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut low = [0u8; 16];
        let mut high = [0u8; 16];
        low.copy_from_slice(&bytes[0..16]);
        high.copy_from_slice(&bytes[16..32]);
        Self([u128::from_le_bytes(low), u128::from_le_bytes(high)])
    }

    /// Serialise back to 32 bytes.
    #[inline]
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0..16].copy_from_slice(&self.0[0].to_le_bytes());
        bytes[16..32].copy_from_slice(&self.0[1].to_le_bytes());
        bytes
    }

    /// Cyclic left shift by `bits` (mod 256). Guards against shift overflow.
    #[inline]
    #[must_use]
    pub fn cyclic_shift_left(&self, bits: u32) -> Self {
        let bits = bits % 256;
        if bits == 0 {
            return *self;
        }
        match bits.cmp(&128) {
            core::cmp::Ordering::Less => {
                let b = bits;
                let inv_b = 128 - b;
                let n0 = (self.0[0] << b) | (self.0[1] >> inv_b);
                let n1 = (self.0[1] << b) | (self.0[0] >> inv_b);
                Self([n0, n1])
            }
            core::cmp::Ordering::Equal => Self([self.0[1], self.0[0]]),
            core::cmp::Ordering::Greater => {
                let b = bits - 128;
                let inv_b = 128 - b;
                let n0 = (self.0[1] << b) | (self.0[0] >> inv_b);
                let n1 = (self.0[0] << b) | (self.0[1] >> inv_b);
                Self([n0, n1])
            }
        }
    }
}
