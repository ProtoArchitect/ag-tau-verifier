<p align="center">
  <img src=".github/assets/top_plaque.svg" width="800" alt="Tau Header" />
</p>
<p align="center">
  
</p>

Reference verifier for the **TAU Protocol** — The Zero-Decompression State Compression Layer.

This crate is a **lightweight, independent re-implementation** of the read-side of the TAU protocol. It is designed for L2 rollups, RPC nodes, and decentralized AI graphs. It depends only on `sha2` and `primitive-types` (`no-std` compatible). It does NOT pull in `tau-core` and contains no proprietary encoder, plugins, or registry mutation logic.

## Capabilities

- **`verify_block_integrity(block_bytes, domain)`** — Re-run the topological spiral accumulator over the block's compressed-tx stream and compare it against the recorded TSC root.
- **`verify_inclusion(domain, leaf, witness)`** — Verify a 76-byte inclusion proof for a single leaf cryptographically.
- **`peek_metadata(compressed_tx)`** — Execute **Query-in-Place**: read the 1-byte semantic header to classify a compressed transaction directly from memory *without decompressing it*.

## Architecture Constraints

- **Cannot Compress:** Transaction and block compression is strictly the job of the `tau-core` sovereign DMA engine (Proprietary).
- **Zero-Decompression:** The verifier checks integrity over *bytes as committed*; it does not require CPU-heavy decompression to recover the underlying state or payload.

## Integration Example

```rust
use tau_verifier::{verify_inclusion, Domain, Leaf, Witness};

fn check_tx_inclusion(domain: Domain, leaf: Leaf, witness: Witness) -> bool {
    // Cryptographically verify a transaction exists in the compressed state
    // Takes exactly 0.31 µs (zero memory allocations).
    verify_inclusion(domain, leaf, &witness)
}

## Contributing

Contributions are welcome! Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a Pull Request.
This project enforces [Code of Conduct](CODE_OF_CONDUCT.md) in all community spaces.

<p align="center">
  <img src=".github/assets/license_plaque.svg" width="800" alt="License Plaque" />
</p>
