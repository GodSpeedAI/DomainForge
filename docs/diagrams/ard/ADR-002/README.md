---
adr: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
generated: 2025-11-01
---

# ADR-002 Diagram Index

- `ADR-002-context-multilanguage-ecosystem.md` — Context: Depicts stakeholders and bindings consuming the Rust core.
- `ADR-002-flow-context-forces.md` — Context: Flow of performance and accessibility pressures behind the FFI decision.
- `ADR-002-container-runtime-options.md` — Alternatives: Compares Rust core against Python, TypeScript, and multi-implementation strategies.
- `ADR-002-flow-decision-tradeoffs.md` — Decision: Evaluates candidates against performance, safety, and consolidation criteria.
- `ADR-002-component-chosen-architecture.md` — Decision: Shows bridging components that wrap the Rust core for each language.
- `ADR-002-component-ffi-pipeline.md` — Implementation: Details CI/build components producing cross-platform bindings.
- `ADR-002-sequence-python-call.md` — Implementation: Traces a Python validation request across the FFI boundary.
- `ADR-002-flow-impact-surface.md` — Impact: Highlights adoption, maintenance, and performance outcomes.
- `ADR-002-context-impact-platforms.md` — Impact: Contextualizes distribution platforms affected by the consolidated core.
- `ADR-002-flow-governance-release.md` — Governance: Documents approval and audit workflow for synchronized releases.
