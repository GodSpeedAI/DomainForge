---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: Sequence
section: Implementation
description: "Python binding invoking Rust core"
generated: 2025-11-01
---

# ADR-002 — Python Binding Call Sequence

Sequence demonstrates how a Python client invocation propagates through the FFI boundary.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: sequenceDiagram
sequenceDiagram
  %% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Dynamic.puml
  autonumber
  participant PyClient as Python Client
  participant PyAPI as PyO3 Layer
  participant RustCore as Rust Core
  participant Safety as Memory Safety Layer
  participant ResultMap as Error Mapper
  PyClient->>PyAPI: validate_graph(model, policy)
  PyAPI->>Safety: Prepare args & lifetimes
  Safety->>RustCore: execute_validation(model_ptr, policy_ptr)
  RustCore-->>Safety: Result<Valid, Error>
  Safety->>ResultMap: Normalize error payload
  ResultMap-->>PyAPI: Python exception or value
  PyAPI-->>PyClient: Return ValidationResult or raise
```

- Related: [Binding pipeline components](ADR-002-component-ffi-pipeline.md)
