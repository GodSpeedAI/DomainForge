# Policy Evaluation API

## Overview

The `evaluate_policy` method provides a unified interface for evaluating SEA DSL policies against a graph in both Python and TypeScript/JavaScript. It supports three-valued logic (True, False, NULL) for handling indeterminate evaluation results.

## API Reference

### Python

```python
from sea_dsl import Graph, EvaluationResult, Violation, Severity

# Create and populate a graph
graph = Graph()
# ... add entities, resources, flows, etc.

# Evaluate a policy
policy_json = '''
{
    "name": "MyPolicy",
    "expression": { ... },
    "severity": "Error"
}
'''

result: EvaluationResult = graph.evaluate_policy(policy_json)

# Access results
is_satisfied: bool = result.is_satisfied
is_satisfied_tristate: Optional[bool] = result.is_satisfied_tristate
violations: List[Violation] = result.violations
```

#### Python Types

**`EvaluationResult`**

- `is_satisfied: bool` - Backward-compatible boolean result (False if evaluation is NULL)
- `is_satisfied_tristate: Optional[bool]` - Three-valued result (True, False, or None for NULL)
- `violations: List[Violation]` - List of policy violations

**`Violation`**

- `name: str` - Policy name that was violated
- `message: str` - Human-readable violation message
- `severity: Severity` - Violation severity level

**`Severity`** (Enum)

- `Severity.Error` - Critical violation
- `Severity.Warning` - Warning-level violation
- `Severity.Info` - Informational violation

### TypeScript/JavaScript

```typescript
import { Graph, EvaluationResult, Violation, Severity } from "@domainforge/sea";

// Create and populate a graph
const graph = new Graph();
// ... add entities, resources, flows, etc.

// Evaluate a policy
const policyJson = `
{
    "name": "MyPolicy",
    "expression": { ... },
    "severity": "Error"
}
`;

const result: EvaluationResult = graph.evaluatePolicy(policyJson);

// Access results
const isSatisfied: boolean = result.isSatisfied;
const isSatisfiedTristate: boolean | undefined = result.isSatisfiedTristate;
const violations: Violation[] = result.violations;
```

#### TypeScript Types

**`EvaluationResult`**

- `isSatisfied: boolean` - Backward-compatible boolean result (false if evaluation is NULL)
- `isSatisfiedTristate?: boolean` - Three-valued result (true, false, or undefined for NULL)
- `violations: Violation[]` - Array of policy violations

**`Violation`**

- `name: string` - Policy name that was violated
- `message: string` - Human-readable violation message
- `severity: Severity` - Violation severity level

**`Severity`** (Enum)

- `Severity.Error` - Critical violation
- `Severity.Warning` - Warning-level violation
- `Severity.Info` - Informational violation

## Three-Valued Logic

The policy evaluator supports three-valued logic to handle cases where evaluation cannot be definitively determined:

| Evaluation State         | `is_satisfied` (Python) / `isSatisfied` (TS) | `is_satisfied_tristate` (Python) / `isSatisfiedTristate` (TS) |
| ------------------------ | -------------------------------------------- | ------------------------------------------------------------- |
| **True** (satisfied)     | `true`                                       | `true`                                                        |
| **False** (violated)     | `false`                                      | `false`                                                       |
| **NULL** (indeterminate) | `false`                                      | `None` (Python) / `undefined` (TS)                            |

### When NULL Occurs

NULL evaluation results occur when:

- Required data for evaluation is missing
- Graph state is incomplete
- Policy expression references non-existent entities or resources

When a NULL result occurs, a `Warning`-level violation is automatically added to explain the indeterminate state.

## Usage Examples

### Example 1: Simple Policy Evaluation (Python)

```python
from sea_dsl import Graph

graph = Graph()
# Populate graph...

policy = {
    "name": "AllEntitiesHaveOwner",
    "expression": {
        "type": "ForAll",
        "variable": "e",
        "collection": {"type": "AllEntities"},
        "condition": {
            "type": "BinaryOp",
            "op": "NotEqual",
            "left": {"type": "MemberAccess", "object": "e", "member": "owner"},
            "right": {"type": "Literal", "value": None}
        }
    },
    "severity": "Error"
}

result = graph.evaluate_policy(json.dumps(policy))

if result.is_satisfied:
    print("✓ Policy satisfied")
else:
    print("✗ Policy violated")
    for violation in result.violations:
        print(f"  - {violation.message}")
```

### Example 2: Handling Indeterminate Results (TypeScript)

```typescript
import { Graph, Severity } from "@domainforge/sea";

const graph = new Graph();
// Populate graph...

const policy = {
  name: "DataFlowsEncrypted",
  expression: {
    type: "ForAll",
    variable: "f",
    collection: { type: "AllFlows" },
    condition: {
      type: "BinaryOp",
      op: "Equal",
      left: { type: "MemberAccess", object: "f", member: "encrypted" },
      right: { type: "Literal", value: true },
    },
  },
  severity: "Error",
};

const result = graph.evaluatePolicy(JSON.stringify(policy));

// Check for indeterminate results
if (result.isSatisfiedTristate === undefined) {
  console.warn("⚠ Policy evaluation is indeterminate");
  console.warn("Reason:", result.violations[0]?.message);
} else if (result.isSatisfied) {
  console.log("✓ All flows are encrypted");
} else {
  console.error("✗ Unencrypted flows detected:");
  result.violations
    .filter((v) => v.severity === Severity.Error)
    .forEach((v) => console.error(`  - ${v.message}`));
}
```

### Example 3: Batch Policy Evaluation (Python)

```python
from sea_dsl import Graph, Severity
import json

def evaluate_policies(graph: Graph, policies: list) -> dict:
    """Evaluate multiple policies and return summary."""
    results = {
        "total": len(policies),
        "satisfied": 0,
        "violated": 0,
        "indeterminate": 0,
        "violations": []
    }

    for policy in policies:
        result = graph.evaluate_policy(json.dumps(policy))

        if result.is_satisfied_tristate is None:
            results["indeterminate"] += 1
        elif result.is_satisfied:
            results["satisfied"] += 1
        else:
            results["violated"] += 1

        # Collect all violations
        results["violations"].extend([
            {
                "policy": v.name,
                "message": v.message,
                "severity": v.severity.name
            }
            for v in result.violations
        ])

    return results

# Usage
graph = Graph()
# ... populate graph

policies = [
    # ... list of policy definitions
]

summary = evaluate_policies(graph, policies)
print(f"Policies: {summary['satisfied']}/{summary['total']} satisfied")
if summary['indeterminate'] > 0:
    print(f"Warning: {summary['indeterminate']} policies could not be evaluated")
```

## Language-Specific Considerations

### Naming Conventions

Due to platform idioms, the API uses different naming conventions:

| Concept              | Python                  | TypeScript/JavaScript |
| -------------------- | ----------------------- | --------------------- |
| Method name          | `evaluate_policy`       | `evaluatePolicy`      |
| Field: satisfied     | `is_satisfied`          | `isSatisfied`         |
| Field: tristate      | `is_satisfied_tristate` | `isSatisfiedTristate` |
| Type: violation list | `List[Violation]`       | `Violation[]`         |

### NULL Representation

- **Python**: `None` (standard Python null value)
- **TypeScript**: `undefined` (missing optional field)

In TypeScript, when `isSatisfiedTristate` is `undefined`, the field is omitted from the object entirely. This is standard NAPI behavior for `Option::None` in Rust.

## Error Handling

### Invalid Policy JSON

Both APIs will throw/raise an error if the policy JSON is malformed:

```python
# Python
try:
    result = graph.evaluate_policy("invalid json")
except Exception as e:
    print(f"Error: {e}")
```

```typescript
// TypeScript
try {
  const result = graph.evaluatePolicy("invalid json");
} catch (error) {
  console.error("Error:", error.message);
}
```

### Policy Evaluation Errors

If a policy cannot be evaluated due to semantic errors (e.g., referencing non-existent fields), the method returns an `EvaluationResult` with:

- `is_satisfied` / `isSatisfied` = `false`
- `is_satisfied_tristate` / `isSatisfiedTristate` = `None` / `undefined`
- A `Severity.Error` violation describing the issue

## Performance Considerations

- Policy evaluation is performed synchronously
- For large graphs with many policies, consider:
  - Batching evaluations
  - Caching policy parse results (future enhancement)
  - Evaluating policies in parallel (if supported by your runtime)

## Testing

Both Python and TypeScript test suites are available:

```bash
# Python tests
just python-test
# or
pytest tests/test_three_valued_eval.py

# TypeScript tests
npm test
# or
npm test -- typescript-tests/three_valued_eval.test.ts
```

## See Also

- [Policy Expression Syntax](./policy_expressions.md) (if exists)
- [Graph API Reference](./graph_api.md) (if exists)
- [Three-Valued Logic in SEA DSL](./three_valued_logic.md) (if exists)
