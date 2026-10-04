//! A DomainForge `semantic_snapshot` pins the world it describes: `world_ref`,
//! the DomainModelIdentity, integrity and the GodSpeed profile declaration. An
//! invalid world is flagged and never pinned. Binding is opt-in (`--world-name`)
//! so emission without it is unchanged.

#![cfg(feature = "cli")]

use domainforge_core::application::envelope::DomainModelIdentity;
use domainforge_core::application::resolve_semantic_envelope;
use domainforge_core::application::world::WorldRef;
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

const BASIC: &str = "domainforge-core/tests/fixtures/envelope/valid-basic.sea";
const INVALID: &str = "domainforge-core/tests/fixtures/envelope/invalid-syntax.sea";

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(path)
}

fn domainforge(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_domainforge"))
        .args(args)
        .output()
        .expect("domainforge binary runs")
}

fn snapshot(extra: &[&str], path: &str) -> (Option<i32>, Value) {
    let entry = fixture(path);
    let mut args = vec!["envelope", "--emit", "cep"];
    args.extend_from_slice(extra);
    let entry = entry.to_string_lossy().to_string();
    args.push(&entry);
    let out = domainforge(&args);
    let value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}): {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code(), value)
}

/// The identity the library computes for the same source, independently of the CLI.
fn library_identity(path: &str) -> DomainModelIdentity {
    let source = std::fs::read_to_string(fixture(path)).unwrap();
    let logical = fixture(path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let sources = json!({ logical.clone(): source }).to_string();
    let doc = resolve_semantic_envelope(&logical, &sources).expect("resolves");
    DomainModelIdentity::from_document(&doc, None)
}

#[test]
fn valid_world_is_pinned_with_identity_integrity_and_profile() {
    let (code, env) = snapshot(
        &[
            "--world-name",
            "godspeed-corporate",
            "--world-alias",
            "world:corporate",
            "--world-label",
            "GodSpeed Corporate World",
        ],
        BASIC,
    );
    assert_eq!(code, Some(0));
    let world_ref = env["scope"]["world_ref"]
        .as_str()
        .expect("world_ref pinned");
    assert!(
        world_ref.starts_with("world:godspeed-corporate@sha256:"),
        "{world_ref}"
    );
    assert_eq!(env["scope"]["world_alias"], "world:corporate");
    assert_eq!(env["scope"]["world_label"], "GodSpeed Corporate World");

    // The identity carried in the snapshot hashes to exactly the pinned digest.
    let identity_value = &env["extensions"]["domainforge.identity"]["domain_model_identity"];
    let identity: DomainModelIdentity = serde_json::from_value(identity_value.clone()).unwrap();
    let parsed: WorldRef = world_ref.parse().expect("canonical world_ref");
    parsed
        .verify_identity(&identity)
        .expect("world_ref verifies against the carried identity");

    assert_eq!(
        env["extensions"]["domainforge.identity"]["semantic_closure_hash"],
        identity.semantic_closure_hash
    );
    assert_eq!(
        env["integrity"]["semantic_hash"],
        identity.semantic_closure_hash
    );
    assert_eq!(env["integrity"]["content_hash"], identity.content_hash);
    assert_eq!(
        env["extensions"]["cep.profile"],
        json!({"profile_id": "godspeed.semantic_snapshot", "profile_version": "1.0.0"})
    );
    assert!(
        env["extensions"].get("domainforge").is_none(),
        "valid world carries no failure flag"
    );
}

#[test]
fn cli_world_ref_equals_the_library_world_ref_for_the_same_source() {
    let (_, env) = snapshot(&["--world-name", "t"], BASIC);
    let id = library_identity(BASIC);
    // The CLI resolves with filesystem source paths, so compare closure and
    // verify the carried identity independently rather than byte-equal digests.
    assert_eq!(
        env["extensions"]["domainforge.identity"]["domain_model_identity"]["semantic_closure_hash"],
        id.semantic_closure_hash
    );
}

#[test]
fn binding_is_stable_across_runs_apart_from_envelope_identity() {
    let (_, a) = snapshot(&["--world-name", "t"], BASIC);
    let (_, b) = snapshot(&["--world-name", "t"], BASIC);
    assert_eq!(a["scope"]["world_ref"], b["scope"]["world_ref"]);
    assert_ne!(a["envelope_id"], b["envelope_id"]);
}

#[test]
fn without_world_name_emission_is_unchanged() {
    let (_, env) = snapshot(&[], BASIC);
    assert!(env["scope"].get("world_ref").is_none());
    assert!(env.get("integrity").is_none());
    assert!(
        env["extensions"].is_null(),
        "no extensions for a plain valid snapshot: {}",
        env["extensions"]
    );
}

#[test]
fn invalid_world_is_flagged_and_never_pinned() {
    let (code, env) = snapshot(
        &[
            "--world-name",
            "godspeed-corporate",
            "--world-alias",
            "world:corporate",
        ],
        INVALID,
    );
    assert_ne!(code, Some(0), "an invalid model must not exit successfully");
    assert!(
        env["scope"].get("world_ref").is_none(),
        "invalid world must not pin a world_ref"
    );
    assert!(
        env["scope"].get("world_alias").is_none(),
        "alias is only attached to a pinned world"
    );
    assert_eq!(
        env["extensions"]["domainforge"]["model_validation_status"],
        "invalid"
    );
    assert_eq!(
        env["extensions"]["cep.profile"]["profile_id"],
        "godspeed.semantic_snapshot"
    );
    assert!(env["extensions"].get("domainforge.identity").is_none());
    assert_eq!(env["completeness_status"], "incomplete_for_declared_state");
    assert!(env["omissions"].as_array().is_some_and(|o| !o.is_empty()));
}

#[test]
fn comment_only_edit_is_a_new_world_with_the_same_closure() {
    // Same logical filename in two directories: the module path is part of the
    // semantic closure, so only the comment may differ.
    let body = std::fs::read_to_string(fixture(BASIC)).unwrap();
    let write = |text: &str| {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.sea");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
        (dir, path)
    };
    let (_keep_a, plain) = write(&body);
    let (_keep_b, commented) = write(&format!("// note\n{body}"));
    let run = |p: &PathBuf| {
        let out = domainforge(&[
            "envelope",
            "--emit",
            "cep",
            "--world-name",
            "t",
            &p.to_string_lossy(),
        ]);
        serde_json::from_slice::<Value>(&out.stdout).unwrap()
    };
    let (a, b) = (run(&plain), run(&commented));
    assert_ne!(a["scope"]["world_ref"], b["scope"]["world_ref"]);
    assert_eq!(
        a["integrity"]["semantic_hash"],
        b["integrity"]["semantic_hash"]
    );
}

#[test]
fn world_flags_are_validated() {
    let entry = fixture(BASIC).to_string_lossy().to_string();
    let bad_name = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        "--world-name",
        "Bad_Name",
        &entry,
    ]);
    assert_eq!(bad_name.status.code(), Some(2));
    let alias_without_name = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        "--world-alias",
        "world:x",
        &entry,
    ]);
    assert_eq!(alias_without_name.status.code(), Some(2));
    let pinned_as_alias = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        "--world-name",
        "t",
        "--world-alias",
        &format!("world:t@sha256:{}", "a".repeat(64)),
        &entry,
    ]);
    assert_eq!(pinned_as_alias.status.code(), Some(2));
    let blank_label = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        "--world-name",
        "t",
        "--world-label",
        "  ",
        &entry,
    ]);
    assert_eq!(blank_label.status.code(), Some(2));
}

#[test]
fn world_binding_applies_to_both_emit_mode() {
    let entry = fixture(BASIC).to_string_lossy().to_string();
    let out = domainforge(&["envelope", "--emit", "both", "--world-name", "t", &entry]);
    assert_eq!(out.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["cep_envelope"]["scope"]["world_ref"].is_string());
}
