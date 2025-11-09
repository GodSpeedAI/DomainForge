use sea_core::policy::{AggregateFunction, Expression};

#[test]
fn test_count_aggregation() {
    let expr = Expression::aggregation(
        AggregateFunction::Count,
        Expression::variable("flows"),
        None::<&str>,
        None,
    );

    assert!(matches!(expr, Expression::Aggregation { .. }));
}

#[test]
fn test_sum_with_field() {
    let expr = Expression::aggregation(
        AggregateFunction::Sum,
        Expression::variable("flows"),
        Some("quantity"),
        None,
    );

    match expr {
        Expression::Aggregation {
            function, field, ..
        } => {
            assert_eq!(function, AggregateFunction::Sum);
            assert_eq!(field, Some("quantity".to_string()));
        }
        _ => panic!("Expected Aggregation"),
    }
}

#[test]
fn test_aggregation_with_filter() {
    let filter = Expression::comparison("resource", "==", "Camera").unwrap();
    let expr = Expression::aggregation(
        AggregateFunction::Count,
        Expression::variable("flows"),
        None::<&str>,
        Some(filter),
    );

    match expr {
        Expression::Aggregation {
            filter: Some(_), ..
        } => {}
        _ => panic!("Expected filter"),
    }
}

#[test]
fn test_min_aggregation() {
    let expr = Expression::aggregation(
        AggregateFunction::Min,
        Expression::variable("flows"),
        Some("quantity"),
        None,
    );

    match expr {
        Expression::Aggregation { function, .. } => {
            assert_eq!(function, AggregateFunction::Min);
        }
        _ => panic!("Expected Aggregation"),
    }
}

#[test]
fn test_max_aggregation() {
    let expr = Expression::aggregation(
        AggregateFunction::Max,
        Expression::variable("flows"),
        Some("quantity"),
        None,
    );

    match expr {
        Expression::Aggregation { function, .. } => {
            assert_eq!(function, AggregateFunction::Max);
        }
        _ => panic!("Expected Aggregation"),
    }
}

#[test]
fn test_avg_aggregation() {
    let expr = Expression::aggregation(
        AggregateFunction::Avg,
        Expression::variable("flows"),
        Some("quantity"),
        None,
    );

    match expr {
        Expression::Aggregation { function, .. } => {
            assert_eq!(function, AggregateFunction::Avg);
        }
        _ => panic!("Expected Aggregation"),
    }
}
