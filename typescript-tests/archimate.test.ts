/**
 * TypeScript binding surface for the ArchiMate 3.0 projection (Graph.exportArchimate).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'archimate', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

describe('Graph.exportArchimate', () => {
    it('emits a single ArchiMate model with elements, relations, and a view', () => {
        const artifacts = JSON.parse(fixtureGraph().exportArchimate(undefined, FIXED_TS));
        expect(Object.keys(artifacts)).toEqual(['model.xml']);

        const xml = artifacts['model.xml'];
        expect(xml).toContain('<model');
        expect(xml).toContain('xsi:type="BusinessRole"');
        expect(xml).toContain('xsi:type="BusinessObject"');
        expect(xml).toContain('xsi:type="BusinessProcess"'); // flows
        expect(xml).toContain('xsi:type="Requirement"'); // policies
        expect(xml).toContain('xsi:type="Association"'); // policy references object
        expect(xml).toContain('xsi:type="Diagram"'); // the business-layer view
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportArchimate(undefined, FIXED_TS)).toEqual(
            graph.exportArchimate(undefined, FIXED_TS),
        );
    });
});
