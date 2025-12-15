#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper function to test help output for a command
fn run_help_test(args: &[&str], expected: &str) {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("sea"));
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

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("sea"));
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

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("sea"));
    cmd.arg("format")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Entity \"Test\" in domain"));
}
