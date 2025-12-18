# ADR: Expression/Policy Language Bindings

**Status:** Proposed  
**Date:** 2025-12-17  
**Authors:** DomainForge Architecture Team  
**Depends On:** [Canonical Expression Normalizer](./canonical-normalizer.md)

## Context

The canonical expression normalizer has been implemented in Rust core, but `Expression` and `Policy` types are not exposed in Python, TypeScript, or WASM bindings. This limits the ability for external consumers to:

1. Programmatically construct policy expressions
2. Use the `normalize()` and `is_equivalent()` methods
3. Evaluate policies from language-native code

## Decision

Implement full Expression and Policy bindings for all three target languages.

## Scope

### Phase 1: Expression Bindings

Create Expression class/struct with factory methods for all variants:

| Variant         | Factory Method                                                |
| --------------- | ------------------------------------------------------------- |
| Literal         | `Expression.literal(value)`                                   |
| Variable        | `Expression.variable(name)`                                   |
| Binary          | `Expression.binary(op, left, right)`                          |
| Unary           | `Expression.unary(op, operand)`                               |
| Quantifier      | `Expression.quantifier(kind, var, collection, condition)`     |
| MemberAccess    | `Expression.member_access(object, member)`                    |
| Cast            | `Expression.cast(operand, target_type)`                       |
| Aggregation     | `Expression.aggregation(function, collection, field, filter)` |
| QuantityLiteral | `Expression.quantity(value, unit)`                            |
| TimeLiteral     | `Expression.time(timestamp)`                                  |
| IntervalLiteral | `Expression.interval(start, end)`                             |

### Phase 2: Normalization Methods

Add to Expression bindings:

- `normalize()` → `NormalizedExpression`
- `is_equivalent(other)` → `bool`
- `to_string()` → display representation

### Phase 3: Policy Bindings

Create Policy class with:

- Constructor: `Policy.new(name, expression)`
- Builder methods: `with_modality()`, `with_priority()`, etc.
- `evaluate(graph)` → `EvaluationResult`
- `normalized_expression()` → `NormalizedExpression`

### Phase 4: NormalizedExpression Bindings

Expose wrapper with:

- `stable_hash()` → deterministic hash
- `inner()` → access normalized expression
- `to_string()` → canonical string form

## Files to Modify

| File                                | Changes                                      |
| ----------------------------------- | -------------------------------------------- |
| `sea-core/src/python/policy.rs`     | Add Expression, Policy, NormalizedExpression |
| `sea-core/src/typescript/policy.rs` | Add Expression, Policy, NormalizedExpression |
| `sea-core/src/wasm/policy.rs`       | Add Expression, Policy, NormalizedExpression |
| `tests/test_*.py`                   | Python binding tests                         |
| `typescript-tests/*.test.ts`        | TypeScript binding tests                     |

## Estimated Effort

- **Per binding**: ~200-300 lines of Rust wrapper code
- **Total**: ~600-900 lines + tests
- **Timeline**: 1-2 days of focused work

## Alternatives Considered

1. **JSON serialization only** - Parse expressions from JSON instead of factory methods (rejected: less ergonomic)
2. **String parsing** - Parse DSL strings in bindings (rejected: duplicates parser logic)
3. **Keep internal** - Only use in Rust/LSP (current state, deferred approach)

---

# ADR: Normalizer CLI Command

**Status:** Proposed  
**Date:** 2025-12-17

## Context

The canonical normalizer is implemented but not accessible from CLI.

## Decision

Add `sea normalize <expr>` command for interactive normalization.

## Proposed Usage

```bash
# Normalize an expression string
sea normalize "b AND a"
# Output: (a AND b)

# Show equivalence
sea normalize --check-equiv "a AND b" "b AND a"
# Output: Equivalent (hash: 0x7f3a...)

# Output normalized form with hash
sea normalize --json "true AND x"
# Output: {"normalized": "x", "hash": "0x..."}
```

## Files to Modify

| File                         | Changes                          |
| ---------------------------- | -------------------------------- |
| `sea-core/src/bin/sea.rs`    | Add `normalize` subcommand       |
| `sea-core/src/parser/mod.rs` | May need expression-only parsing |

## Dependencies

- Requires expression-only parser (currently parses full SEA files)
- May need `clap` subcommand additions

---

# ADR: LSP Hover Normalization Integration

**Status:** Proposed  
**Date:** 2025-12-17  
**Depends On:** Canonical Normalizer (complete)

## Context

LSP hover currently displays policy expressions using `expr.to_string()`. This can produce non-deterministic output for semantically equivalent expressions.

## Decision

Use normalized form in hover signatures for deterministic display.

## Proposed Changes

### domainforge-lsp

| File                           | Changes                                                  |
| ------------------------------ | -------------------------------------------------------- |
| `src/hover/symbol_resolver.rs` | Use `expr.normalize().to_string()` for policy signatures |

### Example

**Before:**

```
Policy tax_check as:
    (rate > 0) AND (valid == true)
```

**After (normalized):**

```
Policy tax_check as:
    ((0 < rate) AND valid)
```

## Considerations

- Normalized form may differ from source - consider showing both
- Performance: normalization is cheap but runs on every hover
- May want configuration option for "show original vs normalized"
