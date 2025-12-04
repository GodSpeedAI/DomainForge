# Implementation Plan - Instance Declarations

This plan addresses **Item 7: Instance Declarations** from the `dsl-completeness-roadmap.md`.

## Goal Description

Enable the definition of concrete data instances in the DSL. This allows users to instantiate Entities with specific data values, which can then be referenced in policies and flows.

**Problem**: Currently, the DSL allows defining `Entity` types (classes) but not specific instances of those entities (objects).
**Solution**: Add `Instance` declarations to the DSL, allowing users to define named instances of an Entity with specific field values.

## Refactoring (Breaking Change)

> [!IMPORTANT] > **Renaming Existing Primitive**: The existing `Instance` primitive (representing a Resource at a Location) will be renamed to `ResourceInstance` to avoid ambiguity and technical debt.
> **New Primitive**: The new primitive for Entity Instances will be named `Instance` in both the DSL and the Rust codebase.

## Proposed Changes

### 1. Refactor Existing `Instance`

#### [RENAME] `sea-core/src/primitives/instance.rs` -> `sea-core/src/primitives/resource_instance.rs`

- Rename struct `Instance` to `ResourceInstance`.
- Update all references in:
  - `sea-core/src/primitives/mod.rs`
  - `sea-core/src/lib.rs`
  - `sea-core/src/graph/mod.rs` (if used)
  - `sea-core/src/python/primitives.rs` (expose as `ResourceInstance`)
  - `sea-core/src/typescript/primitives.rs` (expose as `ResourceInstance`)
  - Tests.

### 2. Implement New `Instance`

#### [MODIFY] [sea.pest](file:///home/sprime01/projects/domainforge/sea-core/grammar/sea.pest)

- Add `instance_decl` rule.
- Update `declaration` rule.
- Add `instance_reference` syntax.

```pest
// Instance Declaration
// Syntax:
// Instance vendor_123 of "Vendor" {
//   name: "Acme Corp",
//   credit_limit: 50_000 "USD"
// }
instance_decl = {
    ^"Instance" ~ name ~ ^"of" ~ string_literal ~ instance_body?
}

instance_body = {
    "{" ~ (instance_field ~ ","?)* ~ "}"
}

instance_field = {
    identifier ~ ":" ~ expression
}

// Update primary_expr to include instance reference
primary_expr = { ... | instance_reference }
instance_reference = @{ "@" ~ identifier }
```

#### [MODIFY] [sea-core/src/parser/ast.rs](file:///home/sprime01/projects/domainforge/sea-core/src/parser/ast.rs)

- Add `Instance` variant to `AstNode`.

```rust
Instance {
    name: String,
    entity_type: String,
    fields: HashMap<String, Expression>,
    annotations: HashMap<String, JsonValue>,
}
```

#### [MODIFY] [sea-core/src/parser/mod.rs](file:///home/sprime01/projects/domainforge/sea-core/src/parser/mod.rs)

- Implement `parse_instance` function.

#### [NEW] [sea-core/src/primitives/instance.rs](file:///home/sprime01/projects/domainforge/sea-core/src/primitives/instance.rs)

- Define new `Instance` struct (Entity Instance).

```rust
pub struct Instance {
    pub id: ConceptId,
    pub entity_type: String, // Reference to Entity name
    pub fields: HashMap<String, Value>, // Evaluated fields
    pub annotations: HashMap<String, Value>,
}
```

#### [MODIFY] [sea-core/src/primitives/mod.rs](file:///home/sprime01/projects/domainforge/sea-core/src/primitives/mod.rs)

- Export new `Instance`.

### 3. Bindings Updates

#### [MODIFY] [sea-core/src/python/primitives.rs](file:///home/sprime01/projects/domainforge/sea-core/src/python/primitives.rs)

- Rename existing `Instance` class to `ResourceInstance`.
- Add new `Instance` class.

#### [MODIFY] [sea-core/src/typescript/primitives.rs](file:///home/sprime01/projects/domainforge/sea-core/src/typescript/primitives.rs)

- Rename existing `Instance` class to `ResourceInstance`.
- Add new `Instance` class.

## Verification Plan

### Automated Tests

- **Refactor Verification**:
  - Ensure existing tests pass after renaming (fix compilation errors).
- **Parser Tests**:
  - Test parsing of valid `Instance` declarations.
  - Test parsing of instance references.
- **Integration Tests**:
  - Verify `Instance` creation and field access.
  - Verify `ResourceInstance` still works as expected.

### Manual Verification

- Run `cargo test`.
- Run `just test-python` and `just test-ts`.
