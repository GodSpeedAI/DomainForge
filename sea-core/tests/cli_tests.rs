use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("sea").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SEA DSL CLI"));
}

#[test]
fn test_validate_help() {
    let mut cmd = Command::cargo_bin("sea").unwrap();
    cmd.arg("validate").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}

#[test]
fn test_import_help() {
    let mut cmd = Command::cargo_bin("sea").unwrap();
    cmd.arg("import").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Import"));
}

#[test]
fn test_project_help() {
    let mut cmd = Command::cargo_bin("sea").unwrap();
    cmd.arg("project").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project"));
}

#[test]
fn test_validate_basic() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Entity \"Test\" in domain").unwrap();
    
    let mut cmd = Command::cargo_bin("sea").unwrap();
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
    
    let mut cmd = Command::cargo_bin("sea").unwrap();
    cmd.arg("format")
        .arg("--check")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Format check passed"));
}
