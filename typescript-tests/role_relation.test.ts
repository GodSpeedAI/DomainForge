import { describe, it, expect } from 'vitest';
import { Role, Relation } from '../index';
import { randomUUID } from 'crypto';

describe('Role', () => {
    it('creates role with name', () => {
        const role = new Role('Approver');
        expect(role.name).toBe('Approver');
        expect(role.id).toHaveLength(36);
    });

    it('supports namespaces', () => {
        const role = new Role('Viewer', 'governance');
        expect(role.namespace).toBe('governance');
    });

    it('manages attributes', () => {
        const role = new Role('Approver');
        role.setAttribute('level', JSON.stringify(1));
        const val = JSON.parse(role.getAttribute('level')!);
        expect(val).toBe(1);
    });
});

describe('Relation', () => {
    it('creates relation between roles', () => {
        const subject = new Role('Payer');
        const object = new Role('Payee');
        const flowId = '00000000-0000-0000-0000-000000000000'; // mock or use uuid

        const relation = new Relation(
            'Payment',
            subject.id,
            'pays',
            object.id,
            null, // namespace
            null  // via_flow_id
        );

        expect(relation.name).toBe('Payment');
        expect(relation.predicate).toBe('pays');
        expect(relation.subjectRoleId).toBe(subject.id);
        expect(relation.objectRoleId).toBe(object.id);
        expect(relation.viaFlowId).toBeNull();
    });

    it('creates relation with namespace and flow', () => {
        const subject = new Role('Payer');
        const object = new Role('Payee');
        // valid UUID required for flowId
        const flowId = '123e4567-e89b-12d3-a456-426614174000'; 

        const relation = new Relation(
            'Payment',
            subject.id,
            'pays',
            object.id,
            'finance',
            flowId
        );

        expect(relation.name).toBe('Payment');
        expect(relation.viaFlowId).toBe(flowId);
    });
});
