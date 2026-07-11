"""Python binding surface for the BPMN 2.0 projection (export_bpmn).

Also runs a live XSD validation of the projected fixture against the vendored
BPMN 2.0 schema when ``lxml`` is available (skipped otherwise so the binding
tests still run in minimal environments).
"""

import json
import pathlib

import pytest

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "bpmn" / "basic"
SCHEMA = ROOT / "schemas" / "bpmn" / "BPMN20.xsd"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def test_export_bpmn_single_file_manifest():
    graph = _fixture_graph()
    artifacts = json.loads(graph.export_bpmn(created_at=FIXED_TS))

    assert set(artifacts) == {"model.bpmn"}
    bpmn = artifacts["model.bpmn"]
    assert "<definitions" in bpmn
    assert 'isExecutable="false"' in bpmn
    assert 'gatewayDirection="Diverging"' in bpmn
    assert 'gatewayDirection="Converging"' in bpmn
    assert 'name="CameraUnits"' in bpmn  # data object
    assert 'name="Operator"' in bpmn  # lane


def test_export_bpmn_is_deterministic():
    graph = _fixture_graph()
    assert graph.export_bpmn(created_at=FIXED_TS) == graph.export_bpmn(created_at=FIXED_TS)


def test_projected_bpmn_is_xsd_valid():
    lxml_etree = pytest.importorskip("lxml.etree")
    graph = _fixture_graph()
    bpmn = json.loads(graph.export_bpmn(created_at=FIXED_TS))["model.bpmn"]

    schema = lxml_etree.XMLSchema(lxml_etree.parse(str(SCHEMA)))
    doc = lxml_etree.fromstring(bpmn.encode("utf-8"))
    # assertValid raises with a detailed message on failure.
    schema.assertValid(doc)
