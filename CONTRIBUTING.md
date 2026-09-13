# Contributing to ag-tau-verifier

Thank you for your interest in contributing to **ag-tau-verifier** — part of the Auriglyph ecosystem.

> **Important:** All contributions are subject to the [Code of Conduct](CODE_OF_CONDUCT.md) and must be made under the terms of the project license.

## Prerequisites

| Tool | Minimum Version |
|------|----------------|
| Rust | 1.78.0 stable |
| Cargo | Bundled with Rust |
| Git | 2.40+ (signed commits required) |

```bash
rustup update stable
rustup component add clippy rustfmt
```

## Development Setup

```bash
git clone https://github.com/auriglyph/ag-tau-verifier.git
cd ag-tau-verifier

cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Coding Standards

- **Formatting:** All code must pass `cargo fmt` with no diff.
- **Lints:** All code must pass `cargo clippy -- -D warnings`. No `unwrap()`/`expect()` in library paths.
- **No unsafe without review:** New `unsafe` blocks require a GitHub Issue and maintainer approval. All `unsafe` must be annotated with `// SAFETY:` comments.
- **No floating-point in deterministic paths:** Use `auriglyph-nu` fixed-point types instead.
- **Integer arithmetic only** in consensus-critical code paths.

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short summary>
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`

**Example:** `feat(core): add deterministic replay support with bit-exact guarantee`

All commits must be **GPG-signed**: `git config commit.gpgsign true`

## Pull Request Process

1. **Open an Issue first** for non-trivial changes.
2. **Fork** and create a feature branch: `git checkout -b feat/my-feature`
3. **Write tests** — coverage must not decrease.
4. **Update documentation** — inline `///` doc comments + markdown.
5. **Run the full suite:** `cargo test --workspace && cargo clippy && cargo fmt --check`
6. **Open a PR** against `main` and fill in the PR template completely.
7. PRs with 30+ days of inactivity will be closed.

## Testing Requirements

```bash
cargo test --workspace            # All unit + integration tests
cargo test --doc                  # All doc-tests
cargo bench -- --quick            # Benchmarks — must not regress >5%
```

## Security Vulnerabilities

**Do not open a public Issue for security vulnerabilities.**
Send an encrypted report to **security@auriglyph.io** with subject:
`[SECURITY] ag-tau-verifier vulnerability report`

We follow a **90-day responsible disclosure policy**.

---

Thank you for helping build **ag-tau-verifier**!
