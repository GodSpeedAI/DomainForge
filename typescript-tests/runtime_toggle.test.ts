/**
 * Test runtime toggle for three-valued logic evaluation.
 *
 * This test validates that the Graph.setEvaluationMode() and
 * Graph.useThreeValuedLogic() methods work correctly.
 */

import { describe, it, expect } from 'vitest';
import { Graph, Severity } from '../index.js';

describe('Runtime toggle for three-valued logic', () => {
  it('should default to three-valued logic enabled', () => {
    const graph = new Graph();
    expect(graph.useThreeValuedLogic()).toBe(true);
  });

  it('should allow toggling between three-valued and boolean logic', () => {
    const graph = new Graph();

    // Start with default (three-valued enabled)
    expect(graph.useThreeValuedLogic()).toBe(true);

    // Disable three-valued logic
    graph.setEvaluationMode(false);
    expect(graph.useThreeValuedLogic()).toBe(false);

    // Re-enable three-valued logic
    graph.setEvaluationMode(true);
    expect(graph.useThreeValuedLogic()).toBe(true);
  });

  it('should return undefined for indeterminate evaluations in three-valued mode', () => {
    const graph = new Graph();
    graph.setEvaluationMode(true); // Enable three-valued logic

    // Create a policy that references a non-existent entity
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

    // Should return undefined (NULL) for tristate
    expect(result.isSatisfiedTristate).toBeUndefined();
    expect(result.isSatisfied).toBe(false);
    expect(result.violations).toHaveLength(1);
    expect(result.violations[0].severity).toBe(Severity.Error);
  });

  it('should use strict boolean logic when three-valued mode is disabled', () => {
    const graph = new Graph();
    graph.setEvaluationMode(false); // Disable three-valued logic

    // Simple policy that always evaluates to true
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

    // Should return true for tristate in boolean mode
    expect(result.isSatisfiedTristate).toBe(true);
    expect(result.isSatisfied).toBe(true);
    expect(result.violations).toHaveLength(0);
  });

  it('should persist evaluation mode across multiple policy evaluations', () => {
    const graph = new Graph();

    const policyTrue = {
      id: '00000000-0000-0000-0000-000000000003',
      name: 'TruePolicy',
      namespace: 'test',
      version: { major: 1, minor: 0, patch: 0 },
      expression: { Literal: true },
      modality: 'Obligation',
      kind: 'Constraint',
      priority: 0,
      rationale: null,
      tags: [],
    };

    // Set to boolean mode
    graph.setEvaluationMode(false);

    // Evaluate first policy
    const result1 = graph.evaluatePolicy(JSON.stringify(policyTrue));
    expect(graph.useThreeValuedLogic()).toBe(false);
    expect(result1.isSatisfied).toBe(true);

    // Evaluate second policy - mode should still be boolean
    const result2 = graph.evaluatePolicy(JSON.stringify(policyTrue));
    expect(graph.useThreeValuedLogic()).toBe(false);
    expect(result2.isSatisfied).toBe(true);

    // Switch to three-valued mode
    graph.setEvaluationMode(true);

    // Evaluate third policy
    const result3 = graph.evaluatePolicy(JSON.stringify(policyTrue));
    expect(graph.useThreeValuedLogic()).toBe(true);
    expect(result3.isSatisfied).toBe(true);
  });

  it('should be independent per graph instance', () => {
    const graph1 = new Graph();
    const graph2 = new Graph();

    // Set different modes for each graph
    graph1.setEvaluationMode(true);
    graph2.setEvaluationMode(false);

    expect(graph1.useThreeValuedLogic()).toBe(true);
    expect(graph2.useThreeValuedLogic()).toBe(false);

    // Changing one should not affect the other
    graph1.setEvaluationMode(false);
    expect(graph1.useThreeValuedLogic()).toBe(false);
    expect(graph2.useThreeValuedLogic()).toBe(false);
  });
});
