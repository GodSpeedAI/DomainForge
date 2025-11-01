---
adr: ADR-004
title: "SBVR-Aligned Rule Expression Language"
diagram_type: Flowchart
section: Decision
description: "Decision logic selecting SBVR-aligned DSL"
generated: 2025-11-01
---

# ADR-004 — Decision Logic Flow

Flowchart demonstrating evaluation of rule language options against decision criteria.

```mermaid
%% Source: ADR-004 - SBVR-Aligned Rule Expression Language
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: Flowchart
graph TD
  A[Evaluate candidate language] --> B{Supports deontic modalities?}
  B -->|No| C[Discard option]
  B -->|Yes| D{Readable by business analysts?}
  D -->|No| C
  D -->|Yes| E{Has formal semantics?}
  E -->|No| C
  E -->|Yes| F[Select SBVR-aligned DSL]
  F --> G[Adopt modal operators + quantifiers]
```

- Related: [Rule engine component view](ADR-004-component-rule-architecture.md)
