# policy — Policy Model, Expressions, and Three-Valued Evaluation

## Overview
Defines `Policy` (modality, kind, priority, normalized expression) and evaluates
it against a `Graph`, producing `Violation`s. Expression trees, quantifier
expansion, normalization with stable hashing, and a `ThreeValuedBool` algebra
back the evaluator.

## Interface
- `[core.rs:Policy]` — `name`, `id` (`ConceptId`), `namespace`, modality, kind, priority, version, rationale, tags, metadata, normalized expression.
- `[core.rs:Policy::new]`, `[core.rs:Policy::new_with_namespace]` — constructors.
- `[core.rs:Policy::with_modality]`, `[core.rs:Policy::with_version]`, `[core.rs:Policy::with_kind]`, `[core.rs:Policy::with_priority]`, `[core.rs:Policy::with_rationale]`, `[core.rs:Policy::with_tags]`, `[core.rs:Policy::with_metadata]` — builder setters.
- `[core.rs:Policy::normalized_expression]`, `[core.rs:Policy::expression]`, `[core.rs:Policy::set_expression]`, `[core.rs:Policy::kind]` — accessors.
- `[core.rs:Policy::evaluate]`, `[core.rs:Policy::evaluate_with_mode]` — evaluate against a graph, optionally forcing the three-valued mode.
- `[core.rs:PolicyModality]` (re-exported as `DeonticModality` in `[core.rs]`), `[core.rs:PolicyKind]` — modality and kind enums.
- `[core.rs:EvaluationResult]` (`+has_errors/error_count`) — violations plus counts produced by one evaluation.
- `[core.rs:PolicyModality::to_severity]` — modality-to-`Severity` mapping.
- `[expression.rs:Expression]` — the expression tree, with `+normalize/is_equivalent/literal/variable/binary/unary/quantifier/cast/comparison/aggregation/member_access`.
- `[expression.rs:BinaryOp]`, `[expression.rs:UnaryOp]`, `[expression.rs:Quantifier]`, `[expression.rs:AggregateFunction]`, `[expression.rs:WindowSpec]` — expression vocabulary.
- `[quantifier.rs:Expression::expand]`, `[quantifier.rs:Expression::substitute]` — quantifier expansion over a graph and variable substitution.
- `[normalize.rs:NormalizedExpression]` (`+new/inner/into_inner/stable_hash`) — canonical form used for equivalence and hashing.
- `[three_valued.rs:ThreeValuedBool]` (`+and/or/not/implies/from_option_bool/into_option/fold_and/fold_or`) plus `sum_nullable`, `sum_nonnull`, `count_all`, `count_nonnull` — three-valued algebra and null-aware aggregates.
- `[violation.rs:Severity]`, `[violation.rs:Violation]` (`+new/with_context`) — violation records.
- `[type_inference.rs:ExprType]`, `[type_inference.rs:TypeError]`, `[type_inference.rs:check_comparison]` — comparison type checking (exposed via `pub mod type_inference;`).
- `[mod.rs]` — re-exports `DeonticModality`, `EvaluationResult`, `Policy`, `PolicyKind`, `PolicyModality`, `AggregateFunction`, `BinaryOp`, `Expression`, `Quantifier`, `UnaryOp`, `WindowSpec`, `NormalizedExpression`, `ThreeValuedBool`, `Severity`, `Violation`.

## Invariants
- `[core.rs:DeonticModality]` — `DeonticModality` is an alias re-export of `PolicyModality` (`pub use PolicyModality as DeonticModality;`), not a distinct type.
- `[core.rs:Policy::evaluate_with_mode]` — evaluation mode is explicit: `Graph::validate` passes the graph's `use_three_valued_logic` flag, so strict-two-valued and three-valued results differ by configuration, not by call site.
- `[normalize.rs:NormalizedExpression::stable_hash]` — two expressions that normalize to the same form must produce the same stable hash; equivalence is defined on the normalized form.
- `[three_valued.rs:ThreeValuedBool]` — `and`/`or`/`not`/`implies` are Kleene three-valued operations; `UNKNOWN` is not collapsed to `false`.
- `[type_inference.rs:check_comparison]` — incompatible operand types are rejected with a `TypeError` rather than evaluated.

## Side Effects
- None observed in source. The policy module is a pure in-memory evaluator; `[three_valued_microbench.rs]` is `#[cfg(test)]`-only.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
