# How-To: Debug Policy Evaluations

This guide explains how to diagnose why a policy expression in DomainForge unexpectedly evaluated to `False` (a violation) or `Null` (`Unknown`).

---

## Goal
Identify the root cause of a failing policy and determine whether the issue is a genuine violation, an expression logic flaw, or missing data.

---

## Diagnostic Procedure

### Step 1: Distinguish `False` from `Null`
In Three-Valued Logic, a rule can fail because:
- **`False`**: A condition was actively violated (e.g. `150 <= 100`).
- **`Null`**: An attribute or related entity was missing/unspecified.

Run validation with `--allow-unknown`:
```bash
domainforge validate model.sea --allow-unknown
```
- If the model **passes** with `--allow-unknown`, the policy evaluated to `Null`. The issue is missing data or an unpopulated optional field.
- If the model **still fails**, the policy evaluated to `False`. The condition was directly violated.

---

### Step 2: Normalize the Expression
If the policy contains nested boolean logic (`and`, `or`, `not`), simplify it with `domainforge normalize`:

```bash
domainforge normalize "not (a > 10 and b < 5)"
```

The output displays the normalized, simplified form:
```text
(not (a > 10)) or (not (b < 5))
```
Use this to verify that your logical operators match your intended business rule.

---

### Step 3: Inspect Graph Collections
Policies evaluate over graph collections (`flows`, `entities`, `resources`, `instances`, `entity_instances`). Check the exact items present in the collection:

```bash
domainforge parse model.sea --format json
```
Look for:
- Did an entity name change, causing a flow filter (`where f.resource = "Payment"`) to match 0 flows?
- Note: `sum()` over an empty collection returns `0`, but `min()` or `max()` over an empty collection returns `Null`!

---

### Step 4: Check Quantifier Scoping
When writing collection quantifiers, ensure you scope them to the intended entity type:

```sea
// ❌ Potential Null: iterates all entity instances; if another entity lacks `credit_limit`, returns Null
forall i in entity_instances: i.credit_limit > 0

// ✅ Safe Scoping: only iterates instances of "Vendor"
forall i in entity_instances of "Vendor": i.credit_limit > 0
```

---

## Source Trail
- `domainforge-core/src/policy/core.rs` — Policy evaluation engine
- `domainforge-core/src/policy/three_valued.rs` — `ThreeValuedBool` logic
- `domainforge-core/src/cli/normalize.rs` — CLI normalize command
