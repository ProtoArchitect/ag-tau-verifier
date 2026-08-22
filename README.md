<!-- ag-doc:v1|tau|readme|#e5532f|τ -->
<div>
<p><span style="display:inline-block;width:34px;border-top:2px solid #e5532f;"></span>
<code style="font-size:10px;letter-spacing:0.18em">TAU · OVERVIEW</code></p>
<p style="font-size:2em;margin:0.35em 0 0"><strong style="font-family:Comfortaa,'Segoe UI',sans-serif">ag<sup style="color:#e5532f">τ</sup></strong>
<span style="font-family:Comfortaa,'Segoe UI',sans-serif;font-weight:500;font-size:0.65em"> tau</span>
<span style="font-family:'JetBrains Mono',monospace;font-size:0.35em;letter-spacing:0.12em;color:#6b6972"> · README</span></p>
<blockquote><p>tau-verifier</p></blockquote>
</div>

---

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
