---
adr: ADR-005
title: "Five Core Primitives Design Pattern"
diagram_type: C4Component
section: Decision
description: "Roles of the five selected primitives"
generated: 2025-11-01
---

# ADR-005 — Primitive Roles Component View

Component view summarizing responsibilities for each of the five primitives.

```mermaid
%% Source: ADR-005 - Five Core Primitives Design Pattern
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-005 Components — Primitive Responsibilities
  Container(primitiveLib, "SEA Primitive Library", "Rust", "Core domain data structures")
  Component(entity, "Entity", "Primitive", "Represents actors or organizational units")
  Component(resource, "Resource", "Primitive", "Denotes items of value (goods, funds, services)")
  Component(flow, "Flow", "Primitive", "Captures movement of resources between entities")
  Component(instance, "Instance", "Primitive", "Physical or logical manifestations of resources")
  Component(policy, "Policy", "Primitive", "Constraints and contextual rules")
  Rel(entity, flow, "Acts as source/target")
  Rel(resource, flow, "Defines payload")
  Rel(instance, resource, "Materializes resource")
  Rel(policy, flow, "Constrains transitions")
  Rel(policy, entity, "Applies obligations to actors")
```

- Related: [Implementation data structures](ADR-005-component-primitive-implementation.md)
