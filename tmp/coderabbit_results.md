Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

## FIX STATUS SUMMARY (2025-11-09)

### COMPLETED CRITICAL FIXES (9)
✅ Windows shell compatibility (.github/workflows/release.yml)
✅ Missing UUID import (graph_integration_tests.rs)
✅ Decimal->f64 precision loss (quantifier.rs - 3 sites)
✅ Python Flow quantity panic (python/primitives.rs)
✅ TypeScript NEG_INFINITY bug (typescript/primitives.rs)
✅ Parser JSON escaping (parser/ast.rs - 3 functions)
✅ UUID parsing duplication (Python & TypeScript - 16 sites)
✅ Redundant clippy attributes (5 removed)
✅ Backup config file (.bak.coderabbit.yml removed)

### DEFERRED (Non-Critical / Breaking Changes)
⏭️ Test assertion improvements (Nitpicks)
⏭️ Units fallback API change (Breaking - 30+ call sites)
⏭️ Documentation updates (task-06)
⏭️ WASM refactoring (Requires dedicated PR)

### TEST RESULTS
✅ cargo test --lib: 67 passed
✅ cargo test --test '*': 236 passed, 1 failed (pre-existing)

See tmp/coderabbit_fixes_summary.md for detailed report.



============================================================================
File: .github/workflows/release.yml
Line: 37
Type: potential_issue

[X] Task: COMPLETED
.github/workflows/release.yml lines 37 and 40: Split into separate Windows (pwsh) and Unix (bash) steps with conditional execution.



============================================================================
File: sea-core/tests/parser_integration_tests.rs
Line: 180 to 193
Type: nitpick

[ ] Task:
In sea-core/tests/parser_integration_tests.rs around lines 180 to 193, the test currently asserts the error by checking the error message via .to_string().contains("Duplicate"), which is fragile; update the test to match the parser's error type/kind instead (for example assert that the Err matches ParseError::DuplicateEntity or the equivalent variant exposed by the parser). Replace the string containment assertion with a pattern match or matches!() on the error enum/variant (or call a .kind() accessor if provided), and import the appropriate error type so the test asserts the specific error variant rather than message text.



============================================================================
File: sea-core/tests/phase_16_unicode_tests.rs
Line: 171
Type: nitpick

[ ] Task:
In sea-core/tests/phase_16_unicode_tests.rs around lines 171, 226, 237, and 247, replace the single-line inline assert! calls with the file's multi-line assertion style: put assert!( on its own line, then place the condition on the next line, and then place the format string and any formatting arguments each on their own lines before closing the ); so the assertion becomes multi-line and matches the existing readability pattern used elsewhere in the file.



============================================================================
File: .github/workflows/ci.yml
Line: 178 to 189
Type: nitpick




============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 236 to 237
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 236 to 237, replace the unreproducible shell session chunk references (shell2, 4193d0, 2fc9bd) with durable, auditable references: list a timestamp or committing SHA for when the clippy run was performed, provide the path to the saved clippy output artifact (e.g., /logs/clippy/run-YYYYMMDD-HHMMSS.txt or /artifacts/clippy-.txt), and add a short plain-text summary of the deprecated usages observed (e.g., “deprecated Flow::new / Instance::new usages in module X”). Ensure the line contains only these durable references and the brief description so reviewers can locate or reproduce the evidence.



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 218 to 223
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 218 to 223, the chained .replace() calls used for escaping are fine for short entity/resource names but can be costly when generating large volumes of Turtle literals; add a short note in this section that explains the performance characteristics and recommends either (a) guarding escaping with a quick test for special characters before running replace chains or (b) deferring optimization to a future batch/export performance pass so reviewers/ops know to monitor and optimize if large-scale exports are needed.



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 1142 to 1145
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 1142–1145, the pending validation items (Python Bindings, TypeScript Bindings, Documentation) lack explicit trigger conditions, owners, and evidence locations; update each pending line to state (1) the trigger condition (e.g., "executed after final Rust cycle completes / or on pre-merge CI job 'bindings-tests'"), (2) the responsible agent or CI job that will run it (e.g., "Agent: Python / CI job: bindings-tests"), and (3) the exact evidence path where results will be stored if run out-of-band (e.g., "/logs/python_tests.txt", "/logs/typescript_tests.txt", "/logs/doc_build.txt"); also add a short footnote or a separate "Pending validations" section linking these items to the CI/CD pipeline stage name and failure handling (where to find logs and who to notify).



============================================================================
File: sea-core/tests/graph_integration_tests.rs
Line: 1 to 7
Type: potential_issue

[ ] Task:
In sea-core/tests/graph_integration_tests.rs lines 1 to 7 and referencing usages at lines ~124 and ~195, the test uses uuid::Uuid::new_v4() but the uuid crate is not imported; add the missing import by adding use uuid::Uuid; to the top imports so the calls to Uuid::new_v4() compile, and ensure Cargo.toml includes the uuid dependency if not already present.



============================================================================
File: .bak.coderabbit.yml
Line: 1 to 3
Type: potential_issue

[ ] Task:
.bak.coderabbit.yml lines 1-3: this backup config file should not be committed; remove it from version control and prevent re-adding. Delete the file from the repo (git rm .bak.coderabbit.yml && git commit -m "remove backup config file"), or if you need to keep it locally only remove it from the index (git rm --cached .bak.coderabbit.yml && git commit -m "untrack backup config file"). Add a rule to .gitignore (e.g. *.bak or .bak.coderabbit.yml) to prevent future commits, and if the content is intended as a real config instead rename it to the documented filename (.coderabbit.yml) and add a brief note to README explaining its purpose.



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 416 to 417
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 416–417, the test currently allows two unit display formats using an OR (accepting either "kg" or "Mass(kg)"); update the assertion to require the single canonical format once Unit::Display is implemented (replace the OR condition with an assert that turtle contains the exact expected string, e.g., sea:unit "kg" or whatever the canonical Display returns). Also add a short follow-up test or a note in the documentation that records the chosen canonical unit display format so future changes are validated against it.



============================================================================
File: sea-core/src/python/graph.rs
Line: 69 to 173
Type: nitpick

[ ] Task:
In sea-core/src/python/graph.rs around lines 69 to 173 the UUID-to-ConceptId parsing is duplicated across has_, get_, and flows_ methods; add a helper function parse_concept_id(id: &str) -> PyResult that parses the UUID and returns ConceptId (using the same Uuid::from_str -> map_err -> ConceptId::from pattern) placed before the impl block, then replace each duplicate block with a single call to parse_concept_id(&id) or parse_concept_id(&entity_id) and propagate the resulting ConceptId (using ? to forward errors) in all has_, get_, and flows_ methods to remove duplication.



============================================================================
File: sea-core/src/python/graph.rs
Line: 1
Type: nitpick

[ ] Task:
In sea-core/src/python/graph.rs around lines 1 to 19, there is a redundant module-level clippy attribute duplicate at line 19 (#![allow(clippy::useless_conversion)]) — remove the duplicate attribute on line 19 so only the module-level attribute remains at the top of the file.



============================================================================
File: sea-core/src/policy/mod.rs
Line: 1
Type: potential_issue




============================================================================
File: .github/workflows/ci.yml
Line: 171 to 172
Type: nitpick

[ ] Task:
In .github/workflows/ci.yml around lines 171-172, the workflow currently installs wasm-pack with cargo install wasm-pack --locked which reinstalls on every run; replace that step by using taiki-e/install-action@wasm-pack (which manages caching automatically) or add an explicit caching step for the wasm-pack binary (cache ~/.cargo/bin/wasm-pack) and then run the install only when cache miss occurs; update the job step to call the install-action or add a cache/key + restore/save steps surrounding the install so subsequent runs reuse the cached wasm-pack.



============================================================================
File: .github/workflows/release.yml
Line: 40 to 44
Type: potential_issue




============================================================================
File: sea-core/tests/turtle_resource_export_tests.rs
Line: 31
Type: nitpick

[ ] Task:
In sea-core/tests/turtle_resource_export_tests.rs around line 31, the test uses a substring .contains(r#"sea:unit "kg""#) which only checks for a fragment and not that the unit triple is attached to the correct Steel resource or that the Turtle is syntactically valid; parse the generated Turtle output into an RDF graph (use the project's RDF/Turtle parser already used elsewhere or add rio/sophia test helper) and assert that the graph contains a triple with the expected Steel subject IRI, the sea:unit predicate, and the literal "kg" (and optionally the correct datatype/lang) so the test verifies both valid Turtle and the full triple relationship rather than a simple substring match.



============================================================================
File: sea-core/examples/parser_demo.rs
Line: 173 to 177
Type: potential_issue

[ ] Task:
In sea-core/examples/parser_demo.rs around lines 173-177 and 180-184, the warning message says "treating as 0.0" but the code actually skips the non-finite flows rather than adding zero; update the warning text to accurately state that the flow is being skipped (e.g., "skipping flow ... with non-finite quantity ...") so logs reflect the real behavior; ensure both warning occurrences are changed to the new wording while keeping flow.id() and flow.quantity() in the message for context.



============================================================================
File: sea-core/src/wasm/graph.rs
Line: 88 to 93
Type: nitpick

[ ] Task:
In sea-core/src/wasm/graph.rs around lines 88-93 (and similarly at 150-155, 205-210, 260-265, 275-280, 290-295, 305-310, 320-325), the code clones every inner element when converting collections to WASM types which can be expensive for large graphs; change the mapping to work with references instead of owned values if possible: update Entity::from_inner to accept a reference (or add a new from_inner_ref/&From constructor) and iterate over &self.inner.all_entities() (or .iter()) to avoid cloning, otherwise keep the clone but add a comment explaining why cloning is necessary for the WASM boundary and add a performance note or benchmark to justify it.



============================================================================
File: sea-core/src/typescript/graph.rs
Line: 75 to 190
Type: nitpick

[ ] Task:
In sea-core/src/typescript/graph.rs around lines 75 to 190, the UUID parsing and conversion to crate::ConceptId is duplicated across many methods; extract a private helper like parse_concept_id(&str) -> Result that parses the UUID string, maps parse errors to Error::from_reason("Invalid UUID: ..."), and returns the ConceptId, then replace each duplicated block in has_entity/has_resource/has_flow/has_instance/get_/flows_from/flows_to with a single call to that helper and use its returned ConceptId for subsequent inner. calls to eliminate repetition.



============================================================================
File: sea-core/src/wasm/graph.rs
Line: 44 to 48
Type: refactor_suggestion

[ ] Task:
In sea-core/src/wasm/graph.rs around lines 44–48 (and also affecting lines 52–55, 63–66, 106–109, 114–117, 125–128, 168–171, 176–179, 187–190, 223–226, 231–234, 242–245, 271–274, 286–289, 301–304, 316–319), extract the duplicated parse-and-convert logic into a private helper on impl Graph with signature fn parse_concept_id(id: &str) -> Result, implementing the existing parsing and map_err behavior, then replace each occurrence that currently does Uuid::from_str(&id) ... and crate::ConceptId::from(uuid) with a call to self.parse_concept_id(&id)? so each method becomes concise and returns/propagates the same JsValue on error.



============================================================================
File: sea-core/tests/turtle_resource_export_tests.rs
Line: 19
Type: nitpick

[ ] Task:
In sea-core/tests/turtle_resource_export_tests.rs around line 19, the test uses a substring contains check which may produce false positives; replace it by parsing the Turtle output into an RDF graph (e.g., with an available Turtle/RDF parser used in the repo or a small helper) and assert that the expected subject node has an rdfs:label triple whose literal value equals "Camera\n(High\tQuality)" (including the newline and tab escapes) — alternatively, if adding a parser is heavy, match a more specific pattern that ties the label to the expected resource IRI (anchor the regex to the resource IRI and the label literal) so the test verifies the label is attached to the correct resource node.



============================================================================
File: sea-core/src/sbvr.rs
Line: 284 to 314
Type: potential_issue

[ ] Task:
In sea-core/src/sbvr.rs around lines 284 to 314, the test_export_to_sbvr test exports SBVR XML but doesn't assert the new SchemaVersion and Destination fields; update the test to assert that sbvr_xml contains the SchemaVersion element (e.g. contains "<sbvr:SchemaVersion") and the Destination element (e.g. contains "<sbvr:Destination" or the expected destination value), placing these assertions after the existing sbvr_xml generation so the test fails if those elements are missing.



============================================================================
File: sea-core/tests/rdf_xml_typed_literal_tests.rs
Line: 39 to 42
Type: nitpick

[ ] Task:
In sea-core/tests/rdf_xml_typed_literal_tests.rs around lines 39 to 42, the test currently allows two different decimal datatype URIs which weakens determinism; either tighten to assert the single expected URI (e.g., require "http://www.w3.org/2001/XMLSchema#decimal") by replacing the OR check with a single contains() assertion, or explicitly document why both namespaces are acceptable by adding a short comment above the assertion explaining the serializer may emit either the standard XMLSchema or the domainforge namespace and both are intentionally supported; implement one of these two fixes to make the test intent explicit and deterministic.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 20
Type: nitpick

[ ] Task:
In sea-core/src/python/primitives.rs around line 20 (also at lines 102, 192, and 300), remove the redundant #[allow(clippy::useless_conversion, clippy::wrong_self_convention)] attributes since a module-level #[allow] at the top of the file already covers these lints; delete those four attribute lines so the file relies on the existing module-level suppression.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 280 to 281
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 280-281 (and similarly 306-307, 339-340, 363-364), the code converts f64 -> string -> Decimal which causes precision loss; replace the intermediate string-based conversion with a direct, precision-preserving conversion (e.g., use Decimal::from_f64_retain(num) or Decimal::from_f64(num) if appropriate) or change the JSON parsing to require decimal values as strings and parse them with Decimal::from_str; update the match/if branches to handle None/error from the chosen Decimal conversion and return/propagate an error instead of silently dropping precision.



============================================================================
File: sea-core/tests/instance_tests.rs
Line: 23 to 25
Type: potential_issue

[ ] Task:
In sea-core/tests/instance_tests.rs around lines 23 to 25, the test currently only asserts the id, resource_id, and entity_id are non-empty which doesn't ensure they are valid UUIDs; change the assertions to validate UUID format by parsing each string back into a uuid::Uuid (use Uuid::parse_str) and assert parsing succeeds (or alternatively assert the string length is 36 and contains hyphens in the expected positions), so replace the is_empty checks with parse assertions and add an appropriate use/import for the uuid crate if not already present.



============================================================================
File: sea-core/src/wasm/primitives.rs
Line: 149
Type: nitpick

[ ] Task:
In sea-core/src/wasm/primitives.rs around lines 149, 259, and 359, the .inspect_err(|_e| {}) calls swallow errors without explanation; add the same explanatory comment used in Entity::get_attribute (near line 64) above each of these .inspect_err invocations to clarify why errors are intentionally suppressed (e.g., non-fatal/missing attribute should not propagate), so future readers understand the rationale and consistency is maintained across Resource, Flow, and Instance.



============================================================================
File: sea-core/src/wasm/primitives.rs
Line: 42
Type: refactor_suggestion

[ ] Task:
In sea-core/src/wasm/primitives.rs around lines 42, 128, 238, and 338, the literal "default" is repeated; add a module-level constant (e.g., const DEFAULT_NS: &str = "default";) near the top of the file and replace each hardcoded comparison (if ns == "default") with if ns == DEFAULT_NS to eliminate duplication; use an appropriate visibility (private by default) and update all four occurrences listed.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 292 to 294
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 292-294 (and also apply to 325-327, 351, 375), the Decimal-to-f64 conversion for Min and Max currently swallows conversion failures and returns null while Sum and Avg propagate conversion errors; standardize behavior by propagating conversion errors consistently: change the Min and Max conversion calls to use ok_or_else with a descriptive error (same style as Sum/Avg) so any to_f64() failure returns an Err with the formatted message instead of silently producing null.



============================================================================
File: sea-core/src/wasm/primitives.rs
Line: 40 to 47
Type: nitpick

[ ] Task:
In sea-core/src/wasm/primitives.rs around lines 40-47 (and also apply to ranges 49-69, 126-133, 135-153, 236-243, 245-263, 336-343, 345-363), several identical wasm-bindgen impl blocks (namespace getter, setAttribute, getAttribute, toJSON) are duplicated for Entity, Resource, Flow, and Instance; create a macro_rules! (e.g., impl_common_wasm_methods) that accepts the struct identifier and its inner rust type and emits the #[wasm_bindgen] impl with the getter and methods, use a single DEFAULT_NAMESPACE constant instead of hardcoded "default", ensure method signatures and conversions (Option, JsValue for JSON) match the original definitions, replace each duplicated impl block by invoking the macro for each of Entity, Resource, Flow, and Instance, and verify visibility/imports so the macro compiles.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 300 to 340
Type: potential_issue

[ ] Task:
docs/plans/tmp/task-07-fix-api-robustness.md around lines 300 to 340: the current _synthesize_fallback_ast() illegally constructs a tree-sitter Tree instance and will crash; remove the invalid Tree(...) return and instead raise a NotImplementedError (or re-raise the original parse exception) with a clear message that fallback AST synthesis is not implemented and that callers should handle parse failures or retry parsing via the parser; ensure no attempt is made to instantiate Tree directly and update the docstring/logging accordingly.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1130 to 1142
Type: potential_issue




============================================================================
File: sea-core/src/python/primitives.rs
Line: 235 to 241
Type: potential_issue

[ ] Task:
In sea-core/src/python/primitives.rs around lines 235 to 241, the quantity getter currently unwraps a Decimal->f64 conversion and will panic on overflow; change the getter to return a Python error instead of panicking by updating the function signature to return PyResult, attempt the to_f64 conversion, and if it returns None map that into a pyo3 Python exception (e.g. PyValueError or OverflowError) with a clear message about the Decimal-to-f64 conversion failure; ensure the #[getter] attribute and any callers are updated to expect PyResult.



============================================================================
File: sea-core/tests/rdf_xml_typed_literal_tests.rs
Line: 82 to 86
Type: refactor_suggestion

[ ] Task:
In sea-core/tests/rdf_xml_typed_literal_tests.rs around lines 82 to 86, the test currently only checks that escaped entities appear anywhere in the RDF/XML output; update the assertions to verify escaping occurs specifically inside the resource name element. Parse the RDF/XML (or use a substring match including the resource name tag/attribute) and assert that the element or attribute that contains the resource name includes "&lt;", "&gt;", "&amp;", "&quot;" and "&apos;" in its text/attribute value rather than just anywhere in the document; ensure you select the correct element/attribute path (e.g. the resource name element or its attribute) and replace the loose contains() assertions with checks against that extracted text or a contextual substring pattern.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 230 to 231
Type: potential_issue

[ ] Task:
In sea-core/src/typescript/primitives.rs around lines 230 to 231, replace the incorrect use of f64::MIN (which is the smallest positive normal value) with f64::NEG_INFINITY for the negative-infinity case; update the branch that returns f64::MIN to return f64::NEG_INFINITY instead so the code correctly represents negative infinity.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 166
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs at line 166, the Decimal-to-f64 conversion uses unwrap_or(0.0) which silently masks failures; replace this with a fallible conversion that returns and propagates an error on failure (follow the pattern used in evaluate_aggregation around lines 292-294), i.e. perform the conversion in a way that maps conversion errors into the function's error type and returns Err when conversion fails rather than defaulting to 0.0, and update the function signature and any callers as needed to handle the propagated Result.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 597 to 635
Type: potential_issue




============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 213 to 247
Type: refactor_suggestion

[ ] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 213–247, parse_file() declares an unused allow_fallback flag and contains unreachable fallback logic because callers bypass parse_file and the flag is never exposed; remove the dead fallback machinery by deleting the allow_fallback parameter and the fallback/synthesis branch and rollback log write, and simplify parse_file to just open/read/parse, set self.source_code, return (tree, source_code), and on any Exception log the error and re-raise immediately so behavior is deterministic and callers need not change.



============================================================================
File: sea-core/tests/parser_tests.rs
Line: 68 to 73
Type: nitpick

[ ] Task:
In sea-core/tests/parser_tests.rs around lines 68 to 73, the test function test_parse_policy_simple is redundant with test_parse_policy_with_colon; remove the entire test_parse_policy_simple function (the fn test_parse_policy_simple() { ... } block) so only the more comprehensive test remains, then run cargo test to ensure the suite still passes.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1508 to 1603
Type: potential_issue

[ ] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 1508 to 1603, the dry-run path (roughly lines 1561–1576) runs transformers and import insertion without error handling; wrap the transformer loop and subsequent import insertion in a try-except block that catches Exception, logs the error (include exception details and stacktrace when verbose), and exits with a non-zero status to mirror production robustness; ensure the ImportInserter.add_import call is inside the same try so import failures are handled the same way.



============================================================================
File: sea-core/src/validation_error.rs
Line: 231 to 246
Type: nitpick

[ ] Task:
In sea-core/src/validation_error.rs around lines 231 to 246, the user-facing suggestion string currently formats Dimension values with Debug ({:?}) which can leak internal representations; change the formatting to use Display ({}) for expected and found (i.e., "{}") assuming Dimension implements Display, or implement Display for Dimension and then use "{}" in the suggestion to produce a user-friendly message.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 224 to 238
Type: potential_issue

[ ] Task:
In sea-core/src/typescript/primitives.rs around lines 224 to 238, the branch handling negative infinity currently returns f64::MIN (the largest-magnitude finite negative), which is incorrect; change that branch to return f64::NEG_INFINITY. Also ensure the function's contract is clear to TypeScript callers: positive infinity maps to f64::MAX, negative infinity to f64::NEG_INFINITY, NaN to 0.0, and conversion failures to 0.0; add a short comment above the match (or update docs) and, if callers need to detect these cases, add optional logging or an API change so callers can observe the special-value conversions.



============================================================================
File: sea-core/src/validation_error.rs
Line: 249 to 266
Type: nitpick

[ ] Task:
In sea-core/src/validation_error.rs around lines 249 to 266, remove the unnecessary clones of expected and found in type_mismatch: after converting inputs to Strings, build the message and suggestion using borrowed references (e.g. format!(..., &expected, &found)) so you can move expected and found directly into expected_type and found_type (Some(expected), Some(found)) without calling clone().



============================================================================
File: sea-core/src/validation_error.rs
Line: 195 to 228
Type: nitpick

[ ] Task:
In sea-core/src/validation_error.rs around lines 195 to 228, the convenience constructors call name.clone() unnecessarily; keep name as a String by doing let name = name.into(); then use &name (a borrow) inside format! for the suggestion and pass name (the owned String) to the struct's name field, and similarly replace location.into() inline where used; remove the .clone() calls in undefined_entity, undefined_resource, and undefined_flow so no extra allocations occur.



============================================================================
File: sea-core/src/units/mod.rs
Line: 281 to 295
Type: potential_issue

[ ] Task:
In sea-core/src/units/mod.rs around lines 281 to 295, the current unit_from_string silently falls back to a Count-dimension Unit for unknown symbols which masks lookup errors; change the API so callers get explicit failures: replace unit_from_string to return Result (or a suitable error type) and have it return Err when registry.get_unit(&symbol) fails, and add a new helper unit_from_string_or_default that implements the existing fallback behavior (creating the Count default) for callers that need it; update call sites to handle the Result or use the new _or_default helper.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 625 to 644
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 625 to 644, the code removes the surrounding quotes and then re-wraps the content with format!("\"{}\"", content) before calling serde_json::from_str, which can produce invalid JSON when content contains characters that need escaping; instead call serde_json::from_str directly on the original pair string slice s (keeping the quotes), remove the manual slicing/re-wrapping, and propagate the serde_json error into the existing ParseError::GrammarError branch so valid JSON escaping is handled by serde_json.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 647 to 667
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 647 to 667, the multiline-string handler builds a JSON string from the raw content but fails when pest preserves literal newlines (JSON disallows unescaped newline chars). Fix by treating the content the same way as parse_string_literal: determine whether pest already provides escaped sequences; if it does, call the same unescape path as parse_string_literal, otherwise pre-escape backslashes, quotes and newline characters in content (e.g., replace '\' with '\\', '"' with '\"', '\n' with '\\n', '\r' with '\\r', '\t' with '\\t') before wrapping with quotes and feeding to serde_json::from_str, or implement a dedicated unescape routine for multiline strings; ensure errors include the offending text and preserve existing ParseError behavior.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 94 to 98
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 94-98, parse_entity currently calls parse_name (which accepts both string_literal and multiline_string) while flows use parse_string_literal (lines ~177-187), creating an inconsistency; decide which form is allowed and make both sides consistent: if multiline entity names are valid, update the flow parsing sites to call parse_name (and ensure downstream handling supports multiline strings); otherwise, restrict entity declarations by replacing parse_name with parse_string_literal at this location so entity parsing only accepts single-line string literals; ensure any usages, types, and error messages are updated accordingly.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 595 to 598
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 595 to 598, the code currently wraps parsed decimals as JsonValue::String which loses numeric typing; replace this by converting the parsed decimal into a serde_json::Number and returning JsonValue::Number. Specifically, import serde_json::Number, parse the decimal string into an f64 (propagating a ParseError::InvalidQuantity on parse failure), use Number::from_f64 and map a None result to ParseError::InvalidQuantity for non-finite values, then construct Expression::Literal(JsonValue::Number(num)) instead of JsonValue::String.


