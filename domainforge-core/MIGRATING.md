# Migrating from `sea-core` to `domainforge-core`

The Rust crate was renamed. The published artifact, CLI binary, and library
name changed:

| Before (`sea-core`)         | After (`domainforge-core`)     |
|-----------------------------|--------------------------------|
| `sea-core` (crates.io)      | `domainforge-core`             |
| `sea_core` (Rust crate)     | `domainforge_core`             |
| `sea` CLI binary            | `domainforge` CLI binary       |

## Cargo

```toml
[dependencies]
# old: sea-core = "0.12"
domainforge-core = "0.13"
```

```rust
// old: use sea_core::...
use domainforge_core::...;
```

## CLI

```bash
# old: sea validate model.sea
domainforge validate model.sea
```

The `.sea` DSL file extension and grammar are unchanged.
