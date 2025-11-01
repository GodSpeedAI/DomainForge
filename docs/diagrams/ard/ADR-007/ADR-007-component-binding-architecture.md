---
adr: ADR-007
title: "Idiomatic Language-Specific Bindings"
diagram_type: C4Component
section: Decision
description: "Components delivering idiomatic bindings"
generated: 2025-11-01
---

# ADR-007 — Binding Architecture Components

Component view of language-specific binding layers supporting idiomatic experiences.

```mermaid
%% Source: ADR-007 - Idiomatic Language-Specific Bindings
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-007 Components — Idiomatic Binding Stack
  Container(bindingCore, "Binding Core", "Rust", "Shared safe abstractions over FFI")
  Component(pyFacade, "Python Facade", "Python", "Dataclasses, snake_case API")
  Component(tsFacade, "TypeScript Facade", "TypeScript", "Interfaces, camelCase API")
  Component(wasmFacade, "WASM Facade", "WebAssembly", "Async functions, JS interop")
  Component(errorMapper, "Error Mapper", "Cross-language", "Maps Rust errors to native exceptions")
  Component(batchCoordinator, "Batch Coordinator", "Perf Layer", "Combines operations to reduce FFI chatter")
  Rel(bindingCore, pyFacade, "Exposes typed bindings")
  Rel(bindingCore, tsFacade, "Exposes typed bindings")
  Rel(bindingCore, wasmFacade, "Exposes typed bindings")
  Rel(errorMapper, pyFacade, "Raises Python exceptions")
  Rel(errorMapper, tsFacade, "Throws JS Error")
  Rel(errorMapper, wasmFacade, "Rejects Promises")
  Rel(batchCoordinator, pyFacade, "Provides batch-ready helpers")
  Rel(batchCoordinator, tsFacade, "Queue flush operations")
```

- Related: [Binding implementation pipeline](ADR-007-component-binding-pipeline.md)
