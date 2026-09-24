# Plan: Repository Documentation Architecture

**Approved:** 2026-09-02
**Context:** Creating a comprehensive, self-contained technical knowledge system for DomainForge adhering to Diátaxis, DeepWiki, and Google Code Wiki principles.

## Objectives
1. Document the repository for both newcomers (progressive disclosure, clear mental models) and experienced engineers (subsystem depth, invariants, source traceability, failure modes).
2. Deliver key architectural deliverables: `documentation-map.md`, `source-map.md`, `architecture.md`.
3. Establish a progressive 9-layer knowledge system (Layer 0 to Layer 8 + Troubleshooting).
4. Connect every architectural assertion to repository source files and symbols with explicit Source Trails.

## Checklist of Deliverables
- [ ] Deliverable: `docs/documentation-map.md`
- [ ] Deliverable: `docs/source-map.md`
- [ ] Deliverable: `docs/architecture.md` (and root `architecture.md` linking to it)
- [ ] Layer 0 (Orientation): `docs/index.md`, `docs/orientation.md`
- [ ] Layer 1 (Mental Model): `docs/mental-model.md`
- [ ] Layer 3 (Subsystems):
  - [ ] `docs/subsystems/parser-grammar.md`
  - [ ] `docs/subsystems/graph-store.md`
  - [ ] `docs/subsystems/policy-engine.md`
  - [ ] `docs/subsystems/units-dimensions.md`
  - [ ] `docs/subsystems/semantic-packs.md`
  - [ ] `docs/subsystems/authority-engine.md`
  - [ ] `docs/subsystems/application-contracts.md`
  - [ ] `docs/subsystems/projections-engine.md`
  - [ ] `docs/subsystems/language-bindings.md`
- [ ] Layer 4 (Workflows):
  - [ ] `docs/workflows/parse-and-validate.md`
  - [ ] `docs/workflows/projection-generation.md`
  - [ ] `docs/workflows/pack-lifecycle.md`
  - [ ] `docs/workflows/application-resolution.md`
  - [ ] `docs/workflows/authority-evaluation.md`
- [ ] Layer 5 (Explanations):
  - [ ] `docs/explanations/canonical-semantic-core.md`
  - [ ] `docs/explanations/indexmap-determinism.md`
  - [ ] `docs/explanations/three-valued-logic-rationale.md`
  - [ ] `docs/explanations/adr-013-application-contract.md`
  - [ ] `docs/explanations/operator-family-design.md`
- [ ] Layer 6 (Tutorials):
  - [ ] `docs/tutorials/01-first-sea-model.md`
  - [ ] `docs/tutorials/02-multi-target-projection.md`
  - [ ] `docs/tutorials/03-building-signing-packs.md`
- [ ] Layer 7 (How-Tos):
  - [ ] `docs/how-tos/add-projection-target.md`
  - [ ] `docs/how-tos/add-grammar-construct.md`
  - [ ] `docs/how-tos/configure-module-resolution.md`
  - [ ] `docs/how-tos/debug-policy-evaluations.md`
- [ ] Layer 8 (Reference):
  - [ ] `docs/reference/dsl-grammar-reference.md`
  - [ ] `docs/reference/cli-reference.md`
  - [ ] `docs/reference/error-code-reference.md`
  - [ ] `docs/reference/primitives-api-reference.md`
  - [ ] `docs/reference/configuration-reference.md`
- [ ] Troubleshooting: `docs/troubleshooting.md`
- [ ] Verification and cross-link validation
