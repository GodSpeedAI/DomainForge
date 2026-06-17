"""
Canonical evaluation-mode tests (Python binding).

The legacy runtime logic toggle (boolean vs three-valued) was removed per the
semantic-infrastructure audit (G1). Three-valued (Kleene) logic is now the single
authoritative semantics, so the binding no longer exposes set_evaluation_mode /
use_three_valued_logic. These tests pin the canonical behavior cross-language.
"""

import json
import pytest
from sea_dsl import Graph, Severity


def test_indeterminate_member_access_returns_null():
    """Missing data evaluates to NULL (None) under canonical three-valued logic."""
    graph = Graph()

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

    # NULL is never silently coerced to False: tristate is None, the fail-closed
    # boolean is False, and a violation is emitted at the policy's severity.
    assert result.is_satisfied_tristate is None
    assert result.is_satisfied is False
    assert len(result.violations) == 1
    assert result.violations[0].severity == Severity.Error


def test_satisfied_policy_has_no_violations():
    """A trivially true policy is satisfied with no violations under canonical logic."""
    graph = Graph()

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

    assert result.is_satisfied_tristate is True
    assert result.is_satisfied is True
    assert len(result.violations) == 0


def test_no_evaluation_mode_toggle_exposed():
    """The boolean-mode toggle must no longer exist on the binding (audit G1)."""
    graph = Graph()
    assert not hasattr(graph, "set_evaluation_mode")
    assert not hasattr(graph, "use_three_valued_logic")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
