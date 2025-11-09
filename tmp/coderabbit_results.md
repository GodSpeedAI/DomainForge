Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: sea-core/src/wasm/graph.rs
Line: 88
Type: potential_issue

[x] Task: ✅ COMPLETED
In sea-core/src/wasm/graph.rs around lines 88 (and likewise for the other affected blocks at 139–147, 189–198, 240–249 and the single-line occurrences 146, 197, 248), the four all_* methods currently swallow serialization failures by using unwrap_or(JsValue::NULL); update each method signature to return Result and stop returning JsValue::NULL on error—propagate the serde_wasm_bindgen serialization error to the caller by converting the serialization error into a JsValue and returning Err(...) instead of unwrapping to NULL, making the body return the successful JsValue on Ok and Err(converted_error) on failure.

============================================================================
File: .github/workflows/release.yml
Line: 88
Type: potential_issue

[x] Task: ✅ COMPLETED
.github/workflows/release.yml around line 88: the upload-artifact path uses target/wheels/.whl but the job runs with working-directory: sea-core so the wheels are actually under sea-core/target/wheels/.whl; update the path to point to sea-core/target/wheels/*.whl (or use a relative path that matches the working-directory) so the artifact uploader can find and upload the generated wheel files.

============================================================================
File: sea-core/src/calm/export.rs
Line: 1 to 12
Type: potential_issue

[x] Task: ✅ COMPLETED
In sea-core/src/calm/export.rs around lines 1 to 12, the code calls chrono::Utc::now() but chrono is not imported; add a top-level import like use chrono::Utc; to the imports block and then update line 12 to call Utc::now().to_rfc3339() (i.e., remove the chrono:: prefix) so the code compiles.

============================================================================
File: sea-core/tests/wasm_tests.rs
Line: 1 to 9
Type: nitpick

[x] Task: ✅ COMPLETED
In sea-core/tests/wasm_tests.rs around lines 1 to 9, the top-level #[allow(unused_imports)] and the wasm_bindgen_test imports are declared outside the #[cfg(feature = "wasm")] module causing the allow attribute to mask unused imports when the wasm feature is disabled; move the lines importing wasm_bindgen_test and wasm_bindgen_test_configure (and remove the file-level #[allow(unused_imports)]) into the feature-gated mod wasm_tests block so the imports only exist when feature="wasm" and the allow attribute is no longer needed.

============================================================================
File: sea-core/Cargo.toml
Line: 21
Type: refactor_suggestion

[x] Task: ✅ COMPLETED
In sea-core/Cargo.toml around line 21, the log dependency is using a floating semver "0.4"; change it to a pinned patch version (e.g., "0.4.21" or the latest 0.4.x) to ensure reproducible builds; also check and pin any other unpinned log entries referenced in the comment (lines 28 and 34) to the same 0.4.x patch version for consistency.

============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1390 to 1394
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 1390 to 1394, the test reaches into Inserter internals (setting inserter.source_code/inserter.parser and calling_find_primitives_use_decls) which verifies implementation instead of the public API; change the test to exercise only the public add_import()/related public methods and then parse the resulting source (or the returned/updated code) to assert the AST contains the expected sea_core::primitives use/import declaration — do not set or read private attributes or call underscored helper methods.

============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 163
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md at line 163, the label "tree-sitter-rust (Python bindings)" is misleading; change the wording to "tree-sitter with tree_sitter_languages" and update the parenthetical to indicate it uses the tree_sitter_languages package (pre-built language parsers including Rust) so readers aren't confused by a non-existent "tree-sitter-rust" package.

============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 866 to 872
Type: potential_issue

[ ] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 866 to 872, the macro name extraction used to detect stringify/concat does not handle fully-qualified paths (e.g. core::stringify), so change the check to use the last path segment: split the extracted macro_name on '::' and use the final element (macro_base) when testing membership in {'stringify','concat'} (or alternatively normalize by rpartition['::'](-1)) so fully-qualified macros are correctly recognized.

============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1594 to 1595
Type: nitpick

[ ] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 1594–1595: the plan describes an allow_fallback flag and regex-based fallback for RustTransformer.parse_file() but the implementation is missing; add logic in RustTransformer.parse_file() so that on a tree-sitter parse exception it logs a warning, writes the raw exception and stack trace to /rollback/parse_failure_.log, then if self.allow_fallback (wired to the CLI --allow-fallback and automatically true for dry-runs) synthesizes and returns a minimalist AST built by scanning the source with a short regex that extracts top-level use statements, top-level function signatures and outer file comments (structured to satisfy downstream transformers/import inserter), otherwise re-raise the original parse exception to fail fast in strict mode.

============================================================================
File: sea-core/tests/rdf_xml_typed_literal_tests.rs
Line: 89 to 93
Type: nitpick

============================================================================
File: sea-core/tests/sbvr_fact_schema_tests.rs
Line: 50 to 57
Type: potential_issue

[x] Task: ✅ COMPLETED
In sea-core/tests/sbvr_fact_schema_tests.rs around lines 50 to 57, the round-trip JSON test checks every field except verb; add a new assertion immediately after line 55 that verifies the deserialized.verb matches the original fact's verb value (for this test case assert that it equals the expected verb, e.g. "installs"), so the test fully covers all SbvrFactType fields.

============================================================================
File: sea-core/tests/flow_tests.rs
Line: 34 to 37
Type: potential_issue

[ ] Task:
In sea-core/tests/flow_tests.rs around lines 34 to 37, the test currently only asserts the ID strings are non-empty; replace these weak checks with actual UUID parsing validation by calling Uuid::parse_str(...) (from the uuid crate) for flow.resource_id(), flow.from_id(), flow.to_id(), and flow.id(), and assert that each parse returns Ok(_); ensure you import uuid::Uuid at the top of the test file if not already present.

============================================================================
File: sea-core/src/python/policy.rs
Line: 30 to 48
Type: potential_issue

[ ] Task:
In sea-core/src/python/policy.rs around lines 30 to 48, the BinaryOp enum is exposed to Python but lacks Python docstrings; add documentation by exposing a class-level __doc__ and per-variant docs using pyo3 class attributes. Concretely: add a #[pymethods] impl BinaryOp block that defines a #[classattr] const or fn __doc__ -> &'static str returning a descriptive docstring for the enum, and add #[classattr] constants for each variant (e.g., pub const And: BinaryOp = BinaryOp::And) with brief docstrings so Python users see documentation for the enum and its members.



============================================================================
File: sea-core/src/python/policy.rs
Line: 8 to 16
Type: potential_issue




============================================================================
File: sea-core/src/calm/import.rs
Line: 198 to 353
Type: nitpick

[ ] Task:
In sea-core/src/calm/import.rs around lines 198 to 353, tests only cover happy-path entity/resource/flow imports; add unit tests to exercise instance imports, error paths, and edge cases: create a test that imports a node with metadata indicating an Instance (use import_instance path) and assert instance_count and properties; add tests that feed malformed JSON to import and assert Err for missing required metadata, relationships referencing unknown node IDs, and invalid quantity formats (e.g., non-numeric strings); add edge-case tests for nodes missing namespace (ensure import tolerates or rejects per spec) and relationships with invalid parties (e.g., missing source/destination or same ID) and assert correct error behavior. For each test, construct serde_json::json fixtures, call import(calm_json), and assert Ok with expected counts/properties or Err with expected error kind/message to validate error handling.



============================================================================
File: sea-core/src/calm/import.rs
Line: 76
Type: nitpick

[ ] Task:
In sea-core/src/calm/import.rs around line 76, the trailing comment "Skip entities, resources, and constraints in second pass" is misleading because constraint handling isn't implemented; update the comment to explicitly state that constraints are not yet supported (e.g., "Skip entities and resources in second pass; constraints not yet supported") or implement the missing constraint handling if intended—make the comment and code consistent.



============================================================================
File: sea-core/src/calm/import.rs
Line: 55
Type: nitpick

[ ] Task:
In sea-core/src/calm/import.rs around line 55, update the trailing comment on the match arm that currently reads "Skip instances and constraints in first pass" to explicitly reflect actual behavior: change it to something like "Skip instances (handled in second pass) and other unsupported node types" so it doesn't claim support for constraints that don't exist; keep the match logic unchanged and only revise the comment text for clarity.



============================================================================
File: .coderabbit.yml
Line: 14 to 22
Type: potential_issue

[ ] Task:
.coderabbit.yml lines 14-22: the current snippet nests a top-level tools section under auto_review and includes unsupported config_file properties for clippy and ruff; move the tools block out to be a sibling of auto_review under reviews, and remove the unsupported config_file keys for clippy and ruff (leave only enabled: true) or, if you need to keep config_file, relocate those settings to a tool that supports config_file (e.g., golangci-lint, swiftlint, or semgrep) and ensure the final YAML conforms to the CodeRabbit schema.



============================================================================
File: sea-core/tests/turtle_entity_export_tests.rs
Line: 1 to 2
Type: nitpick

[ ] Task:
In sea-core/tests/turtle_entity_export_tests.rs around lines 1 to 2, the file is wrapped with an unnecessary #[cfg(test)] and a module declaration because files under tests/ are already integration tests; remove the leading #[cfg(test)] attribute and the surrounding "mod turtle_entity_export_tests {", move the test items from inside that module to the top level of the file, and delete the corresponding closing brace at the end of the file so the tests live at file scope.



============================================================================
File: sea-core/examples/parser_demo.rs
Line: 164 to 174
Type: potential_issue

[ ] Task:
In sea-core/examples/parser_demo.rs around lines 164 to 174, the code currently converts flow.quantity() by calling .to_string().parse::().unwrap_or(0.0) which is inefficient and fragile; replace this with a direct numeric conversion or accessor: if Quantity is already a numeric type cast or convert it directly to f64 (e.g. as f64 or as_primitive), or add/use a Quantity::to_f64()/as_f64() method and call that; handle conversion failures explicitly (e.g. match or unwrap_or with a logged warning) instead of round-tripping through strings so totals are accurate and performant.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 306 to 309
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 306 to 309, the code silently falls back to 0.0 on parse errors (avg.to_string().parse::().unwrap_or(0.0)), corrupting average results; replace the panic/unwrap_or path by actually propagating the parse error: perform the parse into f64 and propagate any Err (using the ? operator or map_err to convert into the function's error type) and then wrap the successfully parsed f64 into serde_json::json!(...). Ensure the function's Result/Err type is used to return the parse error rather than returning 0.0.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 275 to 278
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 275 to 278, the code currently uses unwrap_or(0.0) when parsing the Decimal-to-f64 conversion which hides parse failures and returns 0.0; change it to propagate the parse error instead of defaulting: remove unwrap_or and use the ? operator (or map_err to convert the parse error into the function's error type) so the parse failure is returned to the caller, and ensure the surrounding function signature/return type supports returning that error.



============================================================================
File: sea-core/tests/graph_tests.rs
Line: 535
Type: nitpick

[ ] Task:
In sea-core/tests/graph_tests.rs around line 535, the loop declares an unused variable _i; replace for _i in 1..=5 with for _ in 1..=5 to indicate the index is intentionally unused and silence the unused variable warning.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 238 to 240
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 238 to 240, the code unconditionally calls filter_expr.substitute("flow", item) which breaks substitutions for entity/resource/instance collections; replace the hardcoded "flow" with a variable name chosen from the collection type (e.g., match on the collection enum/variant to map to "flow"/"entity"/"resource"/"instance"), then call substitute with that chosen name and keep the existing unwrap_or_else fallback behavior so errors are handled the same way.



============================================================================
File: sea-core/tests/parser_tests.rs
Line: 68 to 91
Type: refactor_suggestion

[ ] Task:
In sea-core/tests/parser_tests.rs around lines 68 to 91, remove the redundant test_parse_policy_simple function (lines 68-73) because its assertions duplicate test_parse_policy_with_colon; delete the entire test_parse_policy_simple block so only test_parse_policy_with_colon remains, ensuring compilation still passes and test ordering unaffected.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 353 to 355
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 353 to 355, the code silently converts a string to f64 with parse::().unwrap_or(0.0) which masks parse failures by returning 0.0; instead propagate parse errors: perform the parse into a Result and propagate the error (for example use map(|d| d.to_string().parse::()) and then transpose or use and_then with ? to return a Result), removing unwrap_or so failures bubble up to the caller, or convert the parse error into the function's error type and return Err when parsing fails.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 163
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around line 163, the code currently does f.quantity().to_string().parse::().unwrap_or(0.0) which silently converts parse failures to 0.0 and corrupts aggregations; instead remove the unwrap_or(0.0) and propagate parse errors upward (or return a Result) so malformed quantities fail loudly — update the surrounding function signature to return Result (or propagate with ?), convert parse errors into a contextual error (with field info) rather than defaulting, and add a small unit test to assert that invalid quantity strings produce an error.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 330 to 332
Type: potential_issue

[ ] Task:
In sea-core/src/policy/quantifier.rs around lines 330 to 332, the current code calls unwrap_or(0.0) after parsing which swallows parse failures and returns 0.0; instead change the expression to parse into a Result and propagate any parse error back to the caller (do not silently replace with 0.0). Concretely, map the optional value into a parse Result (e.g. min.map(|d| d.to_string().parse::()).transpose() or equivalent), propagate the parse error with ? so failures bubble up, and then wrap the resulting Option into the serde_json::json! call.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 641 to 652
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 641 to 652, parse_multiline_string currently only strips the triple quotes and returns the raw content, missing escape sequence processing; update it to apply the same unescaping logic used by parse_string_literal (or extract a shared helper unescape_string and call it from both functions) so that sequences like \\n, \\t, \\\", unicode escapes, etc., are decoded correctly; ensure errors from the unescape helper map to ParseError::GrammarError and keep the same length/quote validation before unescaping.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 56 to 71
Type: nitpick

[ ] Task:
In sea-core/src/parser/ast.rs around lines 56 to 71, the inner node check uses the uncommon if let Rule::declaration = inner.as_rule() form; change this to idiomatic pattern matching by replacing that if let with either if inner.as_rule() == Rule::declaration { ... } or a match inner.as_rule() { Rule::declaration => { ... }, _ => {} } so the code follows standard Rust matching style while preserving the existing logic and error handling for parsing declarations.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 628 to 639
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 628 to 639, the current parse_string_literal simply strips the surrounding quotes and ignores escape sequences, which will misparse strings containing things like \" \\ \n \t; replace the manual slicing with a proper unescaping step—use a JSON-style string parser (e.g. call serde_json::from_str::(s)) to parse and unescape the quoted string, map any serde_json error into the existing ParseError::GrammarError, and return the resulting unescaped String; keep the existing check that the input is a quoted string or let serde_json handle invalid inputs and translate its errors into your ParseResult.



============================================================================
File: sea-core/src/units/mod.rs
Line: 277 to 279
Type: nitpick

[ ] Task:
In sea-core/src/units/mod.rs around lines 277-281, the doc comment currently on lines 277-278 is attached to DEFAULT_UNIT_REGISTRY but actually documents the unit_from_string function; move that triple-slash doc comment so it sits immediately above the unit_from_string function declaration (line ~281) so the documentation attaches to the correct item, preserving the comment text and formatting and removing it from before DEFAULT_UNIT_REGISTRY.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 299 to 317
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 299 to 317, the parse_not_expr currently only wraps the operand with UnaryOp::Not when the inner rule is Rule::not_expr and otherwise returns the unwrapped expression; change it so that after pulling the first inner pair you always call parse_expression(first) and then return Expression::Unary { op: UnaryOp::Not, operand: Box::new(parsed_expr) }, ensuring any single NOT is unconditionally applied to its operand.



============================================================================
File: sea-core/src/kg.rs
Line: 141 to 145
Type: nitpick

[ ] Task:
In sea-core/src/kg.rs around lines 141 to 145, the code assumes flow.quantity() produces a Turtle-safe decimal literal but doesn't validate or document that; add a short validation or assertion that the decimal string matches a safe numeric pattern (e.g. optional leading sign, digits, optional fractional part, no exponent or non-numeric chars) before constructing the Turtle literal, and if it fails either sanitize/convert it to a safe decimal representation or return/log an error; alternatively add a clear code comment documenting the guarantee that flow.quantity() returns a valid decimal string and consider using a canonical formatting function (or regex check) to enforce the Turtle numeric literal shape before pushing the Triple.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 598 to 601
Type: potential_issue




============================================================================
File: sea-core/src/kg.rs
Line: 339 to 352
Type: potential_issue

[ ] Task:
In sea-core/src/kg.rs around lines 339 to 352, the escape_turtle_literal function misses Turtle escapes for backspace and form feed; add match arms for U+0008 (backspace) and U+000C (form feed) mapping them to "\\b" and "\\f" respectively so that those characters are properly escaped in output, keeping the same push_str style and preserving existing behavior for other characters.



============================================================================
File: sea-core/src/kg.rs
Line: 358 to 366
Type: potential_issue

[ ] Task:
In sea-core/src/kg.rs around lines 358 to 366, clean_uri currently expands any prefix:name into http://domainforge.ai/{prefix}#{name}, which wrongly rewrites standard RDF/XSD prefixes; replace this hard-coded expansion with a proper prefix-to-namespace lookup: parse the input on the first ':' into (prefix,name), consult a static map (or match) containing standard mappings (e.g. rdf -> "http://www.w3.org/1999/02/22-rdf-syntax-ns#", rdfs -> "http://www.w3.org/2000/01/rdf-schema#", xsd -> "http://www.w3.org/2001/XMLSchema#", owl -> "http://www.w3.org/2002/07/owl#", etc.), and if the prefix exists return namespace + name (using '#' or '/' as appropriate for that namespace), otherwise fall back to the original domainforge expansion or uri.to_string(); also handle inputs with more than one ':' by splitting only on the first ':'.



============================================================================
File: sea-core/src/kg.rs
Line: 571 to 593
Type: potential_issue

[ ] Task:
In sea-core/src/kg.rs around lines 571 to 593, the test currently asserts the wrong expanded datatype URI (it expects the buggy value "http://domainforge.ai/xsd#decimal"); update the assertion to the correct XSD decimal URI produced by the fixed clean_uri — replace the expected string with "http://www.w3.org/2001/XMLSchema#decimal" (i.e. assert that xml.contains("rdf:datatype=\"http://www.w3.org/2001/XMLSchema#decimal\"")). Ensure the rest of the assertions remain unchanged.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 201 to 217
Type: nitpick

[ ] Task:
In sea-core/src/typescript/primitives.rs around lines 201 to 217, replace the string-parse conversion with Decimal::to_f64() but keep the original NaN-to-0.0 behavior: call self.inner.quantity().to_f64() and match on the Option, returning the f64 when Some and finite; map positive infinity to f64::MAX, negative infinity to f64::MIN, map NaN to 0.0, and treat None (or any other case) as 0.0.



Review completed ✔
