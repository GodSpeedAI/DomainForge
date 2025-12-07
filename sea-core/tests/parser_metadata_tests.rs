//! Tests for parsing file-level metadata annotations.
use sea_core::parser::ast::FileMetadata;
use sea_core::parser::{parse_source, parse_to_graph};

#[test]
fn test_parse_full_header() {
    let source = r#"
        @namespace "com.acme.finance"
        @version "2.2.0"
        @owner "team-payments"

        Entity "Vendor"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(
        ast.metadata,
        FileMetadata {
            namespace: Some("com.acme.finance".to_string()),
            version: Some("2.2.0".to_string()),
            owner: Some("team-payments".to_string()),
            profile: None,
            imports: Vec::new(),
        }
    );
}

#[test]
fn test_parse_partial_header() {
    let source = r#"
        @namespace "com.acme.logistics"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(
        ast.metadata,
        FileMetadata {
            namespace: Some("com.acme.logistics".to_string()),
            version: None,
            owner: None,
            profile: None,
            imports: Vec::new(),
        }
    );
}

#[test]
fn test_no_header() {
    let source = r#"
        Entity "Product"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.metadata, FileMetadata::default());
}

#[test]
fn test_unknown_annotation() {
    let source = r#"
        @priority "high"
        Entity "Task"
    "#;
    let result = parse_source(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("expected annotation_name"));
}

#[test]
fn test_policy_metadata_applied_to_graph() {
    let source = r#"
        @namespace "com.acme.finance"
        @version "1.2.3"

        Policy ap_cap per Constraint Obligation priority 5
          @rationale "Limit vendor exposure"
          @tags ["sox", "payments"]
        as:
          true
    "#;

    let graph = parse_to_graph(source).unwrap();
    assert_eq!(graph.policy_count(), 1);

    let policy = graph.all_policies().into_iter().next().unwrap();
    assert_eq!(policy.name, "ap_cap");
    assert_eq!(policy.namespace, "com.acme.finance");
    assert_eq!(policy.version.to_string(), "1.2.3");
    assert_eq!(policy.kind, sea_core::policy::PolicyKind::Constraint);
    assert_eq!(
        policy.modality,
        sea_core::policy::PolicyModality::Obligation
    );
    assert_eq!(policy.priority, 5);
    assert_eq!(policy.rationale.as_deref(), Some("Limit vendor exposure"));
    assert_eq!(policy.tags, vec!["sox", "payments"]);
}
