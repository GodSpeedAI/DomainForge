---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: C4Context
section: Context
description: "Stakeholder view of multi-language runtime core"
generated: 2025-11-01
---

# ADR-002 — Multilanguage Ecosystem Context

Context view depicting how stakeholders interface with the Rust core and bindings defined in ADR-002.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Context
%% Diagram: C4Context
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Context.puml
C4Context
  title ADR-002 Context — Rust Core Touchpoints
  Person(pythonDev, "Python Developer", "Builds data science workflows")
  Person(tsDev, "TypeScript Developer", "Delivers web integrations")
  Person(perfTeam, "Performance Engineer", "Benchmarks graph workloads")
  System(seaCore, "SEA Core", "Rust runtime providing domain logic")
  System_Ext(pyBinding, "Python Binding", "PyO3 wheels")
  System_Ext(tsBinding, "TypeScript Binding", "napi-rs module")
  System_Ext(wasmBinding, "WASM Binding", "wit-bindgen package")
  Rel(pythonDev, pyBinding, "Consumes idiomatic API")
  Rel(tsDev, tsBinding, "Uses Node/TypeScript interface")
  Rel(perfTeam, seaCore, "Runs native benchmarks")
  Rel(pyBinding, seaCore, "FFI calls into Rust core")
  Rel(tsBinding, seaCore, "FFI bridge via napi-rs")
  Rel(wasmBinding, seaCore, "Exports to WebAssembly runtimes")
```

- Related: [Context forces flow](ADR-002-flow-context-forces.md)
