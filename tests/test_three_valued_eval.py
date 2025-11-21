"""
Test three-valued logic evaluation semantics for policy evaluation.

This test validates that the EvaluationResult correctly handles:
- True evaluations (policy satisfied)
- False evaluations (policy violated)
- Null evaluations (policy evaluation unknown/indeterminate)
"""

import json
import pytest
from sea_dsl import Graph, EvaluationResult, Violation, Severity


def test_policy_evaluates_to_true():
    """Test that a satisfied policy returns is_satisfied=True and tristate=True."""
    graph = Graph()

    # Create a simple policy that always evaluates to true
    policy = {
        "id": "00000000-0000-0000-0000-000000000001",
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

    assert isinstance(result, EvaluationResult)
    assert result.is_satisfied is True
    assert result.is_satisfied_tristate is True
    assert len(result.violations) == 0


def test_policy_evaluates_to_false():
    """Test that a violated policy returns is_satisfied=False and tristate=False."""
    graph = Graph()

    # Create a simple policy that always evaluates to false
    policy = {
        "id": "00000000-0000-0000-0000-000000000002",
        "name": "AlwaysFalse",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {"Literal": False},
        "modality": "Obligation",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    result = graph.evaluate_policy(json.dumps(policy))

    assert isinstance(result, EvaluationResult)
    assert result.is_satisfied is False
    assert result.is_satisfied_tristate is False
    assert len(result.violations) == 1
    assert result.violations[0].name == "AlwaysFalse"
    assert result.violations[0].severity == Severity.Error


def test_policy_evaluates_to_null():
    """Test that an unknown/null policy evaluation returns is_satisfied=False and tristate=None."""
    graph = Graph()

    # Create a policy that references a non-existent entity attribute
    # This should evaluate to NULL when three_valued_logic feature is enabled
    policy = {
        "id": "00000000-0000-0000-0000-000000000003",
        "name": "NullEvaluation",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {
            "MemberAccess": {"object": "NonExistentEntity", "member": "someAttribute"}
        },
        "modality": "Obligation",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    result = graph.evaluate_policy(json.dumps(policy))

    assert isinstance(result, EvaluationResult)
    # When evaluation is NULL, is_satisfied defaults to False for backwards compatibility
    assert result.is_satisfied is False
    # The tristate field should be None to indicate unknown/NULL
    assert result.is_satisfied_tristate is None
    # Violation severity follows the policy modality even when evaluation is NULL
    assert len(result.violations) == 1
    assert (
        "UNKNOWN" in result.violations[0].message
        or "NULL" in result.violations[0].message
    )
    assert result.violations[0].severity == Severity.Error


def test_evaluation_result_repr():
    """Test that EvaluationResult has a useful string representation."""
    graph = Graph()

    policy = {
        "id": "00000000-0000-0000-0000-000000000004",
        "name": "TestPolicy",
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
    repr_str = repr(result)

    assert "EvaluationResult" in repr_str
    assert "is_satisfied" in repr_str
    assert "violations" in repr_str


def test_violation_repr():
    """Test that Violation has a useful string representation."""
    graph = Graph()

    policy = {
        "id": "00000000-0000-0000-0000-000000000005",
        "name": "ViolatedPolicy",
        "namespace": "test",
        "version": {"major": 1, "minor": 0, "patch": 0},
        "expression": {"Literal": False},
        "modality": "Prohibition",
        "kind": "Constraint",
        "priority": 0,
        "rationale": None,
        "tags": [],
    }

    result = graph.evaluate_policy(json.dumps(policy))

    assert len(result.violations) > 0
    violation = result.violations[0]
    repr_str = repr(violation)

    assert "Violation" in repr_str
    assert "ViolatedPolicy" in repr_str


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
