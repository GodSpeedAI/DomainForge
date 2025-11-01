---
adr: ADR-005
title: "Five Core Primitives Design Pattern"
diagram_type: C4Component
section: Implementation
description: "Implementation constructs enforcing primitive rules"
generated: 2025-11-01
---

# ADR-005 — Primitive Implementation Components

Implementation view focusing on data structures and extensibility for the five primitives.

```mermaid
%% Source: ADR-005 - Five Core Primitives Design Pattern
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-005 Implementation — Primitive Data Structures
  Container(coreRuntime, "Primitive Runtime", "Rust", "Provides storage and identity services")
  Component(uuidService, "UUID Identity Service", "Infrastructure", "Assigns stable identifiers")
  Component(attrStore, "Attribute Store", "Extensibility Layer", "Holds domain-specific attributes")
  Component(relBinder, "Relation Binder", "Validation", "Validates primitive relationships")
  Component(ubmMapper, "UBM Mapper", "Interop", "Maps primitives to ERP5 concepts")
  Component(schemaLinter, "Schema Linter", "Tooling", "Detects illegal primitive usage")
  Rel(uuidService, coreRuntime, "Provides unique IDs")
  Rel(attrStore, coreRuntime, "Extends primitive metadata")
  Rel(relBinder, coreRuntime, "Enrolls flows + instances")
  Rel(ubmMapper, relBinder, "Validates mapping fidelity")
  Rel(schemaLinter, attrStore, "Checks attribute conventions")
```

- Related: [Primitive creation sequence](ADR-005-sequence-primitive-creation.md)
