use sea_core::parser::parse_to_graph;

#[test]
fn test_instance_stored_in_graph() {
    let source = r#"
Entity "Vendor"

Instance vendor_123 of "Vendor" {
    name: "Acme Corp",
    credit_limit: 50000
}
"#;

    let result = parse_to_graph(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let graph = result.unwrap();
    assert_eq!(graph.entity_instance_count(), 1);

    let instance = graph.get_entity_instance("vendor_123");
    assert!(instance.is_some());

    let instance = instance.unwrap();
    assert_eq!(instance.name(), "vendor_123");
    assert_eq!(instance.entity_type(), "Vendor");

    // Check fields
    let name_field = instance.get_field("name");
    assert!(name_field.is_some());
    assert_eq!(name_field.unwrap().as_str(), Some("Acme Corp"));

    let credit_field = instance.get_field("credit_limit");
    assert!(credit_field.is_some());
    assert_eq!(credit_field.unwrap().as_f64(), Some(50000.0));
}

#[test]
fn test_multiple_instances_in_graph() {
    let source = r#"
Entity "Vendor"

Instance vendor_1 of "Vendor" {
    name: "Acme Corp"
}

Instance vendor_2 of "Vendor" {
    name: "Beta Inc"
}
"#;

    let result = parse_to_graph(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let graph = result.unwrap();
    assert_eq!(graph.entity_instance_count(), 2);

    assert!(graph.get_entity_instance("vendor_1").is_some());
    assert!(graph.get_entity_instance("vendor_2").is_some());
}

#[test]
fn test_duplicate_instance_error() {
    let source = r#"
Entity "Vendor"

Instance vendor_1 of "Vendor" {
    name: "Acme Corp"
}

Instance vendor_1 of "Vendor" {
    name: "Beta Inc"
}
"#;

    let result = parse_to_graph(source);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("already exists"));
}

#[test]
fn test_instance_minimal_no_fields() {
    let source = r#"
Entity "Vendor"

Instance vendor_123 of "Vendor"
"#;

    let result = parse_to_graph(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let graph = result.unwrap();
    assert_eq!(graph.entity_instance_count(), 1);

    let instance = graph.get_entity_instance("vendor_123");
    assert!(instance.is_some());
    assert_eq!(instance.unwrap().fields().len(), 0);
}
