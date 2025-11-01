# Diagram Planning Notes

## 01 Overview & Context
- Diagram: prd-overview-context-system-boundaries (C4Context)
  - Highlight SEA DSL boundary containing Rust Core, Expression Engine, Validation Engine, Language Bindings.
  - External actors: Business Analyst, Enterprise Architect, Compliance Officer, Platform Engineer, Data Scientist.
  - External systems: ERP5 UBM, FINOS CALM, CI/CD Pipeline, Analytics Tooling, Next.js Apps.
  - Interfaces emphasise cross-language parity and migration channels.

## 02 Objectives & Success Metrics
- Diagram A: prd-objectives-goal-hierarchy (Flowchart) mapping top-level goal -> pillars -> metrics.
- Diagram B: prd-objectives-metrics-mapping (Flowchart) linking metrics to measurement methodology / owner.

## 03 Stakeholders & Roles
- Diagram: prd-stakeholders-decision-map (Flowchart) showing RACI-coded relationships among Product Manager, Platform Engineer, Analysts, Compliance, QA Lead.

## 04 User Context
- Diagram: prd-user-context-assumptions (Flowchart) capturing environmental assumptions (cross-language parity, regulated domain, offline modeling) and validation requirements. Flag missing assumptions as needed.

## 05 Scope & Out-of-Scope
- Diagram A: prd-scope-system-boundary (C4Context) labeling in-scope modules vs external/out-of-scope concerns (e.g., UI authoring tooling). Use System vs System_Ext for clarity.
- Diagram B: prd-scope-feature-decision-tree (Flowchart) guiding inclusion decisions; include warning about missing explicit exclusions.

## 06 Personas & Scenarios
- Diagram A: prd-personas-business-analyst-journey (journey) across phases (Onboarding, Modeling, Validation, Reporting) with satisfaction scores.
- Diagram B: prd-personas-comparison-map (Flowchart) comparing Analyst vs Compliance Officer vs Platform Engineer touchpoints and pain points.

## 07 Requirements
- Diagram A: prd-requirements-breakdown (Flowchart) grouping requirements into categories with color by EARS pattern.
- Diagram B: prd-requirements-component-mapping (C4Component) linking requirement nodes to components (Rust Core, Graph Runtime, FFI bindings, Tooling).
- Diagram C: prd-requirements-traceability (Flowchart/graph) mapping Requirement -> Component -> Test -> Metric per traceability example using Mermaid graph.
- README traceability matrix with fabricated TEST IDs aligned to acceptance criteria and metrics.

## 08 UX & Design
- Diagram A: prd-ux-sequence-model-validation (sequenceDiagram) showing Business Analyst using Python binding -> Rust core -> Validation engine -> reporting.
- Diagram B: prd-ux-sequence-streaming-validation (sequenceDiagram) covering streaming API across Python async / TypeScript callback.
- Diagram C: prd-ux-state-error-feedback (stateDiagram) for diagnostics states including edge/error flows.

## 09 Dependencies & Constraints
- Diagram A: prd-dependencies-system-map (C4Container) linking SEA containers to external dependencies with SLA/version annotations.
- Diagram B: prd-dependencies-constraint-propagation (Flowchart) showing how constraint changes propagate across layers; warn where PRD lacks numbers (e.g., SLA specifics beyond metrics).

## 10 Acceptance Criteria
- Diagram: prd-acceptance-validation-workflow (Flowchart) connecting requirements to acceptance test stages and linking back to metrics. Ensure edges labelled with REQ IDs.

## Index
- Summarize key insights per section, EARS distribution counts (from JSON), coverage checklist.
