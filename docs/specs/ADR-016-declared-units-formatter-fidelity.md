# ADR-016: Declared Units/Dimensions Graph Fidelity and Mapping/Projection Formatter Round-Trip

**Status:** Proposed
**Date:** 2026-09-25
**Deciders:** DomainForge Architecture Team (proposed by implementation
agent; needs human ratification before this status is final)

> **Ratification record** _(fill in when accepted)_:
> - Ratifier:
> - Approval date:

This ADR records two fidelity fixes that touch watched files
(`src/formatter/printer.rs`, plus graph-conversion code exercised through
`src/parser/ast.rs`) without adding any new syntax. It exists to satisfy the
SEA grammar-change gate, which fires on file presence, not on semantic
content.

## Proposed distinction

Two independent fixes, both restoring already-specified behavior rather than
adding language surface:

1. CLI-resolved graphs now retain declared units and dimensions.
   `Graph::absorb` merges `declared_dimensions`/`declared_units` exactly like
   every other collection, so `parse`, `validate`, and `project` (which merge
   one converted graph per namespace via `build_graph_from_set`) produce the
   same declarations the language bindings' `Graph::parse` already returned.
2. The formatter emits mapping/projection contract keys as identifiers, so
   `fmt` output re-parses. Previously it quoted them (`{ "name": ... }`),
   which the grammar rejects.

## Current representational limitation

There is none at the language level: `Dimension`, `Unit`, `Mapping`, and
`Projection` declarations already parse, validate, and convert. The gaps were
downstream of parsing — one merge function that predated the declared-unit
fields, and one printer call that used the wrong writer. No existing SEA
category or annotation needed to change.

## Existing mechanisms considered and why they fail

No alternative mechanism was needed because no new concept is introduced:

- **Annotations**: nothing new to annotate; the declarations already exist.
- **`Mapping`/`Projection` overrides**: the formatter fix concerns how
  overrides are printed, not a new override kind.
- **A new `Profile`**: no projection behavior changes.
- **An `import`-based adapter**: the absorb fix is in graph assembly, not
  module resolution.

## Cross-target semantics

Declared units feed every projection target that reads the unit registry
(CALM, RDF, Lean, protobuf, and others); dropping them in CLI-resolved graphs
meant CLI-driven projections silently lost unit declarations that
binding-driven consumers saw. Contract-key printing is formatter-level and
target-independent.

## Normative mappings

| Element | IR construct | Realization |
|---|---|---|
| `Dimension "D"` | `DeclaredDimension` | `Graph::add_declared_dimension`, carried through `absorb` |
| `Unit "U" of "D" factor F base "B"` | `DeclaredUnit` | `Graph::add_declared_unit`, carried through `absorb` |
| Mapping rule / projection override keys | grammar `identifier` in `mapping_field` / `projection_field` | `formatter::printer` writes keys with `write`, not `write_string_literal` |

## Grammar / AST impact

- `domainforge-core/grammar/sea.pest`: unchanged.
- `domainforge-core/src/parser/ast.rs`: unchanged by this ADR's fixes (the
  DEBT-003 work on this branch added graph-owned insertion-ordered records
  for declared units/dimensions in the AST-to-graph conversion; no grammar
  rule changed).
- `domainforge-core/src/graph/mod.rs`: `absorb` gains two `extend` lines.
- `domainforge-core/src/formatter/printer.rs`: mapping/projection key
  emission changed from quoted to identifier form.
- `domainforge-core/src/cli/parse.rs`: human summary prints `Resource
  instances:`, `Entity instances:`, `Policies:` (policy count), `Patterns:`.
- Bindings (TypeScript, Python, WASM): read accessors only; no AST shape
  change. `schemas/ast-v3.schema.json`: unchanged.

## Compatibility impact

Additive. No previously valid `.sea` file changes meaning:

- Files with `Dimension`/`Unit` declarations now expose them in CLI graph
  JSON (previously empty) — strictly more information.
- Files with `Mapping`/`Projection` contracts: `fmt` output changes from
  unparseable to parseable — strictly a fix.
- `parse --format human` summary text changes (new labels); no consumer
  should parse it (JSON exists for tooling).

## Migration path

None required (non-breaking). No version bump beyond the release-please
minor driven by the accompanying `feat(bindings)` commit.

## Fixtures

Inline sources in the regression tests (no new fixture files): the
dimension/unit source map in `dimension_unit_tests.rs`, and the
mapping/projection and summary-count models in `cli_tests.rs`.

## Tests

- `domainforge-core/tests/dimension_unit_tests.rs`:
  `test_resolve_path_keeps_declared_units_and_dimensions_in_source_order`
  (resolve path, source order, base factors).
- `domainforge-core/tests/cli_tests.rs`:
  `test_parse_human_summary_counts`,
  `test_fmt_mapping_projection_round_trip` (fmt → unquoted keys → validate).
- Binding parity (same branch): `typescript-tests/graph.test.ts` (12),
  `tests/test_graph.py` accessor test, WASM
  `test_graph_exposes_parsed_declarations_in_source_order` (22/22 wasm).
- Full gates: `just all-tests` exit 0, `just wasm-test` exit 0.

## Failure modes

- `absorb` on duplicate unit/dimension identities is last-wins, identical to
  every other collection it merges; conflicts surface, if at all, at the
  `UnitRegistry` compatibility check during conversion, as before.
- Formatter keys originate from the grammar `identifier` rule, so emitting
  them unquoted cannot produce a non-identifier; programmatically constructed
  ASTs with non-identifier keys were already unrepresentable.

## Disconfirmation criterion

If a future projection target requires non-identifier contract keys (e.g.
dotted or spaced property names), the grammar rules `mapping_field` and
`projection_field` would need to accept quoted keys first — at which point
this decision (print identifiers) should be revisited alongside that grammar
change.
