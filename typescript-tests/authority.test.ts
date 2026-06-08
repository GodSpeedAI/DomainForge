/**
 * Tests for the new Policy Authority API exports added in this PR.
 *
 * Covers:
 * - Enum exports: FinalDecision, PolicyModality, SourceClass, ClaimLevel
 * - Function: evaluateAuthority
 */

import { describe, it, expect } from 'vitest';
import {
    FinalDecision,
    PolicyModality,
    SourceClass,
    ClaimLevel,
    evaluateAuthority,
} from '../index.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeMinimalConfigJson(packs: unknown[] = []): string {
    return JSON.stringify({
        resolver_semantics_version: '0.4',
        specificity_profile: {
            id: 'default',
            dimensions: [],
            scoring_rules: {},
            hash: '',
        },
        unknown_handling: {
            permission: { default: 'escalate' },
            prohibition: { default: 'deny' },
            obligation: { default: 'escalate' },
            override_: { default: 'not_applicable' },
        },
        fact_sources: [],
        fact_transforms: [],
        authority_packs: packs,
        strict_mode: false,
        compatibility_lowering_version: '0.4',
        resolver_version: '0.1',
    });
}

function makeMinimalRequestJson(operation = 'TestAction', resourceType = 'Order'): string {
    return JSON.stringify({
        request_id: 'req-001',
        actor: { id: 'user-1' },
        operation,
        resource: { type: resourceType },
        context: {},
        requested_at: '2026-06-07T00:00:00Z',
    });
}

// ---------------------------------------------------------------------------
// Enum value tests
// ---------------------------------------------------------------------------

describe('FinalDecision enum', () => {
    it('exports Allow=0, Deny=1, Escalate=2, NotApplicable=3, Reject=4', () => {
        expect(FinalDecision.Allow).toBe(0);
        expect(FinalDecision.Deny).toBe(1);
        expect(FinalDecision.Escalate).toBe(2);
        expect(FinalDecision.NotApplicable).toBe(3);
        expect(FinalDecision.Reject).toBe(4);
    });

    it('has exactly five members', () => {
        const keys = Object.keys(FinalDecision).filter((k) => isNaN(Number(k)));
        expect(keys).toHaveLength(5);
    });
});

describe('PolicyModality enum', () => {
    it('exports Permission=0, Prohibition=1, Obligation=2, Override=3', () => {
        expect(PolicyModality.Permission).toBe(0);
        expect(PolicyModality.Prohibition).toBe(1);
        expect(PolicyModality.Obligation).toBe(2);
        expect(PolicyModality.Override).toBe(3);
    });

    it('has exactly four members', () => {
        const keys = Object.keys(PolicyModality).filter((k) => isNaN(Number(k)));
        expect(keys).toHaveLength(4);
    });
});

describe('SourceClass enum', () => {
    it('exports all seven source class values', () => {
        expect(SourceClass.CallerSupplied).toBe(0);
        expect(SourceClass.RuntimeObserved).toBe(1);
        expect(SourceClass.SystemOfRecord).toBe(2);
        expect(SourceClass.Attested).toBe(3);
        expect(SourceClass.ManualApproval).toBe(4);
        expect(SourceClass.Derived).toBe(5);
        expect(SourceClass.UnknownSource).toBe(6);
    });

    it('has exactly seven members', () => {
        const keys = Object.keys(SourceClass).filter((k) => isNaN(Number(k)));
        expect(keys).toHaveLength(7);
    });
});

describe('ClaimLevel enum', () => {
    it('exports AuditBacked=0, Validated=1, FormallyProven=2', () => {
        expect(ClaimLevel.AuditBacked).toBe(0);
        expect(ClaimLevel.Validated).toBe(1);
        expect(ClaimLevel.FormallyProven).toBe(2);
    });

    it('has exactly three members', () => {
        const keys = Object.keys(ClaimLevel).filter((k) => isNaN(Number(k)));
        expect(keys).toHaveLength(3);
    });
});

// ---------------------------------------------------------------------------
// evaluateAuthority
// ---------------------------------------------------------------------------

describe('evaluateAuthority', () => {
    it('is a function', () => {
        expect(typeof evaluateAuthority).toBe('function');
    });

    it('throws on invalid config JSON', () => {
        expect(() =>
            evaluateAuthority('not valid json', makeMinimalRequestJson()),
        ).toThrow();
    });

    it('throws on invalid request JSON', () => {
        expect(() =>
            evaluateAuthority(makeMinimalConfigJson(), 'not valid json'),
        ).toThrow();
    });

    it('throws on invalid facts JSON', () => {
        expect(() =>
            evaluateAuthority(makeMinimalConfigJson(), makeMinimalRequestJson(), 'not valid json'),
        ).toThrow();
    });

    it('returns an object with traceJson and decisionJson', () => {
        const result = evaluateAuthority(makeMinimalConfigJson(), makeMinimalRequestJson());
        expect(result).toHaveProperty('traceJson');
        expect(result).toHaveProperty('decisionJson');
        expect(typeof result.traceJson).toBe('string');
        expect(typeof result.decisionJson).toBe('string');
    });

    it('traceJson and decisionJson are valid JSON strings', () => {
        const result = evaluateAuthority(makeMinimalConfigJson(), makeMinimalRequestJson());
        expect(() => JSON.parse(result.traceJson)).not.toThrow();
        expect(() => JSON.parse(result.decisionJson)).not.toThrow();
    });

    it('returns not_applicable when no packs are configured', () => {
        const result = evaluateAuthority(makeMinimalConfigJson([]), makeMinimalRequestJson());
        const decision = JSON.parse(result.decisionJson);
        // With no packs and no applicable policies, expect not_applicable
        expect(decision).toHaveProperty('final_decision');
        expect(decision.final_decision).toBe('not_applicable');
    });

    it('accepts optional facts JSON as a JSON array', () => {
        const factsJson = JSON.stringify([]);
        const result = evaluateAuthority(
            makeMinimalConfigJson(),
            makeMinimalRequestJson(),
            factsJson,
        );
        expect(result).toHaveProperty('decisionJson');
    });

    it('decision contains a final_decision field', () => {
        const result = evaluateAuthority(makeMinimalConfigJson(), makeMinimalRequestJson());
        const decision = JSON.parse(result.decisionJson);
        expect(decision).toHaveProperty('final_decision');
        const validDecisions = ['allow', 'deny', 'escalate', 'not_applicable', 'reject'];
        expect(validDecisions).toContain(decision.final_decision);
    });

    it('trace contains request_id matching the request', () => {
        const result = evaluateAuthority(makeMinimalConfigJson(), makeMinimalRequestJson());
        const trace = JSON.parse(result.traceJson);
        // Trace should reference the request_id
        const traceStr = JSON.stringify(trace);
        expect(traceStr).toContain('req-001');
    });

    it('handles different operations in requests', () => {
        const result1 = evaluateAuthority(
            makeMinimalConfigJson(),
            makeMinimalRequestJson('CreateOrder'),
        );
        const result2 = evaluateAuthority(
            makeMinimalConfigJson(),
            makeMinimalRequestJson('DeleteOrder'),
        );
        // Both should return valid results
        expect(() => JSON.parse(result1.decisionJson)).not.toThrow();
        expect(() => JSON.parse(result2.decisionJson)).not.toThrow();
    });

    it('two calls with identical input produce the same final_decision', () => {
        const config = makeMinimalConfigJson();
        const request = makeMinimalRequestJson();
        const r1 = JSON.parse(evaluateAuthority(config, request).decisionJson);
        const r2 = JSON.parse(evaluateAuthority(config, request).decisionJson);
        expect(r1.final_decision).toBe(r2.final_decision);
    });
});