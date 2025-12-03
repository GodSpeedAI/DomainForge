use jsonschema::JSONSchema;
use rust_decimal::Decimal;
use sea_core::calm::export;
use sea_core::primitives::{Entity, Flow, Resource, ResourceInstance};
use sea_core::units::unit_from_string;
use sea_core::Graph;

const CALM_SCHEMA: &str = include_str!("../schemas/calm-v1.schema.json");

fn compile_calm_schema() -> JSONSchema {
    let schema = serde_json::from_str(CALM_SCHEMA).expect("Invalid schema JSON");
    JSONSchema::compile(&schema).expect("Failed to compile schema")
}

#[test]
fn test_export_validates_against_schema() {
    let compiled_schema = compile_calm_schema();

    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("TestEntity".to_string(), "default".to_string());
    graph.add_entity(entity).unwrap();

    let calm_json = export(&graph).unwrap();

    let result = compiled_schema.validate(&calm_json);

    if let Err(errors) = result {
        for error in errors {
            eprintln!("Validation error: {}", error);
            eprintln!("Instance path: {}", error.instance_path);
        }
        panic!("CALM export does not validate against schema");
    }
}

#[test]
fn test_complex_export_validates_against_schema() {
    let compiled_schema = compile_calm_schema();

    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse".to_string(), "logistics".to_string());
    let factory = Entity::new_with_namespace("Factory".to_string(), "manufacturing".to_string());
    let cameras = Resource::new_with_namespace(
        "Cameras".to_string(),
        unit_from_string("units"),
        "default".to_string(),
    );

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(cameras_id, warehouse_id, factory_id, Decimal::from(100));
    graph.add_flow(flow).unwrap();

    let calm_json = export(&graph).unwrap();

    let result = compiled_schema.validate(&calm_json);

    if let Err(errors) = result {
        for error in errors {
            eprintln!("Validation error: {}", error);
            eprintln!("Instance path: {}", error.instance_path);
        }
        panic!("CALM export does not validate against schema");
    }
}

#[test]
fn test_schema_is_valid_json_schema() {
    let schema = serde_json::from_str::<serde_json::Value>(CALM_SCHEMA);
    assert!(schema.is_ok(), "CALM schema must be valid JSON");

    let schema_value = schema.unwrap();
    assert!(schema_value.is_object());
    assert_eq!(
        schema_value["$schema"],
        "http://json-schema.org/draft-07/schema#"
    );
}

#[test]
fn test_empty_graph_validates() {
    let compiled_schema = compile_calm_schema();

    let graph = Graph::new();
    let calm_json = export(&graph).unwrap();

    let result = compiled_schema.validate(&calm_json);
    assert!(result.is_ok(), "Empty graph export should validate");
}

#[test]
fn test_all_node_types_validate() {
    let compiled_schema = compile_calm_schema();

    let mut graph = Graph::new();

    // Add one entity
    let entity = Entity::new_with_namespace("TestEntity".to_string(), "default".to_string());
    graph.add_entity(entity).unwrap();

    // Add one resource
    let resource = Resource::new_with_namespace(
        "TestResource".to_string(),
        unit_from_string("units"),
        "default".to_string(),
    );
    graph.add_resource(resource).unwrap();

    // Add one flow
    let flow = Flow::new(
        graph.all_resources()[0].id().clone(),
        graph.all_entities()[0].id().clone(),
        graph.all_entities()[0].id().clone(),
        Decimal::from(100),
    );
    graph.add_flow(flow).unwrap();

    // Add one instance
    let instance = ResourceInstance::new(
        graph.all_resources()[0].id().clone(),
        graph.all_entities()[0].id().clone(),
    );
    graph.add_instance(instance).unwrap(); // Export and validate
    let calm_json = export(&graph).unwrap();
    let result = compiled_schema.validate(&calm_json);
    assert!(
        result.is_ok(),
        "Graph with all node types should export valid CALM JSON"
    );
}
