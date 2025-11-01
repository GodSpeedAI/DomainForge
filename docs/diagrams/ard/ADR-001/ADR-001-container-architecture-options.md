---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: C4Container
section: Alternatives
description: "Comparison of evaluated architectural options"
generated: 2025-11-01
---

# ADR-001 — Architecture Options Map

This container diagram summarizes the alternative architectures evaluated before selecting the layered approach.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Alternatives
%% Diagram: C4Container
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Container.puml
C4Container
  title ADR-001 Alternatives — Architectural Containers
  System_Boundary(options, "Model Architecture Options") {
    Container(optionChosen, "Three-Layer Stack", "Vocabulary • Fact Model • Rule", "Supports SBVR alignment and dependency controls")
    Container(optionTwoLayer, "Two-Layer Schema+Rules", "Combined terminology and structure", "Lacks vocabulary evolution safety")
    Container(optionFlat, "Flat Ontology", "Single undifferentiated model", "High cognitive load; weak traceability")
  }
  ContainerDb(impact, "Change Impact", "Traceability matrix")
  Rel(optionChosen, impact, "Enables predictable propagation")
  Rel(optionTwoLayer, impact, "Obscures vocabulary changes")
  Rel(optionFlat, impact, "Breaks structural governance")
  Rel_D(optionChosen, optionTwoLayer, "Superior maintainability")
  Rel_D(optionChosen, optionFlat, "Lower cognitive load")
```

- Related: [Decision evaluation flow](ADR-001-flow-rationale-evaluation.md)
