/**
 * TypeScript binding surface for the Lean 4 projection (Graph.exportLean).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'lean', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

describe('Graph.exportLean', () => {
    it('emits the Lake package layout with decide proofs', () => {
        const artifacts = JSON.parse(fixtureGraph().exportLean(undefined, FIXED_TS));
        expect(artifacts['lakefile.toml']).toContain('defaultTargets = ["DomainForge"]');
        const policies = artifacts['DomainForge/Policies.lean'];
        expect(policies).toContain(
            'theorem policy_positive_flow_holds : policy_positive_flow := by decide',
        );
        expect(policies).not.toContain('sorry');
        expect(artifacts['Obligations/Stubs.lean']).toContain('by sorry');
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportLean(undefined, FIXED_TS)).toEqual(
            graph.exportLean(undefined, FIXED_TS),
        );
    });
});
