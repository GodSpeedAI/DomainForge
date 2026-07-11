# SEA language evolution policy

SEA is a semantic source language, not a convenient dumping ground for
target-specific configuration.

New technologies normally enter DomainForge as:

1. projection targets;
2. target realization mappings;
3. profiles;
4. adapters;
5. activation modes;
6. override schemas.

They do not automatically justify new SEA syntax.

## When a grammar change is permitted

A grammar change is permitted only when **all** of the following are true:

1. **Semantic necessity** — the distinction cannot be represented clearly
   and safely using existing primitives, annotations, mappings,
   projections, imports, or profiles.
2. **Cross-target value** — the distinction is meaningful beyond one
   implementation target, or is essential to one major first-class
   projection category.
3. **Stable semantics** — the concept has a precise definition, identity,
   validation rules, and lifecycle.
4. **Normative mapping** — its mapping into every affected IR and
   projection is documented before implementation.
5. **Backward compatibility** — existing valid SEA files retain their
   meaning unless a declared breaking change and migration exists.
6. **Parser completeness** — grammar, AST, formatter, serialization,
   bindings, schema, and diagnostics are updated together.
7. **Evidence** — golden fixtures and tests demonstrate a real affordance
   that could not be obtained cleanly without the change.
8. **Migration** — breaking changes use semantic versioning,
   `ConceptChange`, migration documentation, and compatibility tests.
9. **Approval artifact** — the change includes an ADR explaining why
   projection/profile/annotation extension was insufficient.
10. **No renderer convenience changes** — a renderer being easier to write
    is never sufficient reason to change the language.

## Enforcement

`scripts/check-sea-language-change.sh` runs in CI (the `lint` job) and
fails any pull request that touches grammar/AST/schema/formatter files
without also touching an ADR and a test/fixture file. See that script for
the exact watched-path list and required-artifact check.

## Worked example

ADR-012 (`docs/specs/ADR-012-cell-environment-declarations.md`) is the
reference example of a change that met this bar: ten declarations with no
existing representation, cross-target semantics (`SystemDependency` etc.
are meaningful independent of any one renderer), a documented normative
mapping (`docs/cell-environment-projections.md`), full parser/AST/
schema/formatter completeness, additive backward compatibility, and
fixture/test evidence (`fixtures/cell_env/`,
`domainforge-core/tests/cell_projection_tests.rs`).
