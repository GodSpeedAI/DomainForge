use sea_core::parser::parse;
use sea_core::policy::{Expression, AggregateFunction, BinaryOp};

#[test]
fn test_parse_count_syntax() {
    let source = r#"
        Policy flow_count as: count(flows) > 10
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_sum_with_field() {
    let source = r#"
        Policy camera_sum as: sum(flows.quantity) > 1000
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_count_with_where() {
    let source = r#"
        Policy camera_count as: count(flows where resource = "Camera") > 2
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_avg_aggregation() {
    let source = r#"
        Policy avg_check as: avg(flows.quantity) >= 500
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_min_aggregation() {
    let source = r#"
        Policy min_check as: min(flows.quantity) > 0
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_max_aggregation() {
    let source = r#"
        Policy max_check as: max(flows.quantity) > 10000
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_aggregation_in_complex_expression() {
    let source = r#"
        Policy complex_check as: (count(flows) > 5) and (sum(flows.quantity) < 1000)
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}
