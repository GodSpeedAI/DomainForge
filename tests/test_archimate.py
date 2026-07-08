"""Python binding surface for the ArchiMate 3.0 projection (export_archimate).

Also runs a live XSD validation of the projected fixture against the vendored
ArchiMate 3.0 Model Exchange File schema when ``lxml`` is available (skipped
otherwise so the binding tests still run in minimal environments).
"""

import json
import pathlib

import pytest

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "archimate" / "basic"
SCHEMA = ROOT / "schemas" / "archimate" / "archimate3_Model.xsd"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def test_export_archimate_single_file_manifest():
    graph = _fixture_graph()
    artifacts = json.loads(graph.export_archimate(created_at=FIXED_TS))

    assert set(artifacts) == {"model.xml"}
    xml = artifacts["model.xml"]
    assert "<model" in xml
    assert 'xsi:type="BusinessRole"' in xml
    assert 'xsi:type="BusinessObject"' in xml  # entities + resources
    assert 'xsi:type="BusinessProcess"' in xml  # flows
    assert 'xsi:type="Requirement"' in xml  # policies
    assert 'xsi:type="Association"' in xml  # policy references an object


def test_export_archimate_is_deterministic():
    graph = _fixture_graph()
    assert graph.export_archimate(created_at=FIXED_TS) == graph.export_archimate(
        created_at=FIXED_TS
    )


def test_projected_archimate_is_xsd_valid():
    lxml_etree = pytest.importorskip("lxml.etree")
    if not SCHEMA.exists():
        pytest.skip("vendored ArchiMate XSD not present")
    graph = _fixture_graph()
    xml = json.loads(graph.export_archimate(created_at=FIXED_TS))["model.xml"]

    schema = lxml_etree.XMLSchema(lxml_etree.parse(str(SCHEMA)))
    doc = lxml_etree.fromstring(xml.encode("utf-8"))
    # assertValid raises with a detailed message on failure.
    schema.assertValid(doc)
