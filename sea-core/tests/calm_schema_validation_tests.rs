use sea_core::Graph;
use sea_core::primitives::{Entity, Resource, Flow, Instance};
use sea_core::calm::export;
use jsonschema::JSONSchema;
use rust_decimal::Decimal;

const CALM_SCHEMA: &str = include_str!("../schemas/calm-v1.schema.json");

fn compile_calm_schema() -> JSONSchema {
    let schema = serde_json::from_str(CALM_SCHEMA).expect("Invalid schema JSON");
    JSONSchema::compile(&schema).expect("Failed to compile schema")
}

#[test]
fn test_export_validates_against_schema() {
    let compiled_schema = compile_calm_schema();

    let mut graph = Graph::new();
    let entity = Entity::new("TestEntity".to_string());
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
    let cameras = Resource::new("Cameras".to_string(), "units".to_string());

    let warehouse_id = *warehouse.id();
    let factory_id = *factory.id();
    let cameras_id = *cameras.id();

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
    assert_eq!(schema_value["$schema"], "http://json-schema.org/draft-07/schema#");
}

#[test]
fn test_empty_graph_validates() {
    let schema = serde_json::from_str(CALM_SCHEMA).expect("Invalid schema JSON");
    let compiled_schema = JSONSchema::compile(&schema).expect("Failed to compile schema");

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
    let entity = Entity::new("TestEntity".to_string());
    graph.add_entity(entity).unwrap();

    // Add one resource
    let resource = Resource::new("TestResource".to_string(), "units".to_string());
    graph.add_resource(resource).unwrap();

    // Add one flow
    let flow = Flow::new(
        *graph.all_resources()[0].id(),
        *graph.all_entities()[0].id(),
        *graph.all_entities()[0].id(),
        Decimal::from(100)
    );
    graph.add_flow(flow).unwrap();

    // Add one instance
    let instance = Instance::new(
        *graph.all_resources()[0].id(),
        *graph.all_entities()[0].id()
    );
    graph.add_instance(instance).unwrap();    // Export and validate
    let calm_json = export(&graph).unwrap();
    let result = compiled_schema.validate(&calm_json);
    assert!(result.is_ok(), "Graph with all node types should export valid CALM JSON");
}
