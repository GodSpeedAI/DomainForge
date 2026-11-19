use domainforge_core::authority::FactRequirement;
use serde_json::json;

#[test]
fn fact_requirement_duration_wire_uses_normalized_seconds_and_nanoseconds() {
    for wire in [
        json!([0, 0]),
        json!([1, 250_000_000]),
        json!([-2, 750_000_000]),
        json!([9_223_372_036_854_775_i64, 807_000_000]),
    ] {
        let input = json!({"fact_path": "facts.x", "allowed_source_classes": [], "max_age": wire});
        let requirement: FactRequirement = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(requirement).unwrap()["max_age"],
            input["max_age"]
        );
    }
}
