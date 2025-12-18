use sea_core::parser::{parse, parse_to_graph, AstNode};
use sea_core::SemanticVersion;

#[test]
fn test_parse_entity_with_version_and_annotations() {
    let source = r#"
        Entity "Vendor" v2.1.0
          @replaces "Vendor" v2.0.0
          @changes ["added credit_limit field", "removed legacy_id"]
          in procurement
    "#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);

    match &ast.declarations[0].node {
        AstNode::Entity {
            name,
            version,
            annotations,
            ..
        } => {
            assert_eq!(name, "Vendor");
            assert_eq!(version.as_ref().unwrap(), "2.1.0");
            assert!(annotations.contains_key("replaces"));
            assert!(annotations.contains_key("changes"));
        }
        _ => panic!("Expected Entity declaration"),
    }
}

#[test]
fn test_parse_concept_change() {
    let source = r#"
        ConceptChange "Vendor_v2_migration"
          @from_version v2.0.0
          @to_version v2.1.0
          @migration_policy mandatory
          @breaking_change true
    "#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);

    match &ast.declarations[0].node {
        AstNode::ConceptChange {
            name,
            from_version,
            to_version,
            migration_policy,
            breaking_change,
            ..
        } => {
            assert_eq!(name, "Vendor_v2_migration");
            assert_eq!(from_version, "2.0.0");
            assert_eq!(to_version, "2.1.0");
            assert_eq!(migration_policy, "mandatory");
            assert!(*breaking_change);
        }
        _ => panic!("Expected ConceptChange declaration"),
    }
}

#[test]
fn test_entity_version_in_graph() {
    let source = r#"
        Entity "Vendor" v2.1.0
          @replaces "Vendor" v2.0.0
          @changes ["added credit_limit field"]
          in procurement
    "#;
    let graph = parse_to_graph(source).unwrap();

    let entities = graph.all_entities();
    assert_eq!(entities.len(), 1);

    let entity = entities[0];
    assert_eq!(entity.name(), "Vendor");
    assert_eq!(entity.version(), Some(&SemanticVersion::new(2, 1, 0)));
    assert_eq!(entity.replaces(), Some("Vendor v2.0.0"));
    assert_eq!(entity.changes().len(), 1);
    assert_eq!(entity.changes()[0], "added credit_limit field");
}

#[test]
fn test_concept_change_in_graph() {
    let source = r#"
        ConceptChange "Vendor_v2_migration"
          @from_version v2.0.0
          @to_version v2.1.0
          @migration_policy mandatory
          @breaking_change true
    "#;
    let graph = parse_to_graph(source).unwrap();

    let changes = graph.all_concept_changes();
    assert_eq!(changes.len(), 1);

    let change = changes[0];
    assert_eq!(change.name(), "Vendor_v2_migration");
    assert_eq!(change.from_version(), "2.0.0");
    assert_eq!(change.to_version(), "2.1.0");
    assert_eq!(change.migration_policy(), "mandatory");
    assert!(change.is_breaking_change());
}

#[test]
fn test_multiple_entity_versions() {
    let source = r#"
        Entity "Vendor" v2.0.0 in procurement
        Entity "VendorV2" v2.1.0
          @replaces "Vendor" v2.0.0
          @changes ["added credit_limit field"]
          in procurement
    "#;
    let graph = parse_to_graph(source).unwrap();

    let entities = graph.all_entities();
    assert_eq!(entities.len(), 2);
}

#[test]
fn test_entity_without_version() {
    let source = r#"
        Entity "Vendor" in procurement
    "#;
    let graph = parse_to_graph(source).unwrap();

    let entities = graph.all_entities();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].version(), None);
}

#[test]
fn test_invalid_version_format() {
    let source = r#"
        Entity "Vendor" v2.1 in procurement
    "#;
    // This should fail parsing because version must be semver format
    let result = parse(source);
    assert!(result.is_err());
}

#[test]
fn test_concept_change_non_breaking() {
    let source = r#"
        ConceptChange "Vendor_v2_1_migration"
          @from_version v2.1.0
          @to_version v2.1.1
          @migration_policy optional
          @breaking_change false
    "#;
    let graph = parse_to_graph(source).unwrap();

    let changes = graph.all_concept_changes();
    assert_eq!(changes.len(), 1);
    assert!(!changes[0].is_breaking_change());
    assert_eq!(changes[0].migration_policy(), "optional");
}
