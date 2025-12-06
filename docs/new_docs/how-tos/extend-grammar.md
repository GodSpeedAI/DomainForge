# Extend Grammar

Goal: Extend the SEA grammar in `sea-core/grammar/sea.pest`, update the parser/AST, and refresh tests across languages.

## Prerequisites

- Rust toolchain with `cargo` and `just` available.
- Familiarity with Pest grammar syntax and the existing rules in `sea-core/grammar/sea.pest`.
- Ability to run Rust/Python/TypeScript tests (`just all-tests`) to ensure binding parity.

## Steps (be concise)

1. **Plan the syntax change**

   - Identify the new construct (e.g., `Constraint`, `Annotation`).
   - Decide whether it is a statement, expression, or modifier of existing rules.
   - Sketch the EBNF and how it should appear in AST structs.

2. **Update the grammar**

   ```bash
   sed -n '1,160p' sea-core/grammar/sea.pest  # review nearby rules
   ```

   - Add or modify rules using Pest syntax. Prefer `silent` rules (`_`) for whitespace handling.
   - Keep tokens unambiguous; if you add keywords, update `identifier` to prevent collisions.

3. **Regenerate parser expectations**

   - If the change affects parsing order, adjust combinators in `sea-core/src/parser.rs` or helpers in `sea-core/src/ast.rs`.
   - Update AST types to include new fields (e.g., add `Annotation` struct) and ensure `FromStr` implementations parse them.

4. **Propagate to projections**

   - **CALM/KG/SBVR**: Update projectors (`sea-core/src/calm/mod.rs`, `sea-core/src/kg.rs`, `sea-core/src/sbvr.rs`) so the new construct is emitted.
   - **Validation**: Add semantic checks in `sea-core/src/validator.rs` to keep error reporting meaningful.

5. **Expose through bindings**

   - Python: Add new fields/classes in `sea-core/src/python/primitives.rs` and wire them into `graph.rs` if they are part of the `Graph` API.
   - TypeScript: Mirror changes in `sea-core/src/typescript/primitives.rs` and adjust `index.d.ts` typing.
   - WASM: Update `sea-core/src/wasm/` exports if browser consumers need the feature.

6. **Add parser tests (Rust)**

   ```bash
   cargo test -p sea-core parser_tests -- --nocapture
   ```

   - Create or edit fixtures under `sea-core/tests/` or `sea-core/examples/` to cover the new syntax and failure cases.
   - Add golden tests if the construct affects export formats.

7. **Add binding tests (Python + TypeScript)**

   - Python: write a `tests/test_<feature>.py` that parses the DSL snippet and asserts field values.
   - TypeScript: add a Vitest case under `typescript-tests/` that mirrors the Python assertion to maintain parity.

8. **Run cross-language suites**

   ```bash
   just rust-test
   just python-test
   just ts-test
   ```

   - Use `just all-tests` to run everything sequentially before committing.

9. **Document the new syntax**

   - Update `docs/new_docs/reference/grammar-spec.md` with the new rule and an example.
   - Add a short section to the relevant how-to or tutorial to show usage.

## Cross-Bindings and Snapshots

- Keep serialized forms stable; if the new construct appears in CALM, adjust mapping tables and fixtures.
- Update `index.d.ts` and Python type hints to match the Rust structs so auto-completion remains accurate.
- Regenerate any snapshot tests that capture textual output (e.g., KG/Turtle strings) after confirming the change is intentional.

## Spec and Example Tests

- Write minimal DSL examples that highlight the new rule with and without namespaces.
- Add negative tests for malformed input to ensure errors are descriptive (use error codes from `validation_error.rs`).
- If the rule impacts policies, add evaluation tests in `sea-core/src/policy` or binding-specific test suites.

## Common Pitfalls / Troubleshooting

- Forgetting to update one binding often leads to runtime errors (`AttributeError` in Python, `undefined` in TS). Keep a checklist per language.
- Grammar precedence issues: if a new rule causes existing tests to fail, inspect operator precedence and adjust via Pest ordering or explicit `_` separators.
- When changing keywords, ensure backwards compatibility or document the breaking change with a migration note.

## Links

- Tutorials: [Extend Grammar Walkthrough](../tutorials/getting-started.md)
- Reference: [Grammar Spec](../reference/grammar-spec.md), [Primitives API](../reference/primitives-api.md), [Cross-Language Binding Strategy](../explanations/cross-language-binding-strategy.md)

## Verification Checklist

- [ ] Parser accepts the new syntax (Rust tests passing).
- [ ] CALM/KG/SBVR exports reflect the construct without dropping fields.
- [ ] Python and TypeScript bindings expose the new fields with the same naming.
- [ ] Documentation updated (how-to + grammar spec) with a runnable example.
- [ ] Version bumped or changelog entry added if the change is user-visible.

## Example Workflow (adding a `Note` keyword)

1. Add grammar rule:

   ```pest
   note = { "Note" ~ string }
   statement = { role_decl | relation_decl | note | flow_decl | resource_decl | entity_decl }
   ```

2. Update AST:

   ```rust
   pub struct Note { pub text: String }
   ```

3. Wire projection:

   - CALM: add `notes` array under model metadata.
   - KG: emit as annotation triples.

4. Add tests:

   - Rust: `tests/note_tests.rs` covering parse success/failure.
   - Python: assert `graph.notes()[0].text == "Payment delayed"`.
   - TypeScript: similar Vitest assertion.

5. Document:

   - Add a snippet to `grammar-spec.md` and this how-to.
   - Mention migration steps if the keyword conflicts with existing identifiers.
