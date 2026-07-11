/**
 * TypeScript binding surface for the ZenML projection (Graph.exportZenml).
 *
 * The projection is resolver-grounded, so the authority environment is passed
 * explicitly (the recipe's `authority_config` path is not resolved in-memory).
 * Per the plan, the TS test asserts the emitted file map (Python-specific
 * semantics are exercised by the Rust and Python suites).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'zenml', 'basic');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

function recipeJson(): string {
    return readFileSync(join(FIXTURE, 'recipes', 'zenml.json'), 'utf8');
}

function authorityJson(): string {
    return readFileSync(join(FIXTURE, 'authority', 'environment.json'), 'utf8');
}

function exportZenml(graph: Graph): Record<string, string> {
    return JSON.parse(
        graph.exportZenml(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS),
    );
}

function exportAiLearning(graph: Graph): Record<string, string> {
    return JSON.parse(
        graph.exportAiLearning(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS),
    );
}

describe('Graph.exportZenml', () => {
    it('emits the ZenML pipeline file map', () => {
        const artifacts = exportZenml(fixtureGraph());
        expect(Object.keys(artifacts).sort()).toEqual([
            'README.md',
            'pipeline.py',
            'requirements.txt',
            'run.py',
            'steps.py',
            'zenml.config.json',
        ]);
        expect(artifacts['pipeline.py']).toContain('def authority_learning_pipeline():');
    });

    it('references ai-learning dataset files it does not copy', () => {
        const graph = fixtureGraph();
        const config = JSON.parse(exportZenml(graph)['zenml.config.json']);
        const ail = exportAiLearning(graph);
        expect(Object.keys(ail)).toContain(config.dataset.train);
        expect(Object.keys(ail)).toContain(config.dataset.dev);
    });

    it('is deterministic for a fixed createdAt', () => {
        const graph = fixtureGraph();
        expect(graph.exportZenml(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS)).toEqual(
            graph.exportZenml(recipeJson(), authorityJson(), 'test.sea', undefined, FIXED_TS),
        );
    });
});
