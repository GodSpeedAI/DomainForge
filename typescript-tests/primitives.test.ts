import { describe, it, expect } from 'vitest';
import { Entity, Resource, Flow, Instance } from '../index';

describe('Entity', () => {
  it('creates entity with name', () => {
    const entity = new Entity('Warehouse A');
    expect(entity.name).toBe('Warehouse A');
    expect(entity.id).toHaveLength(36);
  });

  it('supports namespaces', () => {
    const entity = new Entity('Factory', 'manufacturing');
    expect(entity.namespace).toBe('manufacturing');
  });

  it('manages attributes', () => {
    const entity = new Entity('Warehouse');
    entity.setAttribute('capacity', JSON.stringify(10000));
    expect(JSON.parse(entity.getAttribute('capacity')!)).toBe(10000);
  });

  it('handles complex attributes', () => {
    const entity = new Entity('Warehouse');
    entity.setAttribute('location', JSON.stringify({ lat: 40.7128, lng: -74.0060 }));
    const location = JSON.parse(entity.getAttribute('location')!);
    expect(location.lat).toBe(40.7128);
    expect(location.lng).toBe(-74.0060);
  });
});

describe('Resource', () => {
  it('creates resource with unit', () => {
    const resource = new Resource('Cameras', 'units');
    expect(resource.name).toBe('Cameras');
    expect(resource.unit).toBe('units');
    expect(resource.id).toHaveLength(36);
  });

  it('supports namespaces', () => {
    const resource = new Resource('Steel', 'kg', 'materials');
    expect(resource.namespace).toBe('materials');
  });

  it('manages attributes', () => {
    const resource = new Resource('Cameras', 'units');
    resource.setAttribute('model', JSON.stringify('HD-2000'));
    expect(JSON.parse(resource.getAttribute('model')!)).toBe('HD-2000');
  });
});

describe('Flow', () => {
  it('creates flow between entities', () => {
    const entity1 = new Entity('Warehouse');
    const entity2 = new Entity('Factory');
    const resource = new Resource('Cameras', 'units');
    
    const flow = new Flow(resource.id, entity1.id, entity2.id, 100);
    expect(flow.resourceId).toBe(resource.id);
    expect(flow.fromId).toBe(entity1.id);
    expect(flow.toId).toBe(entity2.id);
    expect(flow.quantity).toBe(100);
  });

  it('throws on invalid UUID', () => {
    expect(() => {
      new Flow('invalid-uuid', 'also-invalid', 'still-invalid', 100);
    }).toThrow();
  });
});

describe('Instance', () => {
  it('creates instance', () => {
    const entity = new Entity('Warehouse');
    const resource = new Resource('Cameras', 'units');
    
    const instance = new Instance(resource.id, entity.id);
    expect(instance.resourceId).toBe(resource.id);
    expect(instance.entityId).toBe(entity.id);
  });

  it('supports namespaces', () => {
    const entity = new Entity('Warehouse');
    const resource = new Resource('Cameras', 'units');
    
    const instance = new Instance(resource.id, entity.id, 'inventory');
    expect(instance.namespace).toBe('inventory');
  });
});
