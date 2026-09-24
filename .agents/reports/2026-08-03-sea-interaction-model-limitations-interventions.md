# Resolving the SEA Forge Interaction-Model Limitations in DomainForge

**Date:** 2026-08-03
**Author:** Claude (Opus 5), on request
**Subject repo:** `/home/sprime01/projects/domainforge` @ 0.15.0
**Consumer repo:** `/home/sprime01/projects/sea-rs`, branch `ultracode/sea-forge-completion`

## 1. Scope

Inputs read: `sea-rs/.agents/STATUS_CACHE.md`, `sea-rs/.sea/interaction/README.md`,
`.../validation/diagnostics.md`, `.../validation/limitations.md`, and the head of
`.../interaction-model.sea`. On the DomainForge side I read the grammar, the AST,
the graph converter, the module resolver, the application-contract layer, the
policy evaluator, the semantic-pack subsystem, the RDF/KG projector, the CLI
command surface, and the relevant docs. Every claim below cites a file and line
in DomainForge at the working-tree state I inspected.

I did not build, run, or modify anything. No DomainForge file changed.

## 2. Headline finding

**Seven of the ten limitations are not missing language features. They are
existing DomainForge features that the graph pipeline discards and the CLI cannot
reach.**

SEA has two semantic layers that do not currently meet:

| | **Layer A — ontology / graph** | **Layer B — application contract** |
|---|---|---|
| Declarations | `entity`, `role`, `resource`, `flow`, `relation`, `instance`, `policy`, `pattern`, `metric` | typed `entity { … }` bodies, `record`, `enum`, `operation` |
| Typing | none — instance fields are a free-form `HashMap<String, Expression>` | closed v0.1 type model: scalars, `quantity<U>`, `ref<T>`, `list<…>`, named types |
| Integrity | entity-exists check only | `key` fields, `optional`, `min`/`max`/`min_length`/`max_items`/`pattern`, defaults, dangling-reference detection |
| Composition | single entry AST | deterministic multi-module closure, merged by namespace |
| Provenance | none | per-module `semantic_content_hash`, `semantic_closure_hash`, pack refs, document `self_hash` |
| Projections | RDF, BPMN, CMMN, Cedar, protobuf, CALM, domain code, +8 others | **none** |
| CLI access | `validate`, `parse`, `project`, `pack`, … | **none** |

The interaction model is authored entirely in Layer A. Layer A is, by design,
untyped. Layer B — added under ADR-013 / Milestone 0 (`src/application/mod.rs:1`)
— already implements most of what `limitations.md` proposes as future grammar,
but its declarations never reach the graph and it has no command-line surface.

The limitations document proposes seven new general grammar constructs. Based on
the codebase, **five of the seven already exist in the grammar today.** The work
is connection, not invention.

## 3. Limitation-by-limitation evidence

| # | Limitation | Actual status in DomainForge | Precise site |
|---|---|---|---|
| 1 | First-class journey identity | Correctly deferred. Nothing to add. | — |
| 2 | Typed instance references | **Exists.** `ref<Target>` in the grammar; resolves to `FieldType::EntityRef` with dangling-reference diagnostics. Unreachable because instances are untyped and entity bodies are dropped. | `grammar/sea.pest:257`, `src/parser/ast.rs:182`, `src/application/resolve.rs:1293` |
| 3 | Ordered / conditional / recovery transitions | **Partially exists.** `operation` has `state`, `effect creates\|mutates\|reads`, and `failure <code> for <kind…>` over a closed kind set. No ordered multi-step transition graph. | `grammar/sea.pest:299–311` |
| 4 | Entry / completion / terminal conditions | **Exists.** `access policy_governed by <policy> at precondition\|invariant\|postcondition fails with <code>`. This is exactly the enforceable entry/completion predicate the document asks for. | `grammar/sea.pest:296–298` |
| 5 | Instance-aware integrity policies | **Defect.** The policy evaluator's `instances` collection binds `graph.all_instances()` — *resource* instances. `all_entity_instances()` is never bound, so all 76 authored records are invisible to every policy. | `src/policy/quantifier.rs:440` vs `src/graph/mod.rs:508` |
| 6 | Interface-binding declarations | Genuinely absent as a first-class construct, but typed entity bodies + `enum` + `ref<>` cover the enforceable part (valid CJ refs, closed maturity values, required fields). | — |
| 7 | Role-to-entity binding | **Half-exists.** `Graph::assign_role_to_entity` works and is already surfaced to policies as `entity.roles`; it has **no parser path** — only the BPMN and CMMN projectors synthesize bindings. Layer B has `actor <symbol_ref>` on operations. | `src/graph/mod.rs:205`, `src/policy/quantifier.rs:358`, callers at `src/projection/bpmn/mod.rs:146`, `src/projection/cmmn/mod.rs:180`; `grammar/sea.pest:295` |
| 8 | Closed vocabularies | **Exists.** `enum Name { Member = "wire", … }` parses into `EnumDecl`. Layer B only. | `grammar/sea.pest:247`, `src/parser/ast.rs:222` |
| 9 | Same-namespace imported instances | **Defect, and the fix already exists.** See §4. | `src/parser/mod.rs:80–86` vs `src/application/resolve.rs:196` |
| 10 | RDF preservation | **Defect.** `KnowledgeGraph::from_graph` iterates entities, roles, resources, patterns, relations, and flows — and nothing else. No entity instances, no resource instances, no policies, no flow annotations. | `src/kg.rs:145` |

Two cross-cutting root causes explain most of the table:

**Root cause A — the graph converter discards typed entity bodies.**

```rust
// src/parser/ast.rs:3341
AstNode::Entity { name, domain, version, annotations, body: _ } => {
```

`entity "X" { key id: uuid, journey: ref<CanonicalJourney>, maturity: Maturity }`
parses (`grammar/sea.pest:89–91`), survives into the AST and the AST JSON schema
(`src/parser/ast_schema.rs:552`, `src/parser/ast_convert.rs:579`), and is then
thrown away at graph construction. `Entity` in the graph has only
`attributes: HashMap<String, Value>` (`src/primitives/entity.rs:58`). `Record`,
`Enum`, and `Operation` have no match arm in the converter at all.

This single line is why limitations 2, 5, 6, 8 and — transitively — 10 exist.
`docs/rdf-projections.md:79` states the RDF v1 non-goal as *"No typed entity
attributes. Attributes are untyped in the IR today, so the ontology derives
classes, relations, and named individuals only."* The RDF loss is downstream of
the type loss, not independent of it.

**Root cause B — the CLI uses the weaker of two module resolvers.**

`ModuleResolver` (`src/module/resolver.rs:25`) validates that imports exist and
that named targets are exported, then `parse_to_graph_with_options` converts
**only the entry AST** (`src/parser/mod.rs:85–86`). Imported declarations are
never merged, so an instance whose entity type lives in an imported module fails
at `Graph::add_entity_instance` with the exact diagnostic sea-rs recorded
(`src/graph/mod.rs:489`).

## 4. Limitation 9 already has a working implementation

`build_graph_from_set` groups every module in a resolved closure **by effective
namespace**, merges their declarations into one AST per namespace, converts, and
absorbs:

```rust
// src/application/resolve.rs:196
pub(crate) fn build_graph_from_set(set: &ResolvedModuleSet)
    -> Result<Graph, Vec<ApplicationDiagnostic>>
```

Because the instance and its entity type land in the same namespace group, the
same-namespace imported-instance case resolves correctly. `Graph::absorb`
(`src/graph/mod.rs:107`) merges all thirteen collections including
`entity_instances` and `entity_roles`.

`resolve_application_graph` (`src/application/resolve.rs:186`) exposes this. It
has **zero callers in `src/cli/`, `src/bin/`, or `src/lib.rs`.** The Python,
TypeScript, and WASM bindings expose only `resolve_application_contract_json`
(`src/python/graph.rs:278`, `src/typescript/graph.rs:46`, `src/wasm/graph.rs:53`)
— the contract, not the graph.

The resolver behind it is well covered: named and wildcard imports, unexported
symbols, cycles, diamond collapse, escaping specifiers, and duplicate keys all
have tests (`src/module/resolver.rs:905–1058`).

The gap is an adapter: the closure resolver consumes an in-memory `SourceMap`
keyed by normalized logical path (`src/module/resolver.rs:338`), while the CLI
holds filesystem paths. Nothing reads a directory into a `SourceMap`.

## 5. The hashing / ledger question

Your recollection is substantively correct — content-addressed, append-only,
supersession-linked composition exists — but it is **two mechanisms, and neither
is an import channel.** Being precise about this matters, because the difference
determines what it can fix.

**a) Semantic packs — a governed vocabulary ledger.**
`pack_content_hash` (sha256 over canonical JSON, `src/semantic_pack/canonical_json.rs:98`);
`meaning_fingerprint` (sha256 over concept definitions, with a build-time rule
that `meaning_version` must bump whenever it changes, or the build fails
`meaning_version_not_bumped`); Ed25519 signing (`src/semantic_pack/signing.rs:12`);
`replaces_pack_ids` supersession chains; and deterministic multi-pack merge with
explicit precedence, conflict detection, and a `merged_pack_hash`
(`src/semantic_pack/pack_set.rs:50–100`). Two builds from identical inputs are
bit-for-bit identical.

That is a ledger in every meaningful sense. But `validate_graph_with_pack`
(`src/semantic_pack/validator.rs:206`) **checks a graph's terms against the pack**
— three-valued `valid` / `invalid` / `unknown`. A pack never contributes
declarations to a model. It cannot supply `ReusableStep`.

**b) The semantic-closure envelope — this is the "import by hash".**
`CanonicalSemanticEnvelope` (`src/application/envelope.rs:36`) records
`modules: Vec<CanonicalModuleRef>` — each with `logical_id`, `namespace`, and
`semantic_content_hash` — plus `import_graph`, `namespace_bindings`,
`semantic_packs`, `semantic_declarations`, and `resolved_references`. Over all of
it: `semantic_closure_hash` (`src/application/envelope.rs:949`), and over the
document: `self_hash`, verified on load (`src/application/envelope.rs:1209`).

So a model's meaning is identified by the exact bytes of every module that
composed it plus the exact pack hashes that governed it. That is the property you
were reaching for. It lives entirely in Layer B, it is reachable only from Rust
and the language bindings, and — critically — **it is the same code path that
fixes limitation 9.** Wiring the CLI to the closure resolver delivers modular
imports and content-addressed provenance in one change.

## 6. Recommended interventions, ranked

Ranked by unlocked-value ÷ effort. I1–I3 are the ones I would actually do.

### I1 — Stop discarding typed entity bodies *(highest leverage)*

Give `Entity` a `fields: Vec<FieldDecl>` and populate it at
`src/parser/ast.rs:3341` instead of `body: _`. Then type-check
`AstNode::Instance` fields against the owning entity's declared fields in
`Graph::add_entity_instance` (`src/graph/mod.rs:467`): required/optional, scalar
types, `enum` membership, `ref<T>` targets resolving to a declared instance,
`key` uniqueness, and the `FieldConstraintDecl` set that
`src/application/validate.rs` already knows how to check.

Reuses `FieldDecl`, `FieldType`, `FieldConstraintDecl`, `EnumDecl`,
`check_entity_key`, and `check_duplicate_fields` verbatim. No new grammar, no new
type model.

Resolves limitations **2, 5, 6, 8** outright and unblocks **10**. Entirely
backward compatible: bodyless entities keep today's behavior, exactly as Layer B
already handles at `src/application/resolve.rs:757`.

For sea-rs this converts `canonical_journey_ids: "CJ01; CJ02"` into
`journeys: list<ref<CanonicalJourney>>`, and `implementation_maturity: "exercised"`
into a member of `enum Maturity`. `CJ99` starts failing validation.

### I2 — Route the file-based CLI through the closure resolver

Add a filesystem→`SourceMap` adapter (walk the entry file's transitive relative
imports, key by normalized logical path) and call `build_graph_from_set` instead
of converting the entry AST alone. Optionally expose the closure envelope via a
new `domainforge app` / `--emit-envelope` surface so the CLI can print
`semantic_closure_hash`.

Resolves limitation **9** and delivers §5(b) provenance. The merge logic,
diagnostics (APP014/APP015), budgets, and tests already exist; this is an adapter
plus a call-site swap.

sea-rs's 1,059-line single file becomes five real modules. Add the regression
test `limitations.md:95` asks for.

### I3 — Bind entity instances into the policy evaluator

Add an `entity_instances` collection alongside `instances` at
`src/policy/quantifier.rs:440`, projecting each `Instance` to
`{ id, name, entity, …fields }`, and register the singular `entity_instance`
binder at line 494.

Roughly the size of the existing `instances` arm. Resolves the remainder of
limitation **5**: `forall ei in entity_instances` becomes expressible, so the
"exactly 12 journeys, unique IDs, valid CJ bindings" checks move from external
Python and `jq` into SEA policy. Best done after I1 so policies can rely on typed
fields.

### I4 — Emit instances, policies, and flow annotations in RDF

Extend `KnowledgeGraph::from_graph` (`src/kg.rs:145`) to iterate
`all_entity_instances()`, `all_instances()`, and `all_policies()`, and to emit
flow annotations as triples. With I1 in place, typed fields give each field a
correct RDF datatype and each `ref<T>` an object-property triple rather than a
string literal — which is precisely what `docs/rdf-projections.md:79` records as
the blocker.

Resolves limitation **10**. Also fixes the `flow_d63efd98bd7f4004` identity
problem: emit the authored flow name and the `journey_id` annotation. Note the
scope caveat in that doc — `model.ttl` is produced by the legacy `kg.rs`
serializer, so `--base-iri` still applies only to JSON-LD and OWL.

### I5 — Author-visible role-to-entity binding

Add grammar for what `Graph::assign_role_to_entity` (`src/graph/mod.rs:205`)
already supports and what the policy evaluator already exposes as `entity.roles`
(`src/policy/quantifier.rs:358`). Resolves limitation **7** and gives CMMN
`performerRef` real performers instead of absent ones.

Smallest genuinely-new grammar on this list. `limitations.md` is right that this
is a credible general candidate.

### I6 — Ordered transitions *(defer)*

The only item where I agree substantial new grammar is warranted, and the only
one I would not start now. `operation` already covers guards
(`at precondition|postcondition`), outcomes (`failure … for <kind>`), and effects
(`creates|mutates|reads`). What is missing is *ordering* across steps. Do I1–I3
first, model the journeys with typed fields and `operation` clauses, and see what
is still genuinely unrepresentable. Limitation **3** is the residue.

## 7. What I would not do

- **Do not add a `journey` keyword.** `limitations.md` is right to defer it, and
  I1 removes most of the pressure for it.
- **Do not add a third import form.** The resolver's semantics are correct and
  tested; only the CLI wiring is missing.
- **Do not try to make semantic packs supply declarations.** They are a
  validation and governance artifact. Overloading them would blur the
  approval/signature trust model in `docs/semantic-packs.md` for no gain that I2
  does not already deliver.
- **Do not generate CMMN yet.** sea-rs's decision to skip it
  (`diagnostics.md:101`) is correct at the current lowering. I1 and I5 change
  the inputs enough that the question is worth reopening afterward, not before.

## 8. Caveats

- **Layer B has no projections.** `Record`, `Enum`, and `Operation` have no match
  arm in the graph converter, so authoring the interaction model natively in
  Layer B today would trade untyped-but-projectable for typed-but-unprojectable.
  This is exactly why I1 (pull Layer B's type model *into* the graph) is ranked
  above "move the model to Layer B". Anyone recommending the latter should
  confirm this first.
- I read code and docs; I did not compile, run the test suite, or execute
  DomainForge against the sea-rs model. Effort estimates are structural
  judgments, not measurements.
- I did not audit the eight event/authority/verification/activation projectors
  for how they would react to typed entity fields. `docs/projection-target-implementation-status.md`
  says they read entities, resources, and flows only, so I expect I1 to be inert
  for them — worth verifying before merging.
- The `key` requirement in `check_entity_key` (`src/application/validate.rs:77`)
  is mandatory in Layer B: an entity with a body but no key field is an App005
  error. If I1 reuses that check unchanged, every typed entity in sea-rs needs a
  key field. That is probably correct — `journey_id` is already the natural key —
  but it is a real authoring constraint, not a free upgrade.
