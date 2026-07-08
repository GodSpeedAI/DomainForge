"""Python binding surface for the RDF/OWL projection (export_rdf_projection)."""

import json
import pathlib

import domainforge

FIXTURE = pathlib.Path(__file__).parent.parent / "fixtures" / "rdf" / "basic"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def test_export_rdf_manifest_and_jsonld():
    graph = _fixture_graph()
    artifacts = json.loads(graph.export_rdf_projection(created_at=FIXED_TS))

    assert set(artifacts) == {"model.ttl", "model.jsonld", "ontology.owl.ttl"}
    assert "sea:Warehouse rdf:type sea:Entity" in artifacts["model.ttl"]
    assert "sea:from a owl:ObjectProperty" in artifacts["ontology.owl.ttl"]

    doc = json.loads(artifacts["model.jsonld"])
    assert doc["@context"]["sea"] == "http://domainforge.ai/sea#"
    assert isinstance(doc["@graph"], list)


def test_export_rdf_base_iri_override():
    graph = _fixture_graph()
    artifacts = json.loads(
        graph.export_rdf_projection(
            created_at=FIXED_TS, base_iri="https://example.org/demo#"
        )
    )
    doc = json.loads(artifacts["model.jsonld"])
    assert doc["@context"]["sea"] == "https://example.org/demo#"
    assert "@prefix sea: <https://example.org/demo#>" in artifacts["ontology.owl.ttl"]


def test_export_rdf_is_deterministic():
    graph = _fixture_graph()
    first = graph.export_rdf_projection(created_at=FIXED_TS)
    second = graph.export_rdf_projection(created_at=FIXED_TS)
    assert first == second
