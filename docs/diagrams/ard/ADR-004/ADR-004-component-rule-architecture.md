---
adr: ADR-004
title: "SBVR-Aligned Rule Expression Language"
diagram_type: C4Component
section: Decision
description: "Core components enabling SBVR-aligned rules"
generated: 2025-11-01
---

# ADR-004 — Rule Architecture Components

Component view outlining how the SBVR-aligned language satisfies decision goals.

```mermaid
%% Source: ADR-004 - SBVR-Aligned Rule Expression Language
%% Generated: 2025-11-01
%% Section: Decision
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-004 Components — SBVR-Aligned Rule Stack
  Container(rulePlatform, "SEA Rule Platform", "Rust + DSL", "Executes SBVR-style policies")
  Component(lexParser, "Controlled Language Parser", "Parsing", "Transforms sentences into AST")
  Component(modalEvaluator, "Modal Evaluator", "Logic Engine", "Evaluates obligation/permission/prohibition")
  Component(quantifierEngine, "Quantifier Engine", "Logic Engine", "Expands forall/exists over graph")
  Component(explanationSvc, "Explainability Service", "Narrative Engine", "Generates human-readable explanations")
  Component(sbvrExporter, "SBVR Exporter", "Interoperability", "Produces SBVR XMI/XML")
  Rel(lexParser, modalEvaluator, "Builds modal statements")
  Rel(lexParser, quantifierEngine, "Generates quantifier clauses")
  Rel(modalEvaluator, explanationSvc, "Provides decision traces")
  Rel(quantifierEngine, modalEvaluator, "Supplies quantifier results")
  Rel(sbvrExporter, lexParser, "Uses normalized AST")
```

- Related: [Rule pipeline implementation](ADR-004-component-rule-pipeline.md)
