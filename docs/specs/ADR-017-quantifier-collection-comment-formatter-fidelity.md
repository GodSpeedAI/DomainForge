# ADR-017: Quantifier Display, Collection Decoding, and Comment-Preservation Formatter Fidelity

**Status:** Proposed
**Date:** 2026-09-28
**Deciders:** DomainForge Architecture Team (proposed by implementation
agent; needs human ratification before this status is final)

> **Ratification record** _(fill in when accepted)_:
> - Ratifier:
> - Approval date:

This ADR records three fidelity fixes that touch a watched file
(`src/formatter/printer.rs`, plus `src/policy/expression.rs` and
`src/formatter/comments.rs`) without adding any new syntax. It exists to
satisfy the SEA grammar-change gate, which fires on file presence, not on
semantic content.

## Proposed distinction

Three independent fixes, all restoring already-specified behavior rather
than adding language surface:

1. `Expression::Display` for `Quantifier` emitted a debug-style
   `ForAll(...)` wrapper the grammar does not accept. It now emits the
   lowercase SEA spelling the case-insensitive `quantifier` grammar rule
   accepts: `forall x in <collection>: (<condition>)` (likewise `exists`,
   `exists_unique`). Non-`Binary` conditions get their own parens;
   `Binary` conditions keep their self-parenthesizing (no `((...))`).
2. Every collection-printing site (`Quantifier`, `Aggregation`,
   `AggregationComprehension`, `GroupBy`) printed the internal
   `entity_instances:X` encoding verbatim. `parse_collection` encodes
   `entity_instances of "X"` that way (see its doc comment), so the
   formatter now decodes it back to `entity_instances of "X"` — the only
   spelling the grammar accepts.
3. `Formatter::format_ast` only re-emitted file-header comments, silently
   dropping every other `//` comment, and `extract_comments` treated `//`
   inside string literals as a comment start. Comments are now flushed
   exactly once (own-line before the following declaration, trailing ones
   kept on the declaration's line, leftovers at end of file), and the
   scanner tracks `"…"` / `"""…"""` string state per
   `grammar/sea.pest`'s `string_literal`/`multiline_string`.

## Current representational limitation

There is none at the language level: quantified policies, entity-instance
collections, and `//` comments already parse and validate. The gaps were
downstream of parsing — two `Display`/printer call sites that used the
wrong spelling, and a comment pipeline that only knew about headers. No
existing SEA category or annotation needed to change.

## Existing mechanisms considered and why they fail

No alternative mechanism was needed because no new concept is introduced:

- **Annotations**: nothing new to annotate; quantifiers, collections, and
  comments already exist.
- **`Mapping`/`Projection` overrides**: the fixes concern how policy
  expressions and comments are printed, not a new override kind.
- **A new `Profile`**: no projection behavior changes.
- **An `import`-based adapter**: the fixes are in expression printing and
  comment handling, not module resolution.

## Cross-target semantics

Policy-expression printing feeds every consumer of `fmt` output
(irrespective of projection target); unparseable quantifier output broke
all of them equally. Comment preservation is formatter-level and
target-independent.

## Normative mappings

| Element | IR construct | Realization |
|---|---|---|
| `forall` / `exists` / `exists_unique` | `Quantifier::ForAll` / `Exists` / `ExistsUnique` | `quantifier_keyword` in `src/policy/expression.rs` |
| `entity_instances of "T"` | `Expression::Variable("entity_instances:T")` (internal encoding) | `format_collection` decodes at every quantifier/aggregation/group_by print site |
| `//` comments (own-line, trailing, EOF) | `formatter::comments::Comment` | `format_ast` flush + string-aware `extract_comments` scanner |

## Grammar / AST impact

- `domainforge-core/grammar/sea.pest`: unchanged.
- `domainforge-core/src/parser/ast.rs`: unchanged.
- `domainforge-core/src/policy/expression.rs`: `Quantifier` display
  spelling + `format_collection` helper (no AST shape change).
- `domainforge-core/src/formatter/printer.rs`: comment flush in
  `format_ast`; collection decoding at four print sites.
- `domainforge-core/src/formatter/comments.rs`: string-aware `//`
  scanner (no `Comment` shape change).
- Bindings (TypeScript, Python, WASM): unaffected; no AST shape change.
  `schemas/ast-v3.schema.json`: unchanged.

## Compatibility impact

Additive. No previously valid `.sea` file changes meaning:

- Files with quantified policies: `fmt` output changes from unparseable
  to parseable — strictly a fix.
- Files with `entity_instances of "T"`: same — strictly a fix.
- Files with comments: `fmt` output now retains them (previously
  dropped, except headers) — strictly more information. A `//` inside a
  string literal no longer manufactures a comment line.

## Migration path

None required (non-breaking). No version bump.

## Fixtures

Inline sources in the regression tests (no new fixture files): the
quantifier/collection sources and comment models in
`formatter_fidelity_tests.rs` (integration) and the 16 unit tests in
`formatter::printer::tests`.

## Tests

- `domainforge-core/tests/formatter_fidelity_tests.rs` (new, gate
  evidence): quantifier round trips (`forall`, `exists`,
  `entity_instances of`), comment preservation (between declarations,
  trailing, end-of-file), string-literal `//` not treated as comment —
  each asserting reparse + idempotent reformat.
- `formatter::printer::tests` (unit, same branch): 16 tests covering
  `exists_unique`, non-`Binary` conditions, post-`@namespace` comments,
  and exactly-once preservation across a five-comment file.
- Full gates: `cargo test -p domainforge-core --features cli`,
  `cargo clippy -p domainforge-core --all-targets --features cli`,
  `cargo fmt --all --check`, `just python-test`, `just ts-test`.

## Failure modes

- `format_collection` only rewrites `Variable` names with the exact
  `entity_instances:` prefix produced by `parse_collection`; any other
  collection prints exactly as before.
- A trailing comment is attached to the first output line of the
  declaration starting on its source line; comments on blank lines flush
  as own-line comments before the next declaration, so none are dropped
  or duplicated (asserted exactly-once by test).

## Disconfirmation criterion

If the grammar ever gains a new collection encoding or a second comment
syntax (e.g. block comments), the `entity_instances:` prefix check and
the `//` scanner must be revisited alongside that grammar change — at
which point this decision should be re-evaluated with the new syntax.
