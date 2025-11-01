---
adr: ADR-006
title: "CALM Interoperability for Architecture-as-Code"
diagram_type: Sequence
section: Implementation
description: "Round-trip transformation sequence SEA↔CALM↔SEA"
generated: 2025-11-01
---

# ADR-006 — CALM Round-Trip Sequence

Sequence outlines how models undergo CALM export and re-import to guarantee semantic preservation.

```mermaid
%% Source: ADR-006 - CALM Interoperability for Architecture-as-Code
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: sequenceDiagram
sequenceDiagram
  %% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Dynamic.puml
  autonumber
  participant Architect as Enterprise Architect
  participant CLI as Architecture CLI
  participant Export as Export Engine
  participant CALM as FINOS CALM JSON
  participant Import as Import Engine
  participant Diff as Diff Analyzer
  Architect->>CLI: to_calm(model.sea)
  CLI->>Export: Serialize SEA model
  Export-->>CALM: CALM JSON artifact
  Architect->>CLI: from_calm(calm.json)
  CLI->>Import: Parse CALM payload
  Import->>Diff: Compare with original SEA model
  Diff-->>Architect: Semantic diff report
```

- Related: [CALM pipeline components](ADR-006-component-calm-pipeline.md)
