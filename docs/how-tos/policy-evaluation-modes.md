# Policy Evaluation Mode (Canonical Three-Valued Logic)

Goal: understand how policies are evaluated and how to handle `Unknown` results.

## Single canonical mode

Policy evaluation uses **three-valued (Kleene) logic** as the one and only
semantics. There is no runtime toggle: the legacy boolean mode was removed so a
model cannot have two different meanings (semantic-infrastructure audit, G1).

- Missing or `Null` data evaluates to **`Unknown`** (`is_satisfied_tristate = None`).
- Every `EvaluationResult` carries `evaluation_mode = "three_valued"` so output is
  self-describing.
- `is_satisfied` is a **fail-closed** convenience boolean: it is `false` whenever the
  tristate result is `Unknown` or `False`. Consumers that must distinguish
  "violated" from "couldn't tell" should read `is_satisfied_tristate`, not `is_satisfied`.

## Reading results

- Rust: `result.is_satisfied_tristate` (`Option<bool>`) and `result.evaluation_mode`.
- Python: `result.is_satisfied_tristate` (`bool | None`) and `result.evaluation_mode` (str).
- TypeScript/WASM: `result.isSatisfiedTristate` (`boolean | undefined`) and
  `result.evaluationMode` (string).

## Symptoms and fixes

- Seeing `Unknown` when you expect pass/fail: the model is missing the data the
  policy references. Complete the missing fields — do not coerce `Unknown` to a
  boolean, as that silently hides incomplete evidence.
- Treating `Unknown` as failure in a gate: check `is_satisfied` (already fail-closed)
  or treat a `None`/`undefined` tristate as a failed gate explicitly.

## See also

- [Policy Evaluation Logic](../explanations/policy-evaluation-logic.md)
- [Three-Valued Logic](../explanations/three-valued-logic.md)
- [Canonical Entrypoints](../specs/canonical_entrypoints.md)
- [CLI Commands](../reference/cli-commands.md) (`--allow-unknown` parallels three-valued behavior)
