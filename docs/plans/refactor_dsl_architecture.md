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

## 2. [ ] DSL Layering (Profiles)

**Goal**: Prevent the DSL from becoming a "kitchen sink" by enforcing profiles for different use cases (Core, Policy, Observability, etc.).

### Tasks

- [ ] **Define Profiles**:
  - Create an enum `DslProfile` in `sea-core/src/parser/mod.rs`:
    - `Core`: Entity, Resource, Flow, Instance, Role, Relation, Namespace.
    - `Policy`: Adds Policy, Expression.
    - `Observability`: Adds Metric.
    - `Evolution`: Adds ConceptChange.
    - `Integration`: Adds Mapping, Projection.
    - `Full`: All of the above.
- [ ] **Update `ParseOptions`**:
  - Add `active_profiles: Vec<DslProfile>` (or bitflags) to `ParseOptions`.
- [ ] **Implement Profile Validation**:
  - Create a validator (e.g., `validate_profile(ast: &Ast, profiles: &[DslProfile]) -> Result<(), ValidationError>`).
  - In `parse_to_graph` (or `ast_to_graph_with_options`), call this validator.
  - Reject AST nodes that do not belong to the active profiles.
- [ ] **Update CLI**:
  - Add `--profile` flag to `sea validate` and `sea parse` commands.
  - Default to `Full` (or `Core` if we want to be strict) if not specified.

## 3. [ ] Policy & Expression Hardening

**Goal**: Avoid "code in DSL" complexity and ensure policies are debuggable.

### Tasks

- [ ] **Define "Core Policy" Subset**:
  - Identify a safe, fast subset of expressions (e.g., simple comparisons, existence checks) suitable for business users.
  - Mark complex features (quantifiers, windows) as "Advanced".
- [ ] **Add Policy Tracing**:
  - Implement a tracing mechanism in `sea-core/src/policy/` to log evaluation steps.
  - Show which entities/resources were accessed during evaluation.
  - Output: "Policy 'X' evaluated to True because Entity 'Y' has field 'Z' = 10".

## 4. [ ] Canonical Graph & Isomorphism

**Goal**: Reinforce `Graph` as the source of truth.

### Tasks

- [ ] **Verify Round-Trip**:
  - Ensure `Graph -> DSL (PrettyPrint) -> Graph` preserves all semantics.
  - Add a test case `tests/test_round_trip.rs` that fuzzes this cycle.
- [ ] **Update Python Bindings**:
  - Ensure `sea_dsl.Graph` exposes the new profile options if needed.

## 5. [ ]Documentation & Golden Paths

**Goal**: Provide clear, working examples for each profile.

### Tasks

- [ ] **Create Golden Examples**:
  - `examples/golden/core.sea`: Pure structural model.
  - `examples/golden/policy.sea`: Structure + Rules.
  - `examples/golden/observability.sea`: Structure + Metrics.
  - `examples/golden/evolution.sea`: Structure + ConceptChange.
- [ ] **CI Integration**:
  - Add a step in `just all-tests` (or `scripts/test-all.sh`) to parse and validate these golden examples.

## Implementation Steps for Agent

1. **Fix Grammar/Examples**: Start by cleaning up `sea.pest` and `examples/`. Run `cargo test` to ensure no regressions.
2. **Implement Profiles**: Modify `ParseOptions` and add the validation logic. Add unit tests for profile rejection.
3. **Update CLI**: Expose the profile flag.
4. **Refactor Policy**: Add tracing and document the subset.
5. **Final Verification**: Run the round-trip tests and CI checks.
