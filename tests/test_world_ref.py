"""world_ref through the Python binding, driven by the same golden vectors the
Rust core and the independent Python reference use."""
import json
from pathlib import Path

import pytest

import domainforge

VECTORS = json.loads(
    (
        Path(__file__).resolve().parents[1]
        / "domainforge-core/tests/fixtures/world_ref/golden-vectors.json"
    ).read_text(encoding="utf-8")
)
G = domainforge.Graph
SOURCES = json.dumps({"main.sea": '@namespace "t"\nentity "Tank" { key id: uuid }\n'})


@pytest.mark.parametrize("case", VECTORS["valid"], ids=lambda c: c["case"])
def test_binding_reproduces_golden_world_ref(case):
    identity_json = json.dumps(case["identity"])
    assert G.world_ref_from_identity_json(case["name"], identity_json) == case["world_ref"]
    assert G.parse_world_ref(case["world_ref"]) == case["world_ref"]
    G.verify_world_ref(case["world_ref"], identity_json)


@pytest.mark.parametrize("bad", VECTORS["invalid_refs"])
def test_binding_rejects_invalid_refs(bad):
    with pytest.raises(ValueError):
        G.parse_world_ref(bad)


def test_tampered_identity_fails_verification():
    case = VECTORS["valid"][0]
    forged = dict(case["identity"], semantic_closure_hash="sha256:" + "0" * 64)
    with pytest.raises(ValueError, match="digest mismatch"):
        G.verify_world_ref(case["world_ref"], json.dumps(forged))


def test_real_resolution_pins_a_verifiable_world():
    identity_json = G.domain_model_identity_json("main.sea", SOURCES)
    identity = json.loads(identity_json)
    assert identity["identity_scheme_version"] == "v2-full-preimage"
    world = G.world_ref_from_identity_json("t", identity_json)
    assert world.startswith("world:t@sha256:")
    G.verify_world_ref(world, identity_json)
    assert G.domain_model_identity_json("main.sea", SOURCES) == identity_json


def test_semantic_change_yields_new_world_same_name():
    other = json.dumps(
        {"main.sea": '@namespace "t"\nentity "Pump" { key id: uuid }\n'}
    )
    a = G.world_ref_from_identity_json("t", G.domain_model_identity_json("main.sea", SOURCES))
    b = G.world_ref_from_identity_json("t", G.domain_model_identity_json("main.sea", other))
    assert a != b
