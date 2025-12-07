#![cfg(not(target_arch = "wasm32"))]
use sea_core::parser::{parse_to_graph_with_options, ParseOptions};
use sea_core::registry::NamespaceRegistry;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_import_std_core() {
    let source = r#"
@namespace "app"
import { System } from "std:core"

Entity "MyApp" @replaces "System"
"#;

    // We need a registry for module resolution
    let registry = NamespaceRegistry::new_empty(PathBuf::from("."))
        .expect("Failed to create namespace registry");
    let mut entry_file = NamedTempFile::new().expect("Failed to create entry file");
    entry_file
        .write_all(source.as_bytes())
        .expect("Failed to write entry file");
    let entry_path = entry_file
        .path()
        .canonicalize()
        .expect("Failed to canonicalize entry path");
    let options = ParseOptions {
        default_namespace: Some("app".to_string()),
        namespace_registry: Some(registry),
        entry_path: Some(entry_path),
        ..Default::default()
    };

    let result = parse_to_graph_with_options(source, &options);
    assert!(result.is_ok());
}
