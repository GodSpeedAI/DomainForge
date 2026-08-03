use domainforge_core::policy::{AggregateFunction, BinaryOp, Expression, Policy};
use domainforge_core::primitives::{Entity, Instance};
use domainforge_core::{parse_to_graph, Graph};
use serde_json::json;

#[test]
fn parsed_policy_quantifies_over_entity_instances() {
    let graph = parse_to_graph(
        r#"@namespace "interaction"
entity "Journey" {
    key journey_id: string
    maturity: string
}
instance cj01 of "Journey" { journey_id: "cj01", maturity: "exercised" }
instance cj02 of "Journey" { journey_id: "cj02", maturity: "exercised" }
Policy all_exercised as: forall journey in entity_instances: (journey.maturity = "exercised")
"#,
    )
    .expect("entity_instances is valid policy syntax");

    let result = graph.all_policies()[0]
        .evaluate(&graph)
        .expect("entity-instance quantifier evaluates");
    assert!(result.is_satisfied);
}

#[test]
fn aggregate_filter_uses_singular_entity_instance_binding() {
    let mut graph = Graph::new();
    graph
        .add_entity(Entity::new_with_namespace("Journey", "interaction"))
        .unwrap();
    for (name, maturity) in [("cj01", "exercised"), ("cj02", "specified")] {
        let mut instance = Instance::new_with_namespace(name, "Journey", "interaction");
        instance.set_field("maturity", json!(maturity));
        graph.add_entity_instance(instance).unwrap();
    }

    let expression = Expression::aggregation(
        AggregateFunction::Count,
        Expression::variable("entity_instances"),
        None::<String>,
        Some(Expression::binary(
            BinaryOp::Equal,
            Expression::member_access("entity_instance", "maturity"),
            Expression::literal("exercised"),
        )),
    );

    assert_eq!(expression.expand(&graph).unwrap(), Expression::literal(1));
}

#[test]
fn instance_fields_cannot_overwrite_reserved_policy_bindings() {
    let mut graph = Graph::new();
    graph
        .add_entity(Entity::new_with_namespace("Journey", "interaction"))
        .unwrap();
    let mut instance = Instance::new_with_namespace("cj01", "Journey", "interaction");
    instance.set_field("name", json!("spoofed"));
    graph.add_entity_instance(instance).unwrap();

    let expression = Expression::quantifier(
        domainforge_core::policy::Quantifier::Exists,
        "journey",
        Expression::variable("entity_instances"),
        Expression::binary(
            BinaryOp::Equal,
            Expression::member_access("journey", "name"),
            Expression::literal("cj01"),
        ),
    );

    assert!(
        Policy::new("reserved bindings", expression)
            .evaluate(&graph)
            .unwrap()
            .is_satisfied
    );
}
