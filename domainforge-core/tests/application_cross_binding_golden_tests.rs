//! Cross-binding byte parity (M0 gate finding 2): every language binding
//! resolves the same flagship source map to byte-identical canonical JSON.
//!
//! This test runs in Rust and asserts the Rust-produced bytes hash to the
//! fixed golden values below. The Python (`tests/test_parser.py::
//! test_cross_binding_contract_bytes_match_rust_golden`), TypeScript
//! (`typescript-tests/cross-binding-parity.test.ts`), and WASM
//! (`wasm_tests.rs::cross_binding_golden_hashes`) suites embed the same
//! constants and hash their own binding's output, so a stale or divergent
//! binding fails its own test instead of silently drifting.
//!
//! If you intentionally change canonical serialization, regenerate these
//! constants in lockstep across all four suites. The `just all-tests`
//! recipe exercises Rust + Python + TypeScript; WASM runs under
//! `wasm-bindgen-test` via the `wasm` feature.

use domainforge_core::application::{resolve_application_contract, resolve_semantic_envelope};
use sha2::{Digest, Sha256};

/// Fixed golden hash of `serde_json::to_string(&contract_doc)` for the
/// flagship `query-read` closure.
pub const CONTRACT_GOLDEN_SHA256: &str =
    "sha256:ca0255a79a9cee5b7ee8b78db8bb9a93728edfe146aab9c8eb973d9e94495be5";

/// Fixed golden hash of `serde_json::to_string(&envelope_doc)` for the
/// flagship `query-read` closure.
pub const ENVELOPE_GOLDEN_SHA256: &str =
    "sha256:6eed2745d0d75422a6842e5d7b89e1edc25ff542a6fcdf20b969da35a8a7c7cc";

pub fn flagship_sources_json() -> String {
    serde_json::json!({
        "flagship/command-write.sea": include_str!(
            "../../fixtures/application_generation/flagship/command-write.sea"
        ),
        "flagship/query-read.sea": include_str!(
            "../../fixtures/application_generation/flagship/query-read.sea"
        ),
    })
    .to_string()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    format!("sha256:{}", hex_lower(&out))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[test]
fn rust_contract_bytes_match_the_cross_binding_golden() {
    let raw = resolve_application_contract("flagship/query-read.sea", &flagship_sources_json())
        .expect("flagship closure resolves");
    let bytes = serde_json::to_string(&raw).expect("contract serializes");
    let hash = sha256_hex(bytes.as_bytes());
    assert_eq!(
        hash, CONTRACT_GOLDEN_SHA256,
        "Rust contract bytes drifted from the cross-binding golden; if intentional, \
         regenerate CONTRACT_GOLDEN_SHA256 here and in the Python/TS/WASM parity tests"
    );
}

#[test]
fn rust_envelope_bytes_match_the_cross_binding_golden() {
    let raw = resolve_semantic_envelope("flagship/query-read.sea", &flagship_sources_json())
        .expect("flagship closure resolves");
    let bytes = serde_json::to_string(&raw).expect("envelope serializes");
    let hash = sha256_hex(bytes.as_bytes());
    assert_eq!(
        hash, ENVELOPE_GOLDEN_SHA256,
        "Rust envelope bytes drifted from the cross-binding golden; if intentional, \
         regenerate ENVELOPE_GOLDEN_SHA256 here and in the Python/TS/WASM parity tests"
    );
}
