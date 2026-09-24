# Explanation: Why Three-Valued Logic?

Why does DomainForge's policy evaluation engine use SQL-like Kleene Three-Valued Logic (`True`, `False`, `Null`/`Unknown`) instead of standard two-valued boolean logic?

---

## 1. The Core Dilemma: Missing Information in Distributed Domains

In real-world enterprise architectures, the complete state of an organization is rarely available in a single atomic snapshot:
- An entity instance might be partially populated during a multi-step onboarding wizard.
- An optional field (such as `vat_number` or `approval_timestamp`) might be legitimate `NULL`.
- A newly declared microservice may not yet have emitted telemetry.

Under classical **Two-Valued Logic (2VL)**, every expression must resolve to either `True` or `False`. When evaluating a rule against missing data, an engineer faces an unavoidable dilemma:

| Default Coercion | Consequence in Enterprise Compliance |
|---|---|
| **Coerce `NULL` to `False`** | **False Alarm Explosion**: Legitimate, in-flight, or optional records fail validation, triggering spurious compliance violations and training developers to ignore alerts. |
| **Coerce `NULL` to `True`** | **Dangerous Security Vulnerability**: Unverified, incomplete, or unauthenticated requests are silently approved, creating critical security holes. |

---

## 2. The Solution: Kleene Three-Valued Logic (3VL)

DomainForge adopts the mathematically rigorous **Kleene 3VL** model (similar to SQL `NULL` semantics). In this model, truth values represent three distinct states:

$$\mathbb{T} = \{\text{True}, \text{False}, \text{Null}\}$$

where `Null` represents **Unknown** ("we do not have enough information to confirm or deny").

### Algebraic Invariants
- **Conjunction (`and`)**:
  - `False and Anything` $\rightarrow$ `False` (if any condition fails, the whole conjunction fails, even if other facts are unknown).
  - `True and Null` $\rightarrow$ `Null` (the outcome depends on the missing fact).
- **Disjunction (`or`)**:
  - `True or Anything` $\rightarrow$ `True` (if at least one alternative is satisfied, missing information cannot invalidate it).
  - `False or Null` $\rightarrow$ `Null` (the outcome depends on whether the unknown fact is true).
- **Negation (`not`)**:
  - `not Null` $\rightarrow$ `Null` (the opposite of unknown is still unknown).

---

## 3. Impact on Quantifiers over Collections

Consider a regulatory rule checking loan amounts:
```sea
policy max_exposure per Constraint Obligation priority 1
  as:
    forall f in flows: f.quantity <= 1000000
```

1. If all flows have known quantities $\le 1,000,000$, evaluation yields **`True`**.
2. If at least one flow has a quantity $> 1,000,000$, evaluation yields **`False`** (violation triggered immediately, regardless of any unknown flows).
3. If no flow exceeds the threshold, but one flow has an unmeasured/null quantity, evaluation yields **`Null` (`Unknown`)**.

### CLI Handling: Strict vs Warn
- In strict CI mode, `Unknown` can be gated or warned without triggering the same alarm level as an explicit rule violation (`False`).
- With `--allow-unknown`, the validator passes incomplete models while highlighting missing fields for developer attention.

---

## Source Trail
- `domainforge-core/src/policy/three_valued.rs` — Implementation of `ThreeValuedBool` and algebraic operators
- `domainforge-core/src/policy/quantifier.rs` — Null-safe collection folding for `forall` and `exists`
- `domainforge-core/tests/three_valued_quantifiers_tests.rs` — Comprehensive algebraic test cases
