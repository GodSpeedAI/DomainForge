# DSL v2 Implementation Summary

## Overview

This document summarizes the changes made to implement the DSL v2 enhancements as outlined in `docs/work/plans/dsl_v2_plan.md`. The goal was to update the grammar and AST to support roles, relations, instances, flow-level units, and policy severity, along with necessary validations.

## Changes

### 1. Grammar (`sea.pest`)

- **Roles**: Added `role_decl` rule: `Role "Name" for entity "Entity"`.
- **Relations**: Added `relation_decl` rule with support for `subject role`, `object role`, and `via flow`.
- **Instances**: Added `instance_decl` rule: `Instance "Name" of resource "Resource" at entity "Entity" { properties }`.
- **Flow Units**: Updated `flow_decl` to support optional unit string: `quantity 100 "USD"`.
- **Metadata**: Added `collation` and `asof` to file-level annotations, and `severity` to policy annotations.

### 2. AST (`src/parser/ast.rs`)

- **New Nodes**: Added `Role`, `Relation`, and `Instance` variants to `AstNode` enum.
- **Metadata Structs**: Updated `FileMetadata` with `collation` and `as_of`, and `PolicyMetadata` with `severity`.
- **Parsing Logic**: Implemented `parse_role`, `parse_relation`, and `parse_instance` functions.
- **Graph Population**: Updated `ast_to_graph_with_options` to populate the graph with new nodes in the correct dependency order (Entities/Resources -> Roles/Instances -> Flows -> Relations -> Policies).

### 3. Primitives (`src/primitives/`)

- **Role**: Created `role.rs` to define the `Role` struct.
- **Relation**: Created `relation.rs` to define the `Relation` struct. Note: `via_flow` in DSL is mapped to `via_resource_id` in the struct, as it refers to the resource being flowed.

### 4. Graph (`src/graph/mod.rs`)

- **Storage**: Added `roles` and `relations` maps to the `Graph` struct.
- **Methods**: Added methods to add, get, remove, and list roles and relations.
- **Validation**: Implemented validation in `add_relation` to ensure that if a relation implies a flow (via `via flow "Resource"`), a corresponding flow exists between the entities of the subject and object roles.

### 5. Testing (`tests/dsl_v2_tests.rs`)

- **Parsing Tests**: Verified parsing of all new constructs.
- **Graph Population Tests**: Verified that the graph is correctly populated from AST.
- **Validation Tests**: Verified that relations with missing flows trigger validation errors.

### 6. Printer (`src/parser/printer.rs`)

- **Pretty Printing**: Updated `PrettyPrinter` to support printing `Role`, `Relation`, and `Instance` nodes, ensuring that the new DSL constructs can be formatted back to source code.

## Verification

All tests in `dsl_v2_tests.rs` passed, confirming the correctness of the implementation. Existing tests also passed, ensuring no regressions.

## Next Steps

- Update documentation to reflect the new DSL syntax.
- Consider implementing more advanced validations (e.g., checking property types for instances).
- Update Python and TypeScript bindings to support the new primitives (`Role`, `Relation`, `Instance`).
