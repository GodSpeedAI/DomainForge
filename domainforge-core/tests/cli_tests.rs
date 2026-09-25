#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

/// Helper function to test help output for a command
fn run_help_test(args: &[&str], expected: &str) {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.args(args)
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn test_help() {
    run_help_test(&["--help"], "SEA DSL CLI");
}

#[test]
fn test_validate_help() {
    run_help_test(&["validate", "--help"], "Validate");
}

#[test]
fn test_import_help() {
    run_help_test(&["import", "--help"], "Import");
}

#[test]
fn test_project_help() {
    run_help_test(&["project", "--help"], "Project");
}

#[test]
fn test_validate_basic() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Entity \"Test\" in domain").unwrap();
    file.flush().unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("validate")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Validation succeeded"));
}

#[test]
fn test_format_check() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Entity \"Test\" in domain").unwrap();
    file.flush().unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("format")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Entity \"Test\" in domain"));
}

/// The human parse summary must label every count honestly: `Policies:` is the
/// policy count (not patterns), `Patterns:` is separate, and resource vs
/// entity instances are distinguished.
#[test]
fn test_parse_human_summary_counts() {
    let model_dir = tempdir().unwrap();
    let model_path = model_dir.path().join("model.sea");
    std::fs::write(
        &model_path,
        "@namespace \"summary\"\n\
         Entity \"Warehouse\"\n\
         instance warehouse_1 of \"Warehouse\" { code: \"WH\" }\n\
         Pattern \"Code\" matches \"^[A-Z]+$\"\n\
         Policy stock_positive per Constraint Obligation priority 3 as: 1 > 0\n",
    )
    .unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("parse")
        .arg(&model_path)
        .arg("--format")
        .arg("human")
        .assert()
        .success()
        .stdout(predicate::str::contains("Entities: 1"))
        .stdout(predicate::str::contains("Resource instances: 0"))
        .stdout(predicate::str::contains("Entity instances: 1"))
        .stdout(predicate::str::contains("Policies: 1"))
        .stdout(predicate::str::contains("Patterns: 1"));
}

/// `fmt` must be round-trip safe on Mapping/Projection contracts: contract keys
/// are grammar identifiers, so quoting them breaks re-parsing (OBSERVED_DEBT D10).
#[test]
fn test_fmt_mapping_projection_round_trip() {
    let model_dir = tempdir().unwrap();
    let model_path = model_dir.path().join("model.sea");
    std::fs::write(
        &model_path,
        "@namespace \"contracts\"\n\
         Entity \"Warehouse\"\n\
         Mapping \"warehouse_calm\" for calm { Entity \"Warehouse\" -> component { name: \"warehouse\" } }\n\
         Projection \"warehouse_kg\" for kg { Entity \"Warehouse\" { label: \"Warehouse\" } }\n",
    )
    .unwrap();
    let formatted_path = model_dir.path().join("formatted.sea");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("fmt")
        .arg(&model_path)
        .arg("--out")
        .arg(&formatted_path)
        .assert()
        .success();

    let formatted = std::fs::read_to_string(&formatted_path).unwrap();
    assert!(
        formatted.contains("name: \"warehouse\""),
        "mapping keys stay unquoted:\n{formatted}"
    );
    assert!(
        !formatted.contains("\"name\": \"warehouse\""),
        "mapping keys are not JSON-quoted:\n{formatted}"
    );

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("validate")
        .arg(&formatted_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Validation succeeded"));
}

/// A valid Cell model must project via the CLI without going through the
/// generic Graph builder (Cell declarations are single-consumer and not part
/// of the Graph vocabulary — see ADR-012). This guards the early
/// `ProjectFormat::Cell` dispatch in `cli::project::run`.
#[test]
fn test_project_cell_format_via_cli() {
    let model_dir = tempdir().unwrap();
    let model_path = model_dir.path().join("model.sea");
    std::fs::write(
        &model_path,
        "@namespace \"godspeed.cells.cli\"\n\
         Cell \"CliAgent\"\n    @profile \"python-agent-v1\"\n",
    )
    .unwrap();
    let out_dir = tempdir().unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("cell")
        .arg("--created-at")
        .arg("2026-07-11T00:00:00Z")
        .arg(&model_path)
        .arg(out_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Projected Cell environment"));
    // The IR + lock are always written in full.
    assert!(out_dir.path().join("cell.lock").exists());
    assert!(out_dir.path().join("semantic/cell-ir.json").exists());
}
