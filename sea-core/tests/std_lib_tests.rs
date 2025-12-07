use sea_core::parser::{parse_to_graph_with_options, ParseOptions};
use sea_core::registry::NamespaceRegistry;
use std::path::PathBuf;

#[test]
fn test_import_std_core() {
    let source = r#"
@namespace "app"
import { System } from "std:core"

Entity "MyApp" @replaces "System"
"#;

    // We need a registry for module resolution
    let registry = NamespaceRegistry::new_empty(PathBuf::from("."));
    let options = ParseOptions {
        default_namespace: Some("app".to_string()),
        namespace_registry: Some(registry),
        entry_path: Some(PathBuf::from("main.sea")),
    };

    // Note: This test might fail if it tries to resolve "main.sea" on disk.
    // But parse_to_graph_with_options calls validate_entry which parses the source string provided.
    // However, validate_entry calls visit, which might try to canonicalize entry_path.
    // Let's see.

    let result = parse_to_graph_with_options(source, &options);

    // If it fails due to file not found (main.sea), we might need to mock the file or use a real file.
    // But let's try.
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok());
}
