import { describe, expect, it } from 'vitest';
import { validateNativeExports } from '../lib/validate_native_exports';

describe('validateNativeExports helper', () => {
    it('throws a helpful error when required exports are missing', () => {
        const fakeBinding = { Graph: {} }; // minimal missing many required symbols
        const required = ['Graph', 'Entity', 'Resource'];
        expect(() => validateNativeExports(fakeBinding, required)).toThrowError(/Missing required export\(s\): Entity, Resource|missing required export/i);
    });

    it('does not throw when required exports are present', () => {
        const fakeBinding = {
            Graph: {},
            Entity: {},
            Resource: {},
        };
        const required = ['Graph', 'Entity', 'Resource'];
        expect(() => validateNativeExports(fakeBinding, required)).not.toThrow();
    });
});

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Graph } from '../index';

describe('application contract binding (ADR-013 Milestone 0)', () => {
    const fixtureRoot = join(__dirname, '..', 'fixtures', 'application_generation', 'flagship');
    const sourcesJson = JSON.stringify({
        'flagship/command-write.sea': readFileSync(join(fixtureRoot, 'command-write.sea'), 'utf8'),
        'flagship/query-read.sea': readFileSync(join(fixtureRoot, 'query-read.sea'), 'utf8'),
    });

    it('exposes AST JSON and canonical application contract JSON', () => {
        expect(JSON.parse(Graph.parseToAstJson('Entity "A"')).declarations).toHaveLength(1);
        const raw = Graph.resolveApplicationContractJson('flagship/query-read.sea', sourcesJson);
        const again = Graph.resolveApplicationContractJson('flagship/query-read.sea', sourcesJson);
        expect(raw).toBe(again);
        const doc = JSON.parse(raw);
        expect(doc.schema_version).toBe('domainforge-application-contract/v1');
        expect(doc.self_hash.startsWith('sha256:')).toBe(true);
    });

    it('reports diagnostics for a malformed source map', () => {
        expect(() => Graph.resolveApplicationContractJson('a.sea', '[]')).toThrowError(/APP/);
    });
});


describe('canonical source-map graph binding', () => {
    const sources = {
        'organization.sea': '@namespace "organization"\nimport { Purchase } from "./finance.sea"\nimport { Ship } from "./operations.sea"\nentity "Case" { key id: uuid\n purchase: ref<Purchase>\n shipment: ref<Ship>\n}\n',
        'finance.sea': '@namespace "finance"\nexport entity "Purchase" { key id: uuid }\n',
        'operations.sea': '@namespace "operations"\nexport entity "Ship" { key id: uuid }\n',
    };
    it('projects imported entities without collapsing their namespaces', () => {
        const graph = Graph.parseSourceMap('organization.sea', JSON.stringify(sources));
        const ids = graph.allEntities().map((entity) => `${entity.namespace}::${entity.name}`).sort();
        expect(ids).toEqual(['finance::Purchase', 'operations::Ship', 'organization::Case']);
        const again = Graph.parseSourceMap('organization.sea', JSON.stringify(sources));
        expect(again.allEntities().map((entity) => entity.id).sort()).toEqual(graph.allEntities().map((entity) => entity.id).sort());
    });
    it('rejects missing imports rather than quietly projecting an incomplete graph', () => {
        const { ['finance.sea']: omitted, ...missing } = sources;
        void omitted;
        expect(() => Graph.parseSourceMap('organization.sea', JSON.stringify(missing))).toThrow();
    });
});
