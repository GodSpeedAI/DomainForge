# domainforge

[![npm](https://img.shields.io/npm/v/domainforge.svg)](https://www.npmjs.com/package/domainforge)
[![npm downloads](https://img.shields.io/npm/dm/domainforge.svg)](https://www.npmjs.com/package/domainforge)
[![License](https://img.shields.io/npm/l/domainforge.svg)](https://github.com/GodSpeedAI/DomainForge/blob/main/LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)
[![Node Version](https://img.shields.io/node/v/domainforge.svg)](https://nodejs.org)

TypeScript/Node.js bindings for the **SEA DSL** (Semantic Enterprise Architecture) domain-specific language. Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) ecosystem.

## Features

- 🏗️ **Domain Primitives** — Entities, Resources, Flows, Roles, Relations, Instances
- 📐 **Unit System** — First-class dimensional analysis with type-safe quantities
- ✅ **Policy Engine** — Constraint validation with three-valued logic
- 🔄 **DSL Parsing** — Parse SEA source code into queryable graph structures
- 🌐 **CALM Integration** — Export/import FINOS CALM architecture-as-code format
- ⚡ **Native Performance** — Rust-powered core via N-API
- 📦 **Full TypeScript Support** — Complete type definitions included

## Installation

```bash
npm install domainforge
```

```bash
yarn add domainforge
```

```bash
pnpm add domainforge
```

**Requires:** Node.js 18+

## Quick Start

### Parse from DSL

```typescript
import { Graph } from "domainforge";

const source = `
  @namespace "supply_chain"
  
  Entity "Warehouse" in logistics
  Entity "Factory" in manufacturing
  
  Resource "Cameras" units
  
  Flow "Cameras" from "Warehouse" to "Factory" quantity 100
`;

const graph = Graph.parse(source);

console.log(`Entities: ${graph.entityCount()}`);
console.log(`Resources: ${graph.resourceCount()}`);
console.log(`Flows: ${graph.flowCount()}`);
```

### Build Programmatically

```typescript
import { Graph, Entity, Resource, Flow } from "domainforge";

const graph = new Graph();

// Create primitives
const warehouse = new Entity("Warehouse", "logistics");
const factory = new Entity("Factory", "manufacturing");
const cameras = new Resource("Cameras", "units");

graph.addEntity(warehouse);
graph.addEntity(factory);
graph.addResource(cameras);

// Create flow
const flow = new Flow(cameras.id, warehouse.id, factory.id, 100);
graph.addFlow(flow);

// Query the graph
for (const entity of graph.allEntities()) {
  console.log(`${entity.name} in ${entity.namespace}`);
}
```

### Work with Attributes

```typescript
const entity = new Entity("Warehouse");
entity.setAttribute("capacity", JSON.stringify(10000));
entity.setAttribute("location", JSON.stringify({ lat: 40.7128, lng: -74.006 }));

const capacity = JSON.parse(entity.getAttribute("capacity")!); // 10000
const location = JSON.parse(entity.getAttribute("location")!);
```

### CALM Integration

```typescript
// Export to CALM JSON
const calmJson = graph.exportCalm();

// Import from CALM
const imported = Graph.importCalm(calmJson);
```

## API Reference

### Core Classes

| Class              | Description                                            |
| ------------------ | ------------------------------------------------------ |
| `Entity`           | Business actors, locations, organizational units (WHO) |
| `Resource`         | Quantifiable subjects of value (WHAT)                  |
| `Flow`             | Transfers of resources between entities                |
| `Instance`         | Entity type instances with named fields                |
| `ResourceInstance` | Physical instances at entity locations                 |
| `Role`             | Roles that entities can play                           |
| `Relation`         | Relationships between roles                            |
| `Graph`            | Container with validation and query capabilities       |

### Entity

```typescript
class Entity {
  constructor(name: string, namespace?: string | null);

  get id(): string;
  get name(): string;
  get namespace(): string | null;
  setAttribute(key: string, valueJson: string): void;
  getAttribute(key: string): string | null;
}
```

### Resource

```typescript
class Resource {
  constructor(name: string, unit: string, namespace?: string | null);

  get id(): string;
  get name(): string;
  get unit(): string;
  get namespace(): string | null;
}
```

### Flow

```typescript
class Flow {
  constructor(
    resourceId: string,
    fromId: string,
    toId: string,
    quantity: number
  );

  get id(): string;
  get resourceId(): string;
  get fromId(): string;
  get toId(): string;
  get quantity(): number;
}
```

### Graph

```typescript
class Graph {
  constructor();

  // Add primitives
  addEntity(entity: Entity): void;
  addResource(resource: Resource): void;
  addFlow(flow: Flow): void;

  // Counts
  entityCount(): number;
  resourceCount(): number;
  flowCount(): number;

  // Lookup
  findEntityByName(name: string): string | null;
  findResourceByName(name: string): string | null;
  getEntity(id: string): Entity | null;
  getResource(id: string): Resource | null;

  // Flow queries
  flowsFrom(entityId: string): Flow[];
  flowsTo(entityId: string): Flow[];

  // Get all
  allEntities(): Entity[];
  allResources(): Resource[];
  allFlows(): Flow[];

  // Parsing
  static parse(source: string): Graph;

  // CALM integration
  exportCalm(): string;
  static importCalm(calmJson: string): Graph;

  // Policy evaluation
  addPolicy(policyJson: string): void;
  evaluatePolicy(policyJson: string): EvaluationResult;
  setEvaluationMode(useThreeValuedLogic: boolean): void;
}
```

### NamespaceRegistry

```typescript
import { NamespaceRegistry } from "domainforge";

// Load workspace registry
const reg = NamespaceRegistry.fromFile(".sea-registry.toml");

// Resolve files
for (const binding of reg.resolveFiles()) {
  console.log(`${binding.path} => ${binding.namespace}`);
}

// Query namespace for file
const ns = reg.namespaceFor("/path/to/file.sea");
```

## Platform Support

Pre-built binaries are available for:

| Platform | Architecture               |
| -------- | -------------------------- |
| Linux    | x64, arm64                 |
| macOS    | x64, arm64 (Apple Silicon) |
| Windows  | x64                        |

Build from source for other platforms using `npm run build`.

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge

# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test
```

## Related Packages

| Package                                                              | Registry  | Description                        |
| -------------------------------------------------------------------- | --------- | ---------------------------------- |
| [`domainforge-core`](https://crates.io/crates/domainforge-core)                      | crates.io | Rust core library                  |
| [`domainforge`](https://pypi.org/project/domainforge/)                       | PyPI      | Python bindings                    |
| [`domainforge`](https://www.npmjs.com/package/domainforge) | npm       | TypeScript bindings (this package) |

## Policy Authority

DomainForge includes a Policy Authority system for executable business authority:

```typescript
import { evaluateAuthority, FinalDecision, PolicyModality, SourceClass, ClaimLevel } from 'domainforge';

const result = evaluateAuthority(configJson, requestJson, factsJson);
const decision = JSON.parse(result.decisionJson);

if (decision.finalDecision === FinalDecision.Deny) {
  console.log('Action denied:', decision.reasonCode);
}
```

**Available exports:**
- `FinalDecision` enum — Allow, Deny, Escalate, NotApplicable, Reject
- `PolicyModality` enum — Permission, Prohibition, Obligation, Override
- `SourceClass` enum — CallerSupplied, RuntimeObserved, SystemOfRecord, etc.
- `ClaimLevel` enum — AuditBacked, Validated, FormallyProven
- `evaluateAuthority(configJson, requestJson, factsJson?)` — One-shot evaluation

## Documentation

- 📖 [SEA DSL Guide](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/grammar-spec.md) — Language specification
- 🏗️ [Architecture](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/explanations/architecture-overview.md) — Design overview
- 📚 [API Reference](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/typescript-api.md) — Full TypeScript API

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

---

Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) project.
