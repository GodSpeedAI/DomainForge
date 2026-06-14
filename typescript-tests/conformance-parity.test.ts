/**
 * Cross-language conformance parity (Phase 2 of the Semantic Infrastructure Audit).
 *
 * Loads the SHARED `conformance/` corpus, parses each `parse` item via the
 * TypeScript binding, serializes to canonical JSON, normalizes volatile flow
 * UUIDs to positional placeholders, and byte-compares against the Rust-pinned
 * `expected/` files produced by `sea parse --format json`.
 *
 * Run: bun test typescript-tests/conformance-parity.test.ts
 */

import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync, statSync, existsSync } from 'fs';
import { join, resolve } from 'path';
import { Graph } from '../index';

const CONF_DIR = resolve(__dirname, '..', 'conformance');

function normalizeFlowIds(value: any): any {
    if (value && typeof value === 'object' && value.flows && typeof value.flows === 'object') {
        let text = JSON.stringify(value);
        const keys = Object.keys(value.flows);
        for (let i = 0; i < keys.length; i++) {
            text = text.split(keys[i]).join(`flow:${i}`);
        }
        return JSON.parse(text);
    }
    return value;
}

function loadCorpusItems(): Array<{ name: string; input: string; expected: string }> {
    const items: Array<{ name: string; input: string; expected: string }> = [];
    if (!existsSync(CONF_DIR)) return items;
    for (const entry of readdirSync(CONF_DIR).sort()) {
        const dir = join(CONF_DIR, entry);
        if (!statSync(dir).isDirectory()) continue;
        const manifestPath = join(dir, 'manifest.json');
        if (!existsSync(manifestPath)) continue;
        const manifest = JSON.parse(readFileSync(manifestPath, 'utf-8'));
        if (manifest.command !== 'parse') continue;
        items.push({
            name: entry,
            input: join(dir, manifest.input),
            expected: join(dir, manifest.expected),
        });
    }
    return items;
}

describe.each(loadCorpusItems())(
    'Conformance parity: $name',
    ({ name, input, expected }) => {
        it(`produces canonical graph JSON matching the Rust oracle`, () => {
            const source = readFileSync(input, 'utf-8');
            const graph = Graph.parse(source);
            const actual = normalizeFlowIds(JSON.parse(graph.toJson()));
            const expectedData = normalizeFlowIds(JSON.parse(readFileSync(expected, 'utf-8')));
            expect(actual).toEqual(expectedData);
        });
    }
);

describe('Conformance parity corpus', () => {
    it('has at least one parse item', () => {
        expect(loadCorpusItems().length).toBeGreaterThanOrEqual(1);
    });
});
