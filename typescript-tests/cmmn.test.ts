/**
 * TypeScript binding surface for the CMMN 1.1 projection (Graph.exportCmmn).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'cmmn', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

describe('Graph.exportCmmn', () => {
    it('emits a single CMMN file with a case, roles, tasks, and sentries', () => {
        const artifacts = JSON.parse(fixtureGraph().exportCmmn(undefined, FIXED_TS));
        expect(Object.keys(artifacts)).toEqual(['model.cmmn']);

        const cmmn = artifacts['model.cmmn'];
        expect(cmmn).toContain('<definitions');
        expect(cmmn).toContain('<casePlanModel');
        expect(cmmn).toContain('name="Budget"'); // case file item
        expect(cmmn).toContain('name="Approver"'); // case role
        expect(cmmn).toContain('<milestone');
        expect(cmmn).toContain('<sentry'); // policy condition lowered
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportCmmn(undefined, FIXED_TS)).toEqual(
            graph.exportCmmn(undefined, FIXED_TS),
        );
    });
});
