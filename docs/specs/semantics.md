<!--
  SEA DSL — Operator Precedence and Three-Valued NULL Semantics
  Author: Copilot assistant (implementation)
  Place under: docs/specs/semantics.md
-->

# SEA DSL — Operator Precedence and Three-Valued NULL Semantics

Version: 1.0

This document describes operator precedence for expressions in the SEA DSL and the semantics of three-valued NULL logic (True / False / Null) used by the policy evaluator when the optional compile-time feature is enabled.

## Scope and audience

- Audience: engineers and authors of SEA policies — designed to be readable by beginners while precise enough for implementers.
- Scope: expression parsing precedence, three-valued truth tables, aggregation semantics, examples, and guidance for enabling the feature.

## Summary

- Default (Phase 18): evaluator is strict — missing attributes cause an evaluation error. This fail-fast behavior helps catch modelling gaps early.
- Optional behaviour: enable `three_valued_logic` feature to use SQL-like three-valued semantics where missing values are treated as `Null` (unknown). This provides greater expressivity when modelling optional or incomplete data.

## Operator precedence (highest → lowest)

1. Primary: literals, identifiers, member access (`.`), grouping `()`
2. Unary: `-` (arithmetic negation), `not` (logical not)
3. Multiplicative: `*`, `/`
4. Additive: `+`, `-`
5. Comparison: `<`, `>`, `<=`, `>=`, `=`, `!=`, `contains`, `startswith`, `endswith`
6. Logical AND: `and`
7. Logical OR: `or`
8. Implication: `implies` (lowest precedence)

### Parsing notes

- Member access binds tightly: `a.b + c` parses as `(a.b) + c`.
- `not` is unary and applies to the smallest following expression: `not a and b` parses as `(not a) and b`.
- Use parentheses `()` to disambiguate: `not (a and b)`.

## Three-valued NULL logic — Overview

- Values: `True`, `False`, `Null` (unknown). `Null` arises from missing attributes or optional fields — not from an explicit `NULL` literal (Phase 18).
- Logical connectives follow SQL/relational semantics with short-circuit behaviour where safe.
- Aggregations propagate `Null` unless `_nonnull` variants are used (e.g., `sum_nonnull`).

## Canonical truth tables and examples

Use `N` for `Null`, `T` for `True`, `F` for `False`.

### Logical AND (`and`)

| A | B | A and B |
|---:|---:|:-------:|
| T | T | T |
| T | F | F |
| T | N | N |
| F | N | F |
| N | N | N |

Example: `NULL and true` → `NULL`; `NULL and false` → `false` (false dominates).

### Logical OR (`or`)

| A | B | A or B |
|---:|---:|:------:|
| T | N | T |
| F | N | N |
| N | N | N |

Example: `NULL or true` → `true`; `NULL or false` → `NULL`.

### NOT (`not`)

| A | not A |
|---:|:-----:|
| T | F |
| F | T |
| N | N |

Example: `not NULL` → `NULL`.

### IMPLIES (`implies`)

Defined as: `A implies B` ≡ `(not A) or B`.

| A | B | A implies B |
|---:|---:|:----------:|
| T | N | N |
| N | T | T |
| N | F | N |
| N | N | N |

Example: `NULL implies true` → `true`.

### Comparisons and NULL

- Any comparison involving `Null` yields `Null` (unknown). In particular `NULL = NULL` → `NULL` (not `True`). This is intentional: two unknowns are not considered equal.

## Aggregation semantics

- `sum([1, NULL, 3])` → `NULL` (propagates unknowns).
- `sum_nonnull([1, NULL, 3])` → `4` (skips NULLs).
- `count([1, NULL, 3])` → `3` (counts elements including NULL).
- `count_nonnull([1, NULL, 3])` → `2`.
- `avg([NULL, NULL])` → `NULL`.

Rationale: by default aggregations are conservative—presence of unknowns makes the whole aggregate unknown unless the caller explicitly chooses a non-null variant.

## Rule-level behaviour

- Policy evaluation that yields `True` is satisfied; `False` is violated; `Null` is unknown.
- The policy framework's severity mapping can consider `Null` specially (for example, treating unknown as `Warning` or `Info` depending on configuration). See `policy/violation.rs` for severity rules.

## Grammar and language notes

- No explicit `NULL` literal in Phase 18; `Null` arises from missing attributes. Future phases may add a `NULL` literal if required.
- Example: `exists x in flows where x.attr = 10` — if `x.attr` is missing for some `x`, that comparison yields `Null` for those elements; quantifiers treat `Null` per the truth-table rules.

## Feature flag and compatibility

- Feature: `three_valued_logic` (crate feature). When enabled, the evaluator returns `ThreeValuedBool` values for boolean expressions; consumers must decide how to treat `Null`.
- Default behaviour (feature disabled): evaluation is strict and returns errors on missing attribute access. This is the recommended default for Phase 18.

### Enabling the feature

In the `sea-core` crate's `Cargo.toml` enable the feature:

```toml
[features]
three_valued_logic = []
```

Then build or test with:

```bash
cargo test -p sea-core --features three_valued_logic
```

## Examples and migration guidance

- If your domain contains many optional values and you want SQL-like handling, enable `three_valued_logic` and update tests to assert `Null` where appropriate.
- If you prefer fail-fast detection of missing data (recommended when authoring new models), keep the feature disabled and address missing attributes explicitly.

## Performance guidance

- Expect a modest overhead (typical 5–10%) for evaluation-heavy workloads when `three_valued_logic` is enabled. See `benches/null_overhead.rs` for a benchmark harness.

## FAQ

- Q: Why is `NULL = NULL` not `True`?
- A: `NULL` means unknown; unknown equals unknown is itself unknown. This avoids false positives in policy checks where data absence should not be assumed equal.

## Appendix: Full truth tables

(Repeat machine-readable tables here for quick reference.)
