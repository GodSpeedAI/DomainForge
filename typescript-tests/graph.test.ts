import { describe, it, expect } from 'vitest';
import { Graph, Entity, Resource, Flow } from '../index';

describe('Graph', () => {
  it('creates empty graph', () => {
    const graph = new Graph();
    expect(graph.entityCount()).toBe(0);
    expect(graph.resourceCount()).toBe(0);
    expect(graph.flowCount()).toBe(0);
  });

  it('adds entities', () => {
    const graph = new Graph();
    const entity = new Entity('Warehouse');
    
    graph.addEntity(entity);
    expect(graph.entityCount()).toBe(1);
    expect(graph.hasEntity(entity.id)).toBe(true);
  });

  it('adds resources', () => {
    const graph = new Graph();
    const resource = new Resource('Cameras', 'units');
    
    graph.addResource(resource);
    expect(graph.resourceCount()).toBe(1);
    expect(graph.hasResource(resource.id)).toBe(true);
  });

  it('adds flows', () => {
    const graph = new Graph();
    const entity1 = new Entity('Warehouse');
    const entity2 = new Entity('Factory');
    const resource = new Resource('Cameras', 'units');
    
    graph.addEntity(entity1);
    graph.addEntity(entity2);
    graph.addResource(resource);
    
    const flow = new Flow(resource.id, entity1.id, entity2.id, 100);
    graph.addFlow(flow);
    
    expect(graph.flowCount()).toBe(1);
  });

  it('retrieves entities by ID', () => {
    const graph = new Graph();
    const entity = new Entity('Warehouse');
    
    graph.addEntity(entity);
    const retrieved = graph.getEntity(entity.id);
    
    expect(retrieved).not.toBeNull();
    expect(retrieved!.name).toBe('Warehouse');
  });

  it('finds entities by name', () => {
    const graph = new Graph();
    const entity = new Entity('Warehouse');
    
    graph.addEntity(entity);
    const foundId = graph.findEntityByName('Warehouse');
    
    expect(foundId).toBe(entity.id);
  });

  it('queries flows from entity', () => {
    const graph = new Graph();
    const entity1 = new Entity('Warehouse');
    const entity2 = new Entity('Factory');
    const resource = new Resource('Cameras', 'units');
    
    graph.addEntity(entity1);
    graph.addEntity(entity2);
    graph.addResource(resource);
    
    const flow = new Flow(resource.id, entity1.id, entity2.id, 100);
    graph.addFlow(flow);
    
    const flows = graph.flowsFrom(entity1.id);
    expect(flows.length).toBe(1);
    expect(flows[0].quantity).toBe(100);
  });

  it('queries flows to entity', () => {
    const graph = new Graph();
    const entity1 = new Entity('Warehouse');
    const entity2 = new Entity('Factory');
    const resource = new Resource('Cameras', 'units');
    
    graph.addEntity(entity1);
    graph.addEntity(entity2);
    graph.addResource(resource);
    
    const flow = new Flow(resource.id, entity1.id, entity2.id, 100);
    graph.addFlow(flow);
    
    const flows = graph.flowsTo(entity2.id);
    expect(flows.length).toBe(1);
    expect(flows[0].quantity).toBe(100);
  });

  it('lists all entities', () => {
    const graph = new Graph();
    graph.addEntity(new Entity('Warehouse'));
    graph.addEntity(new Entity('Factory'));
    
    const entities = graph.allEntities();
    expect(entities.length).toBe(2);
  });

  it('parses SEA DSL', () => {
    const source = `
      Entity "Warehouse" in logistics
      Entity "Factory" in manufacturing
      Resource "Cameras" units
      Flow "Cameras" from "Warehouse" to "Factory" quantity 100
    `;
    
    const graph = Graph.parse(source);
    expect(graph.entityCount()).toBe(2);
    expect(graph.resourceCount()).toBe(1);
    expect(graph.flowCount()).toBe(1);
  });

  it('parses complex DSL with namespaces', () => {
    const source = `
      Entity "Main Warehouse" in logistics
      Entity "Assembly Line" in manufacturing
      Resource "Electronic Components" units in electronics
      Flow "Electronic Components" from "Main Warehouse" to "Assembly Line" quantity 500
    `;
    
    const graph = Graph.parse(source);
    const entities = graph.allEntities();
    const resources = graph.allResources();
    
    expect(entities.some(e => e.namespace === 'logistics')).toBe(true);
    expect(entities.some(e => e.namespace === 'manufacturing')).toBe(true);
    expect(resources.some(r => r.namespace === 'electronics')).toBe(true);
  });
});
