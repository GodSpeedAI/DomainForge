# Define Policies

Goal: Author policies in SEA DSL and verify they evaluate correctly.

## Prerequisites

- SEA CLI installed (`cargo install --path sea-core --features cli`) or Python/TypeScript bindings.
- A validated model with entities/resources/flows referenced by the policy.

## Write a Policy

```sea
Policy payment_threshold as:
  forall f in flows where f.resource = "Money":
    f.quantity <= 10000 "USD"
```

- Use comprehensions (`forall`, `exists`) and comparisons (`=`, `<=`, `matches`) available in the grammar.
- Namespaces: qualify names (`"finance/Money"`) if you use multiple namespaces.

## Validate the Policy

- CLI: `sea validate --format human model.sea` (prints violations on failure).
- JSON output: `sea validate --format json model.sea` for tooling or CI.
- Keep policies in the same file as the model or include them via the namespace registry (`.sea-registry.toml`).

## Evaluate Programmatically

- **Python**:

  ```python
  from pathlib import Path
  from sea_dsl import Graph

  graph = Graph.parse(Path("model.sea").read_text())
  policy_json = Path("policy.json").read_text()  # JSON representation of crate::policy::Policy
  result = graph.evaluate_policy(policy_json)
  print(result.violations)
  ```

- **TypeScript**:

  ```ts
  import { Graph } from "@domainforge/sea";
  import { readFileSync } from "fs";

  const graph = Graph.parse(readFileSync("model.sea", "utf8"));
  const policyJson = readFileSync("policy.json", "utf8"); // same Policy JSON shape as Rust
  const outcome = graph.evaluatePolicy(policyJson);
  console.log(outcome.violations);
  ```

## Best Practices

- Turn on three-valued logic when data may be incomplete: `graph.setEvaluationMode(true)`.
- Model units consistently; comparisons on quantities with mismatched units will fail validation.
- Keep predicates deterministic (no random data sources) to preserve reproducibility.
