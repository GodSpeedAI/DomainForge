---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: C4Context
section: Impact
description: "Ecosystem impact across tooling and stakeholders"
generated: 2025-11-01
---

# ADR-001 — Impacted Ecosystem Context

This context view highlights systems and stakeholders affected by the layered architecture decision.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Impact
%% Diagram: C4Context
%% !include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/main/C4_Context.puml
C4Context
  title ADR-001 Impact — Interoperability Footprint
  Person(qaLead, "QA Lead", "Receives automated impact reports")
  Person(auditor, "Compliance Auditor", "Verifies dependency rules")
  System(seaDsl, "SEA DSL", "Layered modeling platform")
  System_Ext(sbvrTooling, "SBVR Tooling", "Eclipse MDT / IBM ODM")
  System_Ext(impactDash, "Impact Dashboard", "Change propagation visualizer")
  System_Ext(lintCi, "CI Linting Pipeline", "Architecture rule enforcement")
  Rel(seaDsl, sbvrTooling, "Shares vocabulary/fact exports")
  Rel(seaDsl, lintCi, "Publishes layer constraint checks")
  Rel(lintCi, qaLead, "Flags cross-layer violations")
  Rel(impactDash, qaLead, "Displays propagation scenarios")
  Rel(sbvrTooling, auditor, "Provides compliance traceability")
  Rel(seaDsl, impactDash, "Feeds dependency graphs")
```

- Related: [Impact propagation flow](ADR-001-flow-impact-propagation.md)
