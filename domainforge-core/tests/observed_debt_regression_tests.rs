//! Regression tests for entries in `.agents/OBSERVED_DEBT.md`:
//! D1 (model @version on Graph), D6 (CALM import via CLI),
//! D-doc-egm-1 (no debug stdout from flow-annotation parsing),
//! D-doc-egm-2 (`domainforge test` must not report success).
#![cfg(feature = "cli")]

use assert_cmd::Command;
use domainforge_core::application::resolve_application_graph;
use domainforge_core::parser::parse_to_graph;
use serde_json::json;
use std::fs::write;
use tempfile::tempdir;

const VERSIONED_MODEL: &str = r#"
    @namespace "com.acme.debt"
    @version "2.2.0"

    Entity "Vendor"
    Entity "Buyer"
"#;

const UNVERSIONED_MODEL: &str = r#"
    Entity "Vendor"
"#;

// D1: the declared @version reaches the Graph, on the body-bearing parse path too.
#[test]
fn d1_graph_exposes_declared_version() {
    let graph = parse_to_graph(VERSIONED_MODEL).expect("parse versioned model");
    assert_eq!(graph.version(), Some("2.2.0"));
}

#[test]
fn d1_graph_version_is_none_when_undeclared() {
    let graph = parse_to_graph(UNVERSIONED_MODEL).expect("parse unversioned model");
    assert_eq!(graph.version(), None);
}

// D1: on the resolved (multi-module) path, the version comes from the entry module,
// not from an imported module that also declares one.
#[test]
fn d1_resolved_graph_uses_entry_module_version() {
    let sources = json!({
        "main.sea": "@namespace \"app\"\n@version \"3.0.0\"\nimport { Lib } from \"./lib.sea\"\n\nentity \"Main\" {\n    key id: uuid\n}\n",
        "lib.sea": "@namespace \"lib\"\n@version \"1.0.0\"\n\nexport entity \"Lib\" {\n    key id: uuid\n}\n",
    })
    .to_string();
    let graph = resolve_application_graph("main.sea", &sources).expect("resolve multi-module");
    assert_eq!(graph.version(), Some("3.0.0"));
}

// D1: an imported module's version must not leak onto an unversioned entry module.
#[test]
fn d1_resolved_graph_ignores_imported_version() {
    let sources = json!({
        "main.sea": "@namespace \"app\"\nimport { Lib } from \"./lib.sea\"\n\nentity \"Main\" {\n    key id: uuid\n}\n",
        "lib.sea": "@namespace \"lib\"\n@version \"1.0.0\"\n\nexport entity \"Lib\" {\n    key id: uuid\n}\n",
    })
    .to_string();
    let graph = resolve_application_graph("main.sea", &sources).expect("resolve multi-module");
    assert_eq!(graph.version(), None);
}

// D1: the CLI graph JSON carries the version when declared, and omits it otherwise.
#[test]
fn d1_cli_parse_json_carries_version() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("model.sea");
    write(&path, VERSIONED_MODEL).unwrap();

    let output = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["parse", "--format", "json"])
        .arg(&path)
        .output()
        .expect("run parse");
    assert!(output.status.success(), "parse failed: {:?}", output);
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse --format json emits valid JSON");
    assert_eq!(json["version"], "2.2.0");
}

// D6: a CALM document produced by `project --format calm` imports via `import --format calm`.
#[test]
fn d6_cli_calm_round_trip_imports() {
    let dir = tempdir().unwrap();
    let sea = dir.path().join("model.sea");
    write(&sea, VERSIONED_MODEL).unwrap();

    let calm = dir.path().join("model.calm.json");
    let projected = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["project", "--format", "calm"])
        .arg(&sea)
        .arg(&calm)
        .output()
        .expect("run project");
    assert!(
        projected.status.success(),
        "project failed: {:?}",
        projected
    );

    let imported = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["import", "--format", "calm"])
        .arg(&calm)
        .output()
        .expect("run import");
    assert!(imported.status.success(), "import failed: {:?}", imported);
    let stdout = String::from_utf8_lossy(&imported.stdout);
    assert!(stdout.contains("Imported CALM"), "stdout was: {stdout}");
}

// D-doc-egm-1 (guard, not a regression proof): the debug println was removed. On the
// original code this test also passes, because the line was not reachable from any
// input the grammar accepts. It guards that `parse --format json` stdout stays pure
// JSON when flow annotations are present.
#[test]
fn d_doc_egm_1_parse_stdout_stays_pure_json_with_flow_annotations() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("flow.sea");
    write(
        &path,
        r#"
        Entity "A"
        Entity "B"
        Resource "Ship" units
        Flow "Ship" @cqrs "command" from "A" to "B" quantity 1
        "#,
    )
    .unwrap();

    let output = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["parse", "--format", "json"])
        .arg(&path)
        .output()
        .expect("run parse");
    assert!(output.status.success(), "parse failed: {:?}", output);
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("stdout must be pure JSON with no debug lines");
}

// D-doc-egm-2: the `test` stub must fail, not report success while testing nothing.
#[test]
fn d_doc_egm_2_test_subcommand_fails_loudly() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["test", "some_pattern"])
        .output()
        .expect("run test");
    assert!(
        !output.status.success(),
        "`domainforge test` must exit non-zero while unimplemented"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not implemented"), "stderr was: {stderr}");
}
