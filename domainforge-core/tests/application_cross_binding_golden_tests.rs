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
//! The `producer.version` stamp (the crate version at build time) is
//! normalized out before hashing so release version bumps do not drift the
//! goldens; each suite separately asserts the stamp equals its own package
//! version. If canonical serialization intentionally changes, regenerate
//! these constants in lockstep across all four suites. The `just all-tests`
//! recipe exercises Rust + Python + TypeScript; WASM runs under
//! `wasm-bindgen-test` via the `wasm` feature.

use domainforge_core::application::{resolve_application_contract, resolve_semantic_envelope};
use sha2::{Digest, Sha256};

/// Fixed golden hash of `serde_json::to_string(&contract_doc)` for the
/// flagship `query-read` closure, with the producer version stamp normalized
/// to `0.0.0` (see [`normalize_producer_version`]).
pub const CONTRACT_GOLDEN_SHA256: &str =
    "sha256:38299bd2d0b062d45088f60ceef7018e4f89a1abe839be8edbdd1c961cd303e5";

/// Fixed golden hash of `serde_json::to_string(&envelope_doc)` for the
/// flagship `query-read` closure, with the producer version stamp normalized
/// to `0.0.0` (see [`normalize_producer_version`]).
pub const ENVELOPE_GOLDEN_SHA256: &str =
    "sha256:1b8382dfd3754098b0253a89722f21df9977d4b02e41c7779d06ca8a3f044f91";

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
    assert_producer_version(&bytes);
    let hash = sha256_hex(normalize_producer_version(&bytes).as_bytes());
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
    assert_producer_version(&bytes);
    let hash = sha256_hex(normalize_producer_version(&bytes).as_bytes());
    assert_eq!(
        hash, ENVELOPE_GOLDEN_SHA256,
        "Rust envelope bytes drifted from the cross-binding golden; if intentional, \
         regenerate ENVELOPE_GOLDEN_SHA256 here and in the Python/TS/WASM parity tests"
    );
}

/// Replace the build-time producer version stamp with a fixed token so the
/// goldens above stay stable across releases. The stamp itself is asserted
/// separately by [`assert_producer_version`].
fn normalize_producer_version(bytes: &str) -> String {
    bytes.replace(env!("CARGO_PKG_VERSION"), "0.0.0")
}

/// Assert the canonical documents stamp the running crate version, proving
/// the stamp mechanism the goldens normalize away.
fn assert_producer_version(bytes: &str) {
    let doc: serde_json::Value =
        serde_json::from_str(bytes).expect("canonical document parses");
    assert_eq!(
        doc["producer"]["version"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "producer.version must stamp the running crate version"
    );
}
