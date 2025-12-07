#![cfg(not(target_arch = "wasm32"))]
use sea_core::registry::NamespaceRegistry;
use std::fs;
use tempfile::tempdir;

#[test]
fn registry_loads_and_matches_patterns() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

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

    let registry = NamespaceRegistry::from_file(&registry_path).unwrap();
    let files = registry.resolve_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, file_path);
    assert_eq!(files[0].namespace, "logistics");

    let matched = registry.namespace_for(&file_path).unwrap();
    assert_eq!(matched, "logistics");

    let fallback_path = base.join("domains/misc/other.sea");
    fs::create_dir_all(fallback_path.parent().unwrap()).unwrap();
    fs::write(&fallback_path, "Entity \"Other\"").unwrap();

    let fallback = registry.namespace_for(&fallback_path).unwrap();
    assert_eq!(fallback, "default");
}

#[test]
fn namespace_precedence_longest_prefix() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

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
namespace = "short"
patterns = ["domains/**/*.sea"]

[[namespaces]]
namespace = "long"
patterns = ["domains/logistics/**/*.sea"]
"#,
    )
    .unwrap();

    let registry = NamespaceRegistry::from_file(&registry_path).unwrap();
    // The best prefix is 'domains/logistics/' which selects 'long'
    let ns = registry.namespace_for(&file_path).unwrap();
    assert_eq!(ns, "long");
}

#[test]
fn namespace_precedence_tie_break_alpha() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

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
patterns = ["domains/*/warehouse.sea"]

[[namespaces]]
namespace = "finance"
patterns = ["domains/*/warehouse.sea"]
"#,
    )
    .unwrap();

    let registry = NamespaceRegistry::from_file(&registry_path).unwrap();
    // Two identical prefix lengths and patterns; alphabetical tie-breaker selects 'finance'
    let ns = registry.namespace_for(&file_path).unwrap();
    assert_eq!(ns, "finance");
}

#[test]
fn namespace_precedence_error_on_ambiguity() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

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
patterns = ["domains/*/warehouse.sea"]

[[namespaces]]
namespace = "finance"
patterns = ["domains/*/warehouse.sea"]
"#,
    )
    .unwrap();

    let registry = NamespaceRegistry::from_file(&registry_path).unwrap();
    // fail_on_ambiguity should cause an error
    let res = registry.namespace_for_with_options(&file_path, true);
    assert!(res.is_err());
}

#[test]
fn registry_detects_conflict() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

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
namespace = "finance"
patterns = ["domains/**/*.sea"]

[[namespaces]]
namespace = "logistics"
patterns = ["domains/logistics/**/*.sea"]
"#,
    )
    .unwrap();

    let registry = NamespaceRegistry::from_file(&registry_path).unwrap();
    let files = registry.resolve_files().unwrap();
    // With longest literal prefix precedence, the logistics pattern should win
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].namespace, "logistics");
}

#[test]
fn registry_resolve_files_error_on_ambiguity() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

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
patterns = ["domains/*/warehouse.sea"]

[[namespaces]]
namespace = "finance"
patterns = ["domains/*/warehouse.sea"]
"#,
    )
    .unwrap();

    let registry = NamespaceRegistry::from_file(&registry_path).unwrap();
    let res = registry.resolve_files_with_options(true);
    assert!(res.is_err());
}

#[test]
fn discover_finds_parent_registry() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

    let subdir = base.join("a/b/c");
    fs::create_dir_all(&subdir).unwrap();

    let registry_path = base.join(".sea-registry.toml");
    fs::write(
        &registry_path,
        r#"
version = 1
default_namespace = "default"

[[namespaces]]
namespace = "logistics"
patterns = ["a/b/c/**/*.sea"]
"#,
    )
    .unwrap();

    let file_path = subdir.join("file.sea");
    fs::write(&file_path, "Entity \"X\"").unwrap();

    let reg = NamespaceRegistry::discover(&file_path).unwrap();
    assert!(reg.is_some());
    let reg = reg.unwrap();
    let ns = reg.namespace_for(&file_path).unwrap();
    assert_eq!(ns, "logistics");
}
