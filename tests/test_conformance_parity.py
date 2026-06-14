"""Cross-language conformance parity (Phase 2 of the Semantic Infrastructure Audit).

Loads the SHARED ``conformance/`` corpus, parses each ``parse`` and ``validate``
item via the Python binding, serializes to canonical JSON, normalizes volatile
flow UUIDs to positional placeholders, and byte-compares against the Rust-pinned
``expected/`` files produced by ``sea parse --format json`` and
``sea validate --format json``.

This proves the Rust core produces identical canonical JSON through the PyO3
binding for both the structural spine (``parse``) and the policy-evaluation
aggregate (``validate``) — closing the G5 "one engine, many wrappers" parity gap.

Run: pytest tests/test_conformance_parity.py
"""

import json
from pathlib import Path

import pytest

from sea_dsl import Graph

CONF_DIR = Path(__file__).resolve().parent.parent / "conformance"


def _normalize_flow_ids(value):
    """Replace volatile flow UUIDs (map keys + inner id fields) with flow:0, flow:1, ..."""
    text = json.dumps(value, sort_keys=False)
    flows = value.get("flows") if isinstance(value, dict) else None
    if isinstance(flows, dict):
        for i, key in enumerate(flows.keys()):
            text = text.replace(key, f"flow:{i}")
    data = json.loads(text)
    return data


def _load_corpus_items():
    items = []
    if not CONF_DIR.is_dir():
        return items
    for entry in sorted(CONF_DIR.iterdir()):
        manifest_path = entry / "manifest.json"
        if not manifest_path.is_file():
            continue
        manifest = json.loads(manifest_path.read_text())
        command = manifest.get("command")
        if command not in ("parse", "validate"):
            continue
        expected_path = entry / manifest["expected"]
        input_path = entry / manifest["input"]
        items.append((entry.name, command, input_path, expected_path))
    return items


@pytest.mark.parametrize("item_name,command,input_path,expected_path", _load_corpus_items())
def test_canonical_json_matches_rust(item_name, command, input_path, expected_path):
    source = input_path.read_text()
    graph = Graph.parse(source)
    if command == "validate":
        actual_raw = graph.validate_json()
    else:
        actual_raw = graph.to_json()
    actual = _normalize_flow_ids(json.loads(actual_raw))

    expected = _normalize_flow_ids(json.loads(expected_path.read_text()))

    oracle_cmd = "validate" if command == "validate" else "parse"
    assert actual == expected, (
        f"Conformance parity drift in '{item_name}': "
        f"Python binding {'validate_json()' if command == 'validate' else 'to_json()'} "
        f"does not match Rust-pinned expected file ({expected_path.name}). "
        f"If this change is intentional, regenerate the expected file with: "
        f"sea {oracle_cmd} {input_path} --format json > {expected_path}"
    )


def test_corpus_has_at_least_one_item():
    items = _load_corpus_items()
    assert len(items) >= 1, (
        f"Expected at least 1 conformance item in {CONF_DIR}, found {len(items)}"
    )
