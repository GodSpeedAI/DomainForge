#[cfg(feature = "wasm")]
mod wasm_tests {
    use domainforge_core::wasm::{Entity, Flow, Graph, Instance, Resource};
    use wasm_bindgen_test::wasm_bindgen_test;

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

    #[wasm_bindgen_test]
    fn test_graph_exposes_parsed_declarations_in_source_order() {
        let source = r#"
@namespace "accessors"
Dimension "Length"
Unit "m" of "Length" factor 1 base "m"
Entity "Warehouse"
Pattern "WarehouseCode" matches "^[A-Z]+$"
ConceptChange "warehouse_v2" @from_version v1.0.0 @to_version v2.0.0 @migration_policy mandatory @breaking_change true
instance warehouse_1 of "Warehouse" { code: "WH" }
Policy stock_positive per Constraint Obligation priority 3 as: 1 > 0
Metric "stock_count" as: 1 @unit "items"
Mapping "warehouse_calm" for calm { Entity "Warehouse" -> component { name: "warehouse" } }
Projection "warehouse_kg" for kg { Entity "Warehouse" { label: "Warehouse" } }
"#;

        fn names(value: wasm_bindgen::JsValue) -> Vec<String> {
            let json: serde_json::Value =
                serde_wasm_bindgen::from_value(value).expect("accessor output is JSON");
            json.as_array()
                .expect("accessor output is an array")
                .iter()
                .map(|item| {
                    item["name"]
                        .as_str()
                        .expect("declaration has a name")
                        .to_string()
                })
                .collect()
        }

        let graph = Graph::parse(source.to_string()).unwrap();
        assert_eq!(names(graph.all_policies().unwrap()), vec!["stock_positive"]);
        assert_eq!(names(graph.all_metrics().unwrap()), vec!["stock_count"]);
        assert_eq!(names(graph.all_mappings().unwrap()), vec!["warehouse_calm"]);
        assert_eq!(
            names(graph.all_projections().unwrap()),
            vec!["warehouse_kg"]
        );
        assert_eq!(names(graph.all_dimensions().unwrap()), vec!["Length"]);
        assert_eq!(names(graph.all_units().unwrap()), vec!["m"]);
        assert_eq!(names(graph.all_patterns().unwrap()), vec!["WarehouseCode"]);
        assert_eq!(
            names(graph.all_concept_changes().unwrap()),
            vec!["warehouse_v2"]
        );
        assert_eq!(
            names(graph.all_entity_instances().unwrap()),
            vec!["warehouse_1"]
        );

        // Two parses of the same source must give identical output.
        let again = Graph::parse(source.to_string()).unwrap();
        let snapshot = |graph: &Graph| -> String {
            serde_json::json!({
                "policies": names(graph.all_policies().unwrap()),
                "metrics": names(graph.all_metrics().unwrap()),
                "mappings": names(graph.all_mappings().unwrap()),
                "projections": names(graph.all_projections().unwrap()),
                "dimensions": names(graph.all_dimensions().unwrap()),
                "units": names(graph.all_units().unwrap()),
                "patterns": names(graph.all_patterns().unwrap()),
                "concept_changes": names(graph.all_concept_changes().unwrap()),
                "entity_instances": names(graph.all_entity_instances().unwrap()),
            })
            .to_string()
        };
        assert_eq!(snapshot(&graph), snapshot(&again));
    }
}

#[cfg(feature = "wasm")]
mod application_contract_wasm_tests {
    use domainforge_core::wasm::Graph;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn flagship_sources_json() -> String {
        serde_json::json!({
            "flagship/command-write.sea": include_str!(
                "../../fixtures/application_generation/flagship/command-write.sea"),
            "flagship/query-read.sea": include_str!(
                "../../fixtures/application_generation/flagship/query-read.sea"),
        })
        .to_string()
    }

    #[wasm_bindgen_test]
    fn resolve_application_contract_json_is_canonical() {
        let sources = flagship_sources_json();
        let raw = Graph::resolve_application_contract_json(
            "flagship/query-read.sea".into(),
            sources.clone(),
        )
        .unwrap();
        let again =
            Graph::resolve_application_contract_json("flagship/query-read.sea".into(), sources)
                .unwrap();
        assert_eq!(raw, again);
        let doc: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(doc["schema_version"], "domainforge-application-contract/v1");
    }

    #[wasm_bindgen_test]
    fn resolve_application_contract_json_rejects_bad_source_map() {
        let err =
            Graph::resolve_application_contract_json("a.sea".into(), "[]".into()).unwrap_err();
        let message = err.as_string().unwrap_or_default();
        assert!(message.contains("APP"));
    }

    #[wasm_bindgen_test]
    fn graph_exposes_typed_entity_contract_json() {
        let graph = Graph::parse(
            "@namespace \"binding\"\nentity \"Item\" { key item_id: string value: int }".into(),
        )
        .unwrap();
        let entity_id = graph.find_entity_by_name("Item".into()).unwrap();
        let raw = graph.entity_contract_json(entity_id).unwrap().unwrap();
        let contract: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(contract["key_field"], "item_id");
        assert_eq!(contract["fields"].as_array().unwrap().len(), 2);
    }

    /// Cross-binding byte parity (M0 gate finding 2): WASM must hash to the
    /// same golden as Rust, Python, and TypeScript. See
    /// `application_cross_binding_golden_tests.rs` for the canonical
    /// constants. The producer.version stamp is normalized out before hashing
    /// so release version bumps do not drift the goldens; the stamp itself is
    /// asserted against the running crate version.
    #[wasm_bindgen_test]
    fn cross_binding_golden_hashes() {
        use sha2::{Digest, Sha256};
        const CONTRACT_GOLDEN_SHA256: &str =
            "sha256:38299bd2d0b062d45088f60ceef7018e4f89a1abe839be8edbdd1c961cd303e5";
        let raw = Graph::resolve_application_contract_json(
            "flagship/query-read.sea".into(),
            flagship_sources_json(),
        )
        .unwrap();
        let doc: serde_json::Value =
            serde_json::from_str(&raw).expect("canonical document parses");
        assert_eq!(
            doc["producer"]["version"].as_str(),
            Some(env!("CARGO_PKG_VERSION")),
            "producer.version must stamp the running crate version"
        );
        let normalized = raw.replace(env!("CARGO_PKG_VERSION"), "0.0.0");
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let digest = hasher.finalize();
        let hex: String = {
            let mut s = String::with_capacity(digest.len() * 2);
            for b in digest {
                s.push_str(&format!("{b:02x}"));
            }
            s
        };
        assert_eq!(
            format!("sha256:{hex}"),
            CONTRACT_GOLDEN_SHA256,
            "WASM binding bytes drifted from the Rust golden"
        );
    }
}
