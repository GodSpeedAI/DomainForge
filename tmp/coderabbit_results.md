Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: .github/copilot-instructions.md
Line: 646
Type: potential_issue

[x] Task:
.github/copilot-instructions.md around line 646: the sentence begins with a lowercase "for" ("for example, ...") but should start with a capital letter; update the sentence to start with "For example," and ensure surrounding punctuation/capitalization remains consistent with the document's style.



============================================================================
File: sea-core/examples/parser_demo.rs
Line: 105
Type: potential_issue

[x] Task:
In sea-core/examples/parser_demo.rs around lines 105-106, 118-119 and 143, remove the unchecked .unwrap() calls on graph lookups and replace them with explicit handling: use match or if let to handle the Option returned by graph.get_entity(...) / graph.get_resource(...), return or propagate an error (Result) or skip the flow with a clear log/error message when the entity/resource is missing, and ensure downstream code receives a valid value before use; apply the same pattern to each listed line so missing entities/resources do not cause a panic.



============================================================================
File: sea-core/tests/calm_schema_validation_tests.rs
Line: 84 to 93
Type: refactor_suggestion

[x] Task:
In sea-core/tests/calm_schema_validation_tests.rs around lines 84 to 93, the test duplicates schema compilation by calling serde_json::from_str and JSONSchema::compile; replace those two lines with a call to the existing compile_calm_schema() helper to obtain the compiled_schema variable, ensuring the test uses compile_calm_schema() (and import or adjust scope if necessary) so the test reads compiled_schema = compile_calm_schema(); and then proceeds unchanged.



============================================================================
File: sea-core/deny.toml
Line: 31
Type: potential_issue

[x] Task:
In sea-core/deny.toml around line 31, the allow-registry entry uses a malformed crates.io git URL; replace the value with the registry name "crates-io" (allow-registry = ["crates-io"]) or, if you must use a git URL, change it to the correct Git URL with the .git suffix (e.g., "https://github.com/rust-lang/crates.io-index.git") so the registry is recognized properly.



============================================================================
File: .github/workflows/ci.yml
Line: 101 to 102
Type: nitpick

[x] Task:
.github/workflows/ci.yml around lines 101 to 102: the workflow installs maturin and pytest without pinned versions which risks non-reproducible builds; update the step to install pinned versions (either hardcode vetted versions like maturin== and pytest== or reference workflow inputs/variables that contain approved version strings or a constraints file) so the CI always installs deterministic, tested releases; ensure the chosen versions are documented in the repo (or a single source of truth) and update any related workflow matrix or cache keys if needed.



============================================================================
File: .coderabbit.yml
Line: 10 to 13
Type: potential_issue

[x] Task:
.coderabbit.yml around lines 10 to 13: the auto_review configuration uses the wrong property name "branches:" which fails schema validation; change the key to "base_branches:" preserving the same list values (- main, - dev, - release/*) so the config conforms to the expected schema.



============================================================================
File: sea-core/tests/graph_tests.rs
Line: 1 to 7
Type: potential_issue

[x] Task:
In sea-core/tests/graph_tests.rs around lines 1 to 7, the test code calls uuid::Uuid::new_v4() but never imports the uuid crate; add a top-level import (use uuid::Uuid;) to the file and ensure the uuid dependency is present in Cargo.toml (e.g., uuid = "1" or the project’s chosen version) so the calls to Uuid::new_v4() compile.



============================================================================
File: .coderabbit.yml
Line: 36
Type: potential_issue

[x] Task:
.coderabbit.yml around line 36: change the misuse of auto_title_placeholder which currently holds a pipe-delimited pattern string; move that pattern into auto_title_instructions and set auto_title_placeholder to a placeholder identifier (for example "@coderabbitai") as the schema expects. Update the file so auto_title_placeholder contains the placeholder identifier string and add or adjust auto_title_instructions to contain the pattern(s) or guidance for generating titles instead of placing them in auto_title_placeholder.



============================================================================
File: sea-core/tests/sbvr_fact_schema_tests.rs
Line: 16 to 19
Type: nitpick

[x] Task:
In sea-core/tests/sbvr_fact_schema_tests.rs around lines 16 to 19, the test uses fragile substring checks on the serialized JSON; instead deserialize the JSON string back into a serde_json::Value (or a concrete struct) and assert the specific fields/values directly (e.g., check value["schema_version"] == "2.0", that value has a "destination" key, and that the appropriate resource id equals "resource-123"), replacing the json.contains(...) assertions with structured assertions that fail only on actual mismatches.



============================================================================
File: sea-core/tests/parser_tests.rs
Line: 75 to 81
Type: potential_issue

[x] Task:
In sea-core/tests/parser_tests.rs around lines 75 to 81, the test test_parse_policy_with_colon() duplicates test_parse_policy_simple(); either delete this redundant test or change it to assert a colon-specific behavior. For a replacement, update the test source to a case that exercises colon parsing (for example use a different placement of ':' or the "as:" form) and change the assertions to validate the parser outcome that depends on the colon (e.g., assert the declaration alias/name/token is parsed as expected or that a missing/extra colon produces an error).



============================================================================
File: sea-core/src/validation_error.rs
Line: 192
Type: nitpick

[x] Task:
In sea-core/src/validation_error.rs around line 192, the comment contains an organizational artifact "Phase 15A:"; remove that prefix so the comment reads only "Additional convenience constructors for common error patterns" (matching the project's comment style and consistency).



============================================================================
File: .github/workflows/ci.yml
Line: 170
Type: potential_issue

[x] Task:
.github/workflows/ci.yml around line 170: the workflow uses a curl | sh pattern to install wasm-pack which is a supply-chain risk; replace it with a verified install via Rust's package manager by running cargo install wasm-pack (include --locked if you want reproducible builds), ensure the Rust toolchain is present (rustup/cargo available) in the job beforehand, and update PATH if necessary so the installed binary is found; remove the curl pipe-to-shell line and use the cargo-based installation step instead.



============================================================================
File: .github/workflows/ci.yml
Line: 176 to 183
Type: potential_issue

[x] Task:
.github/workflows/ci.yml around lines 176 to 183: the script assigns SIZE from stat but does not validate it before using an arithmetic comparison, which can fail if SIZE is empty or non-numeric; after setting SIZE, add an explicit validation step that ensures SIZE is a non-empty numeric value (e.g., check for empty or a regex/ctype-digit test), and if the check fails emit a clear ::error:: message and exit 1; only proceed to the numeric comparison against 524288 if the validation passes.



============================================================================
File: requirements.txt
Line: 1 to 2
Type: potential_issue

[x] Task:
In requirements.txt lines 1-2, replace the unmaintained tree-sitter-languages==1.10.2 entry with the maintained alternative tree-sitter-language-pack and pin it to a release compatible with tree-sitter 0.25.x (or use the latest tree-sitter-language-pack that aligns with tree-sitter 0.25.x); remove the tree-sitter-languages line and add tree-sitter-language-pack with a matching version constraint so the tree-sitter and language-pack versions remain compatible, then run pip-compile / pip install to regenerate lockfiles and verify imports/tests still work.



============================================================================
File: .coderabbit.yml
Line: 19 to 21
Type: potential_issue

[x] Task:
In .coderabbit.yml around lines 19 to 21, remove the unsupported rustfmt tool configuration block (the three lines defining "rustfmt: enabled: true config_file: .rustfmt.toml") so the file conforms to the CodeRabbit schema; delete those lines and adjust indentation/spacing if necessary to keep valid YAML (ensure there are no leftover blank keys or broken structure) and rely on the already-configured clippy for Rust checks.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 568
Type: potential_issue

[x] Task:
In sea-core/src/parser/ast.rs around line 568, the call pair.into_inner().next().unwrap() panics when literal content is missing; replace the unwrap with next().ok_or_else(|| / construct a ParseError or appropriate error type with a clear message like "missing literal content" /)? so the function returns a proper error instead of panicking and propagates it with ?. Ensure you construct the error using the existing parser error type/constructor used elsewhere in this module for consistency.



============================================================================
File: sea-core/tests/rdf_xml_typed_literal_tests.rs
Line: 26 to 32
Type: potential_issue

[x] Task:
In sea-core/tests/rdf_xml_typed_literal_tests.rs around lines 26 to 32, the test suppresses deprecation for Flow::new; replace this deprecated constructor with the current non-deprecated API: call the up-to-date Flow constructor (e.g., Flow::try_new or Flow::from_parts — whichever is provided by Flow's public API), adapt the call arguments to the new signature, handle any Result or error return (unwrap or expect in the test), and then remove the #[allow(deprecated)] attribute so the test uses the supported API and fails on future breaking changes.



============================================================================
File: sea-core/tests/rdf_xml_typed_literal_tests.rs
Line: 56
Type: potential_issue

[x] Task:
In sea-core/tests/rdf_xml_typed_literal_tests.rs around line 56, the test currently asserts a fragile exact string fragment for an XML closing tag; instead parse the generated rdf_xml and assert semantically: parse rdf_xml with an XML parser (e.g., roxmltree or quick-xml), locate the element whose local name is "label" and namespace is "http://domainforge.ai/rdfs#" (or resolve via the RDF/XML namespace mappings), extract its text content and assert it equals or contains "Warehouse" (or the expected literal value); this avoids brittle whitespace/prefix/formatting issues while keeping the same semantic check.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 525 to 528
Type: potential_issue

[x] Task:
In sea-core/src/parser/ast.rs around lines 525 to 528, the four .unwrap() calls in parse_quantified_expr can panic on malformed input; replace each unwrap() with ok_or_else that returns a descriptive parse error (matching the file's existing style) before calling the parse_ helpers — e.g., for each inner.next() use inner.next().ok_or_else(|| parse_error_with_context("expected  in quantified expression"))? and then pass that token to the corresponding parse_ function, so errors are propagated instead of panicking.



============================================================================
File: sea-core/tests/parser_integration_tests.rs
Line: 232
Type: potential_issue

[x] Task:
In sea-core/tests/parser_integration_tests.rs around line 232, the Flow declaration Flow "X" from "A" to "A" is missing the required quantity parameter; update this line to include a valid quantity (e.g., quantity 1 or the correct numeric amount for this test) so the Flow declaration matches the expected syntax used by other tests.



============================================================================
File: tmp/coderabbit_results.md
Line: 13 to 14
Type: potential_issue

[x] Task:
Finding 1 now acknowledges that task-06-rdf-sbvr-escaping.md:test_language_tag_preserved asserts both `xml:lang="en"` and the `<http://domainforge.ai/rdfs#label>` element value, so the language-tag handling test already verifies its behavior and no further change is required under this finding.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 537
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around line 537, the code rebuilds constructor args using only the first argument node which will drop additional arguments or complex multi-part expressions; update the transformer to collect all argument nodes (e.g., iterate arguments_node.named_children), reconstruct the original argument text by either joining each child's source text in order or by slicing from the first child's start to the last child's end to preserve nested/complex expressions, and adjust any callers to use that full reconstructed args string; add/extend the test around lines 399–411 to include multi-argument and nested-expression Entity::new(...) cases and verify tree-sitter named_children boundaries for Rust call expressions.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 756 to 758
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 756–758, the current behavior silently warns and returns when Resource::new has an unexpected arg count; change this to record and report skipped transformations instead of silently returning: add a counter and a collection of skipped locations (file and line or call snippet) and increment/populate it when len(args) != 2, keep the warning but do not just return silently; then at the end of the migration run emit a dry-run summary showing total transformed vs skipped and list/sketch of skipped call sites so operators can review; also add an optional strict flag that, when enabled, causes the code to raise an error and halt if any skips were recorded.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1360 to 1391
Type: nitpick

[x] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 1360 to 1391, the main() does not validate that the provided --file is a Rust source; add a check at about line 1398 that inspects parsed_args.file.suffix and if it is not in {'.rs', '.rs.bak'} call logger.error with a clear message and exit with sys.exit(1); also ensure imports for sys (and Path if missing) are present at the top of the file so the validation and exit work.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 88 to 101
Type: nitpick




============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1495 to 1502
Type: potential_issue

[x] Task:
docs/plans/tmp/task-07-fix-api-robustness.md lines 1495–1502: Section 6 promises a regex fallback when tree-sitter parsing fails but the code raises on parse errors; implement the fallback inside RustTransformer.parse_file() so it tries tree-sitter, catches parse exceptions, logs a warning with the error, invokes a simple regex-based fallback parser that extracts imports, function signatures and comments into a minimal parse result compatible with downstream transformers, writes a /rollback/parse_failure_.log entry, and returns the fallback parse result (make this behavior configurable via a flag and keep tree-sitter failure re-raise only when fallback is disabled), or alternatively remove the fallback statement from the plan if you choose to defer it.



============================================================================
File: sea-core/tests/sbvr_flow_facts_tests.rs
Line: 25 to 31
Type: potential_issue

[x] Task:
In sea-core/tests/sbvr_flow_facts_tests.rs around lines 25 to 31, remove the #[allow(deprecated)] attribute and replace the deprecated Flow::new(...) call with the current constructor (e.g., Flow::try_new(...) or the library's documented replacement) and handle the Result (unwrap or expect with a clear message) so the test uses the non-deprecated API; consult the Flow type docs or code to pick the exact replacement name and signature and pass the same camera_id, warehouse_id, factory_id and Decimal::new(100, 0) arguments.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 560 to 561
Type: potential_issue

[x] Task:
In sea-core/src/parser/ast.rs around lines 560 to 561, the code uses inner.next().unwrap() which panics on missing tokens; replace these unwraps with proper error handling by using inner.next().ok_or_else(|| ParseError::new("expected identifier for object/member"))? (or the project’s existing parse error constructor) and then pass the resulting token into parse_identifier(...)? so the function returns a controlled parse error instead of panicking, matching the codebase’s error-handling style.



============================================================================
File: sea-core/tests/rdf_xml_typed_literal_tests.rs
Line: 80 to 85
Type: potential_issue

[x] Task:
In sea-core/tests/rdf_xml_typed_literal_tests.rs around lines 80 to 85, the test currently only checks for presence of escaped entities which allows malformed outputs; replace the loose contains() assertions with a single equality assertion that the escaped result equals the exact expected string "&amp;&lt;&gt;&quot;&apos;". Update the test to call KnowledgeGraph::escape_xml("&\"'") and assert_eq!(escaped, expected) (or equivalent) so the transformation is verified exactly.



============================================================================
File: .github/workflows/release.yml
Line: 88
Type: potential_issue

[x] Task:
.github/workflows/release.yml around line 88: the workflow currently references the wrong maturin wheel output path (sea-core/target/wheels/.whl); change it to the default maturin output path target/wheels/.whl so the upload step picks up the generated wheels from the project root.



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1005 to 1010
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 1005 to 1010, the separator logic incorrectly handles cases where the last non-whitespace character before the closing brace is a comma (including trailing commas with extra spaces or newlines), producing malformed imports; change the logic to inspect the last non-whitespace character before the brace (not just rstrip().endswith(',')) and, if that character is a comma, set the fragment to insert a leading space plus the new import followed by a trailing comma (e.g., " import_name,") so the new item keeps the list comma-terminated, otherwise use ", " + import_name (no trailing comma); also add a unit test after line 942 in test_import_insertion.py that covers multiline imports with trailing commas to validate both inline and multiline spacing variants.



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 800 to 812
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 800 to 812, the test asserts schema_version == "1.0" but the default_schema_version() returns "2.0"; update the test to assert schema_version equals "2.0" (and optionally update the inline comment to reflect that the default injected version is 2.0) so the assertion matches the actual default.



============================================================================
File: sea-core/tests/wasm_tests.rs
Line: 8
Type: potential_issue

[x] Task:
In sea-core/tests/wasm_tests.rs around line 8, the import use sea_core::units::unit_from_string; is unused; remove that line from the file (delete the unused use statement) and save the file so the tests compile without unused-import warnings.



============================================================================
File: sea-core/tests/wasm_tests.rs
Line: 3 to 6
Type: potential_issue

[x] Task:
In sea-core/tests/wasm_tests.rs around lines 3 to 6, the wasm_bindgen_test_configure!(run_in_browser) macro is declared outside the cfg(feature = "wasm") mod wasm_tests block so its run_in_browser setting won't apply to those tests; move the wasm_bindgen_test_configure!(run_in_browser) invocation inside the mod wasm_tests scope (immediately after the mod declaration and before any test functions) so the configuration is in the same module as the tests and wrapped by #[cfg(feature = "wasm")].



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 1003 to 1012
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 1003 to 1012, the test compares flow_fact.destination (Option) directly to factory_id.to_string() (String) causing a type mismatch; change the assertion to compare against Some(factory_id.to_string()) (or unwrap/expect the Option first) so the types match — e.g. replace the failing assert_eq! with one that asserts flow_fact.destination == Some(factory_id.to_string()) or uses flow_fact.destination.as_ref().map(|s| s.as_str()) == Some(factory_id.as_str()) as appropriate.



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 1022 to 1031
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 1022 to 1031, the assignment to the destination field uses a String but the struct expects Option; change the assignment to provide an Option by wrapping the value in Some(...) or using .map(|s| s.to_string()) as appropriate (e.g., destination: Some(flow.to_id().to_string())), ensuring the field type matches Option.



============================================================================
File: sea-core/src/primitives/flow.rs
Line: 46 to 66
Type: nitpick

[x] Task:
In sea-core/src/primitives/flow.rs around lines 46 to 66, the new() constructor duplicates the logic of new_with_namespace by creating the default namespace and UUID itself; refactor new() to call new_with_namespace(DEFAULT_NAMESPACE.to_string(), resource_id, from_id, to_id, quantity) so it delegates creation and avoids duplicated code, preserving the same returned Flow structure and attributes initialization.



============================================================================
File: sea-core/src/wasm/primitives.rs
Line: 11 to 368
Type: nitpick

[x] Task:
In sea-core/src/wasm/primitives.rs around lines 11 to 368, multiple wrapper structs (Entity, Resource, Flow, Instance) repeat identical wasm-bindgen methods (set_attribute, get_attribute, to_json) and helper impls (inner, from_inner, into_inner); introduce a macro (e.g., impl_wasm_common!) that generates those shared #[wasm_bindgen] impl blocks and the helper impls parameterized by wrapper and inner types, place the macro definition above the struct definitions, replace each duplicated block with a single macro invocation like impl_wasm_common!(Entity, RustEntity) for each wrapper, and ensure the macro preserves #[wasm_bindgen(js_name = ...)] attributes, uses serde_wasm_bindgen/console/JsValue as before, and is visible to all usages (move necessary use/imports earlier if needed).



============================================================================
File: docs/plans/tmp/task-07-fix-api-robustness.md
Line: 1022 to 1023
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-07-fix-api-robustness.md around lines 1017–1023, ImportInserter._create_new_import currently sets insert_byte = 0 when no existing use_declaration is found, which can insert imports inside functions or comments; change the logic so that when last_use is None you compute an insertion point at the start of the module-level declarations by finding the end of any shebang and module docstring (or else the first top-level statement start) and use that byte offset instead of 0; then add a unit test in tests/test_import_insertion.py that creates a file with no imports but with a top-level function and asserts the new import is inserted before that function definition.



============================================================================
File: sea-core/src/primitives/instance.rs
Line: 41 to 52
Type: refactor_suggestion

[x] Task:
In sea-core/src/primitives/instance.rs around lines 41 to 52, the new() constructor duplicates the initialization logic present in new_with_namespace(); refactor new() to simply delegate to new_with_namespace() by returning Self::new_with_namespace(resource_id, entity_id, DEFAULT_NAMESPACE.to_string()) (or String::from(DEFAULT_NAMESPACE)) so the UUID, namespace assignment and attribute initialization remain centralized in new_with_namespace().



============================================================================
File: sea-core/src/calm/import.rs
Line: 185 to 189
Type: potential_issue

[x] Task:
In sea-core/src/calm/import.rs around lines 185 to 189, Simple relationship variants are currently no-ops which silently drop data; update the match arm to explicitly handle the "ownership" case (return a descriptive Err indicating Simple relationships are not yet supported for ownership) and for any other Simple rel_type either return a similar unsupported error or explicitly document via a clear comment that those Simple encodings are intentionally skipped; ensure the function propagates the error type used in this module and include the rel_type value in the error message for debugging.



============================================================================
File: sea-core/src/units/mod.rs
Line: 278 to 295
Type: nitpick

[x] Task:
In sea-core/src/units/mod.rs around lines 278 to 295, unit_from_string currently constructs UnitRegistry::default() on every call which wastes CPU and memory; instead create a single, shared, lazily-initialized registry (use std::sync::OnceLock or lazy_static) and call get_or_init to initialize UnitRegistry::default() once, then use that static registry reference inside unit_from_string; update imports accordingly and ensure the static is thread-safe and holds the registry for repeated lookups rather than recreating it each invocation.



============================================================================
File: sea-core/src/kg.rs
Line: 471 to 529
Type: refactor_suggestion

[x] Task:
In sea-core/src/kg.rs around lines 471 to 529 add unit tests that cover URI encoding and escaping edge cases: create entities/resources whose names include space, ':', '/', and '#' and assert exported RDF (turtle) contains distinct, correctly encoded URIs (no collisions); add literals containing escape sequences (\n, \t, \", \\) and assert they appear escaped in turtle output; add values with XML-special characters (&, , ", ') and assert rdf-xml output has them properly escaped or wrapped; include tests for typed literals (e.g. "100"^^xsd:decimal) and language-tagged literals ("hello"@en) and assert those exact tokens are present in the appropriate format; ensure tests use graph.export_rdf("turtle") and graph.export_rdf("rdf-xml") and assert Err(KgError::UnsupportedFormat(_)) for unsupported formats where appropriate.



============================================================================
File: sea-core/src/kg.rs
Line: 380 to 413
Type: potential_issue




============================================================================
File: sea-core/src/kg.rs
Line: 351 to 353
Type: potential_issue

[x] Task:
In sea-core/src/kg.rs around lines 351 to 353, the current uri_encode replaces [' ', ':', '/', '#'] with underscores which creates collisions (different identifiers mapping to the same URI); replace this with proper percent-encoding per RFC 3986 instead: use a percent-encoding routine (e.g. percent-encoding crate or std helpers) that leaves unreserved characters (ALPHA / DIGIT / '-' / '.' / '_' / '~') intact and percent-encodes reserved/unsafe characters (including space, ':', '/', '#'), then return the encoded string (preserve any existing "sea:" prefix logic elsewhere) so different inputs map to distinct, reversible URIs.



Review completed ✔
