Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: sea-core/src/cli/validate.rs
Line: 103 to 109
Type: potential_issue

[x] Task:
In sea-core/src/cli/validate.rs around lines 103 to 109, the parameters _show_source and _source are declared but unused, leaving the --show-source flag non-functional; either implement the feature or revert the API change: to implement, remove the leading underscores (rename to show_source and source), add conditional logic after validation to, when show_source is true and source.is_some(), extract the relevant source snippet(s) for each reported validation error and include them in the output (respecting the chosen OutputFormat and use_color), and update any callers to pass the source and flag; alternatively, if you prefer to delay the feature, remove these parameters and the flag from the public API and callers so no non-functional flag is exposed.



============================================================================
File: examples/observability_metrics.sea
Line: 7 to 8
Type: potential_issue

[x] Task:
In examples/observability_metrics.sea around lines 7 to 8, the entities reference the shorthand namespace "payments" while the file declares "com.example.payments" at line 4; update the entity declarations to use the fully qualified namespace "com.example.payments" (e.g., Entity "PaymentProcessor" in com.example.payments) or, if the DSL explicitly supports shorthand, add a brief comment or a validation step confirming shorthand resolution is enabled and consistent; ensure all entity namespace references match the declared namespace to avoid resolution errors.



============================================================================
File: sea-core/src/lib.rs
Line: 111 to 112
Type: potential_issue

[x] Task:
In sea-core/src/lib.rs around lines 111 to 112, the module-level documentation that claims "five core primitives" is out of sync with the code which registers python::primitives::Mapping and python::primitives::Projection; update the doc comment (lines ~3-8) to list all actual core primitives including Mapping and Projection, and then verify that both python::primitives::Mapping and python::primitives::Projection are implemented and exported in src/python/primitives.rs (or the appropriate module file); if either is missing, implement the struct/class with the expected methods and PyO3 bindings and add the pub use/export so m.add_class::() can find them.



============================================================================
File: sea-core/src/primitives/mapping_contract.rs
Line: 5 to 30
Type: nitpick

[x] Task:
In sea-core/src/primitives/mapping_contract.rs around lines 5 to 30, the MappingContract struct exposes all fields as pub which makes the new() constructor optional and prevents enforcing invariants or changing the internal representation safely; to address this, make the struct fields private (remove pub on each field), retain or adjust the existing pub fn new(...) as the canonical constructor, and add minimal pub getter methods (or implement AsRef/traits) for any fields consumers need to read so the API remains stable and invariants can be enforced internally.



============================================================================
File: examples/observability_metrics.sea
Line: 11
Type: potential_issue

[x] Task:
In examples/observability_metrics.sea around line 11, the Resource "Money" references an undefined namespace "finance"; either verify that "finance" is a valid predefined namespace or declare it explicitly before line 11. Fix by adding a namespace declaration for "finance" above line 11 (or remove/replace the namespace reference on the Resource if it was incorrect) so the Resource no longer refers to an undeclared namespace.



============================================================================
File: sea-core/src/projection/mod.rs
Line: 5 to 7
Type: nitpick

[x] Task:
In sea-core/src/projection/mod.rs around lines 5 to 7, the file currently uses wildcard re-exports (pub use contracts::; pub use engine::; pub use registry::*;); replace these with explicit re-export lists: open each child module (contracts, engine, registry), determine which items (types, traits, functions, constants) are part of the public API, and replace each wildcard with a comma-separated pub use listing of those items (e.g., pub use contracts::{ItemA, ItemB, TraitC};) to avoid name collisions; after editing, run cargo check and the test suite to ensure no missing or accidentally-private items and adjust visibility (pub vs pub(crate>) if necessary.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 556 to 558
Type: potential_issue

[x] Task:
In sea-core/src/python/primitives.rs around lines 556–558, the method target_format currently returns the Debug representation (format!("{:?}", self.inner.target_format)), which exposes internal Debug output to Python users; change it to return a stable, user-friendly string by using the Display implementation (format!("{}", ...)) or by explicitly mapping the enum/struct to a descriptive string (e.g., match on target_format and return human-readable names) and ensure the underlying type implements Display if needed so Python consumers receive a stable, readable format.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 1681 to 1686
Type: nitpick

[x] Task:
In sea-core/src/parser/ast.rs around lines 1681 to 1686, the boolean branch uses a case-sensitive comparison (== "true") while parse_literal_expr (around line 1379) uses eq_ignore_ascii_case("true"); change the boolean parsing here to use value_pair.as_str().eq_ignore_ascii_case("true") (or the same helper used by parse_literal_expr) so boolean parsing is consistent and case-insensitive across the parser.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 561 to 565
Type: refactor_suggestion

[x] Task:
In sea-core/src/python/primitives.rs around lines 561 to 565, the Mapping wrapper defines from_rust but is missing a matching into_inner method used by other wrappers; add a public consuming method pub fn into_inner(self) -> RustMapping that returns the inner field so Python-to-Rust conversions can pass Mapping back into Rust functions, keeping naming and signature consistent with Entity/Resource/Flow/ResourceInstance/Instance.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 567 to 590
Type: refactor_suggestion

[x] Task:
In sea-core/src/python/primitives.rs around lines 567 to 590, add a Python __repr__ method inside the existing #[pymethods] impl block for Projection: implement a fn __repr__(&self) -> String that returns a concise, human-readable representation including the projection name and target_format (e.g., "Projection(name='', target_format='')") by calling the existing getters; ensure it returns String and does not mutate state so it can be used for debugging in Python REPLs.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 586 to 590
Type: refactor_suggestion

[x] Task:
In sea-core/src/python/primitives.rs around lines 586 to 590, the Projection wrapper lacks an into_inner method for consistency with other wrappers; add a public consuming accessor named into_inner that takes self and returns the inner RustProjection (i.e., pub fn into_inner(self) -> RustProjection { self.inner }) so Python-side Projection objects can be converted back to Rust types when passed into Rust functions.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 542 to 565
Type: refactor_suggestion

[x] Task:
In sea-core/src/python/primitives.rs around lines 542 to 565, the Mapping Python wrapper lacks a __repr__ implementation which other similar classes have; add a __repr__ method inside the existing #[pymethods] impl block that returns a descriptive string including the mapping name and target_format (use self.inner.name and the formatted target_format) so Python debugging and REPL prints show a clear representation.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 1708 to 1712
Type: potential_issue

[x] Task:
In sea-core/src/parser/ast.rs around lines 1708 to 1712, the code currently drops precision with unwrap_or(0.0) and may panic on NaN/Infinity via from_f64(...).unwrap(); instead preserve the decimal precisely by converting the parsed decimal to its string form and construct the serde_json::Number via serde_json::Number::from_str(&d.to_string()) (or propagate/return an error if from_str fails), and remove the unwraps — if from_str cannot produce a valid JSON number (e.g., NaN/Infinity), handle that case explicitly (return an error or a safe fallback such as JsonValue::Null) rather than unwrapping.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 537 to 559
Type: nitpick

[x] Task:
In sea-core/src/typescript/primitives.rs around lines 537 to 559, add two helper accessors to Mapping for consistency with Entity/Resource/Flow: implement pub fn into_inner(self) -> RustMapping that consumes the wrapper and returns the inner RustMapping, and pub fn inner_ref(&self) -> &RustMapping that returns a shared reference to the inner field; place them in the impl Mapping block (non-#[napi] impl) so they are available for Rust-side interop but not exported to JS.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 561 to 583
Type: nitpick

[x] Task:
In sea-core/src/typescript/primitives.rs around lines 561 to 583, the Projection wrapper currently exposes getters but lacks the internal helper accessors; add a consuming into_inner(self) -> RustProjection and a non-consuming inner_ref(&self) -> &RustProjection to match the Mapping pattern so other Rust code can either take ownership of the inner RustProjection or borrow it immutably—implement both as inherent impl methods (not napi exported).



============================================================================
File: sea-core/src/python/primitives.rs
Line: 581 to 583
Type: potential_issue

[x] Task:
In sea-core/src/python/primitives.rs around lines 581 to 583, the method target_format currently uses Debug formatting (format!("{:?}", ...)) which leaks internal debug representations to Python; change it to return a stable, user-facing string by using a Display or explicit to_string conversion (e.g., call .to_string() or format!("{}", ...) on the target_format) or map the enum/struct to a clear human-readable string; if target_format lacks Display, implement Display for it or provide a method that returns a stable label and use that here.



============================================================================
File: sea-core/src/wasm/primitives.rs
Line: 281 to 293
Type: potential_issue

[x] Task:
In sea-core/src/wasm/primitives.rs around lines 281 to 293, restore a to_json method on Instance to match Entity/Resource/Flow: add a pub fn to_json(&self) -> JsValue (or the same return type used by the other primitives) that delegates serialization to the inner RustInstance (either by calling inner.to_json() if available, or by serializing self.inner with serde_wasm_bindgen/serde_json to produce the same JS-friendly value used by the other primitives); ensure the signature and returned value match the other primitives so JavaScript consumers see a consistent serialization API.



============================================================================
File: sea-core/src/kg.rs
Line: 158 to 167
Type: potential_issue

[x] Task:
In sea-core/src/kg.rs around lines 158 to 167, rdf_class and property predicate values are taken directly from rule.fields and inserted into RDF triples without validation; add an is_valid_rdf_term(&str) -> bool helper at the end of the impl KnowledgeGraph block (matching the provided validation rules for prefixed names vs local names) and use it when assigning rdf_class and when iterating properties: validate each candidate term and skip (or return an Err/log) for invalid terms instead of inserting them, ensuring no term containing quotes, angle brackets, control chars, backslashes or illegal colons is accepted.



============================================================================
File: sea-core/src/calm/export.rs
Line: 191 to 193
Type: potential_issue




============================================================================
File: sea-core/src/calm/export.rs
Line: 188
Type: potential_issue

[x] Task:
In sea-core/src/calm/export.rs around line 188, the current use of graph.get_resource(flow.resource_id()).map(|r| r.name()).unwrap_or("") silently falls back to an empty string when the resource is missing, which causes mapping lookups to fail without notice; change this to explicitly handle the None case by matching the Option, log or return an error including the flow.resource_id (and flow info) when the resource is missing, and ensure you either skip processing this flow with a clear warning or propagate an Err so mappings are not silently skipped; replace the unwrap_or with a match (or if let) that obtains the resource name when Some, and performs the chosen explicit handling when None.



Review completed ✔
