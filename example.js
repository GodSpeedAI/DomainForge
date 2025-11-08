const { Graph, Entity, Resource, Flow } = require('./index');

console.log('=== DomainForge SEA TypeScript Example ===\n');

// Example 1: Create graph programmatically
console.log('Example 1: Programmatic Graph Creation');
const graph = new Graph();

const warehouse = new Entity('Warehouse', 'logistics');
const factory = new Entity('Factory', 'manufacturing');
const cameras = new Resource('Cameras', 'units');

warehouse.setAttribute('capacity', JSON.stringify(10000));
warehouse.setAttribute('location', JSON.stringify({ city: 'Seattle', country: 'USA' }));

graph.addEntity(warehouse);
graph.addEntity(factory);
graph.addResource(cameras);

const flow = new Flow(cameras.id, warehouse.id, factory.id, 100);
graph.addFlow(flow);

console.log(`  Entities: ${graph.entityCount()}`);
console.log(`  Resources: ${graph.resourceCount()}`);
console.log(`  Flows: ${graph.flowCount()}`);
console.log(`  Warehouse capacity: ${JSON.parse(warehouse.getAttribute('capacity'))}`);
console.log('');

// Example 2: Parse SEA DSL
console.log('Example 2: Parsing SEA DSL');
const source = `
  Entity "Supplier" in supply_chain
  Entity "Distribution Center" in logistics
  Entity "Retail Store" in retail
  Resource "Electronics" units in products
  Flow "Electronics" from "Supplier" to "Distribution Center" quantity 5000
  Flow "Electronics" from "Distribution Center" to "Retail Store" quantity 3000
`;

const parsedGraph = Graph.parse(source);
console.log(`  Parsed entities: ${parsedGraph.entityCount()}`);
console.log(`  Parsed resources: ${parsedGraph.resourceCount()}`);
console.log(`  Parsed flows: ${parsedGraph.flowCount()}`);

// Example 3: Query flows
console.log('\nExample 3: Querying Flow Network');
const dcId = parsedGraph.findEntityByName('Distribution Center');
if (dcId) {
  const inbound = parsedGraph.flowsTo(dcId);
  const outbound = parsedGraph.flowsFrom(dcId);
  console.log(`  Distribution Center receives ${inbound.length} flows`);
  console.log(`  Distribution Center sends ${outbound.length} flows`);
  if (inbound.length > 0) {
    console.log(`  Inbound quantity: ${inbound[0].quantity}`);
  }
  if (outbound.length > 0) {
    console.log(`  Outbound quantity: ${outbound[0].quantity}`);
  }
}

console.log('\n=== Example Complete ===');
