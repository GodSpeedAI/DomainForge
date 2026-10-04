/**
 * world_ref through the napi binding, driven by the same golden vectors the
 * Rust core and the independent Python reference use.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { Graph } from '../index.js';

const vectors = JSON.parse(
    readFileSync(
        new URL('../domainforge-core/tests/fixtures/world_ref/golden-vectors.json', import.meta.url),
        'utf8',
    ),
);
const sources = JSON.stringify({ 'main.sea': '@namespace "t"\nentity "Tank" { key id: uuid }\n' });

describe('world_ref (napi binding)', () => {
    for (const c of vectors.valid) {
        it(`reproduces golden world_ref: ${c.case}`, () => {
            const identity = JSON.stringify(c.identity);
            expect(Graph.worldRefFromIdentityJson(c.name, identity)).toBe(c.world_ref);
            expect(Graph.parseWorldRef(c.world_ref)).toBe(c.world_ref);
            expect(() => Graph.verifyWorldRef(c.world_ref, identity)).not.toThrow();
        });
    }

    for (const bad of vectors.invalid_refs) {
        it(`rejects invalid ref ${JSON.stringify(bad)}`, () => {
            expect(() => Graph.parseWorldRef(bad)).toThrow();
        });
    }

    it('fails verification for a tampered identity', () => {
        const c = vectors.valid[0];
        const forged = { ...c.identity, semantic_closure_hash: 'sha256:' + '0'.repeat(64) };
        expect(() => Graph.verifyWorldRef(c.world_ref, JSON.stringify(forged))).toThrow(/digest mismatch/);
    });

    it('pins a verifiable world from a real resolution, deterministically', () => {
        const identityJson = Graph.domainModelIdentityJson('main.sea', sources);
        expect(JSON.parse(identityJson).identity_scheme_version).toBe('v2-full-preimage');
        const world = Graph.worldRefFromIdentityJson('t', identityJson);
        expect(world.startsWith('world:t@sha256:')).toBe(true);
        expect(() => Graph.verifyWorldRef(world, identityJson)).not.toThrow();
        expect(Graph.domainModelIdentityJson('main.sea', sources)).toBe(identityJson);
    });
});
