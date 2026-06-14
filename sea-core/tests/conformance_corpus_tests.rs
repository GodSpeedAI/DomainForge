//! Conformance corpus harness (Phase 1 of the Semantic Infrastructure Audit).
//!
//! Runs the `sea` CLI — the canonical reference oracle — against every item in the
//! top-level `conformance/` corpus and compares its JSON output against the
//! checked-in `expected/` file for that item.
//!
//! Determinism: every concept ID except flows is content-derived (UUID v5) and
//! therefore stable across runs and runtimes. Flow IDs are deliberately random
//! (UUID v4 event identity, see `src/primitives/flow.rs`), so they are normalized
//! to positional placeholders on BOTH the actual and expected sides before
//! comparison. See `docs/specs/canonical_entrypoints.md`.
//!
//! This corpus IS the spec: any intentional output change requires regenerating the
//! corresponding `expected/` file with the documented `sea ... --format json`
//! command. An unintentional change fails this test.
//!
//! Run with: `cargo test --features cli --test conformance_corpus_tests`
#![cfg(feature = "cli")]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// Location of the top-level `conformance/` corpus (sibling of the `sea-core` crate).
fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sea-core crate has a parent workspace directory")
        .join("conformance")
}

/// Path to the `sea` binary built for this test run (set by Cargo when the
/// `cli`-gated binary target is built).
fn sea_bin() -> &'static str {
    env!("CARGO_BIN_EXE_sea")
}

/// Normalize volatile flow UUIDs (map keys + inner `id` fields) to positional
/// tokens so canonical graph JSON is byte-comparable. No-op for payloads without a
/// top-level `flows` object (e.g. policy-evaluation output).
fn normalize(value: Value) -> Value {
    let replacements: Vec<(String, String)> = match value.get("flows").and_then(Value::as_object) {
        Some(flows) => flows
            .keys()
            .enumerate()
            .map(|(i, key)| (key.clone(), format!("flow:{i}")))
            .collect(),
        None => Vec::new(),
    };
    if replacements.is_empty() {
        return value;
    }
    // Flow UUIDs appear only as the flows-map key and the flow's own `id` field;
    // a textual substitution rewrites both safely (UUIDs never substring-overlap).
    let mut serialized = serde_json::to_string(&value).expect("serialize JSON value");
    for (uuid, token) in &replacements {
        serialized = serialized.replace(uuid, token);
    }
    serde_json::from_str(&serialized).expect("re-parse normalized JSON")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("invalid JSON in {}: {e}", path.display()))
}

#[test]
fn serde_json_preserves_flow_key_parse_order_when_normalizing_uuid_maps() {
    // Given: flow IDs arrive in parse order, not lexicographic order.
    let value: Value = serde_json::from_str(
        r#"{
            "flows": {
                "ffffffff-ffff-4fff-8fff-ffffffffffff": {
                    "id": "ffffffff-ffff-4fff-8fff-ffffffffffff"
                },
                "00000000-0000-4000-8000-000000000000": {
                    "id": "00000000-0000-4000-8000-000000000000"
                }
            }
        }"#,
    )
    .expect("flow fixture is valid JSON");

    // When: normalize enumerates flow keys.
    let keys: Vec<&str> = value
        .get("flows")
        .and_then(Value::as_object)
        .expect("flows is an object")
        .keys()
        .map(String::as_str)
        .collect();

    // Then: positional placeholders match JSON parse order instead of sorted UUIDs.
    assert_eq!(
        keys,
        vec![
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "00000000-0000-4000-8000-000000000000",
        ],
    );
}

fn run_item(dir: &Path) {
    let manifest = read_json(&dir.join("manifest.json"));
    let command = manifest["command"]
        .as_str()
        .expect("manifest.command is a string");
    let input = dir.join(
        manifest["input"]
            .as_str()
            .expect("manifest.input is a string"),
    );
    let expected_path = dir.join(
        manifest["expected"]
            .as_str()
            .expect("manifest.expected is a string"),
    );

    let mut args: Vec<String> = vec![
        command.to_string(),
        input.display().to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if command == "validate" {
        args.push("--no-color".to_string());
    } else if command != "parse" {
        panic!("item {}: unsupported command '{command}'", dir.display());
    }

    let output = Command::new(sea_bin())
        .args(&args)
        .output()
        .unwrap_or_else(|e| panic!("item {}: failed to run sea: {e}", dir.display()));

    // NOTE: `validate` exits non-zero when policies are violated; the JSON oracle is
    // still written to stdout, so we intentionally do not assert on exit status.
    let stdout = String::from_utf8(output.stdout).expect("sea stdout is utf-8");
    let actual: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "item {}: `sea {command}` did not produce JSON ({e}).\nstdout:\n{stdout}\nstderr:\n{}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        )
    });

    let actual = normalize(actual);
    let expected = normalize(read_json(&expected_path));

    assert_eq!(
        actual,
        expected,
        "\nConformance drift in '{}'.\nThe `sea {command}` output no longer matches the pinned spec.\n\
         If this change is intentional, regenerate the expected file:\n  \
         sea {command} {} --format json > {}\n",
        dir.display(),
        input.display(),
        expected_path.display(),
    );
}

#[test]
fn conformance_corpus_matches_cli_oracle() {
    let dir = conformance_dir();
    assert!(
        dir.is_dir(),
        "conformance corpus not found at {}",
        dir.display()
    );

    let mut items: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read conformance dir")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir() && path.join("manifest.json").is_file())
        .collect();
    items.sort();

    assert!(
        items.len() >= 7,
        "expected at least 7 active conformance items, found {}",
        items.len()
    );

    for item in &items {
        run_item(item);
    }
    eprintln!(
        "conformance: {} item(s) verified against the `sea` CLI oracle",
        items.len()
    );
}
