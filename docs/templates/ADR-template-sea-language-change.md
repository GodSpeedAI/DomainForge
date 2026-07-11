# ADR-NNN: <title>

**Status:** Proposed
**Date:** <YYYY-MM-DD>
**Deciders:** <names/team>

See `docs/reference/sea-language-evolution-policy.md` before filling this
in — a grammar change is permitted only when every criterion there is met.

## Proposed distinction

What new concept needs representation, in one or two sentences.

## Current representational limitation

Why no existing SEA category (or annotation on one) already expresses
this.

## Existing mechanisms considered and why they fail

For each of: annotations on an existing category, `Mapping`/`Projection`
overrides, a new `Profile`, an `import`-based adapter — say why it doesn't
work.

## Cross-target semantics

Is this concept meaningful beyond one projection target? If it's
single-target, say so explicitly and justify why it still belongs in the
language rather than a target-specific config file.

## Normative mappings

The table (or reference to a doc containing the table) mapping each new
element to its IR construct and primary realization.

## Grammar / AST impact

List every file that must change together: `grammar/sea.pest`,
`src/parser/ast.rs`, `src/parser/ast_schema.rs`, `src/parser/ast_convert.rs`,
`src/formatter/printer.rs`, `src/parser/printer.rs`,
`src/module/resolver.rs`, `schemas/ast-v3.schema.json`, any language
bindings affected by AST shape.

## Compatibility impact

Additive or breaking? What existing valid `.sea` files, if any, change
meaning?

## Migration path

Required only for breaking changes: semantic version bump, `ConceptChange`
declarations, migration documentation, compatibility test fixtures.

## Fixtures

Where the proving fixtures live.

## Tests

Which test files/suites cover parser, formatter round-trip, IR, renderer,
and golden-determinism behavior.

## Failure modes

What fails closed, and with what error code/message.

## Disconfirmation criterion

What future observation would mean this decision was wrong (e.g. "if a
second projection target never needs these declarations, they should have
been target-specific config instead").
