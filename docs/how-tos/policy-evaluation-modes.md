# Policy Evaluation Modes (Three-Valued vs Boolean)

Goal: choose and troubleshoot the runtime evaluation mode for policies.

## Defaults and toggle

- Three-valued logic is **enabled by default** (returns `Unknown` when data is missing).
- Switch to strict boolean mode when you need binary pass/fail:
  - Rust: `graph.set_evaluation_mode(false)`
  - Python: `graph.set_evaluation_mode(False)`
  - TypeScript/WASM: `graph.setEvaluationMode(false)`

Return to three-valued logic with `true/True`.

## When to use which mode

- **Three-valued (default)**: modeling in-progress systems; want Unknown instead of false negatives.
- **Boolean**: production gates that must fail on any missing data; CI pipelines that treat Unknown as failure.

## Symptoms and fixes

- Seeing `Unknown` results when you expect pass/fail: enable boolean mode for that run or complete the missing fields.
- Tests failing after switching modes: ensure fixtures set `graph.set_evaluation_mode(true/false)` explicitly.
- Performance concerns: toggling is runtime-only; no rebuild needed.

## See also

- [Policy Evaluation Logic](../explanations/policy-evaluation-logic.md)
- [Three-Valued Logic](../explanations/three-valued-logic.md)
- [CLI Commands](../reference/cli-commands.md) (`--allow-unknown` parallels three-valued behavior)
