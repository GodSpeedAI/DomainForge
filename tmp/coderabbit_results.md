Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: schemas/ast-v1.schema.json
Line: 53 to 62
Type: potential_issue

[x] Taskt:
In schemas/ast-v1.schema.json around lines 53 to 62, the Flow.quantity property currently allows any number; restrict it to positive values by adding an exclusive minimum constraint so quantity must be greater than 0 (e.g., add "minimum": 0 and "exclusiveMinimum": true to the quantity schema). Ensure the schema remains valid JSON Schema and update any related validators/tests if they assume non-positive values.



============================================================================
File: schemas/ast-v1.schema.json
Line: 5 to 29
Type: potential_issue

[x] Taskt:
In schemas/ast-v1.schema.json around lines 5–29, the root schema is missing a "required" array so mandatory top-level properties aren't enforced; add a "required" property listing the fields that must be present (for example "version", "namespace", "entities", "resources", and "flows" — adjust to your DSL if some are optional), and optionally add "minItems": 1 to the array schemas (entities/resources/flows) to ensure they are non-empty when required; keep the existing type/pattern constraints intact.



============================================================================
File: sea-core/src/parser/lint.rs
Line: 1 to 33
Type: nitpick

[x] Taskt:
In sea-core/src/parser/lint.rs around lines 1 to 33, add an implementation of the Default trait for Linter since new() takes no parameters; implement Default by returning Self::new() so callers can use Linter::default() and derive/construct more idiomatically across code that expects Default.



============================================================================
File: sea-core/src/parser/mod.rs
Line: 19
Type: potential_issue

[x] Taskt:
In sea-core/src/parser/mod.rs around line 19, the current wildcard re-export pub use lint::*; exposes all public items from the lint module and risks accidental breaking changes and name collisions; replace it with explicit re-exports by listing only the specific public types/functions/constants you intend to expose (e.g., pub use lint::Foo; pub use lint::bar_fn; etc.). Inspect sea-core/src/parser/lint.rs (or lint/mod.rs) for its public items, pick the intended public API surface, and update line 19 to a set of explicit pub use lint::; statements matching those names; run cargo check to ensure no missing exports or name conflicts remain.



============================================================================
File: sea-core/tests/quantity_tests.rs
Line: 27 to 28
Type: potential_issue

[x] Taskt:
In sea-core/tests/quantity_tests.rs around lines 27-28, replace the fragile substring checks that assert separately for "1" and "500" with a single pattern-based assertion that verifies the full thousand-separated number format; use a regex like r#""\d{1,3}(?:[ ,]\d{3})*""# to match numbers with space or comma thousands separators (and keep the existing check that the string contains "\"EUR\""), i.e. assert that formatted matches the regex for the whole number token and that the currency "EUR" is present.



============================================================================
File: sea-core/src/primitives/quantity.rs
Line: 7 to 12
Type: potential_issue

[x] Taskt:
In sea-core/src/primitives/quantity.rs around lines 7–12, the public f64 value (and public unit/dimension) allows callers to set NaN/Infinity or domain-invalid values; make the struct fields private, add a validated constructor (e.g., new or try_new) that checks value.is_finite() and enforces domain rules (e.g., non-negative for mass/other relevant dimensions) and returns Result/Err, add simple getter methods for value, unit, and dimension as suggested, update any serde usage if needed (private fields are fine with derive) and change call sites to use the constructor/getters instead of writing fields directly.



============================================================================
File: sea-core/src/parser/printer.rs
Line: 26 to 27
Type: nitpick

[x] Taskt:
In sea-core/src/parser/printer.rs around lines 26 to 27, the code uses unwrap() on writeln! calls that write to a String; replace those unwrap() calls with explicit, intentional ignoring of the Result by using let _ = writeln!(output, "namespace {}", ns) and let _ = writeln!(output) so the writes remain infallible but signal purposeful omission of error handling.



============================================================================
File: sea-core/src/primitives/quantity.rs
Line: 14 to 22
Type: potential_issue

[x] Taskt:
In sea-core/src/primitives/quantity.rs around lines 14–22, the Quantity::new constructor currently accepts any f64 including NaN and Infinity; update it to validate the value using value.is_finite() and fail fast on invalid numbers. Change the signature to return a Result (e.g., Result or Result), check if value.is_finite() and return an Err with a clear message for NaN/Infinity, otherwise construct and return Ok(Self { ... }); keep the rest of the fields unchanged.



============================================================================
File: sea-core/src/parser/printer.rs
Line: 13 to 19
Type: nitpick

[x] Taskt:
In sea-core/src/parser/printer.rs around lines 13 to 19, implement the Rust Default trait for the struct (replace or augment the existing new() implementation) so the default values (indent_width: 4, max_line_length: 80, trailing_commas: false) are provided via impl Default for ... { fn default() -> Self { ... } }, and then have pub fn new() -> Self simply call Self::default(); update visibility and imports if needed and remove duplicated literal construction to keep new() delegating to Default.



============================================================================
File: sea-core/src/primitives/quantity.rs
Line: 33
Type: nitpick

[x] Taskt:
In sea-core/src/primitives/quantity.rs around line 33, the call uses locale.clone().into() which performs an unnecessary allocation; change it to (&locale).into() (or locale.into() if the conversion accepts a reference) so the FixedDecimalFormatter::try_new receives a borrowed conversion and avoids cloning the locale.



============================================================================
File: sea-core/src/parser/printer.rs
Line: 54 to 65
Type: potential_issue

[x] Taskt:
In sea-core/src/parser/printer.rs around lines 54 to 65 the Policy branch prints the policy expression using debug formatting ({:?}), which is not production-ready; replace the debug output with a proper expression printer (either call an existing expression pretty-print function or implement Display/pretty_print on the expression AST and write that to output with correct indentation) and add a short TODO noting that expression printing should be tested and cover nested/complex expressions; ensure the new printing uses the same writeln! pattern and propagates errors instead of unwrapping where practical.



============================================================================
File: sea-core/src/primitives/quantity.rs
Line: 32 to 35
Type: potential_issue




============================================================================
File: sea-core/src/primitives/quantity.rs
Line: 38 to 45
Type: potential_issue

[x] Taskt:
In sea-core/src/primitives/quantity.rs around lines 38-45, the current code converts f64 -> String -> FixedDecimal and uses unwrap_or_else to silently replace parse failures with 0; this masks precision loss and parsing errors. Replace the silent fallback: construct FixedDecimal directly from the f64 if available (e.g. FixedDecimal::from_f64 or appropriate try_from API) or propagate the conversion error by changing the function signature to return Result (or a domain error type) and return Err on conversion failure; ensure callers handle the Result instead of receiving a silently zeroed value.



Review completed ✔
