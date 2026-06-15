/**
 * Cross-binding authority-trace parity for conformance/08_authority.
 *
 * Loads the shared evaluation inputs (config/request/facts), drives the
 * TypeScript `evaluateAuthority` binding, and byte-compares the emitted trace
 * and decision (volatile-normalized) against the committed goldens.
 *
 * This proves the napi surface reproduces the Rust golden through the same
 * core — closing the authority spine of the cross-binding parity matrix.
 *
 * Run: bun test typescript-tests/authority-parity.test.ts
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';
import { evaluateAuthority } from '../index.js';

const CONF_DIR = join(__dirname, '..', 'conformance', '08_authority');

const VOLATILE_KEYS = new Set(['created_at', 'decision_id', 'trace_ref']);

function normalizeVolatile(value: any): any {
    if (value !== null && typeof value === 'object') {
        if (Array.isArray(value)) {
            return value.map(normalizeVolatile);
        }
        const out: Record<string, any> = {};
        for (const [key, val] of Object.entries(value)) {
            out[key] = VOLATILE_KEYS.has(key) ? '<volatile>' : normalizeVolatile(val);
        }
        return out;
    }
    return value;
}

function loadJson(fileName: string): any {
    return JSON.parse(readFileSync(join(CONF_DIR, fileName), 'utf-8'));
}

function evaluate(): { trace: any; decision: any } {
    const configJson = readFileSync(join(CONF_DIR, 'config.json'), 'utf-8');
    const requestJson = readFileSync(join(CONF_DIR, 'request.json'), 'utf-8');
    const factsJson = readFileSync(join(CONF_DIR, 'facts.json'), 'utf-8');

    const result = evaluateAuthority(configJson, requestJson, factsJson);
    return {
        trace: JSON.parse(result.traceJson),
        decision: JSON.parse(result.decisionJson),
    };
}

describe('Authority trace parity (08_authority)', () => {
    it('trace matches the Rust golden (volatile-normalized)', () => {
        const { trace: actual } = evaluate();
        const expected = loadJson('trace.json');
        expect(normalizeVolatile(actual)).toEqual(normalizeVolatile(expected));
    });

    it('decision matches the Rust golden (volatile-normalized)', () => {
        const { decision: actual } = evaluate();
        const expected = loadJson('decision.json');
        expect(normalizeVolatile(actual)).toEqual(normalizeVolatile(expected));
    });

    it('final_decision is deny', () => {
        const { decision } = evaluate();
        expect(decision.final_decision).toBe('deny');
    });
});
