# Three-Valued Logic

DomainForge uses **Kleene's Three-Valued Logic (3VL)** for policy evaluation. This is a critical design choice for modeling architecture, where information is often incomplete.

## The Three States

1. **True**: The statement is demonstrably true.
2. **False**: The statement is demonstrably false.
3. **Unknown**: There is insufficient information to determine truth.

## Why Not Boolean?

In standard boolean logic, `null` or missing values often default to `False` or cause errors. In architecture modeling:

- **Scenario**: You have a policy "All databases must be encrypted".
- **Model**: You define a database `UserDB` but forget to specify the `encrypted` property.
- **Boolean Logic**: `UserDB.encrypted == true` evaluates to `False`. The policy fails. You think the DB is unencrypted.
- **3VL**: `UserDB.encrypted` is `Unknown`. The comparison evaluates to `Unknown`. The policy result is `Unknown`.

This distinction tells the architect: "I don't know if this is safe yet," rather than "This is unsafe."

## Truth Tables

### NOT (Negation)
| Input | Result |
|-------|--------|
| True | False |
| False | True |
| Unknown | Unknown |

### AND (Conjunction)
| A | B | Result |
|---|---|--------|
| True | True | True |
| True | False | False |
| True | Unknown | Unknown |
| False | True | False |
| False | False | False |
| False | Unknown | False |
| Unknown | True | Unknown |
| Unknown | False | False |
| Unknown | Unknown | Unknown |

### OR (Disjunction)
| A | B | Result |
|---|---|--------|
| True | True | True |
| True | False | True |
| True | Unknown | True |
| False | True | True |
| False | False | False |
| False | Unknown | Unknown |
| Unknown | True | True |
| Unknown | False | Unknown |
| Unknown | Unknown | Unknown |

## Practical Implications

When writing policies:

- **Strict Enforcement**: If you want to fail on unknowns, check for them explicitly or ensure your CI pipeline treats `Unknown` as a failure for production releases.
- **Progressive Modeling**: 3VL allows models to evolve. You can run policies against a partial sketch of a system without getting flooded with false positives.

## See Also

- [Policy Evaluation Logic](policy-evaluation-logic.md)
