# @domainforge/sea

TypeScript/Node.js bindings for the SEA (Semantic Enterprise Architecture) DSL.

## Installation

```bash
npm install @domainforge/sea
```

## Quick Start

```typescript
import { Graph, Entity, Resource, Flow } from '@domainforge/sea';

// Create a graph programmatically
const graph = new Graph();

const warehouse = new Entity('Warehouse', 'logistics');
const factory = new Entity('Factory', 'manufacturing');
const cameras = new Resource('Cameras', 'units');

graph.addEntity(warehouse);
graph.addEntity(factory);
graph.addResource(cameras);

const flow = new Flow(cameras.id, warehouse.id, factory.id, 100);
graph.addFlow(flow);

console.log(`Graph has ${graph.entityCount()} entities`);
console.log(`Graph has ${graph.flowCount()} flows`);
```

## Parsing SEA DSL

```typescript
import { Graph } from '@domainforge/sea';

const source = `
  Entity "Warehouse" in logistics
  Entity "Factory" in manufacturing
  Resource "Cameras" units
  Flow "Cameras" from "Warehouse" to "Factory" quantity 100
`;

const graph = Graph.parse(source);

console.log(`Parsed ${graph.entityCount()} entities`);
console.log(`Parsed ${graph.flowCount()} flows`);
```

## API Reference

### Entity

```typescript
class Entity {
  constructor(name: string, namespace?: string);
  readonly id: string;
  readonly name: string;
  readonly namespace?: string;
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Resource

```typescript
class Resource {
  constructor(name: string, unit: string, namespace?: string);
  readonly id: string;
  readonly name: string;
  readonly unit: string;
  readonly namespace?: string;
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Flow

```typescript
class Flow {
  constructor(resourceId: string, fromId: string, toId: string, quantity: number);
  readonly id: string;
  readonly resourceId: string;
  readonly fromId: string;
  readonly toId: string;
  readonly quantity: number;
  readonly namespace?: string;
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
}
```

### Instance

```typescript
class Instance {
  constructor(resourceId: string, entityId: string, namespace?: string);
  readonly id: string;
  readonly resourceId: string;
  readonly entityId: string;
  readonly namespace?: string;
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
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
  addInstance(instance: Instance): void;
  
  // Counts
  entityCount(): number;
  resourceCount(): number;
  flowCount(): number;
  instanceCount(): number;
  
  // Lookup by ID
  hasEntity(id: string): boolean;
  getEntity(id: string): Entity | null;
  getResource(id: string): Resource | null;
  getFlow(id: string): Flow | null;
  getInstance(id: string): Instance | null;
  
  // Lookup by name
  findEntityByName(name: string): string | null;
  findResourceByName(name: string): string | null;
  
  // Flow queries
  flowsFrom(entityId: string): Flow[];
  flowsTo(entityId: string): Flow[];
  
  // Get all
  allEntities(): Entity[];
  allResources(): Resource[];
  allFlows(): Flow[];
  allInstances(): Instance[];
  
  // Parsing
  static parse(source: string): Graph;
}
```

## Advanced Usage

### Working with Attributes

```typescript
const entity = new Entity('Warehouse');
entity.setAttribute('capacity', JSON.stringify(10000));
entity.setAttribute('location', JSON.stringify({ lat: 40.7128, lng: -74.0060 }));

const capacity = JSON.parse(entity.getAttribute('capacity')!); // 10000
const location = JSON.parse(entity.getAttribute('location')!); // { lat: 40.7128, lng: -74.0060 }
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

## License

MIT OR Apache-2.0
