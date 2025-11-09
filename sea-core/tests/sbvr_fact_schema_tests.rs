#[cfg(test)]
mod sbvr_fact_schema_tests {
    use sea_core::sbvr::SbvrFactType;

    #[test]
    fn test_fact_type_includes_destination_and_schema_version() {
        let fact = SbvrFactType {
            id: "fact-001".to_string(),
            subject: "entity-from".to_string(),
            verb: "transfers".to_string(),
            object: "resource-123".to_string(),
            destination: Some("entity-to".to_string()),
            schema_version: "2.0".to_string(),
        };

        let json = serde_json::to_string(&fact).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["schema_version"], "2.0");
        assert_eq!(value["destination"].as_str(), Some("entity-to"));
        assert_eq!(value["object"], "resource-123");
    }

    #[test]
    fn test_fact_v1_upgrade_without_destination() {
        let legacy = r#"{
            "id":"legacy-fact",
            "subject":"legacy-from",
            "verb":"transfers",
            "object":"legacy-to"
        }"#;

        let upgraded: SbvrFactType = serde_json::from_str(legacy).unwrap();
        assert_eq!(upgraded.schema_version, "2.0");
        assert_eq!(upgraded.destination, None);
        assert_eq!(upgraded.object, "legacy-to");
    }

    #[test]
    fn test_sbvr_fact_round_trip() {
        let fact = SbvrFactType {
            id: "fact-002".to_string(),
            subject: "warehouse".to_string(),
            verb: "transfers".to_string(),
            object: "camera-resource".to_string(),
            destination: Some("factory".to_string()),
            schema_version: "2.0".to_string(),
        };

        let json = serde_json::to_string(&fact).unwrap();
        let deserialized: SbvrFactType = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "fact-002");
        assert_eq!(deserialized.subject, "warehouse");
        assert_eq!(deserialized.verb, "transfers");
        assert_eq!(deserialized.object, "camera-resource");
        assert_eq!(deserialized.destination.as_deref(), Some("factory"));
        assert_eq!(deserialized.schema_version, "2.0");
    }
}
