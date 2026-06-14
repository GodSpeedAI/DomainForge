"""Cross-language conformance parity (Phase 2 of the Semantic Infrastructure Audit).

Loads the SHARED ``conformance/`` corpus, parses each ``parse`` item via the
Python binding, serializes to canonical JSON, normalizes volatile flow UUIDs to
positional placeholders, and byte-compares against the Rust-pinned ``expected/``
files produced by ``sea parse --format json``.

This proves the Rust core produces identical canonical graph JSON through the
PyO3 binding — closing the G5 "one engine, many wrappers" parity gap for the
structural spine.

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
        if manifest.get("command") != "parse":
            continue
        expected_path = entry / manifest["expected"]
        input_path = entry / manifest["input"]
        items.append((entry.name, input_path, expected_path))
    return items


@pytest.mark.parametrize("item_name,input_path,expected_path", _load_corpus_items())
def test_parse_canonical_json_matches_rust(item_name, input_path, expected_path):
    source = input_path.read_text()
    graph = Graph.parse(source)
    actual_raw = graph.to_json()
    actual = json.loads(actual_raw)
    actual = _normalize_flow_ids(actual)

    expected = json.loads(expected_path.read_text())
    expected = _normalize_flow_ids(expected)

    assert actual == expected, (
        f"Conformance parity drift in '{item_name}': "
        f"Python binding Graph.to_json() does not match Rust-pinned expected file "
        f"({expected_path.name}). If this change is intentional, regenerate the "
        f"expected file with: sea parse {input_path} --format json > {expected_path}"
    )


def test_corpus_has_at_least_one_parse_item():
    items = _load_corpus_items()
    assert len(items) >= 1, (
        f"Expected at least 1 parse conformance item in {CONF_DIR}, found {len(items)}"
    )
