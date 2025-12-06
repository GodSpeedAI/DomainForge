# Quick Start: Building WASM Bindings

## 1. Install Prerequisites

### wasm-pack
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### wasm-opt (optional, for optimization)
```bash
# macOS
brew install binaryen

# Ubuntu/Debian
sudo apt install binaryen

# Windows
# Download from https://github.com/WebAssembly/binaryen/releases
```

## 2. Build WASM Package

### Option A: Use Build Script (Recommended)
```bash
chmod +x scripts/build-wasm.sh
./scripts/build-wasm.sh
```

### Option B: Manual Build
```bash
cd sea-core
wasm-pack build --target web --release --out-dir ../pkg --features wasm
```

## 3. Verify Build

```bash
cd pkg
ls -lh
```

You should see:
- sea_core_bg.wasm
- sea_core.js
- sea_core.d.ts
- package.json
- index.js
- README.md

## 4. Check Size

```bash
gzip -k sea_core_bg.wasm
ls -lh sea_core_bg.wasm.gz
```

Target: <500 KB

## 5. Test in Browser

```bash
cd ..
python3 -m http.server 8000
```

Open (in your browser): `http://localhost:8000/examples/browser.html` (after starting the local server)

## 6. Run Tests

```bash
cd sea-core
wasm-pack test --headless --firefox --features wasm
```

## 7. Publish to npm (Optional)

```bash
cd pkg
npm publish --access public
```

## Troubleshooting

### Build fails with "wasm-pack not found"
Install wasm-pack: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`

### Bundle size >500KB
Run optimization: `wasm-opt -Oz pkg/sea_core_bg.wasm -o pkg/sea_core_bg.wasm`

### Browser example doesn't load
- Check browser console for errors
- Ensure local server is running
- Verify pkg/ directory contains WASM files

### Tests fail
```bash
# Try different browser
wasm-pack test --headless --chrome --features wasm

# Or run without headless
wasm-pack test --firefox --features wasm
```

## Files Created

### Source Code
- `sea-core/src/wasm/mod.rs` - WASM module entry
- `sea-core/src/wasm/primitives.rs` - Entity, Resource, Flow, Instance bindings
- `sea-core/src/wasm/graph.rs` - Graph bindings
- `sea-core/tests/wasm_tests.rs` - WASM tests

### Package
- `pkg/package.json` - npm metadata
- `pkg/index.js` - JavaScript wrapper
- `pkg/README.md` - Package docs

### Documentation
- `README_WASM.md` - Implementation guide
- `PHASE9_VERIFICATION.md` - Verification checklist
- `examples/browser.html` - Browser demo
- `scripts/build-wasm.sh` - Build script

## API Changes (November 2025)

- `Entity.new(name)` and `Resource.new(name, unit)` now flow through the namespace-aware overloads, defaulting to `"default"` when you omit the namespace; `namespace()` always returns a string.
- `Resource.new` delegates to `Resource.newWithNamespace(..., "default")`, so the WASM bindings never expose an undefined namespace.
- `Flow.new` takes `ConceptId` values (not references), so clone the IDs returned by `id()` before passing them to the constructor.

## Next Steps

See `PHASE9_VERIFICATION.md` for complete verification checklist.
