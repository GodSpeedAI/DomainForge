# Troubleshoot NAPI/TypeScript Builds

Goal: fix Node.js native module (`.node`) build issues, especially PyO3 symbol leaks.

## Symptoms

- Loading `domainforge-core.*.node` fails with `undefined symbol: PyBaseObject_Type`.
- TypeScript tests crash when `python` feature symbols leak into the NAPI build.

## Root cause

The native binary was built with `python` features enabled or reused artifacts from a Python build, pulling PyO3 symbols into the Node module.

## Fix: Clean, isolated build

```bash
# Remove stale artifacts
cargo clean
cd domainforge-core && cargo clean && cd ..
rm -rf target domainforge-core/target

# Build TypeScript binding only
cd domainforge-core
cargo build --lib --features typescript,three_valued_logic

# Copy the correct artifact to the root package if needed (choose your platform)
cp target/debug/libdomainforge_core.so ../domainforge-core.linux-x64-gnu.node      # Linux
cp target/debug/libdomainforge_core.dylib ../domainforge-core.darwin-x64.node      # macOS
cp target\\debug\\domainforge_core.dll ..\\domainforge-core.win32-x64.node         # Windows

# Verify no Py* symbols remain
nm -D ../domainforge-core.linux-x64-gnu.node | grep Py || echo "clean"
```

Then run `npm test` to confirm the binding loads.

## Prevent future leaks

- Keep Cargo features isolated:

  ```toml
  [features]
  default = []
  python = ["pyo3", "pythonize"]
  typescript = ["napi", "napi-derive"]
  three_valued_logic = []
  ```

- Ensure build scripts (`package.json`, `just`) do **not** enable `python` when building NAPI.
- If you regularly switch between Python and TypeScript builds, run `cargo clean` between them.

## See also

- [Cross-Language Binding Strategy](../explanations/cross-language-binding-strategy.md)
- [TypeScript API Reference](../reference/typescript-api.md)
