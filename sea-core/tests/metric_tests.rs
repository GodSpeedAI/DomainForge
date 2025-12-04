use chrono::Duration;
use sea_core::parser::parse_to_graph;
use sea_core::primitives::Severity;

#[test]
fn test_metric_parsing() {
    let source = r#"
        Metric "total_payment_volume" as: sum(flows.quantity)
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse metrics");

    assert_eq!(graph.metric_count(), 1);

    let metric1 = graph
        .get_metric(&sea_core::ConceptId::from_concept(
            "default",
            "total_payment_volume",
        ))
        .unwrap();
    assert_eq!(metric1.name, "total_payment_volume");
}

#[test]
fn test_metric_with_annotations() {
    let source = r#"
        Metric "high_value_payments" as: count(flows)
            @threshold 100
            @severity "warning"
            @unit "USD"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse metric with annotations");

    assert_eq!(graph.metric_count(), 1);

    let metric = graph
        .get_metric(&sea_core::ConceptId::from_concept(
            "default",
            "high_value_payments",
        ))
        .unwrap();
    assert_eq!(metric.name, "high_value_payments");
    assert!(metric.threshold.is_some());
    assert_eq!(metric.threshold.unwrap().to_string(), "100");
    assert!(matches!(metric.severity, Some(Severity::Warning)));
    assert_eq!(metric.unit.as_deref(), Some("USD"));
}

#[test]
fn test_metric_with_time_annotations() {
    let source = r#"
        Metric "payment_success_rate" as: count(flows)
            @refresh_interval 60 "seconds"
            @window 1 "hour"
            @target 99.9
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse metric with time annotations");

    assert_eq!(graph.metric_count(), 1);

    let metric = graph
        .get_metric(&sea_core::ConceptId::from_concept(
            "default",
            "payment_success_rate",
        ))
        .unwrap();
    assert_eq!(metric.name, "payment_success_rate");
    assert!(metric.refresh_interval.is_some());
    assert!(metric.window.is_some());
    assert!(metric.target.is_some());
    assert_eq!(metric.target.unwrap().to_string(), "99.9");
}

#[test]
fn metric_rejects_unknown_severity() {
    let source = r#"
        Metric "invalid_severity" as: count(flows)
            @severity "urgent"
    "#;

    let err = parse_to_graph(source).expect_err("Expected severity validation to fail");
    let message = format!("{}", err);
    assert!(
        message.contains("Unknown severity") || message.contains("severity"),
        "Unexpected error message: {}",
        message
    );
}

#[test]
fn metric_rejects_unknown_units() {
    let source = r#"
        Metric "invalid_unit" as: count(flows)
            @refresh_interval 5 "weeks"
    "#;

    let err = parse_to_graph(source).expect_err("Expected refresh interval validation to fail");
    let message = format!("{}", err);
    assert!(
        message.contains("duration unit") || message.contains("Invalid"),
        "Unexpected error message: {}",
        message
    );
}

#[test]
fn metric_serializes_durations_with_serde() {
    let source = r#"
        Metric "serde_metric" as: count(flows)
            @refresh_interval 90 "seconds"
            @window 2 "hours"
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse metric with durations");
    let metric = graph
        .get_metric(&sea_core::ConceptId::from_concept(
            "default",
            "serde_metric",
        ))
        .expect("Missing serde_metric")
        .clone();

    let json = serde_json::to_string(&metric).expect("Failed to serialize metric");
    let restored: sea_core::primitives::Metric =
        serde_json::from_str(&json).expect("Failed to deserialize metric");

    assert_eq!(restored.refresh_interval, metric.refresh_interval);
    assert_eq!(restored.window, metric.window);

    // Spot-check that serde preserves values, not just structure.
    assert_eq!(
        restored.refresh_interval,
        Some(Duration::seconds(90)),
        "Refresh interval should round-trip through serde"
    );
    assert_eq!(
        restored.window,
        Some(Duration::hours(2)),
        "Window should round-trip through serde"
    );
}
