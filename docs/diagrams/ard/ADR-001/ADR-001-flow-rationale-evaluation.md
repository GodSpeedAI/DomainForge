---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: Flowchart
section: Decision
description: "Rationale evaluation leading to chosen architecture"
generated: 2025-11-01
---

# ADR-001 — Decision Evaluation Flow

The flowchart visualizes how forces and evaluation criteria direct the decision toward the three-layer architecture.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: Flowchart
graph TD
  A[Start evaluation] --> B{Does option ensure SBVR alignment?}
  B -->|No| C[Reject option]
  B -->|Yes| D{Does option separate stakeholder concerns?}
  D -->|No| C
  D -->|Yes| E{Can dependencies be enforced top-down?}
  E -->|No| C
  E -->|Yes| F[Select Three-Layer Architecture]
  F --> G[Document constraints and dependency rules]
```

- Related: [Component responsibilities](ADR-001-component-layer-responsibilities.md)
