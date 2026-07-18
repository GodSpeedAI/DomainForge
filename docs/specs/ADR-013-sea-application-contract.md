# ADR-013: SEA Application Contract

**Status:** Proposed
**Date:** 2026-07-18
**Deciders:** DomainForge Architecture Team

> **Ratification record** _(fill in only after explicit human acceptance)_:
> - Ratifier:
> - Approval date:

See `docs/reference/sea-language-evolution-policy.md`; every criterion there is
addressed below. The normative surface contract (exact grammar, types,
validation, diagnostics, canonicalization, fixtures) is
`docs/reference/sea-application-contract.md`. This ADR records the single
decision; the reference is its binding specification. Both documents encode the
D1–D10 decisions accepted by the DomainForge repository maintainer on
2026-07-18 in
`.agents/reports/2026-07-18-application-contract-decision-packet.md`.

## Proposed distinction

SEA gains a first-class Application Contract: typed entity state bodies with
one aggregate-key field, identityless `record` declarations for operation
input/output/failure-detail data, closed `enum` declarations, and first-class
`operation` declarations that bind identity, intent, direction, actor, access,
typed input/output records, state entity, effect, transaction boundary,
canonical failures, idempotency, concurrency, evidence, and lifecycle into one
machine-checkable unit of `generation_ready` meaning (spec §5.1). Resolution
happens once, in a shared `ResolvedModuleSet` symbol table that feeds both
Graph and a new resolved `ApplicationContract` model.

## Current representational limitation

- `entity_decl` has no body (`domainforge-core/grammar/sea.pest:84-88`);
  entities cannot declare typed fields, so DTO/storage schema does not exist in
  canonical SEA.
- Domain IR infers fields from `instance of` as `str`
  (`domainforge-core/src/projection/domain/ir.rs:68`, `ir.rs:439`), which spec
  §5.1 forbids as a semantic source.
- There is no port/operation IR (`rg PortIr` returns nothing); repository, bus,
  and read-model traits are synthesized by renderers from flow shape
  (`ir.rs:390` `cqrs_kind`, `ir.rs:403` `method_from_flow`) — outputs, not
  semantic input.
- Every policy guards every method as a no-op (`ir.rs:49`, `ir.rs:202-210`),
  the exact pattern spec §5.1 bans.
- Effect, transaction, failure, idempotency, concurrency, evidence, and
  lifecycle have no grammar or IR representation at all.
- Hashing a formatted entry file cannot capture resolved import meaning; the
  semantic closure needs an import-resolved model.

## Existing mechanisms considered and why they fail

- **Annotations on an existing category** — spec §5.2 bans generic annotations
  unless their keys, types, defaults, and validation become a versioned
  language contract; at that point they are grammar in disguise, without
  formatter, schema, or diagnostic integration. Rejected (packet D1 Option 3,
  D2 Option 3).
- **`Mapping`/`Projection` overrides** — mappings choose realization for
  existing meaning; they cannot introduce the meaning itself. Spec §5.4 assigns
  every adapter concern's authoritative source to explicit SEA, never to
  mapping/projection surfaces.
- **A new `Profile`** — profiles carry nonsemantic configuration (listen
  address, ports, file roots per §5.4). Typed fields and operation semantics in
  a profile would be exactly the smuggling §5.4 prevents. Rejected (packet D1
  Option 4).
- **An `import`-based adapter** — imports bring declarations into scope; they
  cannot create declaration kinds the grammar lacks. An imported "adapter
  model" would be unvalidated foreign content with no diagnostics.

## Cross-target semantics

The Application Contract is meaningful across every projection target: domain
review bundles, the policy-to-operation table, semantic/realization diff, RDF
semantic refs, OpenTelemetry conventions, Adapter IR, and every future
provider family consume the same operation meaning (packet D9 projection
matrix). No single renderer motivates this change; the change exists because
spec §5.1 forbids generation without this meaning anywhere.

## Normative mappings

`docs/reference/sea-application-contract.md` §4 (field-by-field semantic
table), §8 (Rust model), and §13 (traceability matrix) map every new element
to its AST node, resolved `ApplicationContract` construct, validation rule,
diagnostic, and D9 fixture line. The Adapter-field source matrix from spec
§5.4 is reproduced verbatim below as the ownership boundary this contract
satisfies:

| Adapter concern | Authoritative source | Profile/provider may do |
|---|---|---|
| operation ID, intent, direction, actor/access | explicit SEA Application Contract | bind transport/runtime only |
| input/output types, fields, constraints, cardinality | explicit typed SEA declarations | map to native DTO/codec types without changing constraints |
| state/aggregate, effect, transaction boundary | explicit SEA operation contract | implement the declared boundary |
| applicable policies and failure outcomes | explicit SEA policy-to-operation scope | select proven enforcement and canonical error mapping |
| concurrency and idempotency | explicit SEA operation contract, or `not_applicable` with validated reason | use a named, proven strategy |
| evidence and externally visible lifecycle | explicit SEA contract | choose storage/export mechanism |
| provider family | deterministic lowering from operation direction/effect | satisfy the family |
| listen address, test port, local file root | versioned profile plus binding | choose schema-valid nonsemantic configuration |
| dependency versions, SDK features, migrations, wiring | exact compiled provider descriptor/emitter | contribute implementation only |
| health endpoint and proof fixtures | versioned profile | emit as clearly profile-owned behavior |

**Semantic ownership and import resolution (D3, accepted):** module
resolution produces one `ResolvedModuleSet` (logical module IDs, source
hashes, import graph, export visibility, named/wildcard alias bindings,
normalized qualified symbol IDs, declaration origins, one symbol table).
Named imports bind only named exported symbols; wildcard imports stay
alias-qualified; unexported declarations are invisible. Cycles, unresolved
aliases, and non-equivalent duplicate qualified IDs are errors; equivalent
duplicates collapse. Graph and `ApplicationContract` are both constructed
from that symbol table and perform no independent resolution. Existing
concepts keep `ConceptId`; application-only declarations use
`ApplicationSymbolId` (normalized namespace + declaration kind + name).
Canonical closure order is logical module ID, then declaration kind, then
stable semantic ID; authored order survives only in declared sequences.

## Grammar / AST impact

Files that must change together in Milestone 0 (none change under this ADR):

- `domainforge-core/grammar/sea.pest` — add `record_decl`, `enum_decl`,
  `operation_decl`, `entity_body` and field/type/clause rules; extend
  `declaration_inner`; extend the `resource_decl` units-branch lookahead with
  `application_decl_prefix` (reference §2).
- `domainforge-core/src/parser/ast.rs` — new `RecordDecl`, `EnumDecl`,
  `OperationDecl` nodes; optional `body` on the entity node.
- `domainforge-core/src/parser/ast_schema.rs` — additive serialized variants;
  no change to the `{metadata, declarations}` top-level shape.
- `domainforge-core/src/parser/ast_convert.rs` — conversions for the new
  nodes.
- `domainforge-core/src/formatter/printer.rs` and
  `domainforge-core/src/parser/printer.rs` — canonical formatting and
  round-trip printing for the new declarations.
- `domainforge-core/src/module/resolver.rs` — produce `ResolvedModuleSet`
  with the D3 symbol-table invariants.
- `schemas/ast-v3.schema.json` — additive declaration variants.
- New `schemas/application-contract-v1.schema.json` and a new
  `domainforge-core/src/application/` module (`contract.rs`, `resolve.rs`,
  `validate.rs`, `canonical.rs`; reference §8).
- Bindings affected by AST/contract shape: `domainforge-core/src/python/`,
  `domainforge-core/src/typescript/`, `domainforge-core/src/wasm/` gain
  `parse_to_ast_json(source)` and
  `resolve_application_contract_json(entry_logical_path, sources_json)`
  (reference §9).
- `domainforge-core/src/projection/domain/ir.rs` — later lowering consumes
  `ApplicationContract`; heuristic paths remain for legacy domain output only.

## Compatibility impact

Additive. `record`, `enum`, `operation`, and all field/type/clause tokens are
contextual: they are recognized only where the complete new production can
begin, and they are not added to the global `declaration_keyword` alternation.
An entity without a typed body produces exactly the same AST and canonical
formatter output as before. Existing valid AST v3 documents remain valid and
serialize unchanged; no top-level discriminator is added. The compatibility
oracle is before/after equality of formatter and AST output over the existing
corpus, plus a collision fixture placing every new token in every current
identifier, name, namespace, unit, alias, and annotation-key position
(reference §10). No existing valid `.sea` file changes meaning.

## Migration path

Not required: the change is classified additive. Contingency (D8, accepted):
if any accepted token cannot satisfy the corpus and collision tests during
implementation, that token is reclassified breaking and must ship with a
semantic version bump, `ConceptChange` declarations, migration documentation,
and compatibility fixtures before implementation proceeds; a non-additive AST
JSON change requires an explicit ast-v4 migration decision.

## Fixtures

Created only after this ADR is Accepted, at exactly these paths:

- `fixtures/application_generation/flagship/command-write.sea` — D9
  `place_order` (reference §11.1).
- `fixtures/application_generation/flagship/query-read.sea` — D9
  `get_order_status`, importing the command module (reference §11.2).
- `fixtures/application_generation/compat/keyword-collision.sea` — every new
  token used as identifier/name/namespace/unit/alias/annotation-key.
- `fixtures/application_generation/compat/entity-no-body.sea` — legacy entity
  unchanged-output oracle.
- `fixtures/application_generation/invalid/` — one file per APP diagnostic,
  named `app001-missing-semantics.sea` through
  `app015-schema-version.sea` (reference §5).

## Tests

Required by the Milestone 0 implementation (proposed names):

- `domainforge-core/tests/application_parser_tests.rs` — positive parse of
  both flagship fixtures; negative parse/validation for every APP code.
- `domainforge-core/tests/application_contract_tests.rs` — resolution,
  symbol-table invariants, duplicate/cycle/visibility errors, closure
  ordering, `source_set_hash` and `semantic_closure_hash` determinism.
- `domainforge-core/tests/format_golden.rs` (extended) — formatter round-trip
  and canonical-output goldens for the new declarations, plus the
  before/after corpus-equality compatibility oracle.
- `domainforge-core/tests/ast_schema_tests.rs` (extended) — additive-variant
  schema validation against `schemas/ast-v3.schema.json` and the new
  `schemas/application-contract-v1.schema.json`, including
  unknown-field rejection.
- `domainforge-core/tests/semantic_pack_compat_tests.rs` — existing pack
  fixtures' hashes and signatures byte-identical before and after Milestone 0
  (D10).
- Binding parity tests in `domainforge-core/tests/python_binding_tests.rs`,
  `domainforge-core/tests/typescript_binding_tests.rs`, and the WASM test
  suite: identical contract JSON for identical source maps on all targets.

## Failure modes

Everything fails closed. Any missing consequential field emits
`APP001 missing_application_semantics` with operation/source ref, missing
fields, why each matters, and the smallest supported remediation; the compiler
never fills fields from provider defaults, method names, fixture instances, or
language-model guesses. Reference §5 defines APP001–APP015 (unknown type
references, invalid record shapes, duplicate fields, aggregate-key violations,
entity-as-DTO misuse, policy-scope errors including reserved
invariant/postcondition points, failure-code violations, strategy-field
violations, unproven `not_applicable`, effect/state mismatch, constraint
errors, enum errors, closure symbol collisions, and contract-JSON
unknown-field rejection). Policy evaluation fails closed: missing facts,
`UNKNOWN`, evaluator errors, and false obligations map to the policy's
declared failure code.

## Disconfirmation criterion

This decision was wrong if any of the following is observed:

- a second projection family (beyond the local HTTP/SQLite flagship path)
  never consumes operation semantics, in which case operations should have
  been target-specific configuration;
- the flat-record, closed-type v0.1 algebra cannot express the D9 flagship
  without annotations or fixture-derived types;
- the shared `ResolvedModuleSet` cannot serve both Graph and
  `ApplicationContract` without divergent resolution semantics, which would
  invalidate the D3 single-symbol-table ownership choice;
- the compatibility oracle finds any existing corpus file whose AST or
  canonical formatter output changes, which would falsify the additive
  classification and trigger the Migration path contingency.
