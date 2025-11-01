---
adr: ADR-007
title: "Idiomatic Language-Specific Bindings"
diagram_type: Sequence
section: Implementation
description: "TypeScript async call path demonstrating idiomatic bindings"
generated: 2025-11-01
---

# ADR-007 — TypeScript Async Call Sequence

Sequence illustrates how the TypeScript binding surfaces an idiomatic Promise-based API.

```mermaid
%% Source: ADR-007 - Idiomatic Language-Specific Bindings
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: sequenceDiagram
sequenceDiagram
  %% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Dynamic.puml
  autonumber
  participant App as TypeScript App
  participant TsAPI as TypeScript Binding
  participant Batch as Batch Coordinator
  participant FFI as Rust FFI Bridge
  participant ErrMap as Error Mapper
  App->>TsAPI: await validateModel(input)
  TsAPI->>Batch: enqueue(operation="validate", payload)
  Batch->>FFI: flushBatch()
  FFI-->>Batch: Result<Success, RustError>
  Batch->>ErrMap: normalizeError()
  ErrMap-->>TsAPI: resolve or reject Promise
  TsAPI-->>App: Promise<ValidationResult>
```

- Related: [Binding pipeline components](ADR-007-component-binding-pipeline.md)
