# Evolution Semantics Implementation Plan

## Goal

Implement versioning and migration support in the DomainForge DSL to allow tracking changes in entities and concepts over time, enabling semantic drift detection and managed evolution.

## Problem

Currently, the DSL has no support for versioning or migration. Changes to entities or concepts overwrite previous definitions without history, making it impossible to track semantic drift or manage breaking changes in a controlled manner.

## Proposed Syntax

### Entity Versioning

Entities can now have a version suffix and metadata about what they replace and what changed.

```sea
Entity "Vendor" v2.1.0
  @replaces "Vendor" v2.0.0
  @changes ["added credit_limit field", "removed legacy_id"]
  in "procurement"
```

### Concept Change Tracking

A new top-level declaration `ConceptChange` to explicitly model migrations.

```sea
ConceptChange "Vendor_v2_migration"
  @from_version "2.0.0"
  @to_version "2.1.0"
  @migration_policy mandatory
  @breaking_change true
```

## Implementation Steps

### 1. Grammar (`sea.pest`)

- Update `entity_decl` to accept an optional version identifier (e.g., `v<semver>`).
- Add `@replaces` and `@changes` annotations to the allowed annotations for entities.
- Add a new top-level rule `concept_change_decl` with fields:
  - `@from_version`
  - `@to_version`
  - `@migration_policy`
  - `@breaking_change`

### 2. AST (`ast.rs`)

- Update `EntityDecl` struct to include:
  - `version: Option<String>`
  - `replaces: Option<String>`
  - `changes: Vec<String>`
- Create a new struct `ConceptChangeDecl`:
  - `name: String`
  - `from_version: String`
  - `to_version: String`
  - `migration_policy: MigrationPolicy` (enum: Mandatory, Optional, etc.)
  - `breaking_change: bool`
- Add `ConceptChange` to the `Declaration` enum.

### 3. Semantics (`validator.rs`)

- Implement version resolution logic:
  - Ensure `v<semver>` follows Semantic Versioning rules.
  - Validate that `@replaces` points to a valid existing version of the same entity.
- Detect semantic drift:
  - Compare fields between versions.
  - Warn if `@breaking_change` is false but fields were removed or types changed incompatibly.

### 4. Knowledge Graph Store (KGS)

- Update the graph schema to store version history.
- Enable temporal queries to fetch specific versions of an entity.
- Store `ConceptChange` nodes linking entity versions.

### 5. Projections

- **CALM**: Export version metadata and migration paths.
- **KG**: Map versions to `owl` and `rdf` vocabularies.
- **SBVR**: Represent version transitions in business vocabulary.

### 6. Bindings

- Update Python and TypeScript APIs to expose:
  - Version information on `Entity` objects.
  - `ConceptChange` objects.
  - Methods to query history or check compatibility.

## Verification Plan

### Automated Tests

- **Grammar Tests**: Verify parsing of `v2.1.0`, `@replaces`, and `ConceptChange` syntax.
- **Validation Tests**:
  - Fail if `@replaces` references a non-existent version.
  - Fail if `from_version` in `ConceptChange` doesn't match the source entity.
- **Migration Test**: Load v2.0.0 and v2.1.0 side-by-side, verify they coexist and are linked.
- **Breaking Change Detection**: Create a test case where a field is removed without marking `@breaking_change true`, expecting a warning or error.

### Manual Verification

- Create a sample `evolution.sea` file with multiple versions of an entity.
- Run `sea project` and inspect the output (CALM/KG) to ensure version metadata is correctly propagated.
