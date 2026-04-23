<p align="center">
  <img src="https://kura.pro/commons/images/logos/commons.svg" alt="EUXIS Commons logo" width="128" />
</p>

<h1 align="center">EUXIS Commons</h1>

<p align="center">
  <strong>Shared Rust utilities and common patterns — error handling, configuration, logging, validation, retry logic, and more.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/commons/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/commons/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/euxis-commons"><img src="https://img.shields.io/crates/v/euxis-commons.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/euxis-commons"><img src="https://img.shields.io/badge/docs.rs-euxis-commons-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/commons"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/commons?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/euxis-commons"><img src="https://img.shields.io/badge/lib.rs-v0.0.3-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add euxis-commons
```

Or add to `Cargo.toml`:

```toml
[dependencies]
euxis-commons = "0.0.3"
```

You need [Rust](https://rustup.rs/) 1.88.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

EUXIS Commons is the shared foundation library for the Sebastien Rousseau Rust ecosystem. It provides reusable components that eliminate boilerplate across projects.

- **Unified error types** for consistent error propagation
- **Configuration loading** from TOML, YAML, and JSON
- **Structured logging** built on the `tracing` ecosystem
- **Retry logic** with exponential backoff and jitter

---

## Features

| | |
| :--- | :--- |
| **Error handling** | Unified error types and `Result` aliases for consistent error propagation |
| **Configuration** | TOML/YAML/JSON config loading with validation and defaults |
| **Logging** | Structured logging utilities built on `tracing` |
| **Validation** | Input and data validation helpers |
| **Retry logic** | Configurable retry with exponential backoff |
| **Security** | `#![forbid(unsafe_code)]` enforced across the crate |

---

## Usage

```rust
use euxis_commons::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use shared utilities across your project
    println!("EUXIS Commons loaded");
    Ok(())
}
```

---

## Development

```bash
cargo build        # Build the project
cargo test         # Run all tests
cargo clippy       # Lint with Clippy
cargo fmt          # Format with rustfmt
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed commits, and PR guidelines.

---

**THE ARCHITECT** \u1d2b [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** \u1d5e [EUXIS](https://euxis.co) \u1d2b Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#euxis-commons">Back to Top</a></p>