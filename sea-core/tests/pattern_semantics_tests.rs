use sea_core::parser::parse_to_graph;
use sea_core::policy::Expression;
use serde_json::json;

#[test]
fn pattern_declaration_and_match_operator_parse() {
    let source = r#"
Pattern "EmailAddress" matches "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
Policy valid_email as: "user@example.com" matches "EmailAddress"
"#;

    let graph = parse_to_graph(source).expect("pattern DSL should parse");
    assert_eq!(graph.pattern_count(), 1);

    let pattern = graph
        .find_pattern("EmailAddress", Some("default"))
        .expect("pattern should be registered");
    assert!(pattern
        .is_match("user@example.com")
        .expect("regex should compile"));

    // Ensure expression parsing preserves matches operator
    let policies = graph.all_policies();
    let policy = policies.first().expect("policy should be present");
    match policy.expression() {
        Expression::Binary { op, .. } => {
            use sea_core::policy::BinaryOp;
            assert_eq!(*op, BinaryOp::Matches);
        }
        other => panic!("expected binary expression, found {:?}", other),
    }
}

#[test]
fn invalid_regex_reports_error() {
    let source = r#"
Pattern "Broken" matches "^(unclosed"
Policy demo as: "anything" matches "Broken"
"#;

    let err = parse_to_graph(source).expect_err("invalid regex should fail");
    assert!(err.to_string().contains("Invalid regex"));
}

#[test]
fn pattern_matching_participates_in_policy_evaluation() {
    let source = r#"
Pattern "EmailAddress" matches "^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$"
Entity "Customer"
Policy valid_contact as:
  forall e in entities: (e.email matches "EmailAddress")
"#;

    let mut graph = parse_to_graph(source).expect("pattern DSL should parse");
    let customer_id = graph
        .find_entity_by_name("Customer")
        .expect("entity should exist");

    {
        let entity = graph
            .get_entity_mut(&customer_id)
            .expect("entity should be mutable");
        entity.set_attribute("email", json!("customer@example.com"));
    }

    let evaluation = {
        let policies = graph.all_policies();
        let policy = policies.first().cloned().expect("policy should exist");

        policy.evaluate(&graph).expect("evaluation should succeed")
    };
    assert!(evaluation.is_satisfied);

    {
        let entity = graph
            .get_entity_mut(&customer_id)
            .expect("entity should be mutable");
        entity.set_attribute("email", json!("invalid-address"));
    }
    let evaluation = {
        let policies = graph.all_policies();
        let policy = policies.first().cloned().expect("policy should exist");

        policy.evaluate(&graph).expect("evaluation should succeed")
    };
    assert!(!evaluation.is_satisfied);
}
