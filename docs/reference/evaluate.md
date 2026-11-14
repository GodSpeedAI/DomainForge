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
- Operation-bound policies (`access policy_governed ... at precondition`) are evaluated by the application evaluator (`evaluate_precondition`). The output has one row per policy, operation and input set, each with its own `result` and `input_set_id`; there is no aggregate across input sets. A bound policy with no input set yields one row, `unknown`, `"reason": "no_operation_input"`; a missing input field gives `unknown` with `"reason": "unknown_input"`.
- Each binding row has `"source": "model"` (instance declared in the `.sea` model) or `"facts"`. The top level reports `model_instance_count` and `facts_instance_count`.
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

Facts are order-independent: all instances are inserted and validated together, so an entity reference may point at a fact later in the file, and a fact set that fails validation is rejected whole. An operation name declared by more than one operation (for example in two namespaces) is ambiguous and an input naming it is rejected. Instance names are unique across the model. `input` values are the application layer's typed values (`kind`/`data`). The operation's declared actor is used. A field whose value is null must be omitted, not sent as `null`: omitted optional fields are the unknown (NULL) case. Each typed value's kind must match the record field's declared type, and each `operation_inputs` entry must name a policy-governed operation; otherwise it is an input error.

## Exit codes

- `0`: evaluation ran, whatever the results (false and unknown are data, not errors).
- `2`: input or usage error: unreadable file, invalid JSON or schema version, a model that does not resolve, or facts naming an unknown namespace/entity/field/operation, a duplicate name or input id, or a field that violates the entity contract. Nothing is written to stdout.

## What it does not do

- Only the outermost quantifier is bound. Which inner instance failed is not reported.
- Only quantifier policies at the policy root get bindings; a quantifier nested inside `and`/`or`/`not` is evaluated as part of the policy but produces no rows.
- Operation-bound policies are precondition-only (0.21.2 rejects `at postcondition` and `at invariant`) and use the closed application subset: no quantifiers, no entity instances. `pre_state` is not supplied (not verified: policies that read `state.*`).
- A policy the evaluator cannot evaluate is reported `unknown` with `"reason": "evaluation_error"` and the message, not dropped.
- `exists_unique` follows three-valued counting at any depth: two or more true items give `false`; otherwise any unknown item gives `unknown`; otherwise exactly one true item gives `true` (issue https://github.com/GodSpeedAI/DomainForge/issues/172). `validate` shares the expansion: a model whose `exists_unique` has an unknown item and at most one true item used to pass and now reports an UNKNOWN (NULL) violation, as the other quantifiers already do. Boolean (non-three-valued) mode keeps the count-only result.
- `evaluate` refuses a graph in boolean (non-three-valued) mode; the CLI cannot currently select that mode.
- Not verified: multi-module models, role-bound (`role<...>`) policies, `group_by`, policies reading `state.*`, and the Python, TypeScript and wasm bindings (not built).
- Only `--format json` exists.
