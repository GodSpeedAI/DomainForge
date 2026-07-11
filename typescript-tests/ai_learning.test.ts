/**
 * TypeScript binding surface for AI Learning Projections
 * (Graph.exportAiLearning).
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it, expect } from 'vitest';
import { Graph } from '../index.js';

const FIXTURE = join(__dirname, '..', 'fixtures', 'ai_learning', 'manufacturing_quality');
const FIXED_TS = '2026-07-02T00:00:00+00:00';

function fixtureGraph(): Graph {
    const source = readFileSync(join(FIXTURE, 'domain', 'model.sea'), 'utf8');
    return Graph.parse(source);
}

function fixtureRecipe(): string {
    return readFileSync(join(FIXTURE, 'recipes', 'ai_learning.json'), 'utf8');
}

function fixtureAuthority(): string {
    return readFileSync(join(FIXTURE, 'authority', 'environment.json'), 'utf8');
}

describe('Graph.exportAiLearning', () => {
    it('produces the ai-learning layout with resolver-grounded labels', () => {
        const graph = fixtureGraph();
        const artifacts = JSON.parse(
            graph.exportAiLearning(fixtureRecipe(), fixtureAuthority(), undefined, undefined, FIXED_TS),
        );

        for (const path of [
            'llm_dataset/train.jsonl',
            'llm_dataset/validation_report.json',
            'graph_dataset/graph.json',
            'graph_dataset/negative_samples.json',
            'cep_eval/dataset.json',
            'cep_eval/reports/coverage_report.json',
        ]) {
            expect(artifacts[path], `missing artifact ${path}`).toBeDefined();
        }

        expect(artifacts['llm_dataset/train.jsonl']).toContain(
            'does not contain enough authority information',
        );
        const report = JSON.parse(artifacts['llm_dataset/validation_report.json']);
        expect(report.status).toBe('passed');
        expect(report.resolver_disagreement_count).toBe(0);
    });

    it('is deterministic with a fixed created_at', () => {
        const graph = fixtureGraph();
        const run = () =>
            graph.exportAiLearning(fixtureRecipe(), fixtureAuthority(), undefined, undefined, FIXED_TS);
        expect(run()).toBe(run());
    });

    it('fails loudly when resolver-grounded families lack an authority config', () => {
        const graph = fixtureGraph();
        expect(() => graph.exportAiLearning()).toThrow(/authority/);
    });
});
