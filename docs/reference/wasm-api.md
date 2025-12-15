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
- `ResourceInstance` - Physical instances of resources at locations (use `ResourceInstance.new(name, resourceId, quantity?, namespace?, attributes?)`)
- `Role` - A role label used to classify entities and to define relations (construct via `Role.new(name, namespace?)`)
- `Relation` - A relation connecting roles, optionally tied to a flow (`Relation.new(name, subjectRoleId, predicate, objectRoleId, viaFlowId?, namespace?, attributes?)`)
- `Metric` - Observability metrics collected for flows and resources (created via `Metric.new(name, unit?, namespace?)`)
- `Mapping` - Used for data projections and mapping definitions across models (mapping constructor: `Mapping.new(...)` - refer to mapping docs for details)
- `Projection` - Projections allow building derived views (e.g., CALM projections) from the Graph
- `Graph` - Graph container with validation and traversal (uses IndexMap for deterministic iteration)
- `formatSource` - Format SEA-DSL source code
- `checkFormat` - Check if source is already formatted

### Constructor Patterns (November 2025)

**Entities:**

```javascript
// Default namespace
const entity = Entity.new("Warehouse"); // namespace() returns "default"

// Explicit namespace
const entity = Entity.newWithNamespace("Warehouse", "logistics");
```

**Resources:**

```javascript
const resource = Resource.new("Cameras", "units"); // Default namespace
const resource = Resource.newWithNamespace("Cameras", "units", "inventory");
```

**Flows:**

```javascript
// Takes ConceptId values - clone before passing
const flow = Flow.new(resourceId.clone(), fromId.clone(), toId.clone(), 100);
```

## Usage Examples

### Parse from DSL

```javascript
import { Graph } from "@domainforge/sea-wasm";

// Supports multiline strings with """ syntax
const source = `
  Entity "Warehouse" in logistics
  Entity """Multi-line
  Factory Name""" in manufacturing
  Resource "Cameras" units
  Flow "Cameras" from "Warehouse" to "Multi-line\\nFactory Name" quantity 100
`;

const graph = await Graph.parse(source);
console.log("Entities:", graph.entityCount());
console.log("Flows:", graph.flowCount());
```

### Build Programmatically

```javascript
import { Graph, Entity, Resource, Flow } from "@domainforge/sea-wasm";

const graph = new Graph();

// Use new() for default namespace, newWithNamespace() for explicit
const warehouse = Entity.new("Warehouse");
const factory = Entity.newWithNamespace("Factory", "manufacturing");
const cameras = Resource.new("Cameras", "units");

await graph.addEntity(warehouse);
await graph.addEntity(factory);
await graph.addResource(cameras);

// Flow constructor takes ConceptId values - clone before passing
const flow = Flow.new(
  cameras.id().clone(),
  warehouse.id().clone(),
  factory.id().clone(),
  100
);
await graph.addFlow(flow);

// Namespace is always a string (not null)
console.log(warehouse.namespace()); // "default"
console.log(factory.namespace()); // "manufacturing"
```

### Formatting Source Code

```javascript
import { formatSource, checkFormat } from "@domainforge/sea-wasm";

const source = 'Entity   "Foo"  in    bar';

// Format with defaults
const formatted = formatSource(source);
console.log(formatted); // Entity "Foo" in bar

// Format with custom options
const formatted2 = formatSource(source, 2, false, true, true);
// args: source, indentWidth, useTabs, preserveComments, sortImports

// Check if formatted
const isFormatted = checkFormat(source);
console.log(isFormatted); // false
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

| Rust Type       | WASM Boundary | JavaScript Type |
| --------------- | ------------- | --------------- |
| `String`        | `String`      | `string`        |
| `Uuid`          | `String`      | `string`        |
| `Decimal`       | `String`      | `string`        |
| `Option<T>`     | `nullable T`  | `T \| null`     |
| `Result<T, E>`  | `throws E`    | `Promise<T>`    |
| `Vec<T>`        | `Array<T>`    | `T[]`           |
| `HashMap<K, V>` | `Object`      | `object`        |

## Performance

- **Bundle size**: <500KB gzipped ✅
- **Parse time**: ~1ms for typical models
- **Memory**: ~2MB runtime overhead
- **Initialization**: <50ms (lazy loaded)
- **Deterministic**: IndexMap ensures reproducible results across runs

## CALM Integration (Architecture-as-Code)

Export/import graphs to/from FINOS CALM format:

```javascript
import { Graph } from "@domainforge/sea-wasm";

// Build your model
const graph = new Graph();
// ... add entities, resources, flows ...

// Export to CALM JSON
const calmJson = await graph.exportCalm();
console.log(calmJson); // CALM JSON string

// Import from CALM
const importedGraph = await Graph.importCalm(calmJson);
```

## Related Documentation

- [Phase 9 Plan](../../work/plans/Phase 9_WASM Bindings.md)
- [Package README](../../pkg/README.md)
- [Browser Example](../../examples/browser.html)

## License

Apache-2.0
