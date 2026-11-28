# ADR-019: The chrono to time migration touches AST and printer files but makes no language change

**Status:** Accepted
**Date:** 2026-11-28
**Deciders:** Sam Prime (repository owner), recorded during the RUSTSEC-2026-0543 migration

This ADR exists because `scripts/check-sea-language-change.sh` requires an ADR
whenever `parser/ast.rs`, `parser/ast_convert.rs`, `parser/printer.rs` or
`formatter/printer.rs` change. It records that this change does **not** alter
the SEA language, so the criteria in
`docs/reference/sea-language-evolution-policy.md` do not apply.

## Proposed distinction

None. No new SEA syntax, category, annotation or keyword is introduced. No
`grammar/sea.pest` rule changes and no AST node changes shape.

## Current representational limitation

Not applicable. The Rust type of a metric's `@refresh_interval` / `@window`
value changed from `chrono::Duration` to `time::Duration`, because chrono is
unmaintained (RUSTSEC-2026-0543). The printers call `whole_seconds()` instead
of `num_seconds()`; the text they print is identical.

## Existing mechanisms considered and why they fail

Not applicable: no new representation is proposed.

## Cross-target semantics

Not applicable. Serialized durations keep their `[secs, nanos]` form and the
printed `@refresh_interval` / `@window` seconds are unchanged; see
`docs/reference/timestamps.md`.

## Normative mappings

Not applicable.

## Grammar / AST impact

`parser/ast.rs`: `use time::Duration`. `parser/ast_convert.rs`,
`parser/printer.rs`, `formatter/printer.rs`: `num_seconds()` becomes
`whole_seconds()`. The golden tests in `domainforge-core/tests/` pass
unchanged.
