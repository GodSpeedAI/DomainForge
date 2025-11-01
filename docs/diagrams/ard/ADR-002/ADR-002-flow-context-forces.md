---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: Flowchart
section: Context
description: "Context forces guiding technology selection"
generated: 2025-11-01
---

# ADR-002 — Context Forces Flow

The flowchart outlines performance and accessibility pressures leading to the multi-language requirement.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Context
%% Diagram: Flowchart
graph TD
  A[Need sub-100ms graph validation] --> B{Single language sufficient?}
  C[Diverse developer ecosystems] --> B
  D[Cross-platform deployment] --> B
  B -->|No| E[Evaluate Rust core with FFI]
  B -->|Yes| F[Shortlist alternative runtimes]
  E --> G[Assess binding toolchains]
```

- Related: [Alternative runtime options](ADR-002-container-runtime-options.md)
