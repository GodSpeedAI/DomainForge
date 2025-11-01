---
adr: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
diagram_type: Flowchart
section: Governance
description: "Governance workflow ensuring compliance"
generated: 2025-11-01
---

# ADR-001 — Governance Controls Flow

The governance workflow enforces reviews and audit logging for layered architecture decisions.

```mermaid
%% Source: ADR-001 - Layered Architecture: Vocabulary, Facts, Rules
%% Generated: 2025-11-01
%% Section: Governance
%% Diagram: Flowchart
graph TD
  A[Propose vocabulary/fact change] --> B[Run architecture linter]
  B -->|Violations| C[Remediate dependencies]
  B -->|Pass| D[Architect review]
  D -->|Reject| C
  D -->|Approve| E[Audit log update]
  E --> F[Publish change to CI]
  F --> G[Notify stakeholders]
```

- Related: [Impact ecosystem context](ADR-001-context-impact-ecosystem.md)
