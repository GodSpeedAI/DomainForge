# Plan: DSL v2 Enhancements

**Status**: Draft
**Priority**: High
**Goal**: Address architectural critiques to make DomainForge a complete semantic DSL (Roles, Instances, Units, Governance).

## 1. Roles and Relations (Fact Types)

**Critique**: Missing `role` and `relation` declarations to support SBVR FactTypes and Knowledge Graph predicates.

### Implementation

- **Grammar (`sea.pest`)**:

  ```pest
  declaration = { ... | role_decl | relation_decl }

  role_decl = {
      ^"role" ~ string_literal ~ ^"for" ~ ^"entity" ~ string_literal
  }

  relation_decl = {
      ^"relation" ~ string_literal ~ "{" ~
          relation_body_item* ~
      "}"
  }

  relation_body_item = {
      (^"subject" ~ ^"role" ~ string_literal) |
      (^"object" ~ ^"role" ~ string_literal) |
      (^"via" ~ ^"flow" ~ string_literal)
  }
  ```

- **AST (`ast.rs`)**:
  - Add `Role` and `Relation` variants to `AstNode`.
  - `Role { name: String, entity: String }`
  - `Relation { name: String, subject: String, object: String, flow: Option<String> }`
- **Graph (`graph/mod.rs`)**:
  - Add nodes for Roles and Relations.
  - Track dependencies (Role -> Entity, Relation -> Role/Flow).

## 2. Instance Declarations

**Critique**: Missing first-class `instance` keyword to represent specific physical instances (aligning with `instance.rs` primitive).

### Implementation

- **Grammar (`sea.pest`)**:

  ```pest
  declaration = { ... | instance_decl }

  instance_decl = {
      ^"instance" ~ string_literal ~ ^"of" ~ ^"resource" ~ string_literal ~
      ^"at" ~ ^"entity" ~ string_literal ~
      instance_body?
  }

  instance_body = {
      "{" ~ (property_assignment)* ~ "}"
  }

  property_assignment = {
      identifier ~ string_literal
  }
  ```

- **AST (`ast.rs`)**:
  - Add `Instance` variant to `AstNode`.
  - `Instance { name: String, resource: String, entity: String, properties: HashMap<String, String> }`
- **Graph**:
  - Integrate `Instance` primitive into the graph.

## 3. Flow-level Units

**Critique**: Flows implicitly use resource's base unit. Need explicit unit support for conversion/validation.

### Implementation

- **Grammar (`sea.pest`)**:
  - Update `flow_decl`:
  ```pest
  flow_decl = {
      ^"flow" ~ string_literal ~ ^"from" ~ string_literal ~ ^"to" ~ string_literal ~
      (^"quantity" ~ number ~ string_literal?)?
  }
  ```
- **AST (`ast.rs`)**:
  - Update `Flow` struct: `quantity: Option<Decimal>`, `unit: Option<String>`.
- **Validation**:
  - Check if flow unit is compatible with resource dimension.

## 4. Explicit Severity / Governance

**Critique**: Missing `@severity` annotation on policies.

### Implementation

- **Grammar (`sea.pest`)**:
  - Update `policy_annotation_name`:
  ```pest
  policy_annotation_name = {
      ^"rationale" | ^"tags" | ^"severity"
  }
  ```
- **AST (`ast.rs`)**:
  - Update `PolicyMetadata`: add `severity: Option<String>`.
- **Logic**:
  - Map string severity ("Error", "Warning", "Info") to `Severity` enum.

## 5. Collation and Temporal Context

**Critique**: Missing `@collation` and `@asof`.

### Implementation

- **Grammar (`sea.pest`)**:
  - Add file-level annotations:
  ```pest
  annotation_name = {
      ^"namespace" | ^"version" | ^"owner" | ^"collation" | ^"asof"
  }
  ```
- **AST (`ast.rs`)**:
  - Update `FileMetadata`: add `collation: Option<String>`, `as_of: Option<String>`.
- **Logic**:
  - Pass these metadata to the evaluation context.

## Execution Plan

1.  **Grammar & AST**: Update `sea.pest` and `ast.rs` for all features.
2.  **Parser**: Update `parser/mod.rs` and `parser/ast.rs` to handle new nodes.
3.  **Graph/Primitives**: Update `primitives` and `graph` to store new concepts.
4.  **Tests**: Add test cases for each new declaration type.
