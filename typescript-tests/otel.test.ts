/**
 * TypeScript binding surface for the OpenTelemetry SemConv projection
 * (Graph.exportOtelSemconv).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'otel', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

function attrValues(src: string): Set<string> {
    const out = new Set<string>();
    for (const line of src.split('\n')) {
        const m = line.match(/= "([^"]+)"/);
        if (m && m[1].includes('.')) out.add(m[1]);
    }
    return out;
}

describe('Graph.exportOtelSemconv', () => {
    it('emits a registry and three constant files with correlation attrs', () => {
        const artifacts = JSON.parse(fixtureGraph().exportOtelSemconv(undefined, FIXED_TS));
        expect(Object.keys(artifacts).sort()).toEqual([
            'constants/attributes.py',
            'constants/attributes.rs',
            'constants/attributes.ts',
            'registry/telemetry.yaml',
        ]);

        const yaml = artifacts['registry/telemetry.yaml'];
        expect(yaml).toContain('domainforge.model.hash');
        expect(yaml).toContain('domainforge.element.id');
        expect(yaml).toContain('demo.entity.');
        expect(yaml).toContain('span_kind: internal');
        // No OTel-reserved namespace leaked.
        for (const reserved of ['service.', 'otel.', 'telemetry.']) {
            expect(yaml).not.toContain(`- id: "${reserved}`);
        }
    });

    it('registry and constants agree on the attribute set (single producer)', () => {
        const artifacts = JSON.parse(fixtureGraph().exportOtelSemconv(undefined, FIXED_TS));
        const yaml: string = artifacts['registry/telemetry.yaml'];
        const yamlIds = new Set<string>();
        for (const line of yaml.split('\n')) {
            const m = line.trim().match(/^- id: "([^"]+)"/);
            if (m && !m[1].startsWith('registry.') && !m[1].startsWith('span.')) {
                yamlIds.add(m[1]);
            }
        }
        const ts = attrValues(artifacts['constants/attributes.ts']);
        expect([...yamlIds].sort()).toEqual([...ts].sort());
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportOtelSemconv(undefined, FIXED_TS)).toEqual(
            graph.exportOtelSemconv(undefined, FIXED_TS),
        );
    });
});
