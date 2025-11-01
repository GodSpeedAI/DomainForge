---
adr: ADR-005
title: "Five Core Primitives Design Pattern"
diagram_type: Sequence
section: Implementation
description: "Sequence for creating a flow instance using primitives"
generated: 2025-11-01
---

# ADR-005 — Primitive Creation Sequence

Sequence demonstrates how primitives interact during flow creation.

```mermaid
%% Source: ADR-005 - Five Core Primitives Design Pattern
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: sequenceDiagram
sequenceDiagram
  %% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Dynamic.puml
  autonumber
  participant Modeler as Domain Modeler
  participant EntitySvc as Entity Service
  participant ResourceSvc as Resource Service
  participant FlowSvc as Flow Service
  participant PolicySvc as Policy Service
  Modeler->>EntitySvc: ensure_entity("Supplier")
  EntitySvc-->>Modeler: entity_id
  Modeler->>ResourceSvc: ensure_resource("RawMaterial")
  ResourceSvc-->>Modeler: resource_id
  Modeler->>FlowSvc: create_flow(entity_id, resource_id)
  FlowSvc->>PolicySvc: check_applicable_policies(flow)
  PolicySvc-->>FlowSvc: constraint approvals
  FlowSvc-->>Modeler: flow_id
```

- Related: [Primitive implementation components](ADR-005-component-primitive-implementation.md)
