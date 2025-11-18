use sea_core::registry::NamespaceRegistry;
use std::fs;
use tempfile::tempdir;

#[test]
fn registry_loads_and_matches_patterns() {
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
