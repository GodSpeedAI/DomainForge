use sea_core::parser::parse;

#[test]
fn test_parse_instance_minimal() {
    let source = r#"
Instance vendor_123 of "Vendor"
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let _graph = result.unwrap();
    // For now, just verify it parses without error
    // We'll add graph integration in the next step
}

#[test]
fn test_parse_instance_with_fields() {
    let source = r#"
Instance vendor_123 of "Vendor" {
    name: "Acme Corp",
    credit_limit: 50000,
    quantity: 10
}
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_instance_with_quantity_field() {
    let source = r#"
Instance vendor_123 of "Vendor" {
    name: "Acme Corp",
    credit_limit: 50000 "USD"
}
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_multiple_instances() {
    let source = r#"
Instance vendor_1 of "Vendor" {
    name: "Acme Corp"
}

Instance vendor_2 of "Vendor" {
    name: "Beta Inc"
}
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
