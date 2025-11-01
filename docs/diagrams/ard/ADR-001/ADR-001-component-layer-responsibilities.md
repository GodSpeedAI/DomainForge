---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: C4Component
section: Decision
description: "Component responsibilities within chosen layers"
generated: 2025-11-01
---

# ADR-001 — Component Responsibilities

Component interactions clarify how each layer in the selected architecture satisfies the decision rationale.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-001 Components — Separation of Concerns
  Container(seaCore, "SEA Core", "Rust", "DomainForge runtime")
  Component(vocabRegistry, "Vocabulary Registry", "Vocabulary Layer", "Manages business terms and namespaces")
  Component(factIndexer, "Fact Model Indexer", "Fact Model Layer", "Maintains structural relationships")
  Component(ruleEngine, "Rule Engine", "Rule Layer", "Evaluates constraints and policies")
  Component(traceService, "Traceability Service", "Shared Service", "Propagates change impact reports")
  Rel(seaCore, vocabRegistry, "Provides persistence and validation")
  Rel(vocabRegistry, factIndexer, "Supplies controlled terms")
  Rel(factIndexer, ruleEngine, "Exposes relationship graph")
  Rel(ruleEngine, traceService, "Publishes evaluation outcomes")
  Rel(traceService, vocabRegistry, "Signals term impact")
```

- Related: [Implementation dependency view](ADR-001-component-layer-dependencies.md)
