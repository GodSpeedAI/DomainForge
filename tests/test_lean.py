"""Python binding surface for the Lean 4 projection (export_lean)."""

import json
import pathlib

import domainforge

FIXTURE = pathlib.Path(__file__).parent.parent / "fixtures" / "lean" / "basic"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def test_export_lean_layout_and_proofs():
    graph = _fixture_graph()
    artifacts = json.loads(graph.export_lean(created_at=FIXED_TS))

    assert "lakefile.toml" in artifacts
    assert "lean-toolchain" in artifacts
    policies = artifacts["DomainForge/Policies.lean"]
    assert "theorem policy_positive_flow_holds : policy_positive_flow := by decide" in policies
    assert "sorry" not in policies
    assert "by sorry" in artifacts["Obligations/Stubs.lean"]


def test_export_lean_is_deterministic():
    graph = _fixture_graph()
    first = graph.export_lean(created_at=FIXED_TS)
    second = graph.export_lean(created_at=FIXED_TS)
    assert first == second
