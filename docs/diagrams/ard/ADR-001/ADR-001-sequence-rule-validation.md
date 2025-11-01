---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: Sequence
section: Implementation
description: "Sequence for validating rule references after vocabulary change"
generated: 2025-11-01
---

# ADR-001 — Rule Validation Sequence

Sequence illustrates how a vocabulary change triggers validations across layers.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: sequenceDiagram
sequenceDiagram
  %% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Dynamic.puml
  autonumber
  participant BA as Business Analyst
  participant API as Model API Facade
  participant Guard as Layer Guard
  participant Fact as Fact Store
  participant Rule as Rule Catalog
  participant Lint as Architecture Linter
  BA->>API: Submit term update
  API->>Guard: Validate operation(scope=vocabulary)
  Guard->>Fact: Check dependent relationships
  Fact-->>Guard: Impacted flows list
  Guard->>Rule: Verify referencing policies
  Rule-->>Guard: Dependent policy ids
  Guard->>Lint: Trigger impact report
  Lint-->>BA: Send validation findings
```

- Related: [Layer dependency enforcement](ADR-001-component-layer-dependencies.md)
