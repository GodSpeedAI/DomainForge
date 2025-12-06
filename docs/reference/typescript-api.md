# @domainforge/sea

TypeScript/Node.js bindings for the SEA (Semantic Enterprise Architecture) DSL.

## Installation

```bash
# From source (npm package coming soon)
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
npm install
npm run build

# The build produces native .node bindings
```

## Quick Start

```typescript
import { Graph, Entity, Resource, Flow } from '@domainforge/sea';

// Create a graph programmatically
const graph = new Graph();

// Constructor patterns - new() for default namespace, newWithNamespace() for explicit
const warehouse = Entity.new('Warehouse');  // Default namespace
const factory = Entity.newWithNamespace('Factory', 'manufacturing');

// Namespace is always a string (not undefined), defaults to "default"
console.log(warehouse.namespace());  // "default"
console.log(factory.namespace());    // "manufacturing"

const cameras = Resource.new('Cameras', 'units');

graph.addEntity(warehouse);
graph.addEntity(factory);
graph.addResource(cameras);

// Flow constructor takes ConceptId values - clone before passing
const flow = Flow.new(
  cameras.id().clone(),
  warehouse.id().clone(),
  factory.id().clone(),
  100
);
graph.addFlow(flow);

console.log(`Graph has ${graph.entityCount()} entities`);
console.log(`Graph has ${graph.flowCount()} flows`);
```

## Parsing SEA DSL

```typescript
import { Graph } from '@domainforge/sea';

// Supports multiline strings with """ syntax
const source = `
  Entity "Warehouse" in logistics
  Entity """Multi-line
  Factory Name""" in manufacturing
  Resource "Cameras" units
  Flow "Cameras" from "Warehouse" to "Multi-line\\nFactory Name" quantity 100
`;

const graph = Graph.parse(source);

console.log(`Parsed ${graph.entityCount()} entities`);
console.log(`Parsed ${graph.flowCount()} flows`);
```

### Error Handling

`Graph.parse(source: string)` may throw an `Error` on invalid input. The binding typically surfaces syntax issues as a standard `Error` with a message that includes parser details (line/column); some runtimes may implement a specific `ParseError` subclass. Callers should wrap parsing in a try/catch and handle errors accordingly:

```ts
try {
  const g = Graph.parse(source);
} catch (e) {
  // `e` typically includes a message and a line/column in the text
  console.error('Failed to parse', e);
}
```

## API Reference

### Entity

```typescript
class Entity {
  // Constructor patterns (November 2025)
  static new(name: string): Entity;  // Default namespace
  static newWithNamespace(name: string, namespace: string): Entity;  // Explicit namespace

  id(): ConceptId;
  name(): string;
  namespace(): string;  // Always returns string, never undefined (defaults to "default")
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Resource

```typescript
class Resource {
  // Constructor patterns (November 2025)
  static new(name: string, unit: string): Resource;  // Default namespace
  static newWithNamespace(name: string, unit: string, namespace: string): Resource;

  id(): ConceptId;
  name(): string;
  unit(): string;
  namespace(): string;  // Always returns string (defaults to "default")
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Flow

```typescript
class Flow {
  // Constructor takes ConceptId values (not references) - clone before passing
  static new(resourceId: ConceptId, fromId: ConceptId, toId: ConceptId, quantity: number): Flow;

  id(): ConceptId;
  resourceId(): ConceptId;
  fromId(): ConceptId;
  toId(): ConceptId;
  quantity(): number;
  namespace(): string;
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Instance

```typescript
class Instance {
  static new(resourceId: ConceptId, entityId: ConceptId): Instance;  // Default namespace
  static newWithNamespace(resourceId: ConceptId, entityId: ConceptId, namespace: string): Instance;

  id(): ConceptId;
  resourceId(): ConceptId;
  entityId(): ConceptId;
  namespace(): string;  // Always returns string (defaults to "default")
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Graph

```typescript
class Graph {
  constructor();

  // Add primitives (validates referential integrity)
  addEntity(entity: Entity): void;
  addResource(resource: Resource): void;
  addFlow(flow: Flow): void;  // Throws if Entity/Resource references invalid
  addInstance(instance: Instance): void;

  // Counts
  entityCount(): number;
  resourceCount(): number;
  flowCount(): number;
  instanceCount(): number;

  // Lookup by ID
  hasEntity(id: ConceptId): boolean;
  getEntity(id: ConceptId): Entity | null;
  getResource(id: ConceptId): Resource | null;
  getFlow(id: ConceptId): Flow | null;
  getInstance(id: ConceptId): Instance | null;

  // Lookup by name
  findEntityByName(name: string): ConceptId | null;
  findResourceByName(name: string): ConceptId | null;

  // Flow queries
  flowsFrom(entityId: ConceptId): Flow[];
  flowsTo(entityId: ConceptId): Flow[];

  // Get all (IndexMap ensures deterministic iteration order)
  allEntities(): Entity[];
  allResources(): Resource[];
  allFlows(): Flow[];
  allInstances(): Instance[];

  // Parsing (supports multiline strings with """)
  static parse(source: string): Graph;

  // CALM integration (architecture-as-code)
  exportCalm(): string;  // Returns CALM JSON string
  static importCalm(json: string): Graph;  // Import from CALM JSON
}

`Graph.importCalm` will throw if the JSON is invalid or violates schema constraints; callers should also wrap `importCalm` in try/catch and inspect error messages for details.
```

### NamespaceRegistry (Workspace)

```typescript
import { NamespaceRegistry } from '@domainforge/sea';

// Load a registry by path to the file
const reg = NamespaceRegistry.from_file('./.sea-registry.toml');

// Expand files and get bindings
const files = reg.resolve_files(); // or reg.resolve_files(true) to fail on ambiguity via the failOnAmbiguity flag
for (const f of files) {
  console.log(f.path, '=>', f.namespace);
}

// Query namespace for a single file
const ns = reg.namespace_for('/path/to/file.sea'); // or pass true as the failOnAmbiguity flag to error on ambiguous matches
console.log('Namespace:', ns);
```


## Advanced Usage

### Working with Attributes

```typescript
// Use new() for default namespace
const entity = Entity.new('Warehouse');
entity.setAttribute('capacity', JSON.stringify(10000));
entity.setAttribute('location', JSON.stringify({ lat: 40.7128, lng: -74.0060 }));

const capacity = JSON.parse(entity.getAttribute('capacity')!); // 10000
const location = JSON.parse(entity.getAttribute('location')!); // { lat: 40.7128, lng: -74.0060 }

// Namespace is always present (not undefined)
console.log(entity.namespace());  // "default"
```

### Querying Flow Networks

```typescript
const graph = Graph.parse(`
  Entity "Supplier"
  Entity "Warehouse"
  Entity "Retailer"
  Resource "Products" units
  Flow "Products" from "Supplier" to "Warehouse" quantity 1000
  Flow "Products" from "Warehouse" to "Retailer" quantity 800
`);

const warehouseId = graph.findEntityByName('Warehouse');
const inboundFlows = graph.flowsTo(warehouseId!);
const outboundFlows = graph.flowsFrom(warehouseId!);

console.log(`Warehouse receives ${inboundFlows.length} flows`);
console.log(`Warehouse sends ${outboundFlows.length} flows`);
```

## Building from Source

```bash
# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test
```


## Platform Support

Pre-built binaries are available for:
- Linux x64
- macOS ARM64 (Apple Silicon)
- Windows x64

Build from source for other platforms using `npm run build`.

## License

Apache-2.0
