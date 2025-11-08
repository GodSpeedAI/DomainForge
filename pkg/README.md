# @domainforge/sea-wasm

WebAssembly bindings for the SEA DSL (Semantic Entity Architecture Domain Specific Language). This package enables browser and edge runtime usage of the DomainForge SEA parser and graph engine.

## Installation

```bash
npm install @domainforge/sea-wasm
```

## Features

- ✅ WASM module <500KB gzipped
- ✅ Browser and Node.js support
- ✅ TypeScript definitions included
- ✅ Auto-initialization with lazy loading
- ✅ Full SEA DSL parser and graph API

## Quick Start

### Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
  <script type="module">
    import { Graph } from '@domainforge/sea-wasm';

    const source = `
      Entity "Warehouse" in logistics
      Resource "Cameras" units
      Flow "Cameras" from "Warehouse" to "Factory" quantity 100
    `;

    const graph = await Graph.parse(source);
    console.log('Entities:', graph.entityCount());
  </script>
</head>
<body>
  <h1>SEA DSL in Browser</h1>
</body>
</html>
```

### Node.js Usage

```javascript
import { Graph, Entity, Resource, Flow, preloadWasm } from '@domainforge/sea-wasm';

// Optional: Preload WASM for faster first use
await preloadWasm();

// Create entities programmatically
const warehouse = new Entity('Warehouse', 'logistics');
const factory = new Entity('Factory', 'manufacturing');
const cameras = new Resource('Cameras', 'units', null);

// Build a graph
const graph = new Graph();
await graph.addEntity(warehouse);
await graph.addEntity(factory);
await graph.addResource(cameras);

const flow = new Flow(cameras.id, warehouse.id, factory.id, '100');
await graph.addFlow(flow);

console.log('Graph has', graph.entityCount(), 'entities');

// Or parse from DSL
const source = `
  Entity "Warehouse" in logistics
  Entity "Factory" in manufacturing
  Resource "Cameras" units
  Flow "Cameras" from "Warehouse" to "Factory" quantity 100
`;

const parsedGraph = await Graph.parse(source);
console.log('Parsed graph:', parsedGraph.toJSON());
```

## API Reference

### Entity

```typescript
class Entity {
  constructor(name: string, namespace?: string | null);
  
  get id(): string;
  get name(): string;
  get namespace(): string | null;
  
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
  toJSON(): object;
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
  
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
  toJSON(): object;
}
```

### Flow

```typescript
class Flow {
  constructor(
    resourceId: string,
    fromId: string,
    toId: string,
    quantity: string
  );
  
  get id(): string;
  get resourceId(): string;
  get fromId(): string;
  get toId(): string;
  get quantity(): string;
  get namespace(): string | null;
  
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
  toJSON(): object;
}
```

### Instance

```typescript
class Instance {
  constructor(
    resourceId: string,
    entityId: string,
    namespace?: string | null
  );
  
  get id(): string;
  get resourceId(): string;
  get entityId(): string;
  get namespace(): string | null;
  
  setAttribute(key: string, value: any): void;
  getAttribute(key: string): any;
  toJSON(): object;
}
```

### Graph

```typescript
class Graph {
  constructor();
  static parse(source: string): Graph;
  
  // Entity operations
  addEntity(entity: Entity): void;
  hasEntity(id: string): boolean;
  getEntity(id: string): Entity | null;
  removeEntity(id: string): Entity;
  findEntityByName(name: string): string | null;
  entityCount(): number;
  allEntities(): Entity[];
  
  // Resource operations
  addResource(resource: Resource): void;
  hasResource(id: string): boolean;
  getResource(id: string): Resource | null;
  removeResource(id: string): Resource;
  findResourceByName(name: string): string | null;
  resourceCount(): number;
  allResources(): Resource[];
  
  // Flow operations
  addFlow(flow: Flow): void;
  hasFlow(id: string): boolean;
  getFlow(id: string): Flow | null;
  removeFlow(id: string): Flow;
  flowCount(): number;
  allFlows(): Flow[];
  
  // Instance operations
  addInstance(instance: Instance): void;
  hasInstance(id: string): boolean;
  getInstance(id: string): Instance | null;
  removeInstance(id: string): Instance;
  instanceCount(): number;
  allInstances(): Instance[];
  
  // Graph traversal
  flowsFrom(entityId: string): Flow[];
  flowsTo(entityId: string): Flow[];
  upstreamEntities(entityId: string): Entity[];
  downstreamEntities(entityId: string): Entity[];
  
  isEmpty(): boolean;
  toJSON(): object;
}
```

## Performance

- Bundle size: <500KB gzipped
- Lazy initialization for optimal load times
- High-precision decimal arithmetic for flow quantities
- UUID-based referential integrity

## License

MIT OR Apache-2.0

## Links

- [GitHub Repository](https://github.com/GodSpeedAI/domainforge)
- [Documentation](https://github.com/GodSpeedAI/domainforge/tree/main/docs)
- [Issue Tracker](https://github.com/GodSpeedAI/domainforge/issues)
