use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn validate_directory_with_registry_succeeds() {
    let temp = tempdir().unwrap();
    let base = temp.path();

    let logistics_dir = base.join("domains/logistics");
    fs::create_dir_all(&logistics_dir).unwrap();
    let file_path = logistics_dir.join("warehouse.sea");
    fs::write(&file_path, "Entity \"Warehouse\"").unwrap();

    let registry_path = base.join(".sea-registry.toml");
    fs::write(
        &registry_path,
        r#"
version = 1
default_namespace = "default"

[[namespaces]]
namespace = "logistics"
patterns = ["domains/logistics/**/*.sea"]
"#,
    )
    .unwrap();

    let bin = assert_cmd::cargo::cargo_bin!("sea");
    let mut cmd = Command::new(bin);
    cmd.arg("validate").arg(base);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validation succeeded"));
}

#[test]
fn validate_directory_no_registry_fails() {
    let temp = tempdir().unwrap();
    let base = temp.path();

    // Create a file in a sub directory with no registry at any parent
    let logistics_dir = base.join("domains/logistics");
    fs::create_dir_all(&logistics_dir).unwrap();
    let file_path = logistics_dir.join("warehouse.sea");
    fs::write(&file_path, "Entity \"Warehouse\"").unwrap();

    let bin = assert_cmd::cargo::cargo_bin!("sea");
    let mut cmd = Command::new(bin);
    cmd.arg("validate").arg(base);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No .sea-registry.toml found"));
}

#[test]
fn registry_list_and_resolve() {
    let temp = tempdir().unwrap();
    let base = temp.path();

    let logistics_dir = base.join("domains/logistics");
    fs::create_dir_all(&logistics_dir).unwrap();
    let file_path = logistics_dir.join("warehouse.sea");
    fs::write(&file_path, "Entity \"Warehouse\"").unwrap();

    let registry_path = base.join(".sea-registry.toml");
    fs::write(
        &registry_path,
        r#"
version = 1
default_namespace = "default"

[[namespaces]]
namespace = "logistics"
patterns = ["domains/logistics/**/*.sea"]
"#,
    )
    .unwrap();

    let bin = assert_cmd::cargo::cargo_bin!("sea");
    // registry list
    let mut cmd = Command::new(bin);
    cmd.arg("registry").arg("list").arg(base);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("logistics"));

    // registry resolve
    let mut cmd2 = Command::new(bin);
    cmd2.arg("registry").arg("resolve").arg(file_path.clone());
    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("logistics"));
}
