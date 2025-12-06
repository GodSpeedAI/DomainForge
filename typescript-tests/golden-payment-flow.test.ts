import { describe, expect, it } from 'vitest';
import { Graph } from '../index';

const PAYMENT_DSL = `
Role "Payer"
Role "Payee"

Resource "Money" units

Entity "Alice"
Entity "Bob"

Flow "Money" from "Alice" to "Bob" quantity 10

Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  via: flow "Money"
`;

describe('Payment role flow golden path', () => {
    it('parses roles, relations, and flows consistently', () => {
        const graph = Graph.parse(PAYMENT_DSL);

        expect(graph.entityCount()).toBe(2);
        expect(graph.resourceCount()).toBe(1);
        expect(graph.flowCount()).toBe(1);
        expect(graph.roleCount()).toBe(2);
        expect(graph.relationCount()).toBe(1);

        const roles = Object.fromEntries(
            graph.allRoles().map((role) => [role.name, role.id])
        );
        expect(roles).toHaveProperty('Payer');
        expect(roles).toHaveProperty('Payee');

        const relations = graph.allRelations();
        expect(relations).toHaveLength(1);
        expect(relations[0].name).toBe('Payment');
        expect(relations[0].predicate).toBe('pays');
        expect(relations[0].subjectRoleId).toBe(roles.Payer);
        expect(relations[0].objectRoleId).toBe(roles.Payee);
    });
});
