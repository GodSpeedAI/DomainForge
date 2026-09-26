"""Cross-binding byte parity for the application contract (M0 gate finding 2).

The Python binding must produce byte-identical canonical JSON to the Rust
golden. The same constants appear in:

- Rust:   domainforge-core/tests/application_cross_binding_golden_tests.rs
- Python: this file
- TS:     typescript-tests/cross-binding-parity.test.ts
- WASM:   domainforge-core/tests/wasm_tests.rs (cross_binding_golden_hashes)

The producer.version stamp is asserted against the installed package
version; the golden pins the full bytes at the current release version and
must be regenerated with every version bump (see the release notes). If
serialization intentionally changes, regenerate all four in lockstep.
"""

import hashlib
import importlib.metadata
import json
import pathlib

import pytest

import domainforge


CONTRACT_GOLDEN_SHA256 = (
    "sha256:e2510cdb56fa43b684fd6aaeeb87749fb8dab413f5bff3854d122da53be94487"
)


def _flagship_sources_map() -> dict:
    root = (
        pathlib.Path(__file__).resolve().parents[1]
        / "fixtures"
        / "application_generation"
        / "flagship"
    )
    return {
        "flagship/command-write.sea": (root / "command-write.sea").read_text(),
        "flagship/query-read.sea": (root / "query-read.sea").read_text(),
    }


def test_cross_binding_contract_bytes_match_rust_golden():
    """Verify the contract's package version stamp and full-byte parity with Rust."""
    sources = json.dumps(_flagship_sources_map(), separators=(",", ":"))
    raw = domainforge.Graph.resolve_application_contract_json(
        "flagship/query-read.sea", sources
    )
    pkg_version = importlib.metadata.version("domainforge")
    doc = json.loads(raw)
    assert doc["producer"]["version"] == pkg_version, (
        "producer.version must stamp the running package version"
    )
    digest = "sha256:" + hashlib.sha256(raw.encode("utf-8")).hexdigest()
    assert digest == CONTRACT_GOLDEN_SHA256, (
        "Python binding bytes drifted from the Rust golden; if intentional, "
        "regenerate CONTRACT_GOLDEN_SHA256 in the Rust, Python, TS, and WASM "
        "parity tests"
    )


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
