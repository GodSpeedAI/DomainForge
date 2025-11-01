---
adr: ADR-005
title: "Five Core Primitives Design Pattern"
diagram_type: Flowchart
section: Governance
description: "Governance workflow for primitive library changes"
generated: 2025-11-01
---

# ADR-005 — Primitive Library Governance Flow

Workflow ensuring controlled evolution of the primitive set.

```mermaid
%% Source: ADR-005 - Five Core Primitives Design Pattern
%% Generated: 2025-11-01
%% Section: Governance
%% Diagram: Flowchart
graph TD
  A[Request primitive change] --> B[Evaluate against five-core policy]
  B -->|Violates constraint| C[Reject or propose attribute extension]
  B -->|Aligned| D[Run ERP5 compatibility checks]
  D -->|Mismatch| C
  D -->|Pass| E[Architecture council approval]
  E --> F[Record change in audit://adr-005]
  F --> G[Broadcast updated modeling guide]
```

- Related: [Primitive implementation components](ADR-005-component-primitive-implementation.md)
