# Reference: Configuration & Manifests

This document details the configuration files, manifest schemas, and build options used across the DomainForge workspace.

---

## 1. `.sea-registry.toml`

Used by `ModuleResolver` to configure multi-file modular projects and map namespace identifiers to glob patterns.

```toml
# Schema version
version = "1.0.0"

# Namespace mapping
[namespaces.procurement]
path = "models/procurement/**/*.sea"
version = "1.0.0"
owner = "Procurement Core Team"

[namespaces.shared]
path = "models/shared/**/*.sea"
version = "1.0.0"
owner = "Architecture Governance"
```

| Key | Type | Description |
|---|---|---|
| `version` | String | Registry format version. |
| `namespaces.<name>.path` | String (Glob) | File path or glob pattern locating files belonging to this namespace. |
| `namespaces.<name>.version` | String (Semver) | Semantic version of the namespace. |
| `namespaces.<name>.owner` | String | Team or individual responsible for governing the namespace. |

---

## 2. Cargo Feature Flags (`domainforge-core/Cargo.toml`)

DomainForge uses Cargo feature flags to support multiple target environments without bloat:

```toml
[features]
default = []
cli = ["three_valued_logic", "clap", "colored", "signing"]
python = ["pyo3", "pythonize", "signing"]
typescript = ["napi", "napi-derive", "signing"]
wasm = ["wasm-bindgen", "serde-wasm-bindgen", "uuid/js", "lol_alloc"]
wasm-projections = []
signing = ["ed25519-dalek"]
three_valued_logic = []
formatting = ["icu_decimal", "icu_locid", "fixed_decimal"]
```

| Feature Flag | Included Crates | Purpose |
|---|---|---|
| `cli` | `clap`, `colored`, `signing` | Builds the standalone `domainforge` CLI binary. |
| `python` | `pyo3`, `pythonize`, `signing` | Builds native Python C-extension module. |
| `typescript` | `napi`, `napi-derive`, `signing` | Builds native Node-API `.node` binary. |
| `wasm` | `wasm-bindgen`, `lol_alloc` | Compiles to compact WebAssembly for browser execution (< 2.5 MB). |
| `wasm-projections` | N/A | Exposes projection export families through WASM bindings (off by default to meet bundle size limits). |
| `signing` | `ed25519-dalek` | Enables Ed25519 cryptographic signing for Semantic Packs. |

---

## 3. Devbox Manifest (`devbox.json`)

Defines the hermetic local development environment:

```json
{
  "$schema": "https://raw.githubusercontent.com/jetify-com/devbox/0.13.7/.schema/devbox.schema.json",
  "packages": [
    "rustup@latest",
    "python@3.11",
    "bun@latest",
    "just@latest",
    "maturin@latest"
  ],
  "env": {
    "PAGER": "cat"
  }
}
```

---

## 4. `justfile` Key Recipes

The `justfile` provides standard shortcuts for testing and verification:

| Recipe | Command | Purpose |
|---|---|---|
| `just ai-validate` | `cargo test -p domainforge-core --features cli` | Standard automated gate check. |
| `just rust-test` | `cargo test -p domainforge-core --features cli` | Runs all Rust unit and integration tests. |
| `just python-test` | `pytest -q` | Runs Python integration test suite using `.venv`. |
| `just ts-test` | `bun test` or `npm test` | Runs TypeScript Vitest suite. |
| `just all-tests` | Runs Rust, Python, and TypeScript tests sequentially. | Cross-language parity verification gate. |
| `just prove` | Runs the full self-proving harness. | Generates `evidence/latest/proof.json` and `proof.md`. |
| `just python-setup` | Configures virtualenv and runs `maturin develop`. | Rebuilds Python native bindings. |
| `just enterprise-verify` | Runs fmt, clippy, tests, doctests, and audit. | Complete release gate verification. |

---

## Source Trail
- `Cargo.toml` — Workspace root manifest
- `domainforge-core/Cargo.toml` — Feature flag definitions
- `justfile` — Task runner recipes
- `devbox.json` — Hermetic dev environment
- `domainforge-core/src/registry/mod.rs` — `.sea-registry.toml` parsing
