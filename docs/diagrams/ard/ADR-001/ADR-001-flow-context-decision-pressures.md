---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: Flowchart
section: Context
description: "Decision pressures leading to layered separation"
generated: 2025-11-01
---

# ADR-001 — Context Decision Pressures

The flowchart traces the forces identified in ADR-001 that culminate in choosing a layered architecture.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Context
%% Diagram: Flowchart
graph TD
  A[Decentralized terminology updates] --> B{Risk of tangled models}
  C[Need for SBVR alignment] --> B
  D[Reusability across domains] --> B
  B --> E[Assess structural separation options]
  E --> F[Highlight vocabulary, fact, rule responsibilities]
  F --> G[Require clear dependency flow]
  G --> H[Layered architecture exploration]
```

- Related: [Architecture options map](ADR-001-container-architecture-options.md)
