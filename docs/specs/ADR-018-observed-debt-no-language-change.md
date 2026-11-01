# ADR-018: Observed-debt fixes touch the AST file but make no language change

**Status:** Accepted
**Date:** 2026-11-01
**Deciders:** Sam Prime (repository owner), recorded during an unattended debt-resolution run

This ADR exists because `scripts/check-sea-language-change.sh` requires an ADR
whenever `domainforge-core/src/parser/ast.rs` changes. It records that this
change does **not** alter the SEA language. It is not a grammar change, so the
criteria in `docs/reference/sea-language-evolution-policy.md` do not apply.

## Proposed distinction

None. No new SEA syntax, category, annotation, or keyword is introduced. No
`grammar/sea.pest` rule changes. No AST node changes shape.

## Current representational limitation

Not applicable. The change fixes two internal defects:

- A model's declared `@version` header was parsed into `FileMetadata` but never
  exposed on the in-memory `Graph`, so Rust callers could not read it.
- A stray debug `println!` sat in the parser's entity-annotation path.

## Existing mechanisms considered and why they fail

Not applicable: no new representation is proposed. The version fix is a Rust
API addition (`Graph::version()`), which is the correct layer because the value
already exists in the source.

## Cross-target semantics

Not applicable. `Graph::version()` is a host-language accessor. It does not
change what any projection target receives unless a projection opts in.

## Normative mappings

Not applicable. No new IR construct or projection mapping is introduced.

## Grammar / AST impact

Files in the watched set that changed:

- `domainforge-core/src/parser/ast.rs`: removed one debug `println!`; added one
  line that copies `metadata.version` onto the constructed `Graph`. No type,
  variant, or field of the AST changed.

Files not in the watched set but changed: `domainforge-core/src/graph/mod.rs`
(new `version` field, `#[serde(skip_serializing_if = "Option::is_none")]`),
`domainforge-core/src/application/resolve.rs` (sets the version from the entry
module), `domainforge-core/src/cli/import.rs` (a `Calm` import format variant
that calls the existing `calm::import`), `domainforge-core/src/cli/test.rs`
(now fails instead of succeeding).

No file under `schemas/ast-*.schema.json` changed. Unversioned models serialize
exactly as before, because the new `Graph` field is omitted when `None`.

## Compatibility impact

Additive only. Existing valid `.sea` files parse and validate identically. The
only observable CLI change is that `domainforge test <pattern>` now exits 1
instead of 0, and `domainforge import --format calm` is newly accepted.

## Migration path

Not required; no breaking change.

## Fixtures

No new language fixtures. The change is verified by
`domainforge-core/tests/observed_debt_regression_tests.rs`, which uses inline
sources.

## Tests

`domainforge-core/tests/observed_debt_regression_tests.rs` covers the version
accessor on the single-module and multi-module (entry-versus-import) paths, the
CLI JSON carrying the version, the CALM round trip, and the `test` subcommand
failing.

## Failure modes

- `domainforge test <pattern>`: exits 1 with "`domainforge test` is not
  implemented ... no tests were run".
- `domainforge import --format calm` on malformed JSON: exits 1 with "Failed to
  parse CALM JSON".

## Disconfirmation criterion

If a future change needs a grammar or AST shape change to express `@version` at
the graph level, this ADR was insufficient and that change must go through the
full policy in `docs/reference/sea-language-evolution-policy.md`.
