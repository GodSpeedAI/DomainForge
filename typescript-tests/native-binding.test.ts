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
