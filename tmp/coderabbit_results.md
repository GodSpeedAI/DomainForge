Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: docs/plans/feature8-implementation-summary.md
Line: 89 to 143
Type: potential_issue




============================================================================
File: sea-core/src/primitives/metric.rs
Line: 5
Type: potential_issue

[ ] Task:
In sea-core/src/primitives/metric.rs around lines 5, 13, and 18, chrono::Duration is being used but chrono's Duration does not implement Serialize/Deserialize without enabling the "serde" feature; fix by either adding the serde feature to the chrono dependency in Cargo.toml (e.g. enable features = ["serde"]) so chrono::Duration can be serialized, or replace chrono::Duration with std::time::Duration (update the use/import, adjust any arithmetic/formatting differences, and ensure serde derives/attributes work for the new type). Ensure affected structs/enums have matching types and update any serde attributes or conversions accordingly.



============================================================================
File: sea-core/src/primitives/mod.rs
Line: 21 to 22
Type: nitpick

[ ] Task:
In sea-core/src/primitives/mod.rs around lines 21 to 22, the review notes that the metric module may not exist and its declaration breaks the alphabetical ordering of module declarations; ensure that sea-core/src/primitives/metric.rs (or metric/mod.rs) is present, move the pub mod metric; line so it appears alphabetically between the instance and quantity module declarations, and update the re-exports section to include Metric and Severity in the correct alphabetical position among other re-exports so imports remain consistent.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 487 to 490
Type: potential_issue

[ ] Task:
In sea-core/src/python/primitives.rs around lines 487 to 490, the namespace getter currently returns String; change it to return Option and follow the established pattern used by other structs: if self.inner.namespace == "default" return None, otherwise return Some(self.inner.namespace.clone()); update the function signature to fn namespace(&self) -> Option and ensure clone is used when wrapping in Some.



============================================================================
File: sea-core/src/python/primitives.rs
Line: 492 to 495
Type: potential_issue

[ ] Task:
In sea-core/src/python/primitives.rs around lines 492–495, the getter currently maps Decimal to f64 using unwrap_or(0.0), which silently converts failed Decimal->f64 conversions to 0.0; change the mapping to preserve conversion failures instead of returning 0.0 — i.e., replace the unwrap_or usage with logic that attempts the conversion and returns None on failure (for example use the Decimal->f64 conversion result as an Option via .ok() or equivalent) so a failed conversion yields None rather than a potentially ambiguous 0.0.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 52 to 61
Type: potential_issue




============================================================================
File: sea-core/src/python/primitives.rs
Line: 497 to 500
Type: potential_issue

[ ] Task:
In sea-core/src/python/primitives.rs around lines 497 to 500, the target getter silently converts a D::to_f64 failure to 0.0 which masks data corruption; change the function signature from returning Option to returning PyResult> and handle the conversion error explicitly instead of using unwrap_or(0.0): attempt to convert inner.target with to_f64(), map Some(converted) on success, return Ok(None) if inner.target is None, and on conversion failure return an Err PyErr with a clear message so the Python caller receives an exception rather than a silent 0.0.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 492 to 495
Type: potential_issue

[ ] Task:
In sea-core/src/typescript/primitives.rs around lines 492 to 495, the namespace getter currently returns String but must follow the established pattern of returning Option and filtering out the "default" namespace; change the function signature to pub fn namespace(&self) -> Option and return None when self.inner.namespace == "default" (or is empty if other getters treat empty similarly), otherwise return Some(self.inner.namespace.clone()) so the TypeScript API contract is consistent.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 497 to 505
Type: potential_issue

[ ] Task:
In sea-core/src/typescript/primitives.rs around lines 497-505, the getters for threshold and target currently map Decimal values to f64 using unwrap_or(0.0), which hides NaN/Infinity; change the mapping to follow the Flow::quantity() pattern: call to_f64(), check the resulting Option and return Some(f) only if f.is_finite(), otherwise return None (i.e., filter out NaN/Infinity rather than converting them to 0.0) so both getters return None for non-finite conversions.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 2007 to 2046
Type: nitpick

[ ] Task:
In sea-core/src/parser/ast.rs around lines 2007 to 2046, the match arm currently swallows unknown metric annotation keys (the _ => {} case), which hides typos or unsupported annotations; replace the empty fallback with an explicit error return (ParseError::GrammarError) that includes the offending key so callers get a clear failure. Use the already-captured key_pair to build the message (e.g., include key_pair.as_str() or its span text) and return Err(...) from that arm instead of ignoring it.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 1949 to 1958
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 1949–1958, the code currently maps unknown severity strings to Info silently; change this to validate the severity and return a parse error instead of defaulting. Replace the wildcard branch with logic that treats unknown values as an error (including the invalid string and location/span in the error message), propagate that error out of the current function (or convert to the parser's Error type) so callers can handle/report it, and update any call sites to handle the Result/Error return if necessary.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 1964 to 1973
Type: potential_issue




============================================================================
File: sea-core/src/parser/ast.rs
Line: 1930 to 1939
Type: potential_issue

[ ] Task:
In sea-core/src/parser/ast.rs around lines 1930 to 1939, the refresh_interval branch currently silently falls back to seconds for unknown units and can panic on overflow; update it to validate the unit string and return or propagate a descriptive error for unrecognized units instead of defaulting, and replace direct chrono::Duration:: calls with the safe/checked constructors (or checked arithmetic) to detect overflow — if creation fails, clamp to chrono::Duration::max_value()/min_value() or return an error so the overflow is not panicked. Ensure the code maps only the allowed unit tokens ("s","seconds","m","minutes","h","hours","d","days") and surfaces failures to the caller rather than hiding them.



============================================================================
File: sea-core/src/graph/mod.rs
Line: 44 to 45
Type: potential_issue

[ ] Task:
In sea-core/src/graph/mod.rs around lines 44-45, the #[serde(default)] attribute is currently placed before entity_roles but not metrics; move or add a #[serde(default)] attribute immediately above the metrics field declaration (and keep/add #[serde(default)] above entity_roles as well) so both fields have serde defaults during deserialization, then run/adjust tests to ensure existing graphs without metrics still deserialize correctly.



Review completed ✔
