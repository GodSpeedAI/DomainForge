/**
 * TypeScript binding surface for the RDF/OWL projection (Graph.exportRdfProjection).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'rdf', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

describe('Graph.exportRdfProjection', () => {
    it('emits the three-artifact manifest and valid JSON-LD', () => {
        const artifacts = JSON.parse(fixtureGraph().exportRdfProjection(undefined, FIXED_TS));
        expect(Object.keys(artifacts).sort()).toEqual([
            'model.jsonld',
            'model.ttl',
            'ontology.owl.ttl',
        ]);
        expect(artifacts['model.ttl']).toContain('sea:Warehouse rdf:type sea:Entity');
        expect(artifacts['ontology.owl.ttl']).toContain('sea:from a owl:ObjectProperty');

        const doc = JSON.parse(artifacts['model.jsonld']);
        expect(doc['@context']['sea']).toBe('http://domainforge.ai/sea#');
        expect(Array.isArray(doc['@graph'])).toBe(true);
    });

    it('honors a base-iri override', () => {
        const artifacts = JSON.parse(
            fixtureGraph().exportRdfProjection(undefined, FIXED_TS, 'https://example.org/demo#'),
        );
        const doc = JSON.parse(artifacts['model.jsonld']);
        expect(doc['@context']['sea']).toBe('https://example.org/demo#');
        expect(artifacts['ontology.owl.ttl']).toContain(
            '@prefix sea: <https://example.org/demo#>',
        );
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportRdfProjection(undefined, FIXED_TS)).toEqual(
            graph.exportRdfProjection(undefined, FIXED_TS),
        );
    });
});
