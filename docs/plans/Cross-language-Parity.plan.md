Cross-language Parity for evaluate_policy
Goal Description
Implement evaluate_policy in Python and TypeScript bindings for the
Graph
object. This method should return an
EvaluationResult
containing is_satisfied (bool), is_satisfied_tristate (Optional[bool]), and violations.

User Review Required
Confirm the structure of
EvaluationResult
in both languages.
Confirm the handling of Null (tristate) in both languages.
Proposed Changes
Rust Core (sea-core)
EvaluationResult
and Violation in
sea-core/src/policy/core.rs
and
violation.rs
are already suitable. No changes needed in core logic.
Python Bindings
File:
sea-core/src/python/policy.rs
Define Violation class with
name
, message,
severity
.

**Severity Field Specification:**

- **Type**: String enum with allowed values: `"error"`, `"warning"`, `"info"`
- **Required**: Yes (always present, never null/None)
- **Default**: `"warning"` (if source severity is unknown or unmappable)
- **Mapping from Rust**:
  - `crate::policy::Severity::Error` → `"error"`
  - `crate::policy::Severity::Warning` → `"warning"`
  - `crate::policy::Severity::Info` → `"info"`
- **Validation**: Implementations should validate that severity is one of the allowed values and reject invalid values with a clear error message.

Define
EvaluationResult
class with is_satisfied, is_satisfied_tristate, violations.
File:
sea-core/src/python/graph.rs
Add evaluate_policy(self, policy_json: str) -> EvaluationResult.
Implementation: Parse JSON to
Policy
, call policy.evaluate(&self.inner), convert result to Python
EvaluationResult
.
File:
sea-core/src/lib.rs
Register Violation and
EvaluationResult
classes.
TypeScript Bindings
File:
sea-core/src/typescript/policy.rs
Define Violation struct (napi) with
name
, message,
severity
.
Define
EvaluationResult
struct (napi) with is_satisfied, is_satisfiedTristate (camelCase), violations.
File:
sea-core/src/typescript/graph.rs
Add evaluatePolicy(policyJson: string): EvaluationResult.
Implementation: Parse JSON to
Policy
, call policy.evaluate(&self.inner), convert result to JS
EvaluationResult
.
Verification Plan
Automated Tests
Python: Create tests/test_three_valued_eval.py.
Test cases:

- Policy evaluates to True → is_satisfied=True, tristate=True.
- Policy evaluates to False → is_satisfied=False, tristate=False.
- Policy evaluates to Null → is_satisfied=False, tristate=None.
- **Invalid policy JSON**: Assert parsing fails with appropriate error message (e.g., malformed JSON, missing required fields).
- **Multiple violations**: Ensure all violations are returned in the violations list, and is_satisfied/tristate reflect the overall failure state.
- **Zero violations**: Empty violations list yields is_satisfied=True and tristate=True (or appropriate success state).
- **Explicit null/None handling**: Verify serialization/deserialization correctly handles None/null values in tristate field (Python: None, TypeScript: null).
  TypeScript: Create typescript-tests/three_valued_eval.test.ts.
  Test cases mirroring Python tests:
- Policy evaluates to True → isSatisfied=true, isSatisfiedTristate=true.
- Policy evaluates to False → isSatisfied=false, isSatisfiedTristate=false.
- Policy evaluates to Null → isSatisfied=false, isSatisfiedTristate=null.
- **Invalid policy JSON**: Assert parsing fails with appropriate error message.
- **Multiple violations**: Ensure all violations are returned and result reflects failure.
- **Zero violations**: Empty violations array yields isSatisfied=true and isSatisfiedTristate=true.
- **Explicit null handling**: Verify null is correctly serialized/deserialized for isSatisfiedTristate.
  Manual Verification
  None planned, relying on automated tests.
