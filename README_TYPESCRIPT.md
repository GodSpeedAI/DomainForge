# @domainforge/sea

TypeScript/Node.js bindings for the SEA (Semantic Enterprise Architecture) DSL.

## Installation

```bash
npm install @domainforge/sea
```

## Quick Start

```typescript
import { Graph, Entity, Resource, Flow } from "@domainforge/sea";

// Create a graph programmatically
const graph = new Graph();

// Constructor patterns - use new ClassName(args) syntax
const warehouse = new Entity("Warehouse"); // Default namespace
const factory = new Entity("Factory", "manufacturing"); // Explicit namespace

// Namespace can be null if not specified
console.log(warehouse.namespace); // null (default)
console.log(factory.namespace); // "manufacturing"

const cameras = new Resource("Cameras", "units");

graph.addEntity(warehouse);
graph.addEntity(factory);
graph.addResource(cameras);

// Flow constructor takes string IDs
const flow = new Flow(cameras.id, warehouse.id, factory.id, 100);
graph.addFlow(flow);

console.log(`Graph has ${graph.entityCount()} entities`);
console.log(`Graph has ${graph.flowCount()} flows`);
```

## Parsing SEA DSL

```typescript
import { Graph } from "@domainforge/sea";

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

## API Reference

### Entity

```typescript
class Entity {
  // Constructor (December 2025)
  constructor(name: string, namespace?: string | null);

  get id(): string;
  get name(): string;
  get namespace(): string | null; // null if not specified
  setAttribute(key: string, valueJson: string): void;
  getAttribute(key: string): string | null;
  toString(): string;
}
```

### Resource

```typescript
class Resource {
  // Constructor (December 2025)
  constructor(name: string, unit: string, namespace?: string | null);

  get id(): string;
  get name(): string;
  get unit(): string;
  get namespace(): string | null; // null if not specified
  setAttribute(key: string, valueJson: string): void;
  getAttribute(key: string): string | null;
  toString(): string;
}
```

### Flow

```typescript
class Flow {
  // Constructor takes string IDs (December 2025)
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
  get namespace(): string | null;
  setAttribute(key: string, valueJson: string): void;
  getAttribute(key: string): string | null;
  toString(): string;
}
```

### Instance

```typescript
// ResourceInstance - represents a physical instance of a resource at an entity location
class ResourceInstance {
  constructor(resourceId: string, entityId: string, namespace?: string | null);

  get id(): string;
  get resourceId(): string;
  get entityId(): string;
  get namespace(): string | null;
  setAttribute(key: string, valueJson: string): void;
  getAttribute(key: string): string | null;
  toString(): string;
}

// Instance - represents an instance of an entity type with named fields
class Instance {
  constructor(name: string, entityType: string, namespace?: string | null);

  get id(): string;
  get name(): string;
  get entityType(): string;
  get namespace(): string | null;
  setField(key: string, valueJson: string): void;
  getField(key: string): string | null;
  toString(): string;
}
```

### Graph

```typescript
class Graph {
  constructor();

  // Add primitives (validates referential integrity)
  addEntity(entity: Entity): void;
  addResource(resource: Resource): void;
  addFlow(flow: Flow): void; // Throws if Entity/Resource references invalid
  addInstance(instance: ResourceInstance): void;
  addRole(role: Role): void;
  addRelation(relation: Relation): void;

  // Counts
  entityCount(): number;
  resourceCount(): number;
  flowCount(): number;
  instanceCount(): number;
  roleCount(): number;
  relationCount(): number;

  // Lookup by ID (IDs are strings)
  hasEntity(id: string): boolean;
  getEntity(id: string): Entity | null;
  getResource(id: string): Resource | null;
  getFlow(id: string): Flow | null;
  getInstance(id: string): ResourceInstance | null;

  // Lookup by name (returns string ID or null)
  findEntityByName(name: string): string | null;
  findResourceByName(name: string): string | null;
  findRoleByName(name: string): string | null;

  // Flow queries
  flowsFrom(entityId: string): Flow[];
  flowsTo(entityId: string): Flow[];

  // Get all (IndexMap ensures deterministic iteration order)
  allEntities(): Entity[];
  allResources(): Resource[];
  allFlows(): Flow[];
  allInstances(): ResourceInstance[];
  allRoles(): Role[];
  allRelations(): Relation[];

  // Parsing (supports multiline strings with """)
  static parse(source: string): Graph;

  // CALM integration (architecture-as-code)
  exportCalm(): string; // Returns CALM JSON string
  static importCalm(calmJson: string): Graph; // Import from CALM JSON

  // Policy evaluation
  addPolicy(policyJson: string): void;
  addAssociation(owner: string, owned: string, relType: string): void;
  evaluatePolicy(policyJson: string): EvaluationResult;
  setEvaluationMode(useThreeValuedLogic: boolean): void;
  useThreeValuedLogic(): boolean;

  toString(): string;
}
```

### NamespaceRegistry (Workspace)

```typescript
import { NamespaceRegistry } from "@domainforge/sea";

// Load a registry by path to the file
const reg = NamespaceRegistry.fromFile("./.sea-registry.toml");

// Expand files and get bindings
const files = reg.resolveFiles(); // or reg.resolveFiles(true) to fail on ambiguity
for (const f of files) {
  console.log(f.path, "=>", f.namespace);
}

// Query namespace for a single file
const ns = reg.namespaceFor("/path/to/file.sea"); // or pass true to error on ambiguous matches
console.log("Namespace:", ns);
```

## Advanced Usage

### Working with Attributes

```typescript
// Use constructor for creating entities
const entity = new Entity("Warehouse");
entity.setAttribute("capacity", JSON.stringify(10000));
entity.setAttribute("location", JSON.stringify({ lat: 40.7128, lng: -74.006 }));

const capacity = JSON.parse(entity.getAttribute("capacity")!); // 10000
const location = JSON.parse(entity.getAttribute("location")!); // { lat: 40.7128, lng: -74.0060 }

// Namespace is null when not specified
console.log(entity.namespace); // null
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

const warehouseId = graph.findEntityByName("Warehouse");
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
