import { describe, expect, it } from 'vitest';
import { Entity, Flow, ResourceInstance, Resource } from '../index';

describe('Instance Advanced Features', () => {
    it('creates instance with namespace', () => {
        const entity = new Entity('Warehouse', 'logistics');
        const resource = new Resource('Camera', 'units', 'inventory');

        const instance = new ResourceInstance(resource.id, entity.id, 'inventory');
        expect(instance.namespace).toBe('inventory');
    });

    it('manages instance attributes', () => {
        const entity = new Entity('Warehouse');
        const resource = new Resource('Camera', 'units');
        const instance = new ResourceInstance(resource.id, entity.id);

        instance.setAttribute('serial_number', JSON.stringify('CAM-12345'));
        expect(JSON.parse(instance.getAttribute('serial_number')!)).toBe('CAM-12345');
    });

    it('handles complex instance metadata', () => {
        const entity = new Entity('Warehouse');
        const resource = new Resource('Camera', 'units');
        const instance = new ResourceInstance(resource.id, entity.id);

        const metadata = {
            serial: 'CAM-12345',
            manufactured: '2025-01-15',
            warranty_expires: '2027-01-15',
            specifications: {
                resolution: '4K',
                fps: 60
            }
        };

        instance.setAttribute('metadata', JSON.stringify(metadata));
        const retrieved = JSON.parse(instance.getAttribute('metadata')!);
        expect(retrieved.serial).toBe('CAM-12345');
        expect(retrieved.specifications.resolution).toBe('4K');
    });
});

describe('Attribute Edge Cases', () => {
    it('handles null attribute retrieval', () => {
        const entity = new Entity('Test');
        const attr = entity.getAttribute('nonexistent');
        expect(attr).toBeNull();
    });

    it('handles empty string attributes', () => {
        const entity = new Entity('Test');
        entity.setAttribute('empty', JSON.stringify(''));
        expect(JSON.parse(entity.getAttribute('empty')!)).toBe('');
    });

    it('handles numeric attributes', () => {
        const resource = new Resource('Item', 'units');
        resource.setAttribute('quantity', JSON.stringify(42));
        resource.setAttribute('price', JSON.stringify(99.99));

        expect(JSON.parse(resource.getAttribute('quantity')!)).toBe(42);
        expect(JSON.parse(resource.getAttribute('price')!)).toBe(99.99);
    });

    it('handles boolean attributes', () => {
        const entity = new Entity('Warehouse');
        entity.setAttribute('active', JSON.stringify(true));
        entity.setAttribute('requires_signature', JSON.stringify(false));

        expect(JSON.parse(entity.getAttribute('active')!)).toBe(true);
        expect(JSON.parse(entity.getAttribute('requires_signature')!)).toBe(false);
    });

    it('handles array attributes', () => {
        const entity = new Entity('Warehouse');
        const zones = ['A1', 'A2', 'B1', 'B2'];
        entity.setAttribute('zones', JSON.stringify(zones));

        const retrieved = JSON.parse(entity.getAttribute('zones')!);
        expect(retrieved).toEqual(zones);
        expect(retrieved.length).toBe(4);
    });
});

describe('Flow Attributes', () => {
    it('manages flow attributes', () => {
        const entity1 = new Entity('Warehouse');
        const entity2 = new Entity('Factory');
        const resource = new Resource('Cameras', 'units');

        const flow = new Flow(resource.id, entity1.id, entity2.id, 100);
        flow.setAttribute('priority', JSON.stringify('high'));
        flow.setAttribute('scheduled_date', JSON.stringify('2025-12-01'));

        expect(JSON.parse(flow.getAttribute('priority')!)).toBe('high');
        expect(JSON.parse(flow.getAttribute('scheduled_date')!)).toBe('2025-12-01');
    });

    it('handles decimal quantities correctly', () => {
        const entity1 = new Entity('Warehouse');
        const entity2 = new Entity('Factory');
        const resource = new Resource('Steel', 'kg');

        const flow = new Flow(resource.id, entity1.id, entity2.id, 1250.75);
        expect(flow.quantity).toBeCloseTo(1250.75, 2);
    });
});

describe('UUID Validation', () => {
    it('accepts valid UUIDs', () => {
        const validUuid = '550e8400-e29b-41d4-a716-446655440000';
        const entity1 = new Entity('Test1');
        const entity2 = new Entity('Test2');

        // This should not throw
        expect(() => {
            new Flow(validUuid, entity1.id, entity2.id, 100);
        }).not.toThrow();
    });

    it('rejects malformed UUIDs', () => {
        expect(() => {
            new Flow('not-a-uuid', 'also-not', 'still-not', 100);
        }).toThrow();
    });

    it('rejects empty string UUIDs', () => {
        expect(() => {
            new Flow('', '', '', 100);
        }).toThrow();
    });
});

describe('Namespace Handling', () => {
    it('entities without namespace have null namespace', () => {
        const entity = new Entity('Test');
        expect(entity.namespace).toBeNull();
    });

    it('entities with namespace preserve it', () => {
        const entity = new Entity('Test', 'domain');
        expect(entity.namespace).toBe('domain');
    });

    it('resources without namespace have null namespace', () => {
        const resource = new Resource('Item', 'units');
        expect(resource.namespace).toBeNull();
    });

    it('resources with namespace preserve it', () => {
        const resource = new Resource('Item', 'units', 'inventory');
        expect(resource.namespace).toBe('inventory');
    });

    it('instances can have different namespace than resource/entity', () => {
        const entity = new Entity('Warehouse', 'logistics');
        const resource = new Resource('Camera', 'units', 'inventory');
        const instance = new ResourceInstance(resource.id, entity.id, 'tracking');

        expect(entity.namespace).toBe('logistics');
        expect(resource.namespace).toBe('inventory');
        expect(instance.namespace).toBe('tracking');
    });
});
