# Plan: SEA DSL Refactoring & Architecture Hardening

This plan addresses the critique regarding the SEA DSL implementation, focusing on grammar consistency, architectural layering (profiles), and policy simplification.

## 1. [x] Grammar & Syntax Cleanup

**Goal**: Ensure the grammar is robust, consistent, and matches the documentation/examples.

### Tasks

- [x] **Review `sea.pest`**:
  - Verify `declaration_inner` rule is complete and not corrupted.
  - Confirm all keywords use case-insensitive matching (e.g., `^"entity"`).
- [x] **Standardize Casing**:
  - Decision: Keep grammar case-insensitive (`^"keyword"`) to be user-friendly.
  - Action: Enforce "Capitalized" convention (e.g., `Entity`, `Resource`) in all documentation and examples.
  - Action: Update `sea format` (if exists) or `PrettyPrinter` to output Capitalized keywords.
- [x] **Fix Examples**:
  - Scan all `.sea` files in `examples/` and `sea-core/examples/`.
  - Remove any `...` placeholders that are not valid syntax.
  - Ensure all examples parse successfully with the current grammar.

## 2. [x] Grammar Expansion (The `as` Operator)

**Goal**: Support a general-purpose casting/conversion mechanism for units and types.

### Tasks

- [x] **Update Grammar (`sea.pest`)**:
  - Modify `unary_expr` or `primary_expr` to support `as` syntax.
  - Rule: `cast_expr = { primary_expr ~ (^"as" ~ string_literal)? }`.
- [x] **Update AST (`sea-core/src/parser/ast.rs`)**:
  - Add `Expression::Cast(Box<Expression>, String)` variant.
- [x] **Update Parser (`sea-core/src/parser/mod.rs`)**:
  - Map the new grammar rule to the AST variant.
- [x] **Update Evaluator (`sea-core/src/policy/evaluation.rs`)**:
  - Implement `eval_cast` logic (initially for unit conversion, e.g., `1000 "ms" as "s"`).

## 3. [x] DSL Layering (Profiles)

**Goal**: Introduce "Profiles" to allow different DSL dialects (e.g., `profile: "cloud"`, `profile: "data"`).

### Tasks

- [x] **Create Profile Registry (`sea-core/src/parser/profiles.rs`)**:
  - Define `Profile` struct (name, enabled keywords/features).
  - Create a registry of built-in profiles.
- [x] **Update Parser Context**:
  - Pass the active profile into the parser.
  - Validate that used keywords/constructs are allowed in the current profile.
- [x] **Add `profile` Keyword to Grammar**:
  - Allow `profile: "name"` at the top of the file.

## 4. [x] Standard Library (Modules)

**Goal**: Move built-in types (like `HTTP`, `REST`) out of the compiler and into a standard library.

### Tasks

- [x] **Create Standard Library Files**:
  - `sea-core/std/core.sea` (Base types).
  - `sea-core/std/http.sea` (HTTP/REST types).
  - `sea-core/std/aws.sea` (AWS types).
- [x] **Embed Stdlib in Binary**:
  - Use `include_str!` to embed these files in the Rust binary.
- [x] **Update Module Resolver**:
  - Modify `sea-core/src/module/resolver.rs` to handle `std:` imports (e.g., `import { Service } from "std:core"`).
  - When resolving `std:*`, return the embedded content.

## 5. [x] Canonical Graph & Isomorphism

**Goal**: Reinforce `Graph` as the source of truth.

### Tasks

- [x] **Verify Round-Trip**:
  - Ensure `Graph -> DSL (PrettyPrint) -> Graph` preserves all semantics.
  - Add a test case `tests/test_round_trip.rs` that fuzzes this cycle.

## Implementation Steps for Agent

1. **Grammar Expansion**: Implement the `as` operator in `sea.pest` and AST.
2. **Implement Profiles**: Create the `DslProfile` enum and integrate it into `ParseOptions`.
3. **Profile Validation**: Enforce profile constraints during parsing.
4. **Policy Tracing**: Add the tracing context to the evaluator.
5. **Final Verification**: Run all tests.
