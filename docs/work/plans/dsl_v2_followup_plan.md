# DSL v2 Follow-up Implementation Plan

## Goal

Complete the integration of DSL v2 features by updating documentation, adding advanced validations, and exposing new primitives in Python and TypeScript bindings.

## User Review Required

- **Validation Strictness**: Confirm if instance property type checking should be strict (error on mismatch) or lenient (warning).
- **Binding Scope**: Confirm if full CRUD support is needed in bindings or just read-access for now.

## Proposed Changes

### 1. Documentation Updates

- **Update `docs/dsl/syntax.md`** (or equivalent):
  - Document `Role` declaration syntax.
  - Document `Relation` declaration syntax.
  - Document `Instance` declaration syntax.
  - Document `via flow` usage in relations.
  - Document flow units and metadata annotations (`collation`, `asof`, `severity`).
- **Update `README.md`**:
  - Highlight new v2 features.

### 2. Advanced Validations

- **Instance Property Validation**:
  - Check if properties defined in `Instance` match the schema/expectations of the `Resource` (if applicable) or `Entity`. _Note: Currently resources don't define property schemas in DSL, so this might be limited to JSON syntax validation or future schema support._
  - **Proposed**: Validate that `Instance` refers to valid `Entity` and `Resource` (already done in parsing, but maybe add semantic checks).
  - **New**: Validate that `Relation` subject/object roles belong to the correct entities if inferred from context (though `Role` decl makes this explicit).
- **Cycle Detection**:
  - Check for cycles in relations if they imply dependencies.

### 3. Language Bindings

#### Python (`sea-core/src/python/`)

- **Update `primitives.rs`**:
  - Implement `PyRole` struct and methods.
  - Implement `PyRelation` struct and methods.
  - Implement `PyInstance` struct and methods.
- **Update `graph.rs`**:
  - Expose `add_role`, `get_role`, `roles` accessor.
  - Expose `add_relation`, `get_relation`, `relations` accessor.
  - Expose `add_instance`, `get_instance`, `instances` accessor.
- **Update `sea_dsl` module**:
  - Register new classes.

#### TypeScript (`sea-core/src/typescript/`)

- **Update `primitives.rs`**:
  - Implement `TsRole` struct (WASM/NAPI friendly).
  - Implement `TsRelation` struct.
  - Implement `TsInstance` struct.
- **Update `graph.rs`**:
  - Add methods to access roles, relations, instances.
- **Update `index.ts` (if applicable)**:
  - Export new types.

## Verification Plan

### Automated Tests

- **Documentation**: Manual review.
- **Validations**: Add unit tests in `dsl_v2_validation_tests.rs` (or extend `dsl_v2_tests.rs`).
- **Bindings**:
  - **Python**: Run `pytest` (if available) or create a script to verify new classes.
  - **TypeScript**: Run `npm test` or equivalent to verify new types.

### Manual Verification

- Verify `sea-doc` output (if applicable).
- Verify Python shell interaction.
