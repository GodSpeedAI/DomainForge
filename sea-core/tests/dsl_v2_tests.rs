//! Tests for DSL v2 enhancements: roles, relations, instances, flow units, and metadata
use sea_core::parser::ast::AstNode;
use sea_core::parser::{parse_source, parse_to_graph};

#[test]
fn test_parse_role_declaration() {
    let source = r#"
        Role "Payer" for entity "Party"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Role { name, entity } => {
            assert_eq!(name, "Payer");
            assert_eq!(entity, "Party");
        }
        _ => panic!("Expected Role node"),
    }
}

#[test]
fn test_parse_multiple_roles() {
    let source = r#"
        Role "Payer" for entity "Party"
        Role "Payee" for entity "Party"
        Role "Owner" for entity "Asset"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 3);
}

#[test]
fn test_parse_relation_with_all_fields() {
    let source = r#"
        Relation "Payment" {
            subject role "Payer"
            object role "Payee"
            via flow "MoneyTransfer"
        }
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Relation { name, subject_role, object_role, via_flow } => {
            assert_eq!(name, "Payment");
            assert_eq!(subject_role.as_deref(), Some("Payer"));
            assert_eq!(object_role.as_deref(), Some("Payee"));
            assert_eq!(via_flow.as_deref(), Some("MoneyTransfer"));
        }
        _ => panic!("Expected Relation node"),
    }
}

#[test]
fn test_parse_relation_without_flow() {
    let source = r#"
        Relation "Ownership" {
            subject role "Owner"
            object role "Asset"
        }
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Relation { name, subject_role, object_role, via_flow } => {
            assert_eq!(name, "Ownership");
            assert_eq!(subject_role.as_deref(), Some("Owner"));
            assert_eq!(object_role.as_deref(), Some("Asset"));
            assert_eq!(via_flow, &None);
        }
        _ => panic!("Expected Relation node"),
    }
}

#[test]
fn test_parse_instance_without_properties() {
    let source = r#"
        Instance "Batch-123" of resource "Steel" at entity "Warehouse"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Instance { name, resource, entity, properties } => {
            assert_eq!(name, "Batch-123");
            assert_eq!(resource, "Steel");
            assert_eq!(entity, "Warehouse");
            assert!(properties.is_empty());
        }
        _ => panic!("Expected Instance node"),
    }
}

#[test]
fn test_parse_instance_with_properties() {
    let source = r#"
        Instance "Batch-456" of resource "Steel" at entity "Warehouse" {
            status "in-stock"
            quality "premium"
        }
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Instance { name, resource, entity, properties } => {
            assert_eq!(name, "Batch-456");
            assert_eq!(resource, "Steel");
            assert_eq!(entity, "Warehouse");
            assert_eq!(properties.len(), 2);
            assert_eq!(properties.get("status"), Some(&"in-stock".to_string()));
            assert_eq!(properties.get("quality"), Some(&"premium".to_string()));
        }
        _ => panic!("Expected Instance node"),
    }
}

#[test]
fn test_parse_flow_with_unit() {
    let source = r#"
        Flow "Money" from "AP" to "Vendor" quantity 100 "USD"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Flow { resource_name, from_entity, to_entity, quantity, unit } => {
            assert_eq!(resource_name, "Money");
            assert_eq!(from_entity, "AP");
            assert_eq!(to_entity, "Vendor");
            assert_eq!(*quantity, Some(100));
            assert_eq!(unit.as_deref(), Some("USD"));
        }
        _ => panic!("Expected Flow node"),
    }
}

#[test]
fn test_parse_flow_without_unit() {
    let source = r#"
        Flow "Steel" from "Warehouse" to "Factory" quantity 50
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Flow { resource_name, from_entity, to_entity, quantity, unit } => {
            assert_eq!(resource_name, "Steel");
            assert_eq!(from_entity, "Warehouse");
            assert_eq!(to_entity, "Factory");
            assert_eq!(*quantity, Some(50));
            assert_eq!(unit, &None);
        }
        _ => panic!("Expected Flow node"),
    }
}

#[test]
fn test_parse_policy_with_severity() {
    let source = r#"
        Policy data_encrypted per Constraint Obligation priority 9
          @severity "Error"
          @rationale "PII at rest must be encrypted"
        as:
          true
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    match &ast.declarations[0] {
        AstNode::Policy { name, metadata, .. } => {
            assert_eq!(name, "data_encrypted");
            assert_eq!(metadata.severity.as_deref(), Some("Error"));
            assert_eq!(metadata.rationale.as_deref(), Some("PII at rest must be encrypted"));
        }
        _ => panic!("Expected Policy node"),
    }
}

#[test]
fn test_parse_file_metadata_with_collation() {
    let source = r#"
        @namespace "com.acme"
        @collation "en-US:ci"
        
        Entity "Test"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.metadata.namespace.as_deref(), Some("com.acme"));
    assert_eq!(ast.metadata.collation.as_deref(), Some("en-US:ci"));
}

#[test]
fn test_parse_file_metadata_with_asof() {
    let source = r#"
        @namespace "com.acme"
        @asof "2025-11-23T00:00:00Z"
        
        Entity "Test"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.metadata.namespace.as_deref(), Some("com.acme"));
    assert_eq!(ast.metadata.as_of.as_deref(), Some("2025-11-23T00:00:00Z"));
}

#[test]
fn test_comprehensive_dsl_v2_example() {
    let source = r#"
        @namespace "com.acme.finance"
        @version "2.0.0"
        @collation "en-US:ci"
        @asof "2025-11-24T00:00:00Z"
        
        Entity "Party"
        Entity "Account"
        
        Resource "Money" units
        
        Role "Payer" for entity "Party"
        Role "Payee" for entity "Party"
        
        Relation "Payment" {
            subject role "Payer"
            object role "Payee"
            via flow "MoneyTransfer"
        }
        
        Flow "Money" from "Account" to "Account" quantity 1000 "USD"
        
        Instance "Batch-789" of resource "Money" at entity "Account" {
            status "pending"
        }
        
        Policy payment_valid per Constraint Obligation priority 8
          @severity "Error"
          @rationale "Payments must be positive"
        as:
          true
    "#;
    
    let ast = parse_source(source).unwrap();
    
    // Check metadata
    assert_eq!(ast.metadata.namespace.as_deref(), Some("com.acme.finance"));
    assert_eq!(ast.metadata.version.as_deref(), Some("2.0.0"));
    assert_eq!(ast.metadata.collation.as_deref(), Some("en-US:ci"));
    assert_eq!(ast.metadata.as_of.as_deref(), Some("2025-11-24T00:00:00Z"));
    
    // Count declarations
    let entity_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Entity { .. })).count();
    let resource_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Resource { .. })).count();
    let role_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Role { .. })).count();
    let relation_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Relation { .. })).count();
    let flow_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Flow { .. })).count();
    let instance_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Instance { .. })).count();
    let policy_count = ast.declarations.iter().filter(|d| matches!(d, AstNode::Policy { .. })).count();
    
    assert_eq!(entity_count, 2);
    assert_eq!(resource_count, 1);
    assert_eq!(role_count, 2);
    assert_eq!(relation_count, 1);
    assert_eq!(flow_count, 1);
    assert_eq!(instance_count, 1);
    assert_eq!(policy_count, 1);
}

#[test]
fn test_graph_population_with_roles_and_relations() {
    let source = r#"
        @namespace "com.acme.graph"
        
        Entity "Party"
        Resource "Money" units
        
        Role "Payer" for entity "Party"
        Role "Payee" for entity "Party"
        
        Relation "Payment" {
            subject role "Payer"
            object role "Payee"
            via flow "Money"
        }
        
        Flow "Money" from "Party" to "Party" quantity 0
    "#;
    
    let graph = parse_to_graph(source).unwrap();
    
    assert_eq!(graph.entity_count(), 1);
    assert_eq!(graph.resource_count(), 1);
    assert_eq!(graph.role_count(), 2);
    assert_eq!(graph.relation_count(), 1);
    assert_eq!(graph.flow_count(), 1);
    
    let relation = graph.all_relations().into_iter().next().unwrap();
    assert_eq!(relation.name(), "Payment");
    assert!(relation.subject_role_id().is_some());
    assert!(relation.object_role_id().is_some());
    assert!(relation.via_resource_id().is_some()); // This is actually a resource ID in our implementation
}

#[test]
fn test_relation_missing_flow_validation() {
    let source = r#"
        @namespace "com.acme.validation"
        
        Entity "Party"
        Resource "Money" units
        
        Role "Payer" for entity "Party"
        Role "Payee" for entity "Party"
        
        Relation "Payment" {
            subject role "Payer"
            object role "Payee"
            via flow "Money"
        }
    "#;
    
    let result = parse_to_graph(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Relation implies a flow"));
}
