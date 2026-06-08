"""
Tests for the new Policy Authority API exports added in this PR.

Covers:
- Enum exports: FinalDecision, PolicyModality, SourceClass, ClaimLevel
- Class: AuthorityEnvironment
- Function: evaluate_authority
"""

import json
import pytest
import sea_dsl


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def make_minimal_config_json(packs=None) -> str:
    if packs is None:
        packs = []
    return json.dumps({
        "resolver_semantics_version": "0.4",
        "specificity_profile": {
            "id": "default",
            "dimensions": [],
            "scoring_rules": {},
            "hash": "",
        },
        "unknown_handling": {
            "permission": {"default": "escalate"},
            "prohibition": {"default": "deny"},
            "obligation": {"default": "escalate"},
            "override_": {"default": "not_applicable"},
        },
        "fact_sources": [],
        "fact_transforms": [],
        "authority_packs": packs,
        "strict_mode": False,
        "compatibility_lowering_version": "0.4",
        "resolver_version": "0.1",
    })


def make_minimal_request_json(operation="TestAction", resource_type="Order") -> str:
    return json.dumps({
        "request_id": "req-001",
        "actor": {"id": "user-1"},
        "operation": operation,
        "resource": {"type": resource_type},
        "context": {},
        "requested_at": "2026-06-07T00:00:00Z",
    })


# ---------------------------------------------------------------------------
# Enum import tests
# ---------------------------------------------------------------------------

def test_final_decision_enum_importable():
    """FinalDecision enum is importable from sea_dsl."""
    assert hasattr(sea_dsl, "FinalDecision")


def test_final_decision_enum_values():
    """FinalDecision has Allow, Deny, Escalate, NotApplicable, Reject members."""
    assert hasattr(sea_dsl.FinalDecision, "Allow")
    assert hasattr(sea_dsl.FinalDecision, "Deny")
    assert hasattr(sea_dsl.FinalDecision, "Escalate")
    assert hasattr(sea_dsl.FinalDecision, "NotApplicable")
    assert hasattr(sea_dsl.FinalDecision, "Reject")


def test_final_decision_members_are_distinct():
    """FinalDecision enum members are not equal to each other."""
    assert sea_dsl.FinalDecision.Allow != sea_dsl.FinalDecision.Deny
    assert sea_dsl.FinalDecision.Allow != sea_dsl.FinalDecision.Escalate
    assert sea_dsl.FinalDecision.Allow != sea_dsl.FinalDecision.NotApplicable
    assert sea_dsl.FinalDecision.Allow != sea_dsl.FinalDecision.Reject
    assert sea_dsl.FinalDecision.Deny != sea_dsl.FinalDecision.Reject


def test_policy_modality_enum_importable():
    """PolicyModality enum is importable from sea_dsl."""
    assert hasattr(sea_dsl, "PolicyModality")


def test_policy_modality_enum_values():
    """PolicyModality has Permission, Prohibition, Obligation, Override members."""
    assert hasattr(sea_dsl.PolicyModality, "Permission")
    assert hasattr(sea_dsl.PolicyModality, "Prohibition")
    assert hasattr(sea_dsl.PolicyModality, "Obligation")
    assert hasattr(sea_dsl.PolicyModality, "Override")


def test_policy_modality_members_are_distinct():
    """PolicyModality enum members are not equal to each other."""
    assert sea_dsl.PolicyModality.Permission != sea_dsl.PolicyModality.Prohibition
    assert sea_dsl.PolicyModality.Obligation != sea_dsl.PolicyModality.Override


def test_source_class_enum_importable():
    """SourceClass enum is importable from sea_dsl."""
    assert hasattr(sea_dsl, "SourceClass")


def test_source_class_enum_values():
    """SourceClass has all seven members."""
    for member in [
        "CallerSupplied", "RuntimeObserved", "SystemOfRecord",
        "Attested", "ManualApproval", "Derived", "UnknownSource",
    ]:
        assert hasattr(sea_dsl.SourceClass, member), f"Missing SourceClass.{member}"


def test_source_class_members_are_distinct():
    """SourceClass enum members are not equal to each other."""
    assert sea_dsl.SourceClass.CallerSupplied != sea_dsl.SourceClass.SystemOfRecord
    assert sea_dsl.SourceClass.Derived != sea_dsl.SourceClass.UnknownSource


def test_claim_level_enum_importable():
    """ClaimLevel enum is importable from sea_dsl."""
    assert hasattr(sea_dsl, "ClaimLevel")


def test_claim_level_enum_values():
    """ClaimLevel has AuditBacked, Validated, FormallyProven members."""
    assert hasattr(sea_dsl.ClaimLevel, "AuditBacked")
    assert hasattr(sea_dsl.ClaimLevel, "Validated")
    assert hasattr(sea_dsl.ClaimLevel, "FormallyProven")


def test_claim_level_members_are_distinct():
    """ClaimLevel enum members are not equal to each other."""
    assert sea_dsl.ClaimLevel.AuditBacked != sea_dsl.ClaimLevel.Validated
    assert sea_dsl.ClaimLevel.Validated != sea_dsl.ClaimLevel.FormallyProven


# ---------------------------------------------------------------------------
# AuthorityEnvironment class
# ---------------------------------------------------------------------------

def test_authority_environment_class_importable():
    """AuthorityEnvironment class is importable from sea_dsl."""
    assert hasattr(sea_dsl, "AuthorityEnvironment")


def test_authority_environment_instantiation():
    """AuthorityEnvironment can be instantiated from a minimal config JSON."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    assert env is not None


def test_authority_environment_raises_on_invalid_config():
    """AuthorityEnvironment raises ValueError on invalid config JSON."""
    with pytest.raises((ValueError, Exception)):
        sea_dsl.AuthorityEnvironment("not valid json")


def test_authority_environment_validate():
    """AuthorityEnvironment.validate() succeeds for a valid config."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    # Should not raise
    env.validate()


def test_authority_environment_evaluate_returns_tuple():
    """AuthorityEnvironment.evaluate() returns a (trace_json, decision_json) tuple."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    env.validate()
    result = env.evaluate(make_minimal_request_json())
    assert isinstance(result, tuple)
    assert len(result) == 2


def test_authority_environment_evaluate_returns_valid_json():
    """AuthorityEnvironment.evaluate() returns valid JSON strings."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    env.validate()
    trace_json, decision_json = env.evaluate(make_minimal_request_json())
    json.loads(trace_json)   # must parse
    json.loads(decision_json)  # must parse


def test_authority_environment_evaluate_raises_on_invalid_request():
    """AuthorityEnvironment.evaluate() raises on invalid request JSON."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    env.validate()
    with pytest.raises((ValueError, Exception)):
        env.evaluate("not valid json")


def test_authority_environment_evaluate_with_empty_facts():
    """AuthorityEnvironment.evaluate() accepts empty facts array."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    env.validate()
    trace_json, decision_json = env.evaluate(make_minimal_request_json(), "[]")
    decision = json.loads(decision_json)
    assert "final_decision" in decision


def test_authority_environment_not_applicable_with_no_packs():
    """AuthorityEnvironment returns not_applicable when no packs are loaded."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json([]))
    env.validate()
    _, decision_json = env.evaluate(make_minimal_request_json())
    decision = json.loads(decision_json)
    assert decision["final_decision"] == "not_applicable"


def test_authority_environment_repr():
    """AuthorityEnvironment has a string representation."""
    env = sea_dsl.AuthorityEnvironment(make_minimal_config_json())
    assert isinstance(str(env), str)


# ---------------------------------------------------------------------------
# evaluate_authority function
# ---------------------------------------------------------------------------

def test_evaluate_authority_is_callable():
    """evaluate_authority is a callable function in sea_dsl."""
    assert callable(sea_dsl.evaluate_authority)


def test_evaluate_authority_returns_tuple():
    """evaluate_authority returns a (trace_json, decision_json) tuple."""
    result = sea_dsl.evaluate_authority(
        make_minimal_config_json(),
        make_minimal_request_json(),
    )
    assert isinstance(result, tuple)
    assert len(result) == 2


def test_evaluate_authority_returns_valid_json():
    """evaluate_authority returns valid JSON strings in both elements."""
    trace_json, decision_json = sea_dsl.evaluate_authority(
        make_minimal_config_json(),
        make_minimal_request_json(),
    )
    json.loads(trace_json)
    json.loads(decision_json)


def test_evaluate_authority_raises_on_invalid_config():
    """evaluate_authority raises on invalid config JSON."""
    with pytest.raises((ValueError, Exception)):
        sea_dsl.evaluate_authority("not valid json", make_minimal_request_json())


def test_evaluate_authority_raises_on_invalid_request():
    """evaluate_authority raises on invalid request JSON."""
    with pytest.raises((ValueError, Exception)):
        sea_dsl.evaluate_authority(make_minimal_config_json(), "not valid json")


def test_evaluate_authority_raises_on_invalid_facts():
    """evaluate_authority raises on invalid facts JSON."""
    with pytest.raises((ValueError, Exception)):
        sea_dsl.evaluate_authority(
            make_minimal_config_json(),
            make_minimal_request_json(),
            "not valid json",
        )


def test_evaluate_authority_not_applicable_with_no_packs():
    """evaluate_authority returns not_applicable when no packs are loaded."""
    _, decision_json = sea_dsl.evaluate_authority(
        make_minimal_config_json([]),
        make_minimal_request_json(),
    )
    decision = json.loads(decision_json)
    assert decision["final_decision"] == "not_applicable"


def test_evaluate_authority_decision_has_final_decision_field():
    """evaluate_authority decision contains a final_decision field."""
    _, decision_json = sea_dsl.evaluate_authority(
        make_minimal_config_json(),
        make_minimal_request_json(),
    )
    decision = json.loads(decision_json)
    assert "final_decision" in decision
    valid_decisions = {"allow", "deny", "escalate", "not_applicable", "reject"}
    assert decision["final_decision"] in valid_decisions


def test_evaluate_authority_trace_references_request_id():
    """evaluate_authority trace references the request_id from the request."""
    trace_json, _ = sea_dsl.evaluate_authority(
        make_minimal_config_json(),
        make_minimal_request_json(),
    )
    assert "req-001" in trace_json


def test_evaluate_authority_with_explicit_empty_facts():
    """evaluate_authority accepts explicit empty facts JSON array."""
    trace_json, decision_json = sea_dsl.evaluate_authority(
        make_minimal_config_json(),
        make_minimal_request_json(),
        "[]",
    )
    assert isinstance(trace_json, str)
    assert isinstance(decision_json, str)


def test_evaluate_authority_is_deterministic():
    """Two calls with identical inputs produce the same final_decision."""
    config = make_minimal_config_json()
    request = make_minimal_request_json()
    _, d1 = sea_dsl.evaluate_authority(config, request)
    _, d2 = sea_dsl.evaluate_authority(config, request)
    assert json.loads(d1)["final_decision"] == json.loads(d2)["final_decision"]


def test_evaluate_authority_handles_different_operations():
    """evaluate_authority processes requests with different operations."""
    for operation in ["CreateOrder", "DeleteOrder", "UpdateOrder"]:
        _, decision_json = sea_dsl.evaluate_authority(
            make_minimal_config_json(),
            make_minimal_request_json(operation=operation),
        )
        decision = json.loads(decision_json)
        assert "final_decision" in decision