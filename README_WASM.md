# SEA Core WASM Bindings

This document describes the WebAssembly (WASM) bindings for the SEA Core library, enabling browser and edge runtime usage.

## Overview

Phase 9 implements WASM bindings using `wasm-bindgen`, providing a lightweight (<500KB gzipped) module for browser and Node.js environments.

## Prerequisites

1. **Rust toolchain** (1.77+)
2. **wasm-pack**:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```
3. **wasm-opt** (optional, for size optimization):
   ```bash
   # macOS
   brew install binaryen
   
   # Ubuntu/Debian
   sudo apt install binaryen
   
   # Or download from https://github.com/WebAssembly/binaryen/releases
   ```

## Building

### Quick Build

```bash
chmod +x scripts/build-wasm.sh
./scripts/build-wasm.sh
```

### Manual Build

```bash
cd sea-core
wasm-pack build --target web --release --out-dir ../pkg --features wasm
cd ..

# Optional: Optimize with wasm-opt
wasm-opt -Oz -o pkg/sea_core_bg_opt.wasm pkg/sea_core_bg.wasm
mv pkg/sea_core_bg_opt.wasm pkg/sea_core_bg.wasm
```

## Testing

### Unit Tests

```bash
cd sea-core
wasm-pack test --headless --firefox --features wasm
```

### Browser Testing

1. Build the package:
   ```bash
   ./scripts/build-wasm.sh
   ```

2. Start a local server:
   ```bash
   python3 -m http.server 8000
   ```

3. Open `http://localhost:8000/examples/browser.html`

## Package Structure

```
pkg/
├── package.json          # npm package metadata
├── index.js              # JavaScript wrapper with lazy loading
├── README.md             # Package documentation
├── sea_core.js           # Generated WASM bindings
├── sea_core.d.ts         # TypeScript definitions
└── sea_core_bg.wasm      # Compiled WASM binary
```

## API

The WASM bindings expose the same API as the Rust core:

- `Entity` - Business actors, locations, organizational units
- `Resource` - Quantifiable subjects of value
- `Flow` - Transfers of resources between entities
- `Instance` - Physical instances of resources at locations
- `Graph` - Graph container with validation and traversal

See `pkg/README.md` for complete API documentation.

## Usage Examples

### Parse from DSL

```javascript
import { Graph } from '@domainforge/sea-wasm';

const source = `
  Entity "Warehouse" in logistics
  Resource "Cameras" units
`;

const graph = await Graph.parse(source);
console.log('Entities:', graph.entityCount());
```

### Build Programmatically

```javascript
import { Graph, Entity, Resource, Flow } from '@domainforge/sea-wasm';

const graph = new Graph();
const warehouse = new Entity('Warehouse', 'logistics');
const cameras = new Resource('Cameras', 'units', null);

await graph.addEntity(warehouse);
await graph.addResource(cameras);
```

## Size Optimization

The WASM module is optimized for size:

1. **Cargo.toml** optimizations:
   ```toml
   [profile.release]
   opt-level = "z"        # Optimize for size
   lto = true             # Link-time optimization
   codegen-units = 1      # Better optimization
   strip = true           # Strip debug symbols
   panic = 'abort'        # Smaller panic handler
   ```

2. **wasm-opt** post-processing:
   ```bash
   wasm-opt -Oz pkg/sea_core_bg.wasm
   ```

3. **Feature flags** to reduce dependencies:
   ```toml
   uuid = { version = "1.6", features = ["v4", "v7", "serde", "wasm-bindgen"] }
   ```

## Publishing

```bash
cd pkg
npm publish --access public
```

## Troubleshooting

### WASM module fails to load

- Ensure server sends correct MIME type: `application/wasm`
- Check browser console for detailed errors
- Verify WASM file exists and is not corrupted

### Size exceeds 500KB

- Run `wasm-opt -Oz` optimization
- Check for unused dependencies
- Use feature flags to exclude optional code

### TypeScript errors

- Ensure `sea_core.d.ts` is present in `pkg/`
- Check TypeScript version compatibility (4.5+)

## Architecture

### WASM Bindings Layer

```
JavaScript/TypeScript
       ↓
index.js (Wrapper + lazy loading)
       ↓
sea_core.js (wasm-bindgen generated)
       ↓
sea_core_bg.wasm (Compiled Rust)
       ↓
Rust Core (primitives, graph, parser)
```

### Type Conversions

| Rust Type | WASM Boundary | JavaScript Type |
|-----------|---------------|-----------------|
| `String` | `String` | `string` |
| `Uuid` | `String` | `string` |
| `Decimal` | `String` | `string` |
| `Option<T>` | `nullable T` | `T \| null` |
| `Result<T, E>` | `throws E` | `Promise<T>` |
| `Vec<T>` | `Array<T>` | `T[]` |
| `HashMap<K, V>` | `Object` | `object` |

## Performance

- **Bundle size**: <500KB gzipped ✅
- **Parse time**: ~1ms for typical models
- **Memory**: ~2MB runtime overhead
- **Initialization**: <50ms (lazy loaded)

## Related Documentation

- [Phase 9 Plan](../docs/plans/Phase%209%20WASM%20Bindings.md)
- [Package README](../pkg/README.md)
- [Browser Example](../examples/browser.html)

## License

MIT OR Apache-2.0
