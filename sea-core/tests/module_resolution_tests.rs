#![cfg(not(target_arch = "wasm32"))]
use sea_core::module::resolver::ModuleResolver;
use sea_core::registry::NamespaceRegistry;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn write_file(path: &Path, contents: &str) {
    let mut file = fs::File::create(path).expect("unable to create file");
    file.write_all(contents.as_bytes())
        .expect("unable to write file");
}

fn create_registry(dir: &Path) -> std::path::PathBuf {
    let registry_path = dir.join(".sea-registry.toml");
    let registry = r#"
version = 1
default_namespace = "default"

[[namespaces]]
namespace = "acme.order"
patterns = ["order.sea"]

[[namespaces]]
namespace = "acme.common"
patterns = ["common.sea"]
"#;
    write_file(&registry_path, registry);
    registry_path
}

#[test]
fn resolves_dependencies_and_exports() {
    let dir = tempdir().expect("tempdir");
    let registry_path = create_registry(dir.path());

    let order_file = dir.path().join("order.sea");
    write_file(
        &order_file,
        r#"
@namespace "acme.order"
import { Customer } from "acme.common"
export Entity "Order"
"#,
    );

    let common_file = dir.path().join("common.sea");
    write_file(
        &common_file,
        r#"
@namespace "acme.common"
export Entity "Customer"
"#,
    );

    let registry = NamespaceRegistry::from_file(&registry_path).expect("registry");
    let mut resolver = ModuleResolver::new(&registry).expect("resolver");

    let order_content = fs::read_to_string(&order_file).unwrap();
    let order_ast = resolver
        .validate_entry(&order_file, &order_content)
        .expect("entry should validate");

    assert!(resolver
        .validate_dependencies(&order_file, &order_ast)
        .is_ok());
}

#[test]
fn detects_dependency_cycles() {
    let dir = tempdir().expect("tempdir");
    let registry_path = create_registry(dir.path());

    let order_file = dir.path().join("order.sea");
    write_file(
        &order_file,
        r#"
@namespace "acme.order"
import * as common from "acme.common"
export Entity "Order"
"#,
    );

    let common_file = dir.path().join("common.sea");
    write_file(
        &common_file,
        r#"
@namespace "acme.common"
import * as order from "acme.order"
export Entity "Customer"
"#,
    );

    let registry = NamespaceRegistry::from_file(&registry_path).expect("registry");
    let mut resolver = ModuleResolver::new(&registry).expect("resolver");
    let result = resolver.validate_entry(&order_file, &fs::read_to_string(&order_file).unwrap());

    assert!(result.is_err(), "cycle should be detected");
}
