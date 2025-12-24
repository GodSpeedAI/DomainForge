# @domainforge/sea-wasm

[![npm](https://img.shields.io/npm/v/@domainforge/sea-wasm.svg)](https://www.npmjs.com/package/@domainforge/sea-wasm)
[![License](https://img.shields.io/npm/l/@domainforge/sea-wasm.svg)](https://github.com/GodSpeedAI/DomainForge/blob/main/LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)
[![Bundle Size](https://img.shields.io/badge/bundle%20size-%3C500KB%20gzip-brightgreen)](https://bundlephobia.com/package/@domainforge/sea-wasm)

WebAssembly bindings for the **SEA DSL** (Semantic Enterprise Architecture) domain-specific language. Runs in browsers and edge runtimes. Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) ecosystem.

## Features

- 🌐 **Browser & Edge** — Runs anywhere WebAssembly is supported
- 📦 **Lightweight** — <500KB gzipped bundle size
- ⚡ **Fast** — ~1ms parse time for typical models
- 🏗️ **Domain Primitives** — Entities, Resources, Flows, Roles, Relations
- ✅ **Policy Engine** — Constraint validation with three-valued logic
- 🔄 **DSL Parsing** — Parse SEA source into queryable graph structures
- 🌐 **CALM Integration** — Export/import FINOS CALM format
- 📦 **TypeScript Support** — Full type definitions included

## Installation

```bash
npm install @domainforge/sea-wasm
```

```bash
yarn add @domainforge/sea-wasm
```

### CDN (Browser)

```html
<script type="module">
  import init, {
    Graph,
    Entity,
    Resource,
    Flow,
  } from "https://unpkg.com/@domainforge/sea-wasm/sea_core.js";

  await init();
  // Ready to use
</script>
```

## Quick Start

### Browser

```html
<!DOCTYPE html>
<html>
  <head>
    <script type="module">
      import init, { Graph, Entity, Resource } from "@domainforge/sea-wasm";

      async function main() {
        await init();

        const source = `
        Entity "Warehouse" in logistics
        Entity "Factory" in manufacturing
        Resource "Cameras" units
        Flow "Cameras" from "Warehouse" to "Factory" quantity 100
      `;

        const graph = Graph.parse(source);
        console.log(`Entities: ${graph.entityCount()}`);
        console.log(`Flows: ${graph.flowCount()}`);
      }

      main();
    </script>
  </head>
  <body></body>
</html>
```

### Node.js / ESM

```javascript
import init, { Graph, Entity, Resource, Flow } from "@domainforge/sea-wasm";

await init();

const graph = new Graph();

const warehouse = new Entity("Warehouse", "logistics");
const factory = new Entity("Factory", "manufacturing");
const cameras = new Resource("Cameras", "units");

graph.addEntity(warehouse);
graph.addEntity(factory);
graph.addResource(cameras);

const flow = new Flow(cameras.id(), warehouse.id(), factory.id(), "100");
graph.addFlow(flow);

console.log(`Graph has ${graph.entityCount()} entities`);
```

### CALM Integration

```javascript
import init, { Graph } from "@domainforge/sea-wasm";

await init();

// Build your model
const graph = Graph.parse(`
  Entity "Customer"
  Entity "Vendor"
  Resource "Payment" USD
  Flow "Payment" from "Customer" to "Vendor"
`);

// Export to CALM JSON
const calmJson = graph.exportCalm();
console.log(calmJson);

// Import from CALM
const importedGraph = Graph.importCalm(calmJson);
```

## API Reference

### Core Classes

| Class                  | Description                                            |
| ---------------------- | ------------------------------------------------------ |
| `Entity`               | Business actors, locations, organizational units (WHO) |
| `Resource`             | Quantifiable subjects of value (WHAT)                  |
| `Flow`                 | Transfers of resources between entities                |
| `Instance`             | Entity type instances with named fields                |
| `Graph`                | Container with validation and query capabilities       |
| `Expression`           | Programmatic policy expression builder                 |
| `NormalizedExpression` | Canonical form for semantic equivalence                |

### Constructor Patterns

```javascript
// Entities
const entity = new Entity("Warehouse"); // default namespace
const entity = new Entity("Warehouse", "logistics"); // explicit namespace

// Resources
const resource = new Resource("Cameras", "units");
const resource = new Resource("Cameras", "units", "inventory");

// Flows (quantity as string for precision)
const flow = new Flow(resourceId, fromId, toId, "100");

// Instances
const instance = new Instance("order_123", "Order");
instance.setField("status", "pending");
```

### Graph Methods

```javascript
// Add primitives
graph.addEntity(entity);
graph.addResource(resource);
graph.addFlow(flow);

// Counts
graph.entityCount();
graph.resourceCount();
graph.flowCount();

// Lookup
graph.findEntityByName("Warehouse");
graph.findResourceByName("Cameras");

// Flow queries
graph.flowsFrom(entityId);
graph.flowsTo(entityId);

// Get all
graph.allEntities();
graph.allResources();
graph.allFlows();

// Parsing
Graph.parse(source);

// CALM integration
graph.exportCalm();
Graph.importCalm(calmJson);
```

## Performance

| Metric          | Value                   |
| --------------- | ----------------------- |
| Bundle size     | <500KB gzipped          |
| Parse time      | ~1ms for typical models |
| Memory overhead | ~2MB runtime            |
| Initialization  | <50ms (lazy loaded)     |

## Building from Source

### Prerequisites

1. **Rust toolchain** (1.77+)
2. **wasm-pack**:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

### Build

```bash
# Quick build
./scripts/build-wasm.sh

# Manual build
cd sea-core
wasm-pack build --target web --release --out-dir ../pkg --features wasm
```

### Test

```bash
cd sea-core
wasm-pack test --headless --firefox --features wasm
```

## Package Structure

```
pkg/
├── package.json          # npm package metadata
├── index.js              # JavaScript wrapper with lazy loading
├── sea_core.js           # Generated WASM bindings
├── sea_core.d.ts         # TypeScript definitions
└── sea_core_bg.wasm      # Compiled WASM binary
```

## Type Conversions

| Rust Type       | JavaScript Type |
| --------------- | --------------- |
| `String`        | `string`        |
| `Uuid`          | `string`        |
| `Decimal`       | `string`        |
| `Option<T>`     | `T \| null`     |
| `Vec<T>`        | `T[]`           |
| `HashMap<K, V>` | `object`        |

## Related Packages

| Package                                                                        | Registry  | Description                  |
| ------------------------------------------------------------------------------ | --------- | ---------------------------- |
| [`sea-core`](https://crates.io/crates/sea-core)                                | crates.io | Rust core library            |
| [`sea-dsl`](https://pypi.org/project/sea-dsl/)                                 | PyPI      | Python bindings              |
| [`@domainforge/sea`](https://www.npmjs.com/package/@domainforge/sea)           | npm       | Native Node.js bindings      |
| [`@domainforge/sea-wasm`](https://www.npmjs.com/package/@domainforge/sea-wasm) | npm       | WASM bindings (this package) |

## Documentation

- 📖 [SEA DSL Guide](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/sea-dsl-syntax.md) — Language specification
- 🏗️ [Architecture](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/architecture.md) — Design overview
- 📚 [WASM Guide](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/wasm-api.md) — Full WASM API

## Troubleshooting

### WASM module fails to load

- Ensure server sends correct MIME type: `application/wasm`
- Check browser console for detailed errors

### Size exceeds 500KB

- Run `wasm-opt -Oz` optimization
- Check for unused dependencies

### TypeScript errors

- Ensure `sea_core.d.ts` is present in `pkg/`
- Check TypeScript version compatibility (4.5+)

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

---

Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) project.
