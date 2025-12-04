use sea_core::parse_to_graph;
use sea_core::parser::parse;

#[test]
fn test_end_to_end_camera_supply_chain() {
    let source = r#"
        // Entities in the supply chain
        Entity "Component Supplier" in sourcing
        Entity "Camera Factory" in manufacturing
        Entity "Distribution Center" in logistics
        Entity "Retail Store" in sales

        // Resources flowing through the supply chain
        Resource "Camera Components" units in inventory
        Resource "Assembled Cameras" units in inventory
        Resource "Packaged Cameras" units in inventory

        // Supply chain flows
        Flow "Camera Components" from "Component Supplier" to "Camera Factory" quantity 1000
        Flow "Assembled Cameras" from "Camera Factory" to "Distribution Center" quantity 800
        Flow "Packaged Cameras" from "Distribution Center" to "Retail Store" quantity 600

        // Business rules
        Policy positive_flow as: forall f in flows: (f.quantity > 0)
        Policy unique_entities as: exists_unique e in entities: (e.name = "Camera Factory")
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse supply chain");

    // Verify entities
    assert_eq!(graph.all_entities().len(), 4);
    let entities: Vec<_> = graph.all_entities().iter().map(|e| e.name()).collect();
    assert!(entities.contains(&"Component Supplier"));
    assert!(entities.contains(&"Camera Factory"));
    assert!(entities.contains(&"Distribution Center"));
    assert!(entities.contains(&"Retail Store"));

    // Verify resources
    assert_eq!(graph.all_resources().len(), 3);
    let resources: Vec<_> = graph.all_resources().iter().map(|r| r.name()).collect();
    assert!(resources.contains(&"Camera Components"));
    assert!(resources.contains(&"Assembled Cameras"));
    assert!(resources.contains(&"Packaged Cameras"));

    // Verify flows
    assert_eq!(graph.all_flows().len(), 3);

    // Test graph queries
    let entities = graph.all_entities();
    let factory = entities
        .iter()
        .find(|e| e.name() == "Camera Factory")
        .expect("Factory not found");

    let inflows = graph.flows_to(factory.id());
    assert_eq!(inflows.len(), 1);

    let outflows = graph.flows_from(factory.id());
    assert_eq!(outflows.len(), 1);
}

#[test]
fn test_end_to_end_multi_domain_model() {
    let source = r#"
        // Finance domain
        Entity "Bank A" in finance
        Entity "Bank B" in finance
        Resource "USD" currency in finance
        Flow "USD" from "Bank A" to "Bank B" quantity 50000

        // Logistics domain
        Entity "Warehouse" in logistics
        Entity "Store" in logistics
        Resource "Products" units in inventory
        Flow "Products" from "Warehouse" to "Store" quantity 200

        // Cross-domain entity
        Entity "Corporate HQ"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse multi-domain model");

    assert_eq!(graph.all_entities().len(), 5);
    assert_eq!(graph.all_resources().len(), 2);
    assert_eq!(graph.all_flows().len(), 2);

    // Verify namespace separation
    let finance_entities: Vec<_> = graph
        .all_entities()
        .iter()
        .filter(|e| e.namespace() == "finance")
        .map(|e| e.name())
        .collect();
    assert_eq!(finance_entities.len(), 2);

    let logistics_entities: Vec<_> = graph
        .all_entities()
        .iter()
        .filter(|e| e.namespace() == "logistics")
        .map(|e| e.name())
        .collect();
    assert_eq!(logistics_entities.len(), 2);
}

#[test]
fn test_end_to_end_minimal_model() {
    let source = r#"
        Entity "A"
        Resource "X" units
        Flow "X" from "A" to "A" quantity 1
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse minimal model");

    assert_eq!(graph.all_entities().len(), 1);
    assert_eq!(graph.all_resources().len(), 1);
    assert_eq!(graph.all_flows().len(), 1);
}

#[test]
fn test_end_to_end_entities_only() {
    let source = r#"
        Entity "Entity A" in domain_a
        Entity "Entity B" in domain_b
        Entity "Entity C"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse entities");

    assert_eq!(graph.all_entities().len(), 3);
    assert_eq!(graph.all_resources().len(), 0);
    assert_eq!(graph.all_flows().len(), 0);
}

#[test]
fn test_end_to_end_resources_only() {
    let source = r#"
        Resource "Resource A" kg in domain_a
        Resource "Resource B" units
        Resource "Resource C" liters in domain_c
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse resources");

    assert_eq!(graph.all_entities().len(), 0);
    assert_eq!(graph.all_resources().len(), 3);
    assert_eq!(graph.all_flows().len(), 0);
}

#[test]
fn test_end_to_end_parse_and_query() {
    let source = r#"
        Entity "Upstream" in production
        Entity "Midstream" in processing
        Entity "Downstream" in sales

        Resource "Material" units

        Flow "Material" from "Upstream" to "Midstream" quantity 100
        Flow "Material" from "Midstream" to "Downstream" quantity 80
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse");

    // Test upstream/downstream queries
    let entities = graph.all_entities();
    let midstream = entities
        .iter()
        .find(|e| e.name() == "Midstream")
        .expect("Midstream not found");

    let upstream_entities = graph.upstream_entities(midstream.id());
    assert_eq!(upstream_entities.len(), 1);
    assert_eq!(upstream_entities[0].name(), "Upstream");

    let downstream_entities = graph.downstream_entities(midstream.id());
    assert_eq!(downstream_entities.len(), 1);
    assert_eq!(downstream_entities[0].name(), "Downstream");
}

#[test]
fn test_end_to_end_error_handling_duplicate_entity() {
    let source = r#"
        Entity "Duplicate"
        Entity "Duplicate"
    "#;

    let result = parse_to_graph(source);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Duplicate"));
    }
}

#[test]
fn test_end_to_end_error_handling_undefined_reference() {
    let source = r#"
        Entity "Factory"
        Resource "Product" units
        Flow "Product" from "Factory" to "Warehouse"
    "#;

    let result = parse_to_graph(source);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Undefined entity") || e.to_string().contains("Warehouse"));
    }
}

#[test]
fn test_end_to_end_empty_model() {
    let source = "";

    let graph = parse_to_graph(source).expect("Failed to parse empty model");

    assert_eq!(graph.all_entities().len(), 0);
    assert_eq!(graph.all_resources().len(), 0);
    assert_eq!(graph.all_flows().len(), 0);
}

#[test]
fn test_end_to_end_comments_and_whitespace() {
    let source = r#"
        // This is a comment

        Entity "A"   // Inline comment

        // Another comment
        Resource "X" units

        Flow "X" from "A" to "A" quantity 1

        // End comment
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse with comments");

    assert_eq!(graph.all_entities().len(), 1);
    assert_eq!(graph.all_resources().len(), 1);
    assert_eq!(graph.all_flows().len(), 1);
}

#[test]
fn test_end_to_end_case_insensitive_keywords() {
    let source = r#"
        ENTITY "E1"
        entity "E2"
        Entity "E3"

        RESOURCE "R1" units
        resource "R2" kg
        Resource "R3" liters

        FLOW "R1" FROM "E1" TO "E2" QUANTITY 10
        flow "R2" from "E2" to "E3" quantity 20
        Flow "R3" from "E3" to "E1" quantity 30
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse case-insensitive");

    assert_eq!(graph.all_entities().len(), 3);
    assert_eq!(graph.all_resources().len(), 3);
    assert_eq!(graph.all_flows().len(), 3);
}

#[test]
fn test_entity_then_instance_parses() {
    let source = r#"
Entity "Vendor"

Instance vendor_123 of "Vendor" {
    name: "Acme Corp",
    credit_limit: 50000
}
"#;

    let ast = parse(source);
    assert!(ast.is_ok(), "Failed to parse: {:?}", ast.err());
    let ast = ast.unwrap();
    assert_eq!(
        ast.declarations.len(),
        2,
        "Expected both declarations parsed"
    );

    let graph = parse_to_graph(source);
    assert!(graph.is_ok(), "Failed to parse: {:?}", graph.err());

    let graph = graph.unwrap();
    assert_eq!(graph.entity_instance_count(), 1);
    assert!(graph.get_entity_instance("vendor_123").is_some());
}
