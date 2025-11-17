#[cfg(test)]
mod sbvr_flow_facts_tests {
    use rust_decimal::Decimal;
    use sea_core::primitives::{Entity, Flow, Resource};
    use sea_core::sbvr::SbvrModel;
    use sea_core::{unit_from_string, Graph};

    #[test]
    fn test_flow_fact_includes_resource_and_destination() {
        let mut graph = Graph::new();

        let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
        let factory = Entity::new_with_namespace("Factory", "default".to_string());
        let camera = Resource::new_with_namespace(
            "Camera",
            unit_from_string("units"),
            "default".to_string(),
        );

        let warehouse_id = warehouse.id().clone();
        let factory_id = factory.id().clone();
        let camera_id = camera.id().clone();
        let factory_id_str = factory_id.to_string();

        graph.add_entity(warehouse).unwrap();
        graph.add_entity(factory).unwrap();
        graph.add_resource(camera).unwrap();

        let flow = Flow::new_with_namespace(
            camera_id.clone(),
            warehouse_id.clone(),
            factory_id.clone(),
            Decimal::new(100, 0),
            "default",
        );
        graph.add_flow(flow.clone()).unwrap();

        let sbvr_model = SbvrModel::from_graph(&graph).unwrap();

        let flow_fact = sbvr_model
            .facts
            .iter()
            .find(|f| f.id == flow.id().to_string())
            .expect("Flow fact not found");

        assert_eq!(flow_fact.subject, warehouse_id.to_string());
        assert_eq!(flow_fact.verb, "transfers");
        assert_eq!(flow_fact.object, camera_id.to_string());
        assert_eq!(
            flow_fact.destination.as_deref(),
            Some(factory_id_str.as_str())
        );
        assert_eq!(flow_fact.schema_version, "2.0");
    }
}
