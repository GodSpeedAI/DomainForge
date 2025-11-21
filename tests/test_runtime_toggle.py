"""
Test runtime toggle for three-valued logic evaluation.

This test validates that the Graph.set_evaluation_mode() and
Graph.use_three_valued_logic() methods work correctly.
"""

import json
import pytest
from sea_dsl import Graph, Severity


def test_default_evaluation_mode_is_three_valued():
    """Test that the default evaluation mode is three-valued logic."""
    graph = Graph()
    assert graph.use_three_valued_logic() is True


def test_can_toggle_evaluation_mode():
    """Test that we can toggle between three-valued and boolean logic."""
    graph = Graph()

    # Start with default (three-valued enabled)
    assert graph.use_three_valued_logic() is True

    # Disable three-valued logic
    graph.set_evaluation_mode(False)
    assert graph.use_three_valued_logic() is False

    # Re-enable three-valued logic
    graph.set_evaluation_mode(True)
    assert graph.use_three_valued_logic() is True


def test_three_valued_mode_returns_null():
    """Test that three-valued mode returns None for indeterminate evaluations."""
    graph = Graph()
    graph.set_evaluation_mode(True)  # Enable three-valued logic

    # Create a policy that references a non-existent entity
    policy = {
        "id": "00000000-0000-0000-0000-000000000001",
        "name": "TestPolicy",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {"MemberAccess": {"object": "NonExistent", "member": "attr"}},
        "modality": "Obligation",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    result = graph.evaluate_policy(json.dumps(policy))

    # Should return None (NULL) for tristate
    assert result.is_satisfied_tristate is None
    assert result.is_satisfied is False
    assert len(result.violations) == 1
    assert result.violations[0].severity == Severity.Error


def test_boolean_mode_coerces_indeterminate_to_false():
    """Boolean mode turns indeterminate member access into a false result with a violation."""
    graph = Graph()
    graph.set_evaluation_mode(False)

    policy = {
        "id": "00000000-0000-0000-0000-000000000004",
        "name": "MissingDataPolicy",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {"MemberAccess": {"object": "NonExistent", "member": "attr"}},
        "modality": "Obligation",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    result = graph.evaluate_policy(json.dumps(policy))

    assert result.is_satisfied_tristate is False
    assert result.is_satisfied is False
    assert len(result.violations) > 0
    assert result.violations[0].severity == Severity.Error


def test_boolean_mode_behavior():
    """Test that boolean mode uses strict boolean logic."""
    graph = Graph()
    graph.set_evaluation_mode(False)  # Disable three-valued logic

    # Simple policy that always evaluates to true
    policy = {
        "id": "00000000-0000-0000-0000-000000000002",
        "name": "AlwaysTrue",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {"Literal": True},
        "modality": "Obligation",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    result = graph.evaluate_policy(json.dumps(policy))

    # Should return Some(true) for tristate in boolean mode
    assert result.is_satisfied_tristate is True
    assert result.is_satisfied is True
    assert len(result.violations) == 0


def test_mode_persists_across_evaluations():
    """Test that the evaluation mode persists across multiple policy evaluations."""
    graph = Graph()

    policy_true = {
        "id": "00000000-0000-0000-0000-000000000003",
        "name": "TruePolicy",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {"Literal": True},
        "modality": "Obligation",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    # Set to boolean mode
    graph.set_evaluation_mode(False)

    # Evaluate first policy
    result1 = graph.evaluate_policy(json.dumps(policy_true))
    assert graph.use_three_valued_logic() is False
    assert result1.is_satisfied is True

    # Evaluate second policy - mode should still be boolean
    result2 = graph.evaluate_policy(json.dumps(policy_true))
    assert graph.use_three_valued_logic() is False
    assert result2.is_satisfied is True

    # Switch to three-valued mode
    graph.set_evaluation_mode(True)

    # Evaluate third policy
    result3 = graph.evaluate_policy(json.dumps(policy_true))
    assert graph.use_three_valued_logic() is True
    assert result3.is_satisfied is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
