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
Reference verifier for the [Tau][tau] semantic compression engine.

This crate is an **independent re-implementation** of the read-side of the
Tau protocol. It depends only on `sha2` and `primitive-types`. It does NOT
pull in `tau-core` and contains no encoder, no plugins, and no registry
mutation logic.

It is shipped alongside the encoder under the same proprietary licence; see
the project root `LICENSE` for terms.

## What it can do

- **`verify_block_integrity(block_bytes, domain)`** — re-run the topological
  spiral accumulator over the block's compressed-tx stream and compare
  against the recorded TSC root.
- **`verify_inclusion(domain, leaf, witness)`** — verify a 76-byte inclusion
  proof for a single leaf.
- **`peek_metadata(compressed_tx)`** — read the 1-byte semantic header to
  classify a compressed transaction without decompressing it.

## What it cannot do

- **Compress** transactions or blocks. That's the encoder's job (`tau-core`,
  proprietary).
- **Decompress** transaction payloads. The verifier checks integrity over
  *bytes as committed*; it does not require recovering the underlying
  Ethereum transaction.

## Equivalence guarantee

`tau-core`'s CI runs an equivalence test on every commit: random witnesses
and blocks produced by the encoder must verify identically through both
`tau-core`'s internal verifier and this crate's public verifier. Any
divergence fails the build.

## Licence

Proprietary. All components of this repository — encoder, verifier, and
documentation — are governed by Mike's dual-layer proprietary licence.
See the project root `LICENSE`.

[tau]: https://github.com/auriglyph/ag-tau
