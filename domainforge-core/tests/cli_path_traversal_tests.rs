#![cfg(feature = "cli")]

use domainforge_core::projection::protobuf::validate_proto_package_namespace;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

fn write_input(dir: &TempDir, content: &str) -> PathBuf {
    let path = dir.path().join("input.sea");
    fs::write(&path, content).unwrap();
    path
}

fn run_project(input: &PathBuf, output: &PathBuf, extra_args: &[&str]) -> Result<(), String> {
    let mut cmd = assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("protobuf")
        .arg("--multi-file")
        .arg("--package")
        .arg("sea.generated")
        .args(extra_args)
        .arg(input)
        .arg(output);
    let output = cmd.output().unwrap();
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_namespace_rejects_absolute_path() {
    let result = validate_proto_package_namespace("/tmp/escape");
    assert!(result.is_err(), "Expected rejection for '/tmp/escape'");
    assert!(result.unwrap_err().contains("path separators"));
}

#[test]
fn test_namespace_rejects_traversal() {
    let result = validate_proto_package_namespace("../escape");
    assert!(result.is_err(), "Expected rejection for '../escape'");
    assert!(result.unwrap_err().contains("traversal"));
}

#[test]
fn test_namespace_rejects_slash() {
    let result = validate_proto_package_namespace("a/b");
    assert!(result.is_err(), "Expected rejection for 'a/b'");
    assert!(result.unwrap_err().contains("path separators"));
}

#[test]
fn test_namespace_rejects_backslash() {
    let result = validate_proto_package_namespace("a\\b");
    assert!(result.is_err(), "Expected rejection for 'a\\b'");
    assert!(result.unwrap_err().contains("path separators"));
}

#[test]
fn test_namespace_rejects_leading_dot() {
    let result = validate_proto_package_namespace(".hidden");
    assert!(result.is_err(), "Expected rejection for '.hidden'");
}

#[test]
fn test_namespace_rejects_double_dot() {
    let result = validate_proto_package_namespace("foo..bar");
    assert!(result.is_err(), "Expected rejection for 'foo..bar'");
}

#[test]
fn test_namespace_accepts_simple() {
    assert!(validate_proto_package_namespace("logistics").is_ok());
}

#[test]
fn test_namespace_accepts_dotted() {
    assert!(validate_proto_package_namespace("com.example.api").is_ok());
}

#[test]
fn test_namespace_accepts_underscore() {
    assert!(validate_proto_package_namespace("my_pkg").is_ok());
}

#[test]
fn test_namespace_accepts_dotted_with_underscore() {
    assert!(validate_proto_package_namespace("com.example_my.api").is_ok());
}

#[test]
fn test_namespace_accepts_empty() {
    assert!(validate_proto_package_namespace("").is_ok());
}

#[test]
fn test_namespace_rejects_digit_start() {
    let result = validate_proto_package_namespace("9invalid");
    assert!(result.is_err());
}

#[test]
fn test_namespace_rejects_special_chars() {
    let result = validate_proto_package_namespace("ns!@#");
    assert!(result.is_err());
}

#[test]
fn test_multi_file_writes_inside_output_dir() {
    let dir = tempdir().unwrap();
    let input = write_input(&dir, "Entity \"TestEntity\" in domain");
    let output = dir.path().join("out_proto");
    fs::create_dir_all(&output).unwrap();

    let result = run_project(&input, &output, &[]);
    assert!(
        result.is_ok(),
        "Multi-file projection should succeed: {:?}",
        result
    );
    assert!(fs::read_dir(&output).unwrap().count() > 0);
}
