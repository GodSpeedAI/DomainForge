# Define Policies

Goal: Author policies in SEA DSL and verify they evaluate correctly across CLI and bindings.

## Prerequisites

- SEA CLI installed with `cli` feature.
- A validated model that includes entities/resources/flows referenced by the policy.
- Optional bindings (Python/TypeScript) for programmatic evaluation.

## Steps (be concise)

1. **Write a policy in DSL**

   ```sea
   Policy payment_threshold as:
     forall f in flows where f.resource = "Money":
       f.quantity <= 10000 "USD"
   ```

   - Use comprehensions (`forall`, `exists`) and comparisons (`=`, `<=`, `matches`).
   - Qualify names with namespaces when needed (`finance/Money`).

2. **Attach predicates to roles or relations**

   ```sea
   Role "Payer"
   Role "Payee"
   Relation "Payment"
     subject: "Payer"
     predicate: "pays"
     object: "Payee"
     via: flow "Money"

   Policy payer_must_exist as:
     forall r in relations where r.name = "Payment":
       exists role in roles where role.id = r.subject_role_id
   ```

3. **Validate the policy with the CLI**

   ```bash
   sea validate --format human examples/policies.sea
   ```

   - Reports semantic errors (unknown identifiers, unit mismatches) with line/column.
   - Use `--format json` for machine-readable diagnostics.

4. **Evaluate a policy in Python**

   ```python
   from sea_dsl import Graph
   from pathlib import Path

   graph = Graph.parse(Path("examples/policies.sea").read_text())
   policy_json = Path("policy.json").read_text()  # JSON form of crate::policy::Policy
   result = graph.evaluate_policy(policy_json)
   print(result.violations)
   ```

   - Enable three-valued logic for partial data: `graph.set_evaluation_mode(True)`.

5. **Evaluate a policy in TypeScript**

   ```ts
   import { Graph } from "@domainforge/sea";
   import { readFileSync } from "fs";

   const graph = Graph.parse(readFileSync("examples/policies.sea", "utf8"));
   const policyJson = readFileSync("policy.json", "utf8");
   const outcome = graph.evaluatePolicy(policyJson);
   console.log(outcome.violations);
   ```

6. **Test policies as part of CI**

   - Add a Rust test under `sea-core/tests/` that parses the DSL and calls `graph.evaluate_policy` on representative inputs.
   - Mirror the test in Python/TypeScript to maintain parity. Use the same DSL snippet to avoid drift.
   - Run `just all-tests` before merging changes.

## Expression Basics

- **Boolean logic**: `and`, `or`, `not` with standard precedence; wrap complex conditions in parentheses.
- **Comparisons**: `=`, `!=`, `<`, `<=`, `>`, `>=`, `matches` (regex-like), `in` (membership over collections).
- **Quantifiers**: `forall <var> in <collection> where <predicate>: <body>` and `exists ...`.
- **Three-valued logic**: `Unknown` propagates when operands lack data; enable it when modeling incomplete datasets.

## Best Practices

- Keep policies near the DSL constructs they reference or register them via `.sea-registry.toml` so namespace resolution works.
- Normalize units before comparison; rely on `as "<Unit>"` to request conversions explicitly.
- Avoid side effects; policies should be deterministic for reproducible validation.
- Document intent with comments; the parser ignores lines starting with `#`.

## Troubleshooting

- **Unknown identifier**: Ensure entities/resources/roles exist and are in scope; namespaces must match.
- **Unit mismatch**: Define units and dimensions before using them in policies.
- **Empty collections**: If `flows` or `relations` are empty, quantifiers may vacuously pass; add explicit existence checks when necessary.
- **Evaluation differences across bindings**: Rebuild bindings (`maturin develop`, `npm run build`) after changing policy evaluation logic in Rust.

## Verification Checklist

- [ ] `sea validate` passes for the policy file.
- [ ] Policy evaluates consistently in Rust, Python, and TypeScript for the same input.
- [ ] Unit conversions behave as expected where used.
- [ ] Documentation and examples updated when adding new operators or predicates.

## Links

- Tutorials: [Getting Started](../tutorials/getting-started.md)
- Reference: [Grammar Spec](../reference/grammar-spec.md), [Policy Evaluation Logic](../explanations/policy-evaluation-logic.md), [Three-Valued Logic](../explanations/three-valued-logic.md)
