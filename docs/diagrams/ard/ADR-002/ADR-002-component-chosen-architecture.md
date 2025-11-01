---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: C4Component
section: Decision
description: "Chosen runtime components and responsibilities"
generated: 2025-11-01
---

# ADR-002 — Chosen Runtime Components

Component view showing how the Rust core and bindings collaborate to satisfy decision criteria.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-002 Components — Rust Core with Bindings
  Container(rustCore, "SEA Rust Core", "Rust", "Implements graph + policy logic")
  Component(pyBridge, "PyO3 Bridge", "Python Binding", "Exposes idiomatic Python API")
  Component(tsBridge, "napi-rs Bridge", "TypeScript Binding", "Provides Node/TS interfaces")
  Component(wasmBridge, "wit-bindgen Bridge", "WASM Binding", "Delivers WebAssembly module")
  Component(memoryLayer, "Memory Safety Layer", "Rust", "Ownership + error conversion")
  Component(perfBench, "Benchmark Harness", "Rust", "Validates <100ms target")
  Rel(memoryLayer, pyBridge, "Translates Result<T,E> to exceptions")
  Rel(memoryLayer, tsBridge, "Maps errors to JS Error")
  Rel(memoryLayer, wasmBridge, "Wraps host errors")
  Rel(pyBridge, rustCore, "Batch operations via FFI")
  Rel(tsBridge, rustCore, "Calls graph API")
  Rel(wasmBridge, rustCore, "Exports to WASM runtime")
  Rel(perfBench, rustCore, "Validates performance budget")
```

- Related: [FFI pipeline implementation](ADR-002-component-ffi-pipeline.md)
