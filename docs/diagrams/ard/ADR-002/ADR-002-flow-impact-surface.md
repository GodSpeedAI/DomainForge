---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
diagram_type: Flowchart
section: Impact
description: "Impact propagation across developer ecosystems"
generated: 2025-11-01
---

# ADR-002 — Impact Surface Flow

Flowchart highlights the downstream effects of consolidating on a Rust core with FFI bindings.

```mermaid
%% Source: ADR-002 - Rust Core with Multi-Language FFI Bindings
%% Generated: 2025-11-01
%% Section: Impact
%% Diagram: Flowchart
graph TD
  A[Rust core adopted] --> B[Shared semantic consistency]
  A --> C[Prebuilt multi-platform artifacts]
  B --> D[Reduced bug surface across bindings]
  D --> E[Fewer cross-language regressions]
  C --> F[Lockstep release cadence]
  F --> G[Simplified support load]
  B --> H[Accelerated feature rollout]
  H --> I[Developers use latest capabilities]
```

- Related: [Platform impact context](ADR-002-context-impact-platforms.md)
