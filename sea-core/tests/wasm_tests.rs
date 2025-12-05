#[cfg(feature = "wasm")]
mod wasm_tests {
    use sea_core::wasm::{Entity, Flow, Graph, Instance, Resource};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_entity_creation() {
        let entity = Entity::new("Warehouse".to_string(), Some("logistics".to_string()));
        assert_eq!(entity.name(), "Warehouse");
        assert_eq!(entity.namespace(), Some("logistics".to_string()));
        assert!(!entity.id().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_entity_without_namespace() {
        let entity = Entity::new("Factory".to_string(), None);
        assert_eq!(entity.name(), "Factory");
        assert_eq!(entity.namespace(), None);
    }

    #[wasm_bindgen_test]
    fn test_entity_attributes() {
        let mut entity = Entity::new("Store".to_string(), None);
        let value = serde_wasm_bindgen::to_value(&serde_json::json!({"location": "NYC"})).unwrap();
        entity.set_attribute("metadata".to_string(), value).unwrap();

        let retrieved = entity.get_attribute("metadata".to_string());
        assert!(!retrieved.is_null());
    }

    #[wasm_bindgen_test]
    fn test_resource_creation() {
        let resource = Resource::new("Cameras".to_string(), "units".to_string(), None);
        assert_eq!(resource.name(), "Cameras");
        assert_eq!(resource.unit(), "units");
        assert!(!resource.id().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_resource_with_namespace() {
        let resource = Resource::new(
            "Steel".to_string(),
            "kg".to_string(),
            Some("materials".to_string()),
        );
        assert_eq!(resource.name(), "Steel");
        assert_eq!(resource.unit(), "kg");
        assert_eq!(resource.namespace(), Some("materials".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_flow_creation() {
        let entity1 = Entity::new("Source".to_string(), None);
        let entity2 = Entity::new("Dest".to_string(), None);
        let resource = Resource::new("Product".to_string(), "units".to_string(), None);

        let flow = Flow::new(
            resource.id(),
            entity1.id(),
            entity2.id(),
            "100".to_string(),
            None,
        );

        assert!(flow.is_ok());
        let flow = flow.unwrap();
        assert_eq!(flow.quantity(), "100");
        assert!(!flow.id().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_instance_creation() {
        let instance = Instance::new("warehouse_1".to_string(), "Warehouse".to_string(), None);

        assert!(!instance.id().is_empty());
        assert_eq!(instance.name(), "warehouse_1");
        assert_eq!(instance.entity_type(), "Warehouse");
    }

    #[wasm_bindgen_test]
    fn test_graph_creation() {
        let graph = Graph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.entity_count(), 0);
        assert_eq!(graph.resource_count(), 0);
        assert_eq!(graph.flow_count(), 0);
    }

    #[wasm_bindgen_test]
    fn test_graph_add_entity() {
        let mut graph = Graph::new();
        let entity = Entity::new("Factory".to_string(), None);

        let result = graph.add_entity(&entity);
        assert!(result.is_ok());
        assert_eq!(graph.entity_count(), 1);
        assert!(!graph.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_graph_get_entity() {
        let mut graph = Graph::new();
        let entity = Entity::new("Office".to_string(), None);
        let id = entity.id();

        graph.add_entity(&entity).unwrap();
        let retrieved = graph.get_entity(id).unwrap();

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name(), "Office");
    }

    #[wasm_bindgen_test]
    fn test_graph_find_entity_by_name() {
        let mut graph = Graph::new();
        let entity = Entity::new("Distribution Center".to_string(), None);

        graph.add_entity(&entity).unwrap();
        let found_id = graph.find_entity_by_name("Distribution Center".to_string());

        assert!(found_id.is_some());
    }

    #[wasm_bindgen_test]
    fn test_graph_add_resource() {
        let mut graph = Graph::new();
        let resource = Resource::new("Materials".to_string(), "kg".to_string(), None);

        let result = graph.add_resource(&resource);
        assert!(result.is_ok());
        assert_eq!(graph.resource_count(), 1);
    }

    #[wasm_bindgen_test]
    fn test_graph_add_flow_with_validation() {
        let mut graph = Graph::new();
        let entity1 = Entity::new("Source".to_string(), None);
        let entity2 = Entity::new("Target".to_string(), None);
        let resource = Resource::new("Goods".to_string(), "units".to_string(), None);

        graph.add_entity(&entity1).unwrap();
        graph.add_entity(&entity2).unwrap();
        graph.add_resource(&resource).unwrap();

        let flow = Flow::new(
            resource.id(),
            entity1.id(),
            entity2.id(),
            "50".to_string(),
            None,
        )
        .unwrap();

        let result = graph.add_flow(&flow);
        assert!(result.is_ok());
        assert_eq!(graph.flow_count(), 1);
    }

    #[wasm_bindgen_test]
    fn test_graph_flows_from() {
        let mut graph = Graph::new();
        let entity1 = Entity::new("Warehouse".to_string(), None);
        let entity2 = Entity::new("Store".to_string(), None);
        let resource = Resource::new("Items".to_string(), "units".to_string(), None);

        graph.add_entity(&entity1).unwrap();
        graph.add_entity(&entity2).unwrap();
        graph.add_resource(&resource).unwrap();

        let flow = Flow::new(
            resource.id(),
            entity1.id(),
            entity2.id(),
            "100".to_string(),
            None,
        )
        .unwrap();
        graph.add_flow(&flow).unwrap();

        let flows = graph.flows_from(entity1.id());
        assert!(flows.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_graph_parse_simple() {
        let source = r#"
Entity "Warehouse" in logistics
Resource "Cameras" units
"#;

        let result = Graph::parse(source.to_string());
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.entity_count(), 1);
        assert_eq!(graph.resource_count(), 1);
    }

    #[wasm_bindgen_test]
    fn test_graph_parse_with_flow() {
        let source = r#"
Entity "Warehouse" in logistics
Entity "Factory" in manufacturing
Resource "Materials" kg
Flow "Materials" from "Warehouse" to "Factory" quantity 500
"#;

        let result = Graph::parse(source.to_string());
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.entity_count(), 2);
        assert_eq!(graph.resource_count(), 1);
        assert_eq!(graph.flow_count(), 1);
    }

    #[wasm_bindgen_test]
    fn test_graph_serialization() {
        let mut graph = Graph::new();
        let entity = Entity::new("TestEntity".to_string(), None);
        graph.add_entity(&entity).unwrap();

        let json_result = graph.to_json();
        assert!(json_result.is_ok());
    }
}
