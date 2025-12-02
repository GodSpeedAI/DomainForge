Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: docs/plans/enhanced-aggregations-implementation.md
Line: 14
Type: potential_issue

[x] Task:
In docs/plans/enhanced-aggregations-implementation.md around lines 14 and 25, the callout lines contain a duplicated '>' character (e.g. "> [!IMPORTANT] > ..."); remove the extra '>' that appears immediately after the callout tag so the lines read with a single '>' before the callout or just the callout tag followed by the content (e.g. "> [!IMPORTANT] Syntax Decision:"), ensuring consistent callout block syntax.



============================================================================
File: docs/plans/enhanced-aggregations-implementation.md
Line: 42
Type: potential_issue

[x] Task:
In docs/plans/enhanced-aggregations-implementation.md around lines 42 and 48, the grammar and examples disagree about whether the "where" clause is mandatory; update the grammar rules to make "where" optional (use ("where" ~ expression)? in both the group_by_expr and aggregation_comprehension rules) and/or update the examples to include an explicit where clause so they match the grammar—ensure both rule definitions and all usage examples consistently either include the optional "where" or demonstrate omission.



============================================================================
File: sea-core/grammar/sea.pest
Line: 148 to 154
Type: nitpick

[x] Task:
In sea-core/grammar/sea.pest around lines 148 to 154, the aggregation_comprehension rule uses a case-sensitive "in" token on line 149 which is inconsistent with other case-insensitive tokens; change that literal to a case-insensitive token (use ^"in") so the grammar treats "in" the same way as other keywords, keeping spacing and surrounding sequence intact.



============================================================================
File: sea-core/grammar/sea.pest
Line: 135 to 137
Type: nitpick

[x] Task:
In sea-core/grammar/sea.pest around lines 135 to 137, the group_by_expr rule uses a case-sensitive "in" while entity_decl uses ^"in"; update the group_by_expr rule to use ^"in" instead of "in" so the keyword matching is consistent (accepting the same case variants) with the rest of the grammar, and run the parser tests to confirm no regressions.



============================================================================
File: sea-core/src/policy/expression.rs
Line: 5 to 9
Type: potential_issue

[x] Task:
In sea-core/src/policy/expression.rs around lines 5-9, the WindowSpec.duration field is currently i64 which allows negative values; change its type to an unsigned integer (e.g., u64) to represent non-negative durations, update any code that constructs or uses WindowSpec to accept/produce u64, and ensure serde derives still work (adjust any deserialization sites and tests); if changing the type is infeasible, add explicit validation in constructors/deserializers to reject negative durations and return a clear error instead.



============================================================================
File: sea-core/src/parser/ast.rs
Line: 1127 to 1139
Type: nitpick

[x] Task:
In sea-core/src/parser/ast.rs around lines 1127 to 1139, the code currently clones Pair values when calling parse_expression; change this to consume the vector so you don't clone Pairs. Collect remaining pairs as Vec> then match on remaining.as_slice() (or destructure via let [a,b,c] = &remaining[..] for refs then std::mem::take/remove to own them) or better: build the vec via into_iter().collect::>() and pattern-match using remaining.into_iter() to extract ownership of each Pair and pass them directly to parse_expression without .clone(); update the branches to destructure into owned variables and call parse_expression(owned_pair)?, and keep the same error branch for unexpected lengths.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 600 to 606
Type: potential_issue

[x] Task:
In sea-core/src/policy/quantifier.rs around lines 600 to 606, the match currently falls back to seconds for unknown window units; change this so unknown units produce an explicit error instead of silently using seconds. Replace the default arm with an error return (e.g., return Err(...) or anyhow::bail!) that includes the invalid unit string and context, and if the containing function does not already return a Result, update its signature to propagate the error and adjust callers accordingly so invalid units fail fast with a clear message.



============================================================================
File: sea-core/src/policy/quantifier.rs
Line: 89 to 93
Type: nitpick

[x] Task:
In sea-core/src/policy/quantifier.rs around lines 89–93, the code uses v.to_string() on a serde_json::Value which produces JSON-encoded output (e.g., string "foo" becomes "\"foo\""); replace this with explicit matching on Value variants and produce a cleaner key string: if Value::String(s) use s.clone(), if Number use its string form, if Bool use "true"/"false", and handle/null or other variants consistently (e.g., "null" or return an error) so group keys are human-readable and comparable outside JSON formatting.



Review completed ✔
