Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing



============================================================================
File: sea-core/tests/instance_parsing_tests.rs
Line: 30 to 41
Type: potential_issue

Prompt for AI Agent:
In sea-core/tests/instance_parsing_tests.rs around lines 30 to 41, the test named test_parse_instance_with_quantity_field does not include a quantity field in its source data; update the test source so it includes a quantity field (e.g., add a line like quantity: 10 or quantity: 1 with appropriate formatting and trailing comma) inside the Instance block before the closing brace, then re-run the test to ensure parsing still succeeds.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 607 to 628
Type: potential_issue

Prompt for AI Agent:
In sea-core/src/parser/ast.rs around lines 607 to 628, the code unconditionally calls HashMap::insert which silently overwrites duplicate field names; change this to detect duplicates and return a ParseError instead of overwriting: before inserting, check if fields already contains the field_name (or use the Entry API) and if so return Err(ParseError::GrammarError(format!("Duplicate field name '{}'", field_name))) so DSL authors get a clear error; optionally include the field_pair span or line info in the error message for better diagnostics.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 1520 to 1523
Type: potential_issue

Prompt for AI Agent:
In sea-core/src/parser/ast.rs around lines 1520–1523, the catch-all branch currently serializes unsupported expressions with format!("{:?}", expr), producing non-portable debug strings; replace this fallback with an explicit error return that identifies the unsupported expression kind and its span (e.g., return Err(ParseError::UnsupportedExpression { kind: expr.kind(), span: expr.span() }.into()) ), or implement a proper Serialize-to-JSON path for complex Expr nodes and use that instead; ensure the resulting Result uses a clear, structured error rather than a debug-formatted string so downstream consumers receive machine-readable failures.



============================================================================
File: sea-core/src/graph/mod.rs
Line: 34
Type: potential_issue

Prompt for AI Agent:
In sea-core/src/graph/mod.rs around line 34, entity_instances is keyed by String (instance name) while all other Graph collections use ConceptId; either document why string keys are intentional or change the map to IndexMap and add a name-based lookup helper (e.g., find_instance_by_name) mirroring find_entity_by_name. Update all usages, insert/lookup code, and any unit tests to use ConceptId keys (or keep String and add a clear comment explaining rationale and tradeoffs), and update module docs/comments to reflect the chosen approach.



============================================================================
File: sea-core/grammar/sea.pest
Line: 122 to 124
Type: nitpick

Prompt for AI Agent:
sea-core/grammar/sea.pest around lines 122 to 124: the keyword casing is inconsistent — change ^"Instance" to lowercase ^"instance" to match other declarations (entity/resource/flow) so the grammar stays consistent; update the rule to use ^"instance" and run the parser tests to ensure no behavioral change since the token is case-insensitive.



============================================================================
File: sea-core/src/typescript/primitives.rs
Line: 432 to 445
Type: potential_issue




============================================================================
File: sea-core/src/graph/mod.rs
Line: 432 to 438
Type: potential_issue

Prompt for AI Agent:
In sea-core/src/graph/mod.rs around lines 432 to 438, add_entity_instance currently inserts an Instance without validating that any entity references inside the Instance actually exist in the graph; inspect crate::primitives::Instance to see which field(s) hold entity references (e.g., an entity_name or referenced_entities list) and, if present, iterate those references and check each against the graph's entity map (the same map used by add_instance around lines 362-367), returning an Err with a clear message for the first missing entity; only insert the instance after all referenced entities are confirmed to exist.



============================================================================
File: sea-core/src/primitives/resource_instance.rs
Line: 5
Type: nitpick

Prompt for AI Agent:
In sea-core/src/primitives/resource_instance.rs around line 5, replace the crate-level import use uuid; with a specific import use uuid::Uuid;, and on line 52 change any fully-qualified uses like uuid::Uuid (or calls such as uuid::Uuid::new_v4()) to the direct Uuid form (e.g., Uuid::new_v4() or Uuid type) so the code uses the imported type directly.



============================================================================
File: sea-core/src/graph/mod.rs
Line: 427 to 457
Type: nitpick

Prompt for AI Agent:
In sea-core/src/graph/mod.rs around lines 427 to 457, the methods repeatedly use the fully qualified type crate::primitives::Instance which is verbose; add a module-level alias or import (for example use crate::primitives::Instance; or type Instance = crate::primitives::Instance;) near the top of the module, then update the signatures and return types in these methods to use the shorter Instance identifier (e.g., change crate::primitives::Instance to Instance) and adjust any other occurrences in this section accordingly to improve readability.



Review completed ✔
