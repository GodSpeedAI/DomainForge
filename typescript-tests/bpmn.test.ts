/**
 * TypeScript binding surface for the BPMN 2.0 projection (Graph.exportBpmn).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'bpmn', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

describe('Graph.exportBpmn', () => {
    it('emits a single BPMN file with a branching process', () => {
        const artifacts = JSON.parse(fixtureGraph().exportBpmn(undefined, FIXED_TS));
        expect(Object.keys(artifacts)).toEqual(['model.bpmn']);

        const bpmn = artifacts['model.bpmn'];
        expect(bpmn).toContain('<definitions');
        expect(bpmn).toContain('isExecutable="false"');
        expect(bpmn).toContain('gatewayDirection="Diverging"');
        expect(bpmn).toContain('gatewayDirection="Converging"');
        expect(bpmn).toContain('name="CameraUnits"'); // data object
        expect(bpmn).toContain('name="Operator"'); // lane
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportBpmn(undefined, FIXED_TS)).toEqual(
            graph.exportBpmn(undefined, FIXED_TS),
        );
    });
});
