"""Python binding surface for the CMMN 1.1 projection (export_cmmn).

Also runs a live XSD validation of the projected fixture against the vendored
CMMN 1.1 schema when ``lxml`` is available (skipped otherwise so the binding
tests still run in minimal environments).
"""

import json
import pathlib

import pytest

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "cmmn" / "basic"
SCHEMA = ROOT / "schemas" / "cmmn" / "CMMN11.xsd"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def test_export_cmmn_single_file_manifest():
    graph = _fixture_graph()
    artifacts = json.loads(graph.export_cmmn(created_at=FIXED_TS))

    assert set(artifacts) == {"model.cmmn"}
    cmmn = artifacts["model.cmmn"]
    assert "<definitions" in cmmn
    assert "<casePlanModel" in cmmn
    assert 'name="Budget"' in cmmn  # case file item from resource
    assert 'name="Approver"' in cmmn  # case role
    assert "<milestone" in cmmn
    assert "<sentry" in cmmn  # policy condition lowered to a sentry


def test_export_cmmn_is_deterministic():
    graph = _fixture_graph()
    assert graph.export_cmmn(created_at=FIXED_TS) == graph.export_cmmn(created_at=FIXED_TS)


def test_projected_cmmn_is_xsd_valid():
    lxml_etree = pytest.importorskip("lxml.etree")
    graph = _fixture_graph()
    cmmn = json.loads(graph.export_cmmn(created_at=FIXED_TS))["model.cmmn"]

    schema = lxml_etree.XMLSchema(lxml_etree.parse(str(SCHEMA)))
    doc = lxml_etree.fromstring(cmmn.encode("utf-8"))
    # assertValid raises with a detailed message on failure.
    schema.assertValid(doc)
