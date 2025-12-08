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

### Casting semantics and validation

- **Compatibility matrix**:
  - Quantity ↔ Quantity (same dimension): allowed, with conversion via the registry.
  - Number → Quantity: allowed, wraps the numeric value with the target unit.
  - String/Boolean/Unknown unit: rejected with a clear diagnostic.
  - Dimension mismatch (e.g., duration → currency): rejected.
- **Conversion mechanics**:
  - Use a canonical base unit per dimension with deterministic scale factors from the registry.
  - Perform arithmetic with `Decimal` to avoid float rounding; no silent truncation.
  - Unknown or non-convertible unit pairs surface an evaluation error.
- **Failure modes**:
  - Unsupported casts return a deterministic error message that names the offending unit/value.
  - Parsing/typecheck can pre-reject obviously invalid casts; runtime still guards conversions.
- **Type inference**:
  - Successful casts return a concrete quantity typed with the requested unit, not `Any`, and downstream checks propagate that concrete type.
  - Example: `1000 "ms" as "s"` produces `1 "s"` with the target unit recorded for later comparisons.

#### Parser contract for cast expressions

- `cast_expr` is considered the outermost form of `primary_expr` (i.e., `primary_expr = { cast_expr | ... }`) so the `as` suffix binds tighter than binary operators but looser than atom-level access (field/index) and is left-associative for chains like `value as "ms" as "s"`.
- The grammar accepts any string literal after `as` so validation is deferred until evaluation; the parser only ensures the operand itself is a valid `primary_expr`.
- The AST carries the literal verbatim (`Expression::Cast { operand, target_type }`) and keeps the operand boxed so any downstream stage can further inspect or optimize before the cast runs.

#### Evaluator validation and type propagation

- The evaluator must resolve the target unit at runtime by looking it up in the global unit registry; if the string literal does not name a registered unit, return an `EvalError` (e.g., `UnknownUnit`) that mentions the missing symbol so diagnostics stay actionable.
- After a successful conversion the expression result should include concrete unit/type metadata (for example, wrapping the numeric value into a `Quantity { value, unit }` structure or annotating the evaluation context) so downstream policies and projections can reason about the actual dimension rather than treating the value as untyped.
- Unknown units, mismatched dimensions, or forbidden conversions must fail at evaluation time, not at parse time, because the registry contents and user-defined units are only known once the runtime context is available.

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

### Profile enforcement rules

- **Default and precedence**:
  - Active profile = `profile:` declared in the file, else project-level option, else `"default"`.
  - `"default"` is permissive and allows all core/cloud/data constructs; additional profiles may restrict keywords.
- **Error semantics**:
  - Profile violations are hard parse errors by default.
  - Parser option `tolerate_profile_warnings` can downgrade violations to warnings for soft-rollout; violations are still logged.
- **Conflict resolution**:
  - Explicit file profile > project-level > default.
  - If multiple profiles define rules for a keyword, explicit allow wins over deny; tie breaks deterministically by precedence above.
- **Parser API**:
  - `ParseOptions` carries `active_profile` and `tolerate_profile_warnings`.
  - Diagnostics include the requested profile and the list of available profiles from `ProfileRegistry`.

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
