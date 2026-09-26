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
//! The `producer.version` stamp is asserted separately against the running
//! crate version; the goldens pin the full bytes at the current release
//! version and must be regenerated with every version bump (see the release
//! notes). If canonical serialization intentionally changes, regenerate
//! these constants in lockstep across all four suites. The `just all-tests`
//! recipe exercises Rust + Python + TypeScript; WASM runs under
//! `wasm-bindgen-test` via the `wasm` feature.

use domainforge_core::application::{resolve_application_contract, resolve_semantic_envelope};
use sha2::{Digest, Sha256};

/// Fixed golden hash of `serde_json::to_string(&contract_doc)` for the
/// flagship `query-read` closure at the current release version. These
/// constants embed the release version (directly and via derived hashes),
/// so they must be regenerated with every version bump — see the release
/// notes. The stamp itself is asserted separately below.
pub const CONTRACT_GOLDEN_SHA256: &str =
    "sha256:78fa1c173ca7da383c5082b6bc2a442faf6280e24c2a6796bc99a34aa8a369cb";

/// Fixed golden hash of `serde_json::to_string(&envelope_doc)` for the
/// flagship `query-read` closure at the current release version (same
/// regeneration rule as above).
pub const ENVELOPE_GOLDEN_SHA256: &str =
    "sha256:1e01c9960deb2f78172b3b173824ddb97293347b6d900fc9a75957426b25abc7";

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
    assert_producer_version(&bytes);
    let hash = sha256_hex(bytes.as_bytes());
    assert_eq!(
        hash, ENVELOPE_GOLDEN_SHA256,
        "Rust envelope bytes drifted from the cross-binding golden; if intentional, \
         regenerate ENVELOPE_GOLDEN_SHA256 here and in the Python/TS/WASM parity tests"
    );
}

/// Assert the canonical documents stamp the running crate version, proving
/// the stamp mechanism the goldens above pin to the release version.
fn assert_producer_version(bytes: &str) {
    let doc: serde_json::Value = serde_json::from_str(bytes).expect("canonical document parses");
    assert_eq!(
        doc["producer"]["version"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "producer.version must stamp the running crate version"
    );
}
