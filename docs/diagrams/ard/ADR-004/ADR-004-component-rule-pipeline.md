---
adr: ADR-004
title: "SBVR-Aligned Rule Expression Language"
diagram_type: C4Component
section: Implementation
description: "Rule processing pipeline components"
generated: 2025-11-01
---

# ADR-004 — Rule Pipeline Components

Implementation components enforcing semantics and interoperability for SBVR-aligned rules.

```mermaid
%% Source: ADR-004 - SBVR-Aligned Rule Expression Language
%% Generated: 2025-11-01
%% Section: Implementation
%% Diagram: C4Component
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Component.puml
C4Component
  title ADR-004 Implementation — Rule Processing Pipeline
  Container(authoringApi, "Authoring API", "Python/TypeScript Bindings", "Collects controlled language input")
  Component(syntaxChecker, "Syntax Checker", "Validation", "Guards SBVR grammar compliance")
  Component(astBuilder, "AST Builder", "Parser", "Produces typed abstract syntax tree")
  Component(modalNormalizer, "Modal Normalizer", "Semantic Layer", "Expands modal expressions")
  Component(executionEngine, "Execution Engine", "Runtime", "Evaluates rules against graph")
  Component(sbvrSerializer, "SBVR Serializer", "Interop", "Exports SBVR XMI/XML")
  Component(testHarness, "Rule Test Harness", "Automated QA", "Runs scenario-based validation")
  Rel(authoringApi, syntaxChecker, "Validates input")
  Rel(syntaxChecker, astBuilder, "Produces parse tokens")
  Rel(astBuilder, modalNormalizer, "Builds semantic clauses")
  Rel(modalNormalizer, executionEngine, "Provides normalized logic")
  Rel(executionEngine, testHarness, "Delivers evaluation results")
  Rel(astBuilder, sbvrSerializer, "Feeds interoperable export")
```

- Related: [Rule evaluation sequence](ADR-004-sequence-rule-evaluation.md)
