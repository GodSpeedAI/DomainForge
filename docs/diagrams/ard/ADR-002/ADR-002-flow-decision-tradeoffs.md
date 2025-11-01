---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: Flowchart
section: Decision
description: "Decision trade-offs that favor Rust core"
generated: 2025-11-01
---

# ADR-002 — Decision Trade-offs Flow

The flow reveals how evaluation criteria eliminate alternatives and confirm the Rust core.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: Flowchart
graph TD
  A[Start runtime assessment] --> B{Meets performance target?}
  B -->|No| C[Discard option]
  B -->|Yes| D{Ensures memory safety across FFI?}
  D -->|No| C
  D -->|Yes| E{Single source of truth?}
  E -->|No| C
  E -->|Yes| F[Adopt Rust core with bindings]
  F --> G[Commit to PyO3, napi-rs, wit-bindgen]
```

- Related: [Chosen runtime component view](ADR-002-component-chosen-architecture.md)
