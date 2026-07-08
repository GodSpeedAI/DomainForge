/**
 * TypeScript binding surface for the BAML projection (Graph.exportBaml).
 *
 * The projection is resolver-grounded, so the authority environment is passed
 * explicitly (the recipe's `authority_config` path is not resolved in-memory).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'baml', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

function recipeJson(): string {
    return readFileSync(join(FIXTURE, 'recipes', 'baml.json'), 'utf8');
}

function authorityJson(): string {
    return readFileSync(join(FIXTURE, 'authority', 'environment.json'), 'utf8');
}

function exportBaml(graph: Graph): Record<string, string> {
    return JSON.parse(
        graph.exportBaml(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS),
    );
}

describe('Graph.exportBaml', () => {
    it('emits the BAML package with types, a function, and a placeholder client', () => {
        const artifacts = exportBaml(fixtureGraph());
        expect(Object.keys(artifacts).sort()).toEqual([
            'README.md',
            'baml_src/clients.baml',
            'baml_src/domain.baml',
            'baml_src/functions.baml',
            'baml_src/tests.baml',
        ]);
        expect(artifacts['baml_src/domain.baml']).toContain('enum ActorRole {');
        expect(artifacts['baml_src/functions.baml']).toContain(
            'function DecideAuthority(request: AuthorityRequest) -> AuthorityRuling',
        );
        expect(artifacts['baml_src/clients.baml']).toContain(
            '// client<llm> DomainForgeAuthorityClient',
        );
    });

    it('seeds a test from a resolver-grounded authority case', () => {
        const tests = exportBaml(fixtureGraph())['baml_src/tests.baml'];
        expect(tests).toContain('actor_role "CertifiedAuditor"');
        expect(tests).toContain('operation "close_audit_finding"');
        expect(tests).toContain('resource_type "AuditFinding"');
        expect(tests).toContain('resolves to decision "allow"');
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportBaml(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS)).toEqual(
            graph.exportBaml(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS),
        );
    });
});
