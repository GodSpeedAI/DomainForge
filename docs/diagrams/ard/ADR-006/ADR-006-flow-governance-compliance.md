---
adr: ADR-006
title: "CALM Interoperability for Architecture-as-Code"
diagram_type: Flowchart
section: Governance
description: "Governance workflow for CALM interoperability lifecycle"
generated: 2025-11-01
---

# ADR-006 — CALM Governance Flow

Workflow ensuring CALM mappings and exports remain compliant with evolving specifications.

```mermaid
%% Source: ADR-006 - CALM Interoperability for Architecture-as-Code
%% Generated: 2025-11-01
%% Section: Governance
%% Diagram: Flowchart
graph TD
  A[Monitor CALM spec updates] --> B[Assess impact on mappings]
  B -->|Breaking| C[Update mapping library & schema loader]
  C --> D[Run round-trip regression suite]
  B -->|Non-breaking| E[Record compliance review]
  D -->|Failures| C
  D -->|Pass| F[Architecture council sign-off]
  F --> G[Publish new version + audit://adr-006 entry]
  G --> H[Notify integration partners]
```

- Related: [CALM pipeline components](ADR-006-component-calm-pipeline.md)
