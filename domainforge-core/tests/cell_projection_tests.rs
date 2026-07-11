//! Integration tests for the cell-environment projection driven by real
//! fixture files under `fixtures/cell_env/` (the flagship end-to-end proof
//! is `repair-agent`). Unit-level coverage (renderer content, IR merge
//! rules) lives in `domainforge-core/src/projection/cell/*` `#[cfg(test)]`
//! modules; these tests exercise the fixture -> library boundary the CLI
//! actually uses.

use domainforge_core::formatter::{format as format_source, FormatConfig};
use domainforge_core::parser::{parse, ParseResult};
use domainforge_core::projection::cell::overrides::{self, CellOverrides};
use domainforge_core::projection::cell::{project_cell_in_memory, CellComponents};
use domainforge_core::projection::sink::ArtifactSink;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const FIXED_TS: &str = "2026-07-11T00:00:00+00:00";

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/cell_env")
        .join(rel)
}

fn read(rel: &str) -> String {
    fs::read_to_string(fixture(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

/// Project a fixture the way the CLI does: with its real on-disk directory
/// as `source_dir`, so `DependencySet` lockfile hashing actually resolves
/// (in-memory/binding callers pass `None` because they have no filesystem).
fn project_fixture(
    model_rel: &str,
    overrides_source: Option<&str>,
) -> Result<BTreeMap<String, String>, String> {
    let source = read(model_rel);
    let ast: ParseResult<_> = parse(&source);
    let ast = ast.map_err(|e| format!("parse failed: {e}"))?;
    let overrides: Option<CellOverrides> = overrides_source.map(overrides::parse).transpose()?;
    let source_dir = fixture(model_rel);
    let source_dir = source_dir.parent().unwrap();
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    domainforge_core::projection::cell::emit(
        &ast,
        &source,
        model_rel,
        Some(source_dir),
        Some(FIXED_TS.to_string()),
        overrides.as_ref(),
        overrides_source,
        &CellComponents::default(),
        &mut sink,
    )?;
    Ok(map)
}

#[test]
fn repair_agent_flagship_end_to_end_proof() {
    let overrides_source = read("repair-agent/domainforge.cell.toml");

    // 1. DomainForge parses and validates the SEA file.
    let files = project_fixture("repair-agent/model.sea", Some(&overrides_source))
        .expect("flagship fixture must project successfully");

    // 2. Cell IR contains every expected category (spot-checked via its
    // serialized form; full structural coverage is in ir::CellIr unit
    // tests).
    let ir: serde_json::Value = serde_json::from_str(&files["semantic/cell-ir.json"]).unwrap();
    for key in [
        "system_dependencies",
        "runtimes",
        "tools",
        "dependency_sets",
        "mounts",
        "endpoints",
        "network_flows",
        "credentials",
        "policies",
        "evidence_probes",
    ] {
        assert!(
            !ir[key].as_array().unwrap().is_empty(),
            "expected non-empty CellIr.{key}"
        );
    }

    // 3. Devbox output contains the expected system dependencies.
    let devbox: serde_json::Value = serde_json::from_str(&files["devbox/devbox.json"]).unwrap();
    let packages: Vec<&str> = devbox["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(packages
        .iter()
        .any(|p| p.starts_with("gitFull") || p.starts_with("git@")));
    // `ca-certificates` is the cross-ecosystem SEA name; the devbox
    // renderer maps it to its real nixpkgs attribute `cacert` (verified
    // against a real `devbox generate` run — nixpkgs has no
    // `ca-certificates` attribute).
    assert!(packages.iter().any(|p| p.starts_with("cacert")));

    // 4. Mise output contains the expected runtime and tools.
    let mise = &files["mise/mise.toml"];
    assert!(mise.contains("python ="));
    assert!(mise.contains("uv ="));
    assert!(mise.contains("just ="));

    // 5. The dependency-set output references pyproject.toml + uv.lock.
    let deps: serde_json::Value =
        serde_json::from_str(&files["dependencies/dependency-sets.json"]).unwrap();
    let set = &deps["dependency_sets"]["python-application"];
    assert_eq!(set["manifest"], "pyproject.toml");
    assert_eq!(set["lockfile"], "uv.lock");
    assert!(set["lockfile_sha256"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    // 6. CubeSandbox/sandbox output is deny-by-default.
    let template: serde_json::Value =
        serde_json::from_str(&files["sandbox/template.json"]).unwrap();
    assert_eq!(template["network_default"], "deny");

    // 7. Only the declared endpoint appears in allowed egress.
    let network: serde_json::Value = serde_json::from_str(&files["sandbox/network.json"]).unwrap();
    let endpoints = network["endpoints"].as_array().unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0]["name"], "package-mirror");

    // 8. No credential value appears in generated output.
    for (path, content) in &files {
        assert!(
            !content.to_lowercase().contains("token-value")
                && !content.contains("package-mirror-token-secret"),
            "unexpected secret material in {path}"
        );
    }

    // 9. SEA-Forge contracts govern dependency mutation and credential issuance.
    let authority: serde_json::Value =
        serde_json::from_str(&files["authority/dependency-mutation-policy.json"]).unwrap();
    assert!(!authority["dependency_mutations"][0]["manifest_change"]
        .as_str()
        .unwrap()
        .is_empty());
    let creds: serde_json::Value =
        serde_json::from_str(&files["authority/credential-contracts.json"]).unwrap();
    assert_eq!(creds["credentials"][0]["scope"], "package-mirror:read");

    // 10. Evidence contracts retain SEA semantic references.
    let evidence: serde_json::Value =
        serde_json::from_str(&files["evidence/environment-proof-contract.json"]).unwrap();
    assert!(!evidence["probes"].as_array().unwrap().is_empty());

    // 11. cell.lock hashes every projection.
    let lock: serde_json::Value = serde_json::from_str(&files["cell.lock"]).unwrap();
    assert!(lock["files"]
        .as_object()
        .unwrap()
        .contains_key("devbox/devbox.json"));
    assert!(lock["files"]
        .as_object()
        .unwrap()
        .contains_key("mise/mise.toml"));
    assert!(lock["overrides"]["present"].as_bool().unwrap());

    // 12. A repeated generation is byte-identical.
    let files2 = project_fixture("repair-agent/model.sea", Some(&overrides_source)).unwrap();
    assert_eq!(files, files2);

    // 13. An undeclared endpoint is denied by validation.
    let bad = read("invalid/02_unknown_flow_reference.sea");
    let err = project_cell_in_memory(&bad, "bad.sea", Some(FIXED_TS.to_string()), None, None)
        .unwrap_err();
    assert!(err.starts_with("CELL020"), "{err}");

    // 14. A lockfile-changing (version-weakening) override is rejected
    // unless explicitly authorized.
    let weak_src = read("invalid/06_version_weakening.sea");
    let weak_overrides = read("invalid/06_version_weakening.toml");
    let err = project_cell_in_memory(
        &weak_src,
        "weak.sea",
        Some(FIXED_TS.to_string()),
        Some(&weak_overrides),
        None,
    )
    .unwrap_err();
    assert!(err.starts_with("CELL011"), "{err}");

    // 15. Existing domain-code projections remain unchanged: proven by the
    // full `cargo test` run staying green (this crate's other projection
    // suites are untouched by this feature — see the completion report).
}

#[test]
fn invalid_fixtures_fail_with_documented_codes() {
    let cases: &[(&str, Option<&str>, &str)] = &[
        ("invalid/01_unknown_profile.sea", None, "CELL001"),
        ("invalid/02_unknown_flow_reference.sea", None, "CELL020"),
        ("invalid/03_duplicate_declaration.sea", None, "CELL005"),
        ("invalid/04_wildcard_endpoint.sea", None, "CELL021"),
        (
            "invalid/05_resource_increase.sea",
            Some("invalid/05_resource_increase.toml"),
            "CELL012",
        ),
        (
            "invalid/06_version_weakening.sea",
            Some("invalid/06_version_weakening.toml"),
            "CELL011",
        ),
        (
            "invalid/07_expired_unsafe.sea",
            Some("invalid/07_expired_unsafe.toml"),
            "CELL016",
        ),
        (
            "invalid/08_unknown_override_key.sea",
            Some("invalid/08_unknown_override_key.toml"),
            "CELL010",
        ),
    ];

    for (model, overrides_rel, expected_code) in cases {
        let source = read(model);
        let overrides_source = overrides_rel.map(read);
        let err = project_cell_in_memory(
            &source,
            model,
            Some(FIXED_TS.to_string()),
            overrides_source.as_deref(),
            None,
        )
        .unwrap_err();
        assert!(
            err.starts_with(expected_code),
            "{model}: expected {expected_code}, got: {err}"
        );
    }
}

#[test]
fn overrides_schema_rejects_unknown_keys_before_reaching_ir() {
    let source = read("invalid/08_unknown_override_key.sea");
    let overrides_source = read("invalid/08_unknown_override_key.toml");
    let err = overrides::parse(&overrides_source).unwrap_err();
    assert!(err.starts_with("CELL010"), "{err}");
    let _ = source; // model isn't reached; the override document fails to parse first.
}

#[test]
fn duplicate_cell_annotations_are_rejected() {
    // Duplicate normalized annotation keys (including case-insensitive
    // collisions) are an authoring error — the last-wins HashMap::insert
    // semantics would otherwise silently drop the earlier value.
    let dup_profile =
        "Cell \"X\"\n    @profile \"python-agent-v1\"\n    @profile \"rust-agent-v1\"\n";
    let err = project_cell_in_memory(
        dup_profile,
        "dup.sea",
        Some(FIXED_TS.to_string()),
        None,
        None,
    )
    .unwrap_err();
    assert!(
        err.contains("Duplicate annotation '@profile'"),
        "expected duplicate-annotation error, got: {err}"
    );

    let dup_case = "Cell \"Y\"\n    @profile \"python-agent-v1\"\n    @Profile \"rust-agent-v1\"\n";
    let err2 = project_cell_in_memory(
        dup_case,
        "dupcase.sea",
        Some(FIXED_TS.to_string()),
        None,
        None,
    )
    .unwrap_err();
    assert!(
        err2.contains("Duplicate annotation '@profile'"),
        "expected case-insensitive duplicate error, got: {err2}"
    );
}

#[test]
fn basic_profile_fixtures_project_successfully() {
    for rel in [
        "basic-typescript/model.sea",
        "basic-rust/model.sea",
        "polyglot/model.sea",
    ] {
        let files =
            project_fixture(rel, None).unwrap_or_else(|e| panic!("{rel} should project: {e}"));
        // Lockfile hashing must resolve against the real fixture dir, not be
        // silently nulled by the in-memory `.` sentinel.
        let deps: serde_json::Value =
            serde_json::from_str(&files["dependencies/dependency-sets.json"]).unwrap();
        for (_, set) in deps["dependency_sets"].as_object().unwrap() {
            let sha = set["lockfile_sha256"]
                .as_str()
                .unwrap_or_else(|| panic!("lockfile_sha256 missing for {rel}"));
            assert!(
                sha.starts_with("sha256:"),
                "{rel}: expected real lockfile hash, got '{sha}' (null = in-memory sentinel leak)"
            );
        }
    }
}

#[test]
fn format_stable_fixture_round_trips_and_is_idempotent() {
    let source = read("format_stable.sea");
    let ast1 = parse(&source).expect("fixture parses");

    let formatted = format_source(&source, FormatConfig::default()).expect("formats");
    let ast2 = parse(&formatted).expect("formatted output reparses");
    assert_eq!(ast1.declarations.len(), ast2.declarations.len());

    let formatted_again =
        format_source(&formatted, FormatConfig::default()).expect("formats again");
    assert_eq!(formatted, formatted_again, "formatter must be idempotent");
}
