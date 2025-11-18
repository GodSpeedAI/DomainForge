# Contributing: Build & Feature Notes

This short guide clarifies build and testing conventions for contributors.

## CLI gating (short)

- The `sea` CLI is an optional build artifact, gated with the `cli` Cargo feature.
- Build CLI: `cargo build --features cli`
- Run CLI-dependent tests: `cargo test --features cli`

## TypeScript (N-API)

- Build TypeScript N-API bindings:

```bash
cd sea-core
cargo build --release --features typescript
```

or via npm:

```bash
npm ci
npm run build
```

Do not enable the `cli` feature for the TypeScript build unless explicitly needed; mixing both can cause linker errors.

## WASM

- Build WASM for web targets:

```bash
cd sea-core
wasm-pack build --target web --features wasm
```

## CI

- CI runs `cargo test --features cli` to ensure CLI integration tests can spawn the binary.
- Release builds include `--features cli` to include the CLI in artifacts.

### Artifact verification

- CI and Release workflows verify the built CLI binary is runnable (they run `sea --version` against the built binary). This detects runtime/linker issues early.
- Release workflow also validates the packaged artifacts and checks native binary artifact sizes and WASM artifact size thresholds to prevent accidental bloat.

## Testing tip

- Use `just all-tests` to run Rust, Python and TypeScript tests locally.
