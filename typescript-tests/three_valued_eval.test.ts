/**
 * Test three-valued logic evaluation semantics for policy evaluation.
 *
 * This test validates that the EvaluationResult correctly handles:
 * - True evaluations (policy satisfied)
 * - False evaluations (policy violated)
 * - Null evaluations (policy evaluation unknown/indeterminate)
 */

import { describe, it, expect } from 'vitest';
import { Graph, Severity } from '../index.js';

describe('Three-valued policy evaluation', () => {
  it('should return is_satisfied=true and tristate=true when policy is satisfied', () => {
    const graph = new Graph();

    // Create a simple policy that always evaluates to true
    const policy = {
      id: '00000000-0000-0000-0000-000000000001',
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

    expect(result.isSatisfied).toBe(true);
    expect(result.isSatisfiedTristate).toBe(true);
    expect(result.violations).toHaveLength(0);
  });

  it('should return is_satisfied=false and tristate=false when policy is violated', () => {
    const graph = new Graph();

    // Create a simple policy that always evaluates to false
    const policy = {
      id: '00000000-0000-0000-0000-000000000002',
      name: 'AlwaysFalse',
      namespace: 'test',
      version: { major: 1, minor: 0, patch: 0 },
      expression: { Literal: false },
      modality: 'Obligation',
      kind: 'Constraint',
      priority: 0,
      rationale: null,
      tags: [],
    };

    const result = graph.evaluatePolicy(JSON.stringify(policy));

    expect(result.isSatisfied).toBe(false);
    expect(result.isSatisfiedTristate).toBe(false);
    expect(result.violations).toHaveLength(1);
    expect(result.violations[0].name).toBe('AlwaysFalse');
    expect(result.violations[0].severity).toBe(Severity.Error);
  });

  it('should return is_satisfied=false and tristate=null when evaluation is unknown', () => {
    const graph = new Graph();

    // Create a policy that references a non-existent entity attribute
    // This should evaluate to NULL when three_valued_logic feature is enabled
    const policy = {
      id: '00000000-0000-0000-0000-000000000003',
      name: 'NullEvaluation',
      namespace: 'test',
      version: { major: 1, minor: 0, patch: 0 },
      expression: {
        MemberAccess: {
          object: 'NonExistentEntity',
          member: 'someAttribute',
        },
      },
      modality: 'Obligation',
      kind: 'Constraint',
      priority: 0,
      rationale: null,
      tags: [],
    };

    const result = graph.evaluatePolicy(JSON.stringify(policy));

    // When evaluation is NULL, is_satisfied defaults to false for backwards compatibility
    expect(result.isSatisfied).toBe(false);
    // The tristate field should be undefined (missing) to indicate indeterminate result
    expect(result.isSatisfiedTristate).toBeUndefined();
    // Violation severity follows the policy modality even when evaluation is NULL
    expect(result.violations).toHaveLength(1);
    expect(
      result.violations[0].message.includes('UNKNOWN') ||
        result.violations[0].message.includes('NULL')
    ).toBe(true);
    expect(result.violations[0].severity).toBe(Severity.Error);
  });

  it('should handle invalid policy JSON', () => {
    const graph = new Graph();

    expect(() => {
      graph.evaluatePolicy('invalid json');
    }).toThrow();
  });

  it('should include violation details', () => {
    const graph = new Graph();

    // Ensure violation messaging reflects the modality (Prohibition) rather than a generic failure.
    const policy = {
      id: '00000000-0000-0000-0000-000000000005',
      name: 'ViolatedPolicy',
      namespace: 'test',
      version: { major: 1, minor: 0, patch: 0 },
      expression: { Literal: false },
      modality: 'Prohibition',
      kind: 'Constraint',
      priority: 0,
      rationale: null,
      tags: [],
    };

    const result = graph.evaluatePolicy(JSON.stringify(policy));

    expect(result.violations.length).toBeGreaterThan(0);
    const violation = result.violations[0];
    expect(violation.name).toBe('ViolatedPolicy');
    expect(violation.message).toMatch(new RegExp(policy.name, 'i'));
    expect(violation.severity).toBe(Severity.Error);
  });
});
