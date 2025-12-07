use sea_core::parser::parse_to_graph;

#[test]
fn test_cast_operator_parsing() {
    let source = r#"
    Dimension "Time1"
    Unit "s1" of "Time1" factor 1 base "s1"
    Unit "ms1" of "Time1" factor 0.001 base "s1"

    Policy test_cast as: 1000 "ms1" as "s1" > 0.5 "s1"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse");
    let policy = graph
        .all_policies()
        .into_iter()
        .next()
        .expect("No policy found");

    let result = policy.evaluate(&graph).expect("Evaluation failed");
    assert!(result.is_satisfied);
}

#[test]
fn test_cast_operator_evaluation() {
    let source = r#"
    Dimension "Time2"
    Unit "s2" of "Time2" factor 1 base "s2"
    Unit "ms2" of "Time2" factor 0.001 base "s2"

    Policy conversion_check as: (1000 "ms2" as "s2") = 1 "s2"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse");
    let policy = graph
        .all_policies()
        .into_iter()
        .next()
        .expect("No policy found");

    let result = policy.evaluate(&graph).expect("Evaluation failed");
    assert!(result.is_satisfied);
}

#[test]
fn test_cast_number_to_unit() {
    let source = r#"
    Dimension "Time3"
    Unit "s3" of "Time3" factor 1 base "s3"

    Policy assign_unit as: (10 as "s3") = 10 "s3"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse");
    let policy = graph
        .all_policies()
        .into_iter()
        .next()
        .expect("No policy found");

    let result = policy.evaluate(&graph).expect("Evaluation failed");
    assert!(result.is_satisfied);
}

#[test]
fn test_cast_incompatible_units_errors() {
    let source = r#"
    Dimension "Time4"
    Unit "s4" of "Time4" factor 1 base "s4"
    Dimension "Money4"
    Unit "USD4" of "Money4" factor 1 base "USD4"

    Policy invalid_cast as: 1 "s4" as "USD4"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse");
    let policy = graph
        .all_policies()
        .into_iter()
        .next()
        .expect("No policy found");

    let result = policy.evaluate(&graph);
    assert!(result.is_err(), "Expected cast to fail across dimensions");
}
