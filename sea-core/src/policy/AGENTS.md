# policy — expression evaluation & three-valued logic

The semantic heart. Evaluates policy expressions against a `Graph` under
**three-valued (Kleene) logic** — the canonical semantics of the standard layer.

## Files

| File | Role |
|------|------|
| `core.rs` (1.2k) | `Policy`, `EvaluationResult`, `EvaluationMode`, `evaluate_with_mode`, member-access → NULL |
| `expression.rs` | `Expression` AST: Binary/Unary/Comparison/Quantifier/Aggregation/GroupBy |
| `three_valued.rs` | `ThreeValuedBool` (True/False/Null) Kleene AND/OR/NOT, forall/exists folds |
| `quantifier.rs` (940) | Collection iteration + substitution; missing/Null attribute → `Literal(Null)` |
| `normalize.rs` (1k) | Canonical/normalized expression form (equivalence, deterministic display) |
| `violation.rs` | `Violation`, `Severity` (Error/Warning/Info) |
| `type_inference.rs` | Static type inference over expressions |

## Critical invariants

- `is_satisfied_tristate: Option<bool>` is authoritative. `is_satisfied` is the
  **fail-closed** back-compat boolean (`tristate.unwrap_or(false)`) — consumers MUST
  read the tristate, not the bool.
- NULL is produced by member access to a missing/Null attribute (e.g. `f.tag` where
  flow has no `tag`). Built-in `flow.quantity` defaults to `0`, never NULL.
- `evaluate()` uses the graph's mode flag; `evaluate_with_mode(graph, three_valued)`
  forces it. Every result stamps `evaluation_mode` so it is self-describing.
- Aggregation in boolean context errors pre-eval (`validate_aggregation_usage`) —
  must be wrapped in a comparison (`count(...) > 0`).

## Gotchas

- `exists`/`forall` short-circuit on Kleene rules (Or stops only on True) — see
  `three_valued.rs` folds; don't "optimize" by skipping Null elements.
- Changing `EvaluationResult` fields ripples into `python/`, `typescript/`, `wasm/`
  `policy.rs` `From` impls and the `cli/validate.rs` JSON oracle.
- Tristate behavior is pinned by `../../conformance/02_*`–`06_*` and
  `tests/three_valued_quantifiers_tests.rs`.
