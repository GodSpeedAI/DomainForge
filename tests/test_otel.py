"""Python binding surface for the OpenTelemetry SemConv projection
(export_otel_semconv).

Also validates the projected registry YAML against the DomainForge SemConv
registry JSON schema when PyYAML + jsonschema are available (skipped otherwise
so the binding tests still run in minimal environments).
"""

import json
import pathlib

import pytest

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "otel" / "basic"
SCHEMA = ROOT / "schemas" / "otel" / "semconv_registry.schema.json"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def _artifacts():
    return json.loads(_fixture_graph().export_otel_semconv(created_at=FIXED_TS))


def test_export_otel_semconv_manifest():
    artifacts = _artifacts()
    assert set(artifacts) == {
        "registry/telemetry.yaml",
        "constants/attributes.rs",
        "constants/attributes.py",
        "constants/attributes.ts",
    }
    yaml_text = artifacts["registry/telemetry.yaml"]
    assert "domainforge.model.hash" in yaml_text
    assert "domainforge.element.id" in yaml_text
    assert "demo.entity." in yaml_text
    assert "span_kind: internal" in yaml_text


def test_export_otel_semconv_is_deterministic():
    graph = _fixture_graph()
    assert graph.export_otel_semconv(created_at=FIXED_TS) == graph.export_otel_semconv(
        created_at=FIXED_TS
    )


def test_registry_and_constants_agree_on_attribute_set():
    yaml = pytest.importorskip("yaml")
    artifacts = _artifacts()
    registry = yaml.safe_load(artifacts["registry/telemetry.yaml"])

    yaml_ids = set()
    for group in registry["groups"]:
        if group["type"] == "attribute_group":
            for attr in group["attributes"]:
                yaml_ids.add(attr["id"])

    def const_values(src, sep):
        out = set()
        for line in src.splitlines():
            if sep in line:
                val = line.split(sep, 1)[1].strip().rstrip(";").strip('"')
                if "." in val:
                    out.add(val)
        return out

    rs = const_values(artifacts["constants/attributes.rs"], '= "')
    py = const_values(artifacts["constants/attributes.py"], ': str = "')
    ts = const_values(artifacts["constants/attributes.ts"], '= "')

    assert yaml_ids == rs
    assert yaml_ids == py
    assert yaml_ids == ts
    assert "domainforge.model.hash" in yaml_ids


def test_projected_registry_is_schema_valid():
    yaml = pytest.importorskip("yaml")
    jsonschema = pytest.importorskip("jsonschema")
    if not SCHEMA.exists():
        pytest.skip("registry schema not present")

    registry = yaml.safe_load(_artifacts()["registry/telemetry.yaml"])
    schema = json.loads(SCHEMA.read_text())
    jsonschema.validate(registry, schema)


def test_reserved_namespace_never_appears():
    # No attribute is minted under an OTel-reserved namespace.
    yaml_text = _artifacts()["registry/telemetry.yaml"]
    for reserved in ("service.", "otel.", "telemetry.", "http.", "rpc."):
        assert f'- id: "{reserved}' not in yaml_text
