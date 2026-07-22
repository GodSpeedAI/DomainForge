"""Cross-binding byte parity for the application contract (M0 gate finding 2).

The Python binding must produce byte-identical canonical JSON to the Rust
golden. The same constants appear in:

- Rust:   domainforge-core/tests/application_cross_binding_golden_tests.rs
- Python: this file
- TS:     typescript-tests/cross-binding-parity.test.ts
- WASM:   domainforge-core/tests/wasm_tests.rs (cross_binding_golden_hashes)

If serialization intentionally changes, regenerate all four in lockstep.
"""

import hashlib
import json
import pathlib

import pytest

import domainforge


CONTRACT_GOLDEN_SHA256 = (
    "sha256:ca0255a79a9cee5b7ee8b78db8bb9a93728edfe146aab9c8eb973d9e94495be5"
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
    sources = json.dumps(_flagship_sources_map(), separators=(",", ":"))
    raw = domainforge.Graph.resolve_application_contract_json(
        "flagship/query-read.sea", sources
    )
    digest = "sha256:" + hashlib.sha256(raw.encode("utf-8")).hexdigest()
    assert digest == CONTRACT_GOLDEN_SHA256, (
        "Python binding bytes drifted from the Rust golden; if intentional, "
        "regenerate CONTRACT_GOLDEN_SHA256 in the Rust, Python, TS, and WASM "
        "parity tests"
    )


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
