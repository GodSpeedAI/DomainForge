# ADR-015: Role Bindings, Array Literals, and Typed `entity_instances` Collections

**Status:** Proposed
**Date:** 2026-08-13
**Deciders:** DomainForge Architecture Team (proposed by implementation
agent; needs human ratification before this status is final)

> **Ratification record** _(fill in when accepted)_:
> - Ratifier:
> - Approval date:

## Proposed distinction

Three small, independent additive grammar changes, all discovered while
resolving documented limitations (`limitations.md` L2, L3, L6) in a
consuming project's canonical `.sea` model:

1. `role_binding "Role" for "Entity"` — a declaration binding one
   previously-declared role to one previously-declared entity.
2. `[expr, expr, ...]` — an array literal usable anywhere `literal` is
   valid.
3. `entity_instances of "EntityType"` — an optional type-narrowing suffix
   on the existing `entity_instances` collection keyword.

## Current representational limitation

1. **Role bindings**: `Graph` has carried `entity_roles: IndexMap<ConceptId,
   Vec<ConceptId>>` and `assign_role_to_entity()` since role support was
   added, but no `.sea` syntax ever populated it — `entity_roles` was
   unconditionally empty coming out of the parser. The graph-level
   capability existed; the language had no way to invoke it.
2. **Array literals**: `literal` had no array form. A field declared
   `list<T>` or `list<ref<T>>` could never be given a concrete value by any
   instance — the entity was silently uninstantiable for that field, with
   no diagnostic. The evaluator side (`FieldType::List` recursive element
   validation, `MinItems`/`MaxItems`, dangling-reference rejection per
   element) was already fully implemented; only the parser could not
   produce an array-valued `Expression::Literal`.
3. **Typed `entity_instances`**: a policy quantifying over the bare
   `entity_instances` collection sees every declared instance regardless of
   entity type. A `forall`/`exists` predicate referencing a field present on
   only one entity type produces a NULL (not a violation, per the
   language's three-valued logic) for every instance of every other type,
   which is not the intended semantics for a policy meant to constrain one
   entity type in a heterogeneous model.

## Existing mechanisms considered and why they fail

- **Annotations**: none of the three concepts fit an annotation's
  free-form, per-declaration, unvalidated shape. A role binding is a
  relationship between two named declarations, not a property of one; an
  array literal is a value form, not metadata; a type filter changes what a
  quantifier iterates, which annotations cannot express.
- **`Mapping`/`Projection` overrides**: these retarget existing primitive
  categories into external formats. None of the three changes is
  target-specific — the missing behavior is native evaluation (graph
  construction and expression literals), not rendering.
- **A new `Profile`**: profiles configure projection behavior; none of
  these three gaps is a projection concern.
- **An `import`-based adapter**: none of the three gaps is an
  interoperability problem with an external system.
- **Reusing an existing collection form with a filter expression** (for
  array literals' sibling gap, and for the type-narrowing case):
  considered encoding `entity_instances of "EntityType"` as a new
  `Expression` variant instead of a string-encoded collection name. Rejected
  in favor of matching the codebase's existing idiom — `get_collection` in
  `policy/quantifier.rs` already dispatches on collection-name strings, and
  a new `Expression` variant would have required touching every consumer
  of `Expression` (serialization, formatter, IR builders) for a
  parser-local distinction. String-encoding as `entity_instances:{Type}`
  keeps the change local to `parse_collection`/`get_collection` and one new
  `Expression`-free dispatch arm.

## Cross-target semantics

All three are evaluation/graph-construction concerns, not renderer-specific:

- Role bindings populate `Graph::entity_roles`, consumed by any future
  responsibility/authorization-aware projection, not one target.
- Array literals make `list<T>` fields instantiable at all; every consumer
  of instance data (validation, every projection that reads instance
  fields) benefits identically.
- Typed `entity_instances` changes what a policy's quantifier iterates —
  policy evaluation is target-independent.

## Normative mappings

No new IR or projection target is introduced. Mapping is confined to
existing structures:

- `role_binding_decl` → `Graph::assign_role_to_entity(entity_id, role_id)`,
  resolved by name against previously-parsed `Role`/`Entity` declarations
  in a dedicated post-pass (mirrors the existing relation-resolution pass).
- `array_literal` → `Expression::Literal(JsonValue::Array(values))`, each
  element itself a `literal` (recursive), consumed by the existing
  `FieldType::List` validator unchanged.
- `entity_instances of "EntityType"` → encoded as the string
  `entity_instances:{EntityType}` in the AST's collection-name slot (same
  representation as the existing bare collection keywords), decoded by a
  new `collect_entity_instances()` helper in `policy/quantifier.rs` that
  filters the full instance set by entity type before the quantifier or
  aggregation consumes it.

## Grammar / AST impact

- `domainforge-core/grammar/sea.pest`:
  - `role_binding_decl = { ^"role_binding" ~ string_literal ~ ^"for" ~
    string_literal }`, added to `declaration_inner` before `role_decl` (PEG
    ordered-choice safety — both start differently enough not to require
    ordering, but placed defensively).
  - `array_literal = { "[" ~ (literal ~ ("," ~ literal)* ~ ","?)? ~ "]" }`,
    added to the `literal` alternation.
  - `collection` extended with an optional `(^"of" ~ string_literal)?`
    suffix on `^"entity_instances"` only.
- `domainforge-core/src/parser/ast.rs`:
  - New `AstNode::RoleBinding { role: String, entity: String }` variant.
  - `parse_role_binding()` and the `Rule::role_binding_decl` dispatch arm.
  - A "Role binding pass" after relation resolution, resolving both names
    via `resolve_by_name()` and calling `graph.assign_role_to_entity()`;
    fails closed on either name not resolving.
  - `parse_literal_expr()` gained a `Rule::array_literal` arm.
  - `parse_collection()` rewritten to detect the inner `string_literal` and
    encode as `entity_instances:{EntityType}`, with `.trim()` — pest's
    implicit whitespace-skip was found to leak into the base keyword's span
    when the optional suffix is present but does not match on a given
    input, so the bare-keyword case required an explicit trim (see
    "Errors and fixes" note below; this was a real bug caught by the
    consuming project's semantic-teeth regression suite, not a
    hypothetical).
- `domainforge-core/src/parser/ast_schema.rs` + `ast_convert.rs`: mirrored
  `RoleBinding` schema variant and `From` conversion (additive to
  `ast-v3.schema.json` — no new schema version; array literals and typed
  collections need no schema-visible variant since they reuse
  `Expression::Literal` and a `String` collection field, both already
  representable in the schema).
- `domainforge-core/src/formatter/printer.rs` and
  `domainforge-core/src/parser/printer.rs`: `RoleBinding` round-trip
  formatting (`role_binding "Role" for "Entity"`); array literals and the
  `of` suffix format through the existing `Expression`/collection-name
  printers unchanged.
- `domainforge-core/src/module/resolver.rs`: `declaration_name` extended so
  `RoleBinding` participates in module export resolution like other
  declarations.
- `domainforge-core/src/application/envelope.rs`: `CanonicalRoleBindingDecl`
  struct and `CanonicalSemanticPayload::RoleBinding` variant, so role
  bindings are represented in the Canonical Semantic Envelope.

## Compatibility impact

Purely additive for all three:

- `role_binding` is a new top-level keyword; no existing declaration form
  changes.
- `array_literal` only extends what `literal` can match; every previously
  valid `literal` still parses identically.
- The `collection` rule's `of "EntityType"` suffix is optional; every
  previously valid bare `entity_instances`/`flows`/`entities`/etc. still
  parses identically, and the whitespace-leak bug found while implementing
  this (see above) was fixed before merge, not shipped and patched later —
  bare `entity_instances` never produced a broken span in any released
  version.

No existing valid `.sea` file changes meaning.

## Migration path

None required — additive only.

## Fixtures

Inline fixtures in each new test file (no shared `fixtures/` directory
entries were needed — every case is a small, self-contained `.sea` source
string):

- `domainforge-core/tests/role_binding_tests.rs`
- `domainforge-core/tests/array_literal_tests.rs`
- `domainforge-core/tests/typed_entity_instances_collection_tests.rs`

## Tests

- `role_binding_tests.rs` (4 tests): binds a role to an entity and asserts
  `Graph::entity_roles`; rejects a binding to an undeclared role; rejects a
  binding to an undeclared entity; round-trips through the formatter.
- `array_literal_tests.rs` (6 tests): `list<string>` instantiation,
  `min_items`/`max_items` rejection, `list<ref<T>>` per-element reference
  validation (valid and dangling), empty-array parsing.
- `typed_entity_instances_collection_tests.rs` (5 tests): the type filter
  narrows a quantifier's iteration set correctly, including the regression
  case for the whitespace-leak bug (bare `entity_instances` in a
  comprehension-form aggregation, with no `of` suffix present at all).
- `policy_arithmetic_where_tests.rs` and `graph_operation_bound_policy_tests.rs`
  cover adjacent limitation fixes (L5, L9) landed in the same branch; not
  grammar changes, listed here only because `cargo fmt` touched them in the
  same commit series.

Full `cargo test --package domainforge-core --features cli` suite grew from
116 to 122 test groups across this branch's seven fix commits, 0 failures.
Re-verified against the real-world consuming model
(`sea-rs/.sea/interaction/interaction-model.sea`, 16 policies, one of which
depends on the `entity_instances of` filter) after every commit; the
model's `semantic_closure_hash` stayed byte-identical throughout, and 15 of
its 16 policies would have been broken by the whitespace-leak bug had it
shipped unfixed.

## Failure modes

- `role_binding` referencing an undeclared role or entity fails closed with
  an error naming the missing declaration (`role_binding references
  undefined role '<name>'` / `'<name>'` for entity), raised in the
  post-parse role-binding resolution pass.
- `array_literal` elements are validated exactly like scalar field values:
  a `list<ref<T>>` element with no matching instance is rejected with an
  error naming the dangling reference; `min_items`/`max_items` violations
  produce the existing field-constraint diagnostics.
- `entity_instances of "UnknownType"` narrows to an empty set (not an
  error) if no instances of that type exist, matching the existing
  behavior of the bare collection keywords when no instances exist at all.

## Disconfirmation criterion

If `role_binding` ever needs to carry a responsibility kind (e.g.
"accountable" vs. "consulted") rather than a bare role↔entity edge, this
declaration's shape was too thin and should have taken a body from the
start — that would be a breaking change to `role_binding_decl` requiring
its own ADR and migration path. This was deliberately deferred rather than
guessed at here because the graph data model
(`Graph::entity_roles: IndexMap<ConceptId, Vec<ConceptId>>`) has no field
for responsibility kind today, and adding one speculatively would violate
the "no renderer convenience changes" / semantic-necessity bar this policy
sets.
