---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: C4Container
section: Alternatives
description: "Runtime architecture options compared"
generated: 2025-11-01
---

# ADR-002 — Runtime Architecture Options

Container view comparing evaluated runtime strategies before selecting the Rust core.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Alternatives
%% Diagram: C4Container
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Container.puml
C4Container
  title ADR-002 Alternatives — Runtime Containers
  System_Boundary(runtimes, "SEA Runtime Strategies") {
    Container(rustCore, "Rust Core + FFI", "Chosen", "Single implementation with bindings")
    Container(pyCore, "Python Core + Native Extensions", "Alternative", "Python logic with C hotspots")
    Container(tsCore, "TypeScript/Node Core", "Alternative", "JS runtime with WASM acceleration")
    Container(multiCore, "Parallel Language Implementations", "Alternative", "Duplicates logic per language")
  }
  Rel(rustCore, pyCore, "Outperforms under heavy traversal")
  Rel(rustCore, tsCore, "Offers stronger type safety")
  Rel(rustCore, multiCore, "Reduces maintenance overhead")
  ContainerDb(targets, "Language Targets", "Python / TypeScript / WASM")
  Rel(rustCore, targets, "Bindings expose core services")
  Rel(pyCore, targets, "Requires rewrite per target")
  Rel(tsCore, targets, "Limited by JS runtime performance")
```

- Related: [Decision trade-off flow](ADR-002-flow-decision-tradeoffs.md)
