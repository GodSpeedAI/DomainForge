# DomainForge — SEA Core

Rust core library implementing the SEA DSL primitives and validation engine.

## Status

⚠️ **Phase 0: Foundation Bootstrap** - Workspace infrastructure only

## Building

```bash
# Build the library
cargo build
cargo test
cargo doc --no-deps --open

# Build the CLI binary (optional) - the CLI is gated behind the `cli` feature
cargo build --features cli

# Build TypeScript bindings (N-API) - produces a release cdylib
cargo build --release --features typescript
```
