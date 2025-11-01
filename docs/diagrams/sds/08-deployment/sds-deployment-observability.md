---
sds_section: "8. Deployment & Operations"
diagram_type: "C4Container"
component_ids: ["OPS-PIPE-ReleaseAutomation", "CORE-API-RustCore", "VAL-SVC-PolicyEvaluator", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-012", "REQ-013", "REQ-016", "REQ-018", "REQ-019"]
related_diagrams:
  - sds-deployment-scaling-strategy.md
  - ../03-architecture/sds-architecture-error-handling.md
  - ../09-testing/sds-testing-strategy.md
updated: "2025-11-01"
reviewed_by: "DevOps Team"
purpose: "Shows observability architecture including metrics, logs, and traces for validation workflows."
---

## Observability Integration

```mermaid
C4Container
    %% Source: docs/specs/sds.md SDS-006, SDS-014, SDS-015
    %% Implements: ADR-002, ADR-004, ADR-008
    %% Satisfies: REQ-005, REQ-012, REQ-013, REQ-016, REQ-018, REQ-019
    %% Components: OPS-PIPE-ReleaseAutomation, CORE-API-RustCore, VAL-SVC-PolicyEvaluator, DOC-PIPE-DocGenerator

    Container(core, "CORE-API-RustCore", "Rust crate", "Emits metrics/events")
    Container(policy, "VAL-SVC-PolicyEvaluator", "Rust module", "Publishes validation metrics")
    Container(ci, "OPS-PIPE-ReleaseAutomation", "GitHub Workflow", "Collects metrics, publishes dashboards")
    Container(doc, "DOC-PIPE-DocGenerator", "Docs pipeline", "Publishes observability playbooks")
    Container(mixture, "Metrics Collector", "Prometheus/Grafana", "Stores metrics")
    Container(logs, "Log Aggregator", "OpenSearch", "Stores structured logs")
    Container(traces, "Tracing Backend", "OpenTelemetry Collector", "Stores spans")
    Person(qateam, "QA Team", "Monitors dashboards")
    Person(secops, "Security Ops", "Monitors alerts")

    Rel(core, mixture, "Metric: validation_duration_ms")
    Rel(core, logs, "Structured logs w/ namespace details")
    Rel(policy, mixture, "Metric: violation_emit_latency_ms")
    Rel(policy, traces, "Spans: policy evaluation depth")
    Rel(ci, mixture, "Benchmark results per release")
    Rel(ci, doc, "Publish runbooks")
    Rel(doc, qateam, "Provide dashboards & playbooks")
    Rel(doc, secops, "Compliance audit docs")
    Rel(mixture, qateam, "Dashboards & alerts")
    Rel(logs, secops, "Alerting rules")
```

### Design Rationale
- Differentiates metrics/logs/traces to pinpoint validation slowdown vs structural errors.
- Observability playbooks ensure docs remain updated (REQ-019).

### Related Components
- Testing integration ensures metrics validated in [sds-testing-strategy](../09-testing/sds-testing-strategy.md).
