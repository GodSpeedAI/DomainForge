---
adr: ADR-007
title: "Idiomatic Language-Specific Bindings"
diagram_type: Flowchart
section: Context
description: "Pressures that necessitate idiomatic bindings"
generated: 2025-11-01
---

# ADR-007 — Context Pressure Flow

Flowchart highlighting experience and performance pressures addressed by idiomatic bindings.

```mermaid
%% Source: ADR-007 - Idiomatic Language-Specific Bindings
%% Generated: 2025-11-01
%% Section: Context
%% Diagram: Flowchart
graph TD
  A[Developers expect native conventions] --> B{Can shared API satisfy?}
  C[Need for ecosystem integration] --> B
  D[Minimize FFI overhead] --> E{Bindings support batching?}
  B -->|No| F[Design idiomatic layer per language]
  E -->|No| G[Introduce batch-oriented APIs]
  F --> H[Define naming & error guidelines]
  G --> H
```

- Related: [Binding strategy options](ADR-007-container-binding-options.md)
