"""Cross-binding authority-trace parity for conformance/08_authority.

Loads the shared evaluation inputs (config/request/facts), drives the Python
``AuthorityEnvironment.evaluate`` binding, and byte-compares the emitted trace
and decision (volatile-normalized) against the committed goldens.

This proves the PyO3 surface reproduces the Rust golden through the same core —
closing the authority spine of the cross-binding parity matrix.

Run: pytest tests/test_authority_parity.py
"""

import json
from pathlib import Path

import pytest

from sea_dsl import AuthorityEnvironment

CONF_DIR = Path(__file__).resolve().parent.parent / "conformance" / "08_authority"

_VOLATILE_KEYS = frozenset({"created_at", "decision_id", "trace_ref"})


def _normalize_volatile(value):
    if isinstance(value, dict):
        return {
            k: ("<volatile>" if k in _VOLATILE_KEYS else _normalize_volatile(v))
            for k, v in value.items()
        }
    if isinstance(value, list):
        return [_normalize_volatile(item) for item in value]
    return value


def _load_json(file_name):
    return json.loads((CONF_DIR / file_name).read_text())


@pytest.fixture(scope="module")
def evaluation_result():
    config_json = (CONF_DIR / "config.json").read_text()
    request_json = (CONF_DIR / "request.json").read_text()
    facts_json = (CONF_DIR / "facts.json").read_text()

    env = AuthorityEnvironment(config_json)
    env.validate()
    trace_json, decision_json = env.evaluate(request_json, facts_json)
    return json.loads(trace_json), json.loads(decision_json)


def test_trace_matches_golden(evaluation_result):
    actual_trace, _ = evaluation_result
    expected_trace = _load_json("trace.json")
    assert _normalize_volatile(actual_trace) == _normalize_volatile(expected_trace), (
        "Python binding trace does not match the Rust golden (volatile-normalized). "
        "If the trace shape changed deliberately, regenerate with: "
        "cargo test --features cli --test authority_fixture_tests generate_fixtures -- --ignored"
    )


def test_decision_matches_golden(evaluation_result):
    _, actual_decision = evaluation_result
    expected_decision = _load_json("decision.json")
    assert _normalize_volatile(actual_decision) == _normalize_volatile(expected_decision), (
        "Python binding decision does not match the Rust golden (volatile-normalized). "
        "If the decision shape changed deliberately, regenerate with: "
        "cargo test --features cli --test authority_fixture_tests generate_fixtures -- --ignored"
    )


def test_final_decision_is_deny(evaluation_result):
    _, actual_decision = evaluation_result
    assert actual_decision["final_decision"] == "deny"
