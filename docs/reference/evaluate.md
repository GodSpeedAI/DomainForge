# `domainforge evaluate`

Evaluate a model's policies against a facts file and report a three-valued result per policy and per bound instance.

```bash
domainforge evaluate --model <MODEL.sea> --facts <FACTS.json> --format json [--world-name <NAME>] [--registry <FILE>] [--default-namespace <NS>]
```

Schemas: [`schemas/evaluate-facts.v1.json`](../../schemas/evaluate-facts.v1.json) (input), [`schemas/evaluate-output.v1.json`](../../schemas/evaluate-output.v1.json) (output). `validate` and every other command are unchanged.

## What it does

- Loads the model, adds the facts' instances (checked against the entity contracts), and evaluates each policy with DomainForge's own evaluators. It adds no evaluation semantics.
- Graph policies use the existing three-valued evaluator (`Policy::evaluate_bindings`). `result` is `true`, `false` or `unknown`; `unknown` is a NULL that nothing else resolved (for example an omitted optional attribute).
- `support` is the number of instances the outermost quantifier ranged over. `support: 0` with `result: true` is vacuous truth, not verification. Policies without a quantifier have `support: null` and `bindings: []`.
- `bindings` has one row per instance of the outermost quantifier: `{entity, namespace, instance, instance_id, result}`, sorted by instance name. The row is the same evaluator applied to the quantified condition for that instance. Nested quantifiers are folded into that row (`"binding_depth": "outermost"`).
- Operation-bound policies (`access policy_governed ... at precondition`) are evaluated by the application evaluator (`evaluate_precondition`) for each `operation_inputs` entry of their operation. Rows carry `operation_bound: true`, `operation`, and one binding per input set (`input_set_id`). A bound policy with no input set is `unknown` with `"reason": "no_operation_input"`; a missing input field gives a binding `unknown` with `"reason": "unknown_input"`. The policy-level result is false if any input set is false, else unknown if any is unknown, else true.
- Output records `model_sha256` and `facts_sha256` (raw file bytes), `source_set_hash`, the compiler version, and `world_ref` when `--world-name` is given (same derivation as `envelope --world-name`; otherwise `null`).
- Output is deterministic: policies ordered by `policy_id` then operation name, bindings by instance name, keys sorted.

## Facts file

```json
{
  "schema": "domainforge.evaluate-facts.v1",
  "instances": [{"namespace": "ns", "entity": "Step", "name": "s1", "fields": {"step_id": "s1", "status": "executed"}}],
  "operation_inputs": [{"id": "set1", "operation": "close_case", "input": {"case_status": {"kind": "string", "data": "open"}}}]
}
```

Instance names are unique across the model. `input` values are the application layer's typed values (`kind`/`data`). The operation's declared actor is used.

## Exit codes

- `0`: evaluation ran, whatever the results (false and unknown are data, not errors).
- `2`: input or usage error: unreadable file, invalid JSON or schema version, a model that does not resolve, or facts naming an unknown namespace/entity/field/operation, a duplicate name or input id, or a field that violates the entity contract. Nothing is written to stdout.

## What it does not do

- Only the outermost quantifier is bound. Which inner instance failed is not reported.
- Only quantifier policies at the policy root get bindings; a quantifier nested inside `and`/`or`/`not` is evaluated as part of the policy but produces no rows.
- Operation-bound policies are precondition-only (0.21.2 rejects `at postcondition` and `at invariant`) and use the closed application subset: no quantifiers, no entity instances. `pre_state` is not supplied (not verified: policies that read `state.*`).
- A policy the evaluator cannot evaluate is reported `unknown` with `"reason": "evaluation_error"` and the message, not dropped.
- `exists_unique` follows the existing evaluator, which expands it eagerly and counts only literal-true items: one true plus one NULL gives policy `true` while the binding rows show the NULL. Binding rows are exact; read the policy-level result of `exists_unique` with that in mind (probe: `evaluate_probe_exists_unique_under_null_and_quantifier_free_policy`).
- Not verified: multi-module models, role-bound (`role<...>`) policies, `group_by`, and the Python, TypeScript and wasm bindings (not built).
- Only `--format json` exists.
