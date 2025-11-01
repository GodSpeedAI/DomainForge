---
adr: ADR-007
title: "Idiomatic Language-Specific Bindings"
diagram_type: Flowchart
section: Governance
description: "Governance workflow maintaining idiomatic standards"
generated: 2025-11-01
---

# ADR-007 — Binding Quality Governance Flow

Workflow ensuring idiomatic standards remain enforced across languages.

```mermaid
%% Source: ADR-007 - Idiomatic Language-Specific Bindings
%% Generated: 2025-11-01
%% Section: Governance
%% Diagram: Flowchart
graph TD
  A[Propose binding change] --> B[Run style and naming lint]
  B -->|Violations| C[Align with idiom guidelines]
  B -->|Clean| D[Execute cross-language tests]
  D -->|Fail| C
  D -->|Pass| E[DX review sign-off]
  E --> F[Update language-specific docs]
  F --> G[Record change in audit://adr-007]
```

- Related: [Binding pipeline components](ADR-007-component-binding-pipeline.md)
