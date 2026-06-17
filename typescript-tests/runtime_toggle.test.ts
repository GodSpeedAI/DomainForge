/**
 * Canonical evaluation-mode tests (TypeScript binding).
 *
 * The legacy runtime logic toggle (boolean vs three-valued) was removed per the
 * semantic-infrastructure audit (G1). Three-valued (Kleene) logic is now the
 * single authoritative semantics, so the binding no longer exposes
 * setEvaluationMode / useThreeValuedLogic. These tests pin the canonical behavior.
 */

import { describe, it, expect } from 'vitest';
import { Graph, Severity } from '../index.js';

describe('Canonical three-valued evaluation', () => {
  it('returns undefined (NULL) for indeterminate evaluations', () => {
    const graph = new Graph();

    const policy = {
      id: '00000000-0000-0000-0000-000000000001',
      name: 'TestPolicy',
      namespace: 'test',
      version: { major: 1, minor: 0, patch: 0 },
      expression: {
        MemberAccess: { object: 'NonExistent', member: 'attr' },
      },
      modality: 'Obligation',
      kind: 'Constraint',
      priority: 0,
      rationale: null,
      tags: [],
    };

    const result = graph.evaluatePolicy(JSON.stringify(policy));

    // NULL is never silently coerced: tristate is undefined, the fail-closed
    // boolean is false, and a violation is emitted at the policy's severity.
    expect(result.isSatisfiedTristate).toBeUndefined();
    expect(result.isSatisfied).toBe(false);
    expect(result.violations).toHaveLength(1);
    expect(result.violations[0].severity).toBe(Severity.Error);
  });

  it('satisfies a trivially true policy with no violations', () => {
    const graph = new Graph();

    const policy = {
      id: '00000000-0000-0000-0000-000000000002',
      name: 'AlwaysTrue',
      namespace: 'test',
      version: { major: 1, minor: 0, patch: 0 },
      expression: { Literal: true },
      modality: 'Obligation',
      kind: 'Constraint',
      priority: 0,
      rationale: null,
      tags: [],
    };

    const result = graph.evaluatePolicy(JSON.stringify(policy));

    expect(result.isSatisfiedTristate).toBe(true);
    expect(result.isSatisfied).toBe(true);
    expect(result.violations).toHaveLength(0);
  });

  it('no longer exposes a boolean-mode toggle (audit G1)', () => {
    const graph = new Graph() as unknown as Record<string, unknown>;
    expect(typeof graph.setEvaluationMode).toBe('undefined');
    expect(typeof graph.useThreeValuedLogic).toBe('undefined');
  });
});
