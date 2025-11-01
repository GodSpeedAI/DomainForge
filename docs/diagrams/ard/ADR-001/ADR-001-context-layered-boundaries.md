---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: C4Context
section: Context
description: "Layered system boundaries and stakeholder touchpoints"
generated: 2025-11-01
---

# ADR-001 — Context Layered Boundaries

This diagram highlights how core stakeholders interact with the SEA DSL layers defined in ADR-001.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Context
%% Diagram: C4Context
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Context.puml
C4Context
  title ADR-001 Context — Layered Interactions
  Person(businessAnalyst, "Business Analyst", "Curates business vocabulary")
  Person(domainArchitect, "Domain Architect", "Shapes fact model structure")
  Person(developer, "Developer", "Implements and validates rules")
  System(seaDsl, "SEA DSL", "Semantic Enterprise Architecture platform")
  Container_Boundary(layerBoundary, "Layered Model Space") {
    Container(vocabularyLayer, "Vocabulary Layer", "Terminology registry")
    Container(factLayer, "Fact Model Layer", "Structural relationships")
    Container(ruleLayer, "Rule Layer", "Constraints & policies")
  }
  Rel(businessAnalyst, vocabularyLayer, "Defines terms and glossaries")
  Rel(domainArchitect, factLayer, "Builds reusable relationships")
  Rel(developer, ruleLayer, "Author policies and checks")
  Rel(ruleLayer, factLayer, "Validates structural references")
  Rel(factLayer, vocabularyLayer, "Reuses approved terminology")
  Rel(seaDsl, ruleLayer, "Executes rule evaluations")
```

- Related: [Decision evaluation flow](ADR-001-flow-rationale-evaluation.md)
