# Last Mile to V1: Stabilization & Release

## Executive Summary

**Status**: Moving from "Fragmented" toward "Production-Ready" — bindings are aligned, docs refreshed, and release pipelines scaffolded.

- **Core (Rust)**: Advanced (Phase 2 features like Roles, Relations, Module System implemented).
- **Bindings (Python/TS)**: Lagging (Missing Roles, Relations, Module interfaces).
- **Meta**: Documentation is outdated; Release pipelines are scaffolded (secrets and final verification required).

**Goal**: Synchronize all layers of the stack and establish a release pipeline to achieve a "Production-Ready" Alpha/Beta state.

## Phase 1: Binding Parity (Critical)

**Objective**: Ensure specific V1 features available in Rust are accessible in Python and TypeScript.

**Outcome**: ✅ Completed (Python + TypeScript bindings updated; parity tests added).

### 1.1 Python Bindings (`sea-core/src/python`)

- [x] **Role Primitive**:
  - Wrap `sea_core::primitives::Role`.
  - Expose `new`, `attributes`, `namespace` accessors.
- [x] **Relation Primitive**:
  - Wrap `sea_core::primitives::RelationType`.
  - Expose `subject`, `predicate`, `object`, `via`.
- [x] **Tests**:
  - Added `tests/test_role_relation_parity.py` + `tests/test_golden_payment_flow.py` for bindings parity.

### 1.2 TypeScript Bindings (`sea-core/src/typescript`)

- [x] **Role Primitive**:
  - Create `Role` class (napi-rs).
- [x] **Relation Primitive**:
  - Create `Relation` class (napi-rs).
- [x] **Tests**:
  - Added `typescript-tests/role_relation.test.ts` + `typescript-tests/golden-payment-flow.test.ts`.

## Phase 2: Documentation Reconciliation

**Objective**: Ensure documentation accurately reflects the codebase to prevent "Agent Rot".

**Outcome**: ✅ Completed (roadmaps and how-tos refreshed to match current APIs).

- [x] **Update `dsl-completeness-roadmap.md`**:
  - Mark "Roles & Relations" complete across core and bindings.
  - Confirm module system status.
- [x] **Update `project_state.md`**:
  - Remove "Roles & Relations" from Gaps.
  - Call out release automation as the primary remaining risk.
- [x] **Publish How-Tos**:
  - Added actionable guides for CALM export, policy authoring, CLI install, and custom units under `docs/new_docs/how-tos/`.

## Phase 3: Release Engineering

**Objective**: Automate distribution.

**Outcome**: ✅ Release workflows and version baseline in place (secrets still required to publish).

- [x] **CI Workflows**:
  - `release-pypi.yml`: Publishes `sea-dsl` (PyPI) via maturin with Python/TS feature gates.
  - `release-npm.yml`: Publishes `@domainforge/sea` via `npm publish` after napi build.
  - `release-crates.yml`: Publishes `sea-core` to crates.io.
- [x] **Versioning Strategy**:
  - Established `0.1.0` baseline across all packages (Rust crate bumped to 0.1.0).

## Verification Strategy

- **Cross-Language Golden Tests**:
  - ✅ "Payment Role Flow" scenario covered in Rust (`roles_relations_tests.rs`), Python (`test_golden_payment_flow.py`), and TypeScript (`golden-payment-flow.test.ts`).
