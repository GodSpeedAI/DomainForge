import pytest
import domainforge


def test_parse_basic():
    source = '''
        Entity "Warehouse" in logistics
        Entity "Factory" in logistics
        Resource "Cameras" units
    '''
    
    graph = domainforge.Graph.parse(source)
    assert graph.entity_count() == 2
    assert graph.resource_count() == 1


def test_parse_with_flow():
    source = '''
        Entity "Warehouse"
        Entity "Factory"
        Resource "Cameras" units
        Flow "Cameras" from "Warehouse" to "Factory" quantity 100
    '''
    
    graph = domainforge.Graph.parse(source)
    assert graph.entity_count() == 2
    assert graph.resource_count() == 1
    assert graph.flow_count() == 1


def test_parse_complex():
    source = '''
        Entity "Supplier" in supply_chain
        Entity "Warehouse" in supply_chain
        Entity "Store" in retail
        
        Resource "Widgets" units
        Resource "Gadgets" units
        
        Flow "Widgets" from "Supplier" to "Warehouse" quantity 500
        Flow "Widgets" from "Warehouse" to "Store" quantity 300
        Flow "Gadgets" from "Supplier" to "Warehouse" quantity 200
    '''
    
    graph = domainforge.Graph.parse(source)
    assert graph.entity_count() == 3
    assert graph.resource_count() == 2
    assert graph.flow_count() == 3


def test_parse_invalid_syntax():
    source = '''
        Entity "Warehouse"
        Invalid syntax here
    '''
    
    with pytest.raises(ValueError, match="Parse error"):
        domainforge.Graph.parse(source)


def test_parse_empty():
    source = ""
    graph = domainforge.Graph.parse(source)
    assert graph.entity_count() == 0


def test_parsed_graph_query():
    source = '''
        Entity "Warehouse"
        Entity "Factory"
        Resource "Steel" tons
        Flow "Steel" from "Warehouse" to "Factory" quantity 50
    '''
    
    graph = domainforge.Graph.parse(source)
    
    warehouse_id = graph.find_entity_by_name("Warehouse")
    assert warehouse_id is not None
    
    flows = graph.flows_from(warehouse_id)
    assert len(flows) == 1
    assert flows[0].quantity == 50.0


# ---- application contract binding (ADR-013 Milestone 0) ----

def _flagship_sources_json() -> str:
    import json
    import pathlib

    root = (
        pathlib.Path(__file__).resolve().parents[1]
        / "fixtures"
        / "application_generation"
        / "flagship"
    )
    return json.dumps(
        {
            "flagship/command-write.sea": (root / "command-write.sea").read_text(),
            "flagship/query-read.sea": (root / "query-read.sea").read_text(),
        },
        separators=(",", ":"),
    )


def test_resolve_application_contract_json_is_canonical():
    import json

    sources = _flagship_sources_json()
    raw = domainforge.Graph.resolve_application_contract_json(
        "flagship/query-read.sea", sources
    )
    again = domainforge.Graph.resolve_application_contract_json(
        "flagship/query-read.sea", sources
    )
    assert raw == again, "canonical contract JSON must be byte-deterministic"
    doc = json.loads(raw)
    assert doc["schema_version"] == "domainforge-application-contract/v1"
    assert doc["self_hash"].startswith("sha256:")
    assert doc["semantic_closure_hash"].startswith("sha256:")


def test_resolve_application_contract_json_rejects_bad_source_map():
    import pytest

    with pytest.raises(ValueError) as err:
        domainforge.Graph.resolve_application_contract_json("a.sea", "[]")
    assert "APP" in str(err.value)


# ---- cross-binding byte parity (M0 gate finding 2) ----
#
# These constants must match the Rust golden hashes in
# `domainforge-core/tests/application_cross_binding_golden_tests.rs`. If a
# binding drifts, its own parity test fails; if the canonical serializer
# intentionally changes, regenerate all four constants in lockstep.
CONTRACT_GOLDEN_SHA256 = (
    "sha256:57c81f0cddc0cec87eaef86cca6692134376076620c9c73844b016869cc31640"
)
ENVELOPE_GOLDEN_SHA256 = (
    "sha256:6c0e730fd4207cb847fac0f8f0ba62c80d3cba0acf6a0e45fe8e555bd633d9e5"
)


def test_cross_binding_contract_bytes_match_rust_golden():
    import hashlib

    sources = _flagship_sources_json()
    raw = domainforge.Graph.resolve_application_contract_json(
        "flagship/query-read.sea", sources
    )
    digest = "sha256:" + hashlib.sha256(raw.encode("utf-8")).hexdigest()
    assert digest == CONTRACT_GOLDEN_SHA256, (
        "Python binding bytes drifted from the Rust golden; if intentional, "
        "regenerate CONTRACT_GOLDEN_SHA256 in the Rust, Python, TS, and WASM "
        "parity tests"
    )
