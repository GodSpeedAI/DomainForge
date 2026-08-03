import { describe, expect, it } from 'bun:test';
import { Graph } from '../domainforge-typescript';

describe('typed entity contract binding', () => {
  it('exposes resolved fields and preserves bodyless compatibility', () => {
    const typed = Graph.parse(
      '@namespace "binding"\nentity "Item" { key item_id: string value: int }',
    );
    const entityId = typed.findEntityByName('Item');
    const contract = JSON.parse(typed.entityContractJson(entityId!)!);

    expect(contract.key_field).toBe('item_id');
    expect(contract.fields.map((field: { name: string }) => field.name)).toEqual([
      'item_id',
      'value',
    ]);

    const legacy = Graph.parse('entity "Legacy"');
    expect(legacy.entityContractJson(legacy.findEntityByName('Legacy')!)).toBeNull();
  });
});
