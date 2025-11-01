---
prd_section: "6. User Personas & Scenarios"
diagram_type: "journey"
ears_requirements: ["PRD-001", "PRD-002", "PRD-003", "PRD-004", "PRD-012", "PRD-017", "PRD-019"]
updated: "2025-11-01"
purpose: "Captures the business analyst journey across SEA DSL touchpoints, annotating satisfaction and pain points derived from PRD user stories."
---

## PRD Section 6: Business Analyst Journey

```mermaid
journey
    title Business Analyst Journey (PRD Sections PRD-001, PRD-002, PRD-012)
    %% Source: docs/specs/prd.md - User stories for PRD-001/002/012 & success metrics
    section Onboarding
      "Install SEA tooling via Python package" as BA1:4
      "Review documentation quick start" as BA2:3
    section Modeling
      "Define vocabulary terms" as BA3:4
      "Model flows & policies" as BA4:3
    section Validation
      "Run validation and inspect violations" as BA5:2
      "Iterate with streaming feedback" as BA6:3
    section Reporting
      "Export compliance report" as BA7:3
      "Share model diffs with stakeholders" as BA8:2
```

- Pain points flagged at BA5 and BA8 align with PRD requirements for richer diagnostics (PRD-017) and documentation improvements (PRD-019).
- Satisfaction assumptions need validation with UX research; defaults set to neutral (3) where PRD lacks explicit signal.
```
