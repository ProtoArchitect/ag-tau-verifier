//! Spiral inclusion witness: 76 bytes proving membership at a known sequence depth.

/// O(1)-size inclusion proof.
///
/// Together with the original (compressed) leaf data and the chain's
/// `domain_separator`, a `SpiralWitness` proves that the leaf was the
/// transition between `pre_state` and `post_state` of the topological
/// spiral accumulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiralWitness {
    /// 1-based depth of this leaf in the chain.
    pub sequence_depth: u64,
    /// Fibonacci momentum at this depth (drives the cyclic shift).
    pub fib_curr: u32,
    /// Accumulator state immediately before ingesting this leaf.
    pub pre_state: [u8; 32],
    /// Accumulator state immediately after ingesting this leaf.
    pub post_state: [u8; 32],
}
