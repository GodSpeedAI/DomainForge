#[cfg(feature = "wasm")]
mod wasm_tests {
    use sea_core::wasm::{Entity, Flow, Graph, Instance, Resource};
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
    fn test_conformance_corpus_minimal_parse() {
        let source = r#"
Role "Payer"
Role "Payee"

Resource "Money" units

Entity "Alice"
Entity "Bob"

Flow "Money" from "Alice" to "Bob" quantity 10

Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  via: flow "Money"
"#;
        let result = Graph::parse(source.to_string());
        assert!(result.is_ok(), "conformance corpus 01 must parse in WASM");

        let graph = result.unwrap();
        assert_eq!(graph.entity_count(), 2, "2 entities expected");
        assert_eq!(graph.resource_count(), 1, "1 resource expected");
        assert_eq!(graph.flow_count(), 1, "1 flow expected");

        let json_result = graph.to_json();
        assert!(
            json_result.is_ok(),
            "canonical JSON serialization must work in WASM"
        );

        let json = json_result.unwrap();
        let value: serde_json::Value =
            serde_wasm_bindgen::from_value(json).expect("JsValue round-trips to serde_json");
        let text = serde_json::to_string(&value).expect("serialize to string");
        assert!(
            text.contains("Alice"),
            "canonical JSON must contain entity names"
        );
        assert!(
            text.contains("Money"),
            "canonical JSON must contain resource names"
        );
    }

    fn normalize_flow_uuids(value: serde_json::Value) -> serde_json::Value {
        let replacements: Vec<(String, String)> =
            match value.get("flows").and_then(|f| f.as_object()) {
                Some(flows) => flows
                    .keys()
                    .enumerate()
                    .map(|(i, key)| (key.clone(), format!("flow:{i}")))
                    .collect(),
                None => Vec::new(),
            };
        if replacements.is_empty() {
            return value;
        }
        let mut serialized = serde_json::to_string(&value).expect("serialize JSON value");
        for (uuid, token) in &replacements {
            serialized = serialized.replace(uuid, token);
        }
        serde_json::from_str(&serialized).expect("re-parse normalized JSON")
    }

    fn normalize_authority_volatile(value: serde_json::Value) -> serde_json::Value {
        const VOLATILE: &[&str] = &["created_at", "decision_id", "trace_ref"];
        match value {
            serde_json::Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (key, val) in map {
                    if VOLATILE.contains(&key.as_str()) {
                        out.insert(key, serde_json::Value::String("<volatile>".to_string()));
                    } else {
                        out.insert(key, normalize_authority_volatile(val));
                    }
                }
                serde_json::Value::Object(out)
            }
            serde_json::Value::Array(items) => serde_json::Value::Array(
                items
                    .into_iter()
                    .map(normalize_authority_volatile)
                    .collect(),
            ),
            other => other,
        }
    }

    #[wasm_bindgen_test]
    fn test_conformance_01_parse_canonical_json() {
        let input = include_str!("../../conformance/01_minimal_domain/input.sea");
        let expected = include_str!("../../conformance/01_minimal_domain/expected/graph.json");

        let graph = Graph::parse(input.to_string()).expect("item 01 must parse in WASM");
        let actual_text = graph.to_canonical_json().expect("canonical JSON string");
        let actual: serde_json::Value = serde_json::from_str(&actual_text).expect("parse actual");
        let expected: serde_json::Value = serde_json::from_str(expected).expect("parse expected");

        let actual_norm = normalize_flow_uuids(actual);
        let expected_norm = normalize_flow_uuids(expected);
        assert_eq!(
            actual_norm, expected_norm,
            "WASM parse output must byte-match the Rust expected graph.json (flow IDs normalized)"
        );
    }

    #[wasm_bindgen_test]
    fn test_conformance_02_validate_tristate_true() {
        let input = include_str!("../../conformance/02_policy_complete_facts/input.sea");
        let expected =
            include_str!("../../conformance/02_policy_complete_facts/expected/validate.json");

        let graph = Graph::parse(input.to_string()).expect("item 02 must parse in WASM");
        let actual_text = graph.validate_json().expect("validate JSON string");
        let actual: serde_json::Value = serde_json::from_str(&actual_text).expect("parse actual");
        let expected: serde_json::Value = serde_json::from_str(expected).expect("parse expected");

        assert_eq!(
            actual, expected,
            "WASM validate output must byte-match the Rust expected validate.json for item 02"
        );

        let tristate = &actual["policies"][0]["is_satisfied_tristate"];
        assert_eq!(
            tristate.as_bool(),
            Some(true),
            "item 02 must report tristate=true (all facts present)"
        );
    }

    #[wasm_bindgen_test]
    fn test_conformance_03_validate_tristate_null() {
        let input = include_str!("../../conformance/03_policy_null_facts/input.sea");
        let expected =
            include_str!("../../conformance/03_policy_null_facts/expected/validate.json");

        let graph = Graph::parse(input.to_string()).expect("item 03 must parse in WASM");
        let actual_text = graph.validate_json().expect("validate JSON string");
        let actual: serde_json::Value = serde_json::from_str(&actual_text).expect("parse actual");
        let expected: serde_json::Value = serde_json::from_str(expected).expect("parse expected");

        assert_eq!(
            actual, expected,
            "WASM validate output must byte-match the Rust expected validate.json for item 03"
        );

        let tristate = &actual["policies"][0]["is_satisfied_tristate"];
        assert!(
            tristate.is_null(),
            "item 03 must report tristate=null (genuine NULL from missing attribute)"
        );

        let violations = actual["policies"][0]["violations"]
            .as_array()
            .expect("violations array");
        assert!(
            !violations.is_empty(),
            "item 03 must have at least one violation"
        );
        let msg = violations[0].as_str().unwrap_or("");
        assert!(
            msg.contains("UNKNOWN"),
            "violation text must contain UNKNOWN, got: {msg}"
        );
    }

    #[wasm_bindgen_test]
    fn test_authority_08_trace_parity() {
        let config = include_str!("../../conformance/08_authority/config.json");
        let request = include_str!("../../conformance/08_authority/request.json");
        let facts = include_str!("../../conformance/08_authority/facts.json");
        let trace_golden = include_str!("../../conformance/08_authority/trace.json");
        let decision_golden = include_str!("../../conformance/08_authority/decision.json");

        let result = sea_core::wasm::evaluate_authority(config, request, Some(facts.to_string()))
            .expect("WASM evaluateAuthority must succeed");

        let result_obj: serde_json::Value =
            serde_wasm_bindgen::from_value(result).expect("JsValue round-trips to serde_json");
        let actual_trace = result_obj.get("trace").expect("result has trace").clone();
        let actual_decision = result_obj
            .get("decision")
            .expect("result has decision")
            .clone();

        let expected_trace: serde_json::Value =
            serde_json::from_str(trace_golden).expect("parse trace golden");
        let expected_decision: serde_json::Value =
            serde_json::from_str(decision_golden).expect("parse decision golden");

        assert_eq!(
            normalize_authority_volatile(actual_trace),
            normalize_authority_volatile(expected_trace),
            "WASM authority trace must byte-match the Rust golden (volatile-normalized)"
        );
        assert_eq!(
            normalize_authority_volatile(actual_decision.clone()),
            normalize_authority_volatile(expected_decision),
            "WASM authority decision must byte-match the Rust golden (volatile-normalized)"
        );

        assert_eq!(
            actual_decision["final_decision"].as_str().unwrap_or(""),
            "deny",
            "08_authority must deny the credit-Hold shipment"
        );

        let trace_obj = serde_json::from_str::<serde_json::Value>(trace_golden)
            .expect("parse trace golden for pack_hashes check");
        let pack_hashes = trace_obj
            .get("pack_hashes")
            .and_then(|h| h.as_array())
            .expect("trace has pack_hashes array");
        assert_eq!(
            pack_hashes.len(),
            2,
            "08_authority pins exactly two pack hashes"
        );
    }
}
