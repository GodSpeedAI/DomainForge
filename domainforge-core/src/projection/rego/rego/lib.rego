# Three-valued helpers mirroring DomainForge's policy evaluator
# (policy/quantifier.rs `expand` + policy/core.rs three-valued evaluation).
#
# Every boolean-position helper returns "true" | "false" | "unknown" |
# "error", optionally "~"-prefixed (see below). "unknown" is DomainForge's
# NULL; "error" is an evaluation failure (the evaluator returns `Err`).
# Neither is ever represented by Rego `undefined`.
package domainforge.sea.lib

import rego.v1

tri(b) := "true" if {
	b == true
} else := "false"

# The evaluator has two regimes, and they disagree on NULL:
#
# - `expand` folds a source-level `and`/`or` whose operands both reduced to
#   literals, and that fold is NULL-absorbing (`false and NULL` is NULL).
# - Anything left unreduced (every `not`, every non-empty forall/exists, and
#   any `and`/`or` over those) is evaluated afterwards with Kleene logic
#   (`false and NULL` is false).
#
# A result therefore carries which regime produced it: a bare string is a
# reduced literal, a "~" prefix marks an unreduced tree. `exists_unique` and
# filtered counts only ever count reduced "true", exactly like the
# evaluator's `is_true_literal`.
value(r) := trim_prefix(r, "~")

_reduced(a, b) if {
	not startswith(a, "~")
	not startswith(b, "~")
}

_kleene_and(a, b) := "false" if {
	"false" in {a, b}
} else := "unknown" if {
	"unknown" in {a, b}
} else := "true"

_kleene_or(a, b) := "true" if {
	"true" in {a, b}
} else := "unknown" if {
	"unknown" in {a, b}
} else := "false"

and3(a, b) := "error" if {
	"error" in {a, b}
} else := "unknown" if {
	_reduced(a, b)
	"unknown" in {a, b}
} else := _kleene_and(a, b) if {
	_reduced(a, b)
} else := concat("", ["~", _kleene_and(value(a), value(b))])

or3(a, b) := "error" if {
	"error" in {a, b}
} else := "unknown" if {
	_reduced(a, b)
	"unknown" in {a, b}
} else := _kleene_or(a, b) if {
	_reduced(a, b)
} else := concat("", ["~", _kleene_or(value(a), value(b))])

_negate(v) := "true" if {
	v == "false"
} else := "false" if {
	v == "true"
} else := v

# `not` is never folded by `expand`, so its result is always unreduced.
not3(a) := "error" if {
	a == "error"
} else := concat("", ["~", _negate(value(a))])

# Quantifiers take the array of per-item results. Errors are strict: every
# item is expanded before anything is folded, so one failing item fails the
# policy even when another item would have decided it. An empty collection
# reduces to a literal; a non-empty one stays an unreduced Kleene fold.
_values(rs) := {value(r) | some r in rs}

forall3(rs) := "error" if {
	"error" in rs
} else := "true" if {
	count(rs) == 0
} else := "~false" if {
	"false" in _values(rs)
} else := "~unknown" if {
	"unknown" in _values(rs)
} else := "~true"

exists3(rs) := "error" if {
	"error" in rs
} else := "false" if {
	count(rs) == 0
} else := "~true" if {
	"true" in _values(rs)
} else := "~unknown" if {
	"unknown" in _values(rs)
} else := "~false"

# exists_unique counts only items whose condition reduced to literal true;
# unknown and unreduced items are not counted, so the result is a literal
# and never "unknown".
exists_unique(rs) := "error" if {
	"error" in rs
} else := "true" if {
	count([1 | some r in rs; r == "true"]) == 1
} else := "false"

# count(...) with a filter keeps only definitely-true items, but a failing
# filter still fails the policy.
guard(rs, r) := "error" if {
	"error" in rs
} else := r

_null(l, _) if l == null

_null(_, r) if r == null

_numbers(l, r) if {
	is_number(l)
	is_number(r)
}

_strings(l, r) if {
	is_string(l)
	is_string(r)
}

cmp_eq(l, r) := "unknown" if {
	_null(l, r)
} else := tri(l == r)

cmp_ne(l, r) := "unknown" if {
	_null(l, r)
} else := tri(l != r)

cmp_lt(l, r) := "unknown" if {
	_null(l, r)
} else := tri(l < r) if {
	_numbers(l, r)
} else := "error"

cmp_lte(l, r) := "unknown" if {
	_null(l, r)
} else := tri(l <= r) if {
	_numbers(l, r)
} else := "error"

cmp_gt(l, r) := "unknown" if {
	_null(l, r)
} else := tri(l > r) if {
	_numbers(l, r)
} else := "error"

cmp_gte(l, r) := "unknown" if {
	_null(l, r)
} else := tri(l >= r) if {
	_numbers(l, r)
} else := "error"

str_contains(l, r) := "unknown" if {
	_null(l, r)
} else := tri(contains(l, r)) if {
	_strings(l, r)
} else := "error"

str_startswith(l, r) := "unknown" if {
	_null(l, r)
} else := tri(startswith(l, r)) if {
	_strings(l, r)
} else := "error"

str_endswith(l, r) := "unknown" if {
	_null(l, r)
} else := tri(endswith(l, r)) if {
	_strings(l, r)
} else := "error"

# Field access with DomainForge's missing-field rule: absent means NULL.
field(item, name) := object.get(item, name, null)
