# SEA Application Contract — Gate 0 Decision Packet (2026-07-18)

Governing spec: `.agents/specs/domainforge-conversational-application-generator-spec.md`
(Status: Proposed v0.2, adversarially revised). Governing plan:
`.agents/plans/2026-07-18-conversational-application-generator.md`.

This packet gathers verified repository evidence and records the accepted
D1–D10 language/ownership decisions that must govern ADR-013 and the Milestone
0 code plan. The DomainForge repository maintainer explicitly accepted the
reviewed recommendations, including the amendments recorded below, on
2026-07-18.

> **Post-Gate-A amendment status: Accepted by the DomainForge repository
> maintainer on 2026-07-18.** The Gate B adversarial review proposed narrow
> corrections to D3, D5, and D6:
> exact-byte legacy `ConceptId` identity; strict bounds; closed enum evolution;
> entity defaults plus deterministic state/output lowering; reachable-only
> `not_applicable`; precise concurrency ordering; and deferral of structured
> failure details. The maintainer explicitly accepted ADR-013 and these
> amendments; Human Gate B is closed.

## Baseline verification (Task 1 Steps 1–2)

Recorded on branch `agent/projection-targets` (tracking
`origin/agent/projection-targets`).

| Command | Result |
|---|---|
| `git status --short --branch` | ` M .gitignore`, `?? .agents/` — unrelated user work, left untouched. |
| `test ! -e docs/specs/ADR-013-sea-application-contract.md` | exit 0 — ADR-013 number is available. |
| `test -e docs/specs/ADR-012-cell-environment-declarations.md` | exit 0 — ADR-012 present. |
| `just rust-test` | exit 0 — full suite passed (final doc-test block: 25 passed; 0 failed; 4 ignored). |
| `devbox run -- cargo run -p domainforge-core --features cli --bin domainforge -- parse fixtures/projection_cell/basic/model.sea --ast --format json` | exit 0 — JSON with top-level keys `metadata`, `declarations`; 14 declarations. |

Baseline is green; documentation work below is not masking any failure.

## Current-capability evidence table (Task 1 Step 3)

| Question | Current evidence | Required conclusion |
|---|---|---|
| Do `Entity` declarations own typed fields? | `domainforge-core/grammar/sea.pest:84-88` — `entity_decl` is `^"entity" ~ name ~ version? ~ entity_annotation* ~ (in_keyword ~ identifier)?`; there is no field/body block. `domainforge-core/src/projection/domain/ir.rs:68` documents fields as `(field, "str")` gathered from `instance of`, and `ir.rs:439` confirms "v1 is string-typed". | Existing entities cannot define application DTO/storage schema. |
| Do current ports own application contracts? | `rg PortIr domainforge-core/src/` returns no matches; `projection/domain/ir.rs` has no port IR type. Repository/bus/read-model traits are synthesized by renderers from flow/CQRS shape (`ir.rs:390` `cqrs_kind`, `ir.rs:403` `method_from_flow`). | Generated traits are outputs, not semantic input. |
| Are policies scoped to operations? | `domainforge-core/src/projection/domain/ir.rs:49` — "Every policy guards every method in v1 (no-op bodies)"; `ir.rs:202-210` turns each policy into one domain error plus a `check_<slug>()` no-op guard applied uniformly. | v0.1 must define explicit scope and proven enforcement. |
| Does Graph preserve Cell-like declarations? | `domainforge-core/src/cli/project.rs:199-204` — the Cell projection "parses the Ast directly (ADR-012 deviation)" and is dispatched before Graph because Graph drops Cell context. | Ownership must be deliberately settled; Cell is not automatic precedent. |
| Are AST JSON shapes public? | `domainforge-core/src/parser/ast_schema.rs` defines stable serializable schema types (`ImportDecl:46`, `FileMetadata:70`, `PolicyMetadata:105`, `TargetFormat:155`, …); `parse --ast --format json` emits `{metadata, declarations}`. Parser/AST changes trigger `docs/reference/sea-language-evolution-policy.md`. | Any new public declaration needs schema and compatibility treatment. |
| Do bindings expose graph/core structures? | `domainforge-core/src/python/graph.rs`, `src/typescript/graph.rs`, and `src/wasm/graph.rs` all exist and wrap core Graph surfaces. | Public Graph/primitive additions require parity planning. |
| Can the current source hash close imports? | Canonicalization today is the semantic-pack primitive `domainforge-core/src/semantic_pack/canonical_json.rs:7` (`canonical_json`) over pack content; hashing a formatted entry file does not capture resolved import meaning. | The accepted ownership model must support an import-resolved semantic closure. |

## Decision records (Task 1 Step 4)

The decisions below form one coherent set: typed entity state plus identityless
operation records (D1), first-class operations (D2), one shared resolved symbol
table feeding Graph and a dedicated application-contract model (D3), explicit
and executable precondition policy scope (D4), a small closed type system (D5),
closed effect/failure/idempotency semantics (D6), additive AST v3 evolution and
a versioned public application-contract schema (D7), contextual-keyword
compatibility (D8), an order-management flagship bounded to local deterministic
behavior (D9), and a canonicalization boundary that leaves existing pack hashes
untouched (D10).

## D1 — Typed application data category

**Question:** Is typed application data a new first-class SEA category, an
extension of an existing category, or target configuration?

**Why it matters:** Spec §5.1 forbids generating any operation whose typed
input/output records are not explicit canonical SEA meaning. Where those types
live decides grammar, AST, schema, formatter, and binding scope.

**Repository evidence:** `entity_decl` has no body (`sea.pest:84-88`); Domain
IR fields are inferred string-typed from instances (`ir.rs:68`, `ir.rs:439`),
which spec §5.1 explicitly forbids as a semantic source.

**Options considered:**
1. New first-class `record`/typed-body declaration category in the grammar.
2. Extend `entity_decl` with an optional typed field body.
3. Generic annotations carrying type strings.
4. Target/profile configuration outside SEA.

**Recommended option:** Adopt one explicit hybrid, recorded as Option 5:

- `entity` gains an optional typed state body. An entity used as generated
  persisted state declares exactly one aggregate-key field.
- a new first-class, identityless `record` declaration defines operation input,
  output, failure-detail, and value data;
- v0.1 records are flat: fields may use a scalar, enum, quantity, entity
  reference, or list of those types, but may not contain another record;
- operation input and output types MUST be records, never entities;
- entity references carry the referenced entity's aggregate-key type, not the
  entity's language-level `ConceptId`.

This keeps domain identity and persisted state on `entity`, prevents transport
DTOs from masquerading as entities, and makes every generated field
first-class grammar rather than an annotation.

**Trade-offs:** The hybrid adds one declaration category and extends an existing
rule, but it removes ambiguity between aggregate state, DTOs, and language
identity. Flat records intentionally defer nested object graphs. Options 3 and
4 fail §5.2's ban on unvalidated annotations and §5.4's authoritative-source
matrix.

**Human decision:** Accepted as amended (Option 5) by the DomainForge repository
maintainer on 2026-07-18.

## D2 — Use cases/operations as declarations

**Question:** Are use cases/operations first-class SEA declarations or a typed
extension of an existing semantic declaration (e.g. `flow`)?

**Why it matters:** Operations are the unit of `generation_ready` (§5.1); the
compiler must locate identity, direction, actor, types, effect, policies, and
failures on one declaration.

**Repository evidence:** Flows exist but Domain IR derives command/query kind
heuristically (`ir.rs:390` `cqrs_kind`, `ir.rs:403` `method_from_flow`) and
carries none of the §5.1 fields.

**Options considered:**
1. New first-class `operation` (or `use_case`) declaration with a typed body.
2. Typed extension block on existing `flow_decl`.
3. Annotations on flows.

**Recommended option:** Option 1 — a new first-class operation declaration
that references entities/records, policies, and state. Flows describe process
narrative; operations carry the machine contract. Keeping them separate makes
the change additive and leaves existing flows untouched.

**Trade-offs:** Two declaration kinds can describe overlapping behavior;
ADR-013 must define how an operation may reference a flow. Option 2 risks
breaking existing flow fixtures and formatter output; Option 3 fails §5.2.

**Human decision:** Accepted as recommended (Option 1) by the DomainForge
repository maintainer on 2026-07-18.

## D3 — Semantic ownership and import behavior

**Question:** Does the application contract live in Graph, a new resolved
semantic model, or a justified AST-owned model?

**Why it matters:** This decides where import resolution, validation, and the
semantic closure happen, and which surfaces need binding parity. Plan
constraint: ADR-012's direct-AST Cell path is a disputed deviation, not
precedent.

**Repository evidence:** `cli/project.rs:199-204` shows Cell bypasses Graph
because Graph drops Cell context — evidence that Graph today loses
declaration classes it was not designed for. Bindings wrap Graph
(`python/graph.rs`, `typescript/graph.rs`, `wasm/graph.rs`).

**Options considered:**
1. Extend Graph to carry application-contract nodes.
2. New resolved semantic model (`ApplicationContract`) built from the AST
   after module/import resolution, alongside Graph.
3. AST-owned contract (repeat the ADR-012 deviation with justification).

**Recommended option:** Option 2, with one mandatory ownership pipeline:

1. module resolution produces one `ResolvedModuleSet` containing logical module
   IDs, source hashes, the import graph, export visibility, named/wildcard alias
   bindings, qualified symbol IDs, declaration origins, and a single
   symbol table;
2. named imports bind only the named exported symbols; wildcard imports remain
   alias-qualified; unexported imported declarations are not visible;
3. import cycles, unresolved aliases, and two distinct declarations with the
   same qualified ID are errors; repeated reachability of the same declaration
   origin collapses to one symbol;
4. Graph and the dedicated `ApplicationContract` are both constructed from that
   symbol table. Neither performs independent name or import resolution;
5. references to existing SEA concepts use the current exact-byte,
   case-sensitive `ConceptId` inputs; application-only declarations use a new
   stable `ApplicationSymbolId` derived from NFC-normalized, case-preserving
   namespace, declaration kind, and name;
6. canonical closure order is logical module ID, then declaration kind, then
   canonical declaration ID. Authored order is retained only for fields
   explicitly declared as sequences.

This avoids overloading Graph while preventing two independently resolved
semantic worlds.

**Trade-offs:** The resolver must expose a new typed output instead of returning
only an entry AST, and the application model needs its own schema and binding
parity. That cost buys one authoritative identity/import boundary shared with
Graph.

**Human decision:** Accepted as amended (Option 2 with the shared resolver and
symbol-table invariants above) by the DomainForge repository maintainer on
2026-07-18.

## D4 — Policy-to-operation scope and enforcement

**Question:** What is the exact policy-to-operation scope and
enforcement-point semantics? Universal no-op guards are forbidden.

**Why it matters:** §5.1 requires "applicable policies and their enforcement
point" per operation; §5.4 assigns policy scope to explicit SEA, never to
providers.

**Repository evidence:** `ir.rs:49` and `ir.rs:202-210` — every policy guards
every method as a no-op, the exact pattern the spec bans.

**Options considered:**
1. Operations declare `governed_by <policy>` explicitly and each reference
   names an enforcement point.
2. Policies declare which operations they scope to.
3. Implicit scoping by aggregate/state touched.

**Recommended option:** Option 1 with this closed v0.1 contract:

- every operation declares an actor role and exactly one access mode: `public`
  or `policy_governed`; `authenticated` is unsupported in v0.1 because the
  release boundary explicitly excludes authentication;
- `policy_governed` names one or more policies and one declared canonical
  failure per policy;
- v0.1 supports `precondition` enforcement only. Invariant and postcondition
  enforcement remain reserved future values and make an operation unsupported;
- the evaluator receives a typed context containing the operation ID, actor
  role, normalized input record, and pre-operation aggregate state when one
  exists;
- only constraint policies whose expressions can evaluate from that context are
  generation-ready. Missing facts, `UNKNOWN`, evaluator errors, and false
  obligation results fail closed and map to the policy's declared failure;
- dangling policy references and policies incompatible with their operation
  context are errors. A general or legacy policy that no application operation
  references remains valid.

Operation-side scoping keeps readiness-relevant meaning on the operation while
the typed context makes enforcement executable rather than a named no-op hook.

**Trade-offs:** v0.1 cannot generate authenticated access, invariants, or
postconditions. Those capabilities require a later authority/evaluator contract
and conformance evidence. Operation-side scope duplicates policy names but
avoids implicit joins and preserves unrelated policies.

**Human decision:** Accepted as amended (Option 1 with the closed v0.1 context
and fail-closed rules above) by the DomainForge repository maintainer on
2026-07-18.

## D5 — Minimal v0.1 type system

**Question:** What are the v0.1 types, field constraints, required/optional
rules, cardinality, identity, and reference semantics?

**Why it matters:** Every typed record (D1) and operation signature (D2)
depends on this closed set; anything omitted becomes an unvalidated escape
hatch.

**Repository evidence:** Today's only field typing is the string-typed
instance inference (`ir.rs:68`); the grammar already has `unit_decl` and
`dimension_decl` (`sea.pest:34`) for quantity semantics that the type system
should reference rather than duplicate.

**Options considered:**
1. Closed scalar set — `string`, `int`, `decimal`, `bool`, `timestamp`,
   `uuid` — plus `optional` marker (default required), `list of T`
   cardinality, exactly one `identity` field per entity, and typed
   `ref <Entity>` references; constraints limited to `min`/`max`/`pattern`/
   `length` with declared semantics.
2. The same plus user-defined enums and nested records in v0.1.
3. Open/extensible type strings validated by providers.

**Recommended option:** Adopt a new closed Option 4:

- scalar types are `string`, signed 64-bit `int`, `decimal`, `bool`, RFC 3339
  UTC `timestamp`, and `uuid`;
- decimal supports at most 28 significant digits and scale 0–28, matching the
  canonical Rust decimal substrate;
- `quantity<Unit>` is a decimal value whose unit is fixed by the field type and
  must resolve to an existing SEA `unit`; canonical values normalize through
  that unit's declared dimension and base-unit factor;
- a closed `enum` declaration has stable member identifiers and explicit wire
  strings. v0.1 exposes no unknown-member fallback, so adding, removing, or
  changing a member/wire value is breaking. A future version may classify
  addition as compatible only after it defines explicit unknown-member handling;
- fields are required unless marked `optional`; optional means absent or a
  value, never an untyped JSON `null`;
- required non-key entity fields of scalar, quantity, or enum type may declare
  a type-checked literal default; record, optional, ref, list, and key fields may
  not. An enum default is its explicit wire string;
- `list<T>` is the only collection. v0.1 forbids nested lists, nested records,
  maps, unions, and arbitrary type strings;
- `min`/`max` are inclusive and `exclusive_min`/`exclusive_max` are strict;
  all four apply only to numeric or quantity fields;
  `min_length`/`max_length` count Unicode scalar values after NFC and apply only
  to strings; `min_items`/`max_items` apply only to lists; `pattern` references
  an existing validated SEA `Pattern` and uses the repository's Rust regex
  dialect;
- each generated persisted entity declares exactly one aggregate-key field of
  type `uuid` or `string`. This is runtime domain identity and is distinct from
  `ConceptId`, which identifies the SEA declaration itself;
- identityless records may contain typed `ref<Entity>` fields. A reference's
  wire/storage type is exactly the target entity's aggregate-key type.

The closed algebra is sufficient for D9 and leaves extension points explicit.

**Trade-offs:** Flat records and a fixed type set limit v0.1 domains. Quantity
normalization adds language work but reuses existing unit/dimension meaning
instead of inventing provider-specific money semantics. Option 3 violates §5.4
because providers may map types but never define them.

**Human decision:** Accepted as amended (Option 4) by the DomainForge repository
maintainer on 2026-07-18.

## D6 — Effect, transaction, failure, idempotency, evidence, lifecycle

**Question:** What are the exact effect/state, transaction, canonical
failure, concurrency, idempotency, evidence, and lifecycle declarations,
including an explicit `not_applicable` representation?

**Why it matters:** These are the remaining §5.1 `generation_ready` items;
§5.4 requires idempotency/concurrency to be explicit SEA "or `not_applicable`
with validated reason".

**Repository evidence:** None of these exist in the grammar or Domain IR; the
only failure modeling is the per-policy `<Pascal>Violation` error
(`ir.rs:94-97`).

**Options considered:**
1. Closed enumerations declared on the operation: effect kind
   (`creates`/`mutates`/`reads`/`emits` + target state), transaction boundary
   (`single_aggregate` v0.1-only), named canonical failures with stable
   codes, idempotency/concurrency strategy from a named closed set or
   `not_applicable("reason")`, evidence obligations as named closed kinds,
   lifecycle only when externally visible.
2. Free-form strings validated by convention.
3. Defer some items (evidence, lifecycle) out of v0.1.

**Recommended option:** Option 1 with these exhaustive v0.1 values and
invariants:

| Concern | Accepted v0.1 values and rules |
|---|---|
| Effect | Exactly one of `creates <Entity>`, `mutates <Entity>`, or `reads <Entity>`. `emits` is deferred with messaging. The target must resolve to the operation's declared state entity. Creates construct same-named state fields from required input and explicit entity defaults; mutates preserve the key and replace same-named supplied fields; reads do not mutate. Required state or output fields without a same-named typed source make the operation not generation-ready. |
| Transaction | Writes use `single_aggregate`; reads use `read_only`. No multi-aggregate or provider-selected boundary exists. |
| Failure | A stable lower-snake-case code unique in the resolved closure and a plain-language meaning. Every validation, policy, missing-state, idempotency, and concurrency path maps to one declared code. Structured detail payloads are deferred; transport status is a later adapter mapping, not SEA meaning. |
| Idempotency | Harmful writes use `keyed_by <input-field>`. The field must be required and scalar. Same key plus the same canonical input fingerprint returns the stored result without another effect; same key plus a different fingerprint returns the declared idempotency-conflict failure. Reads use `inherent`. |
| Concurrency | Creates use `unique_key <input-field>` backed by a same-named compatible state field. Mutates use `optimistic_version <input-field>` backed by a same-named required state `int`; equality is required and successful mutation atomically increments it. Reads use one consistent `read_snapshot`. Idempotency lookup precedes create uniqueness. |
| Evidence | Every inbound v0.1 operation declares `operation_trace`, producing a structured record with semantic operation ID, outcome code, and correlation ID. No payload or secret is evidence by default. |
| Lifecycle | Every inbound v0.1 operation declares `synchronous_request_response`. Background, streaming, and externally managed lifecycles are unsupported. |
| Not applicable | The v0.1 first-class form is `idempotency not_applicable(read_only, "explanation")`, legal only on a `reads` operation and identical to `inherent` at runtime; its reason and explanation remain in the canonical contract for review. `no_shared_state` and `no_external_lifecycle` are deferred because v0.1 requires one state entity and synchronous inbound lifecycle; unreachable values MUST NOT enter the grammar. |

Multiple effects, implicit ordering, omitted conditionally relevant fields, and
free-form strategy names make the operation not generation-ready.

**Trade-offs:** The contract supports only one aggregate and synchronous local
HTTP behavior. Closed values require a language revision for new strategies;
that is preferable to allowing providers to invent consequential semantics.

**Human decision:** Accepted as amended (Option 1 with the exhaustive v0.1 table
above) by the DomainForge repository maintainer on 2026-07-18.

## D7 — Public schema version and Rust types

**Question:** What is the stable serialized schema version and which Rust
types are public (hence require Python/TypeScript/WASM parity)?

**Why it matters:** `ast_schema.rs` is a stable public surface; every public
addition triggers the language evolution policy and the plan's cross-binding
parity rule.

**Repository evidence:** `parser/ast_schema.rs` (serializable schema types),
`parse --ast --format json` emitting `{metadata, declarations}`, and the
three binding graph wrappers (`python/graph.rs`, `typescript/graph.rs`,
`wasm/graph.rs`).

**Options considered:**
1. New AST declaration variants join the existing serialized AST schema under
   its additive-variant policy, and the resolved `ApplicationContract` gets its
   own versioned public schema; both receive explicit cross-binding access.
2. Keep the resolved model private (Rust-only) in v0.1; only AST shapes are
   public.
3. Everything private behind the CLI.

**Recommended option:** Option 1 with this exact public-surface policy:

- additive declaration variants and optional entity fields remain in
  `ast-v3.schema.json`, matching the repository's existing additive AST policy;
- existing valid AST v3 documents remain valid, existing documents serialize
  as before, and no top-level discriminator is added to the current
  `{metadata, declarations}` JSON shape;
- the resolved contract uses schema ID
  `domainforge-application-contract/v1`, contains a required `schema_version`,
  and rejects unknown input fields during validation;
- Rust exposes typed `ApplicationContract` and constituent contract types plus
  canonical JSON serialization;
- Python, TypeScript, and WASM each expose `parse_to_ast_json(source)` and
  `resolve_application_contract_json(entry_logical_path, sources_json)`. The
  latter consumes an in-memory map of logical paths to SEA source, so import
  behavior is identical on native and WASM targets;
- v0.1 bindings return the stable contract JSON rather than duplicating a large
  mutable object hierarchy. Typed binding wrappers may be added later without
  changing the v1 JSON contract;
- any future non-additive AST JSON change requires an explicit ast-v4 migration
  decision rather than silently changing v3.

The review/approval workflow consumes the resolved model, so its schema is
public from Foundation conformance onward.

**Trade-offs:** Adding the missing TypeScript AST JSON path and a source-map
resolver API increases Milestone 0 work. JSON parity is smaller and more stable
than three handwritten mutable wrapper hierarchies.

**Human decision:** Accepted as amended (Option 1 with additive AST v3 and
`domainforge-application-contract/v1`) by the DomainForge repository maintainer
on 2026-07-18.

## D8 — Compatibility, keywords, formatter, modules, closure

**Question:** What are the backward-compatibility, reserved-keyword,
formatter/printer, module-resolution, and semantic-closure-contribution
rules?

**Why it matters:** §5.2 items 4–6 require formatter, envelope, and
resolution treatment; the language evolution policy requires an explicit
additive/breaking classification.

**Repository evidence:** `sea.pest:120` shows the existing reserved-word
alternation that any new keywords must join; `ImportDecl`/`ImportSpecifier`
exist in `ast_schema.rs:46-61`, so module resolution machinery already has a
schema surface to extend.

**Options considered:**
1. Additive contextual syntax: new declaration/body tokens are recognized only
   where their full grammar production can begin; existing identifier uses
   retain their parse and meaning. New declarations format canonically and
   round-trip, and imported contracts follow D3's resolution rules.
2. Breaking change with migration (needed only if D1 alters `entity_decl`
   in a way that changes existing parses).
3. Feature-flagged grammar.

**Recommended option:** Option 1 with these compatibility requirements:

- `record`, `operation`, `enum`, field/type tokens, and D6 strategy tokens are
  contextual. Do not reject them globally from every `identifier` position;
- where an existing rule needs declaration lookahead, including `resource_decl`,
  test the complete new declaration prefix instead of banning the bare word;
- an entity without a typed body produces exactly the same AST and canonical
  formatter output as before;
- every new token receives a collision fixture in every current identifier,
  name, namespace, unit, alias, and annotation-key position;
- the compatibility oracle is equality between formatter/AST output from the
  compiler before and after the change for the same existing corpus. It is not
  byte equality between authored source and canonical formatted source;
- named imports, wildcard aliases, exports, cycles, qualified identity,
  collisions, origins, and deterministic closure order follow D3 exactly;
- if any accepted syntax cannot satisfy those corpus and collision tests,
  ADR-013 must classify that token as breaking and supply `ConceptChange`,
  migration documentation, and compatibility tests before implementation.

**Trade-offs:** Contextual lookahead makes the grammar more deliberate and
requires a larger compatibility corpus. It avoids silently invalidating models
that already use ordinary words such as `operation` as identifiers. Feature
flags would fragment the language contract and remain rejected.

**Human decision:** Accepted as amended (Option 1 with contextual-token and
collision-proof requirements) by the DomainForge repository maintainer on
2026-07-18.

## D9 — Flagship domain

**Question:** Which flagship domain proves v0.1, with exact command, query,
policy/public-access decision, invalid/conflict path, persisted state,
rollback/restart expectations, projection matrix, deterministic outputs, and
expected outcome?

**Why it matters:** Milestone 0's stop gate requires two nontrivial SEA
fixtures expressing the v0.1 use cases without annotations or fixture-derived
types; every later proof (§18) runs against this domain.

**Repository evidence:** Future fixture paths are already fixed by the plan:
`fixtures/application_generation/flagship/command-write.sea` and
`fixtures/application_generation/flagship/query-read.sea` (to be created during
Milestone 0 now that ADR-013 is accepted).

**Options considered:**
1. Local order management: command `place_order`, governed by a deterministic
   local order-limit policy, creates an `Order`, is idempotent by client order
   key, and proves rollback/restart behavior; query `get_order_status` publicly
   reads only non-sensitive status by opaque order ID.
2. Task/todo management (simpler, but weak on policy and conflict paths).
3. Inventory reservation (strong concurrency story, harder v0.1 scope).

**Recommended option:** Option 1 with this preregistered contract.

### Flagship declarations

- role: `Customer`;
- unit/dimension: `USD` in `Currency`;
- enum: `OrderStatus` with wire member `placed`;
- entity `Order`: aggregate key `order_id: uuid`; required
  `client_order_id: string` (length 1–64), `total: quantity<USD>` (`min > 0`),
  `item_count: int` (`min 1`), `status: OrderStatus` with explicit default wire
  value `"placed"`;
- `PlaceOrderInput`: the same four input fields except `status`;
- `PlaceOrderOutput`: `order_id: uuid`, `status: OrderStatus`;
- `GetOrderStatusInput`: `order_id: uuid`;
- `GetOrderStatusOutput`: `order_id: uuid`, `status: OrderStatus`.

### Operation acceptance matrix

| §5.1 concern | `place_order` | `get_order_status` |
|---|---|---|
| Stable ID / intent | `place_order`: persist one valid order exactly once | `get_order_status`: return the current non-sensitive status for one opaque ID |
| Direction / actor / access | inbound; `Customer`; `policy_governed` | inbound; anonymous public caller; `public` |
| Input / output | `PlaceOrderInput` / `PlaceOrderOutput` | `GetOrderStatusInput` / `GetOrderStatusOutput` |
| State / effect | `Order`; `creates Order` | `Order`; `reads Order` |
| Policy / point | `order_total_within_limit` at `precondition`; total MUST be at most `10000 USD`; failure `order_limit_exceeded` | none; explicitly public and unguarded |
| Transaction | `single_aggregate` | `read_only` |
| Failures | `invalid_order`, `order_limit_exceeded`, `idempotency_conflict` | `order_not_found` |
| Idempotency | `keyed_by client_order_id` | `inherent` |
| Concurrency | `unique_key client_order_id` | `read_snapshot` |
| Evidence | `operation_trace` | `operation_trace` |
| Lifecycle | `synchronous_request_response` | `synchronous_request_response` |

The public query returns only opaque ID and status; it exposes no customer,
payment, address, or line-item data. Payment authorization, authentication, and
external authority/evidence are explicitly outside this fixture and v0.1.

### Exact behavioral vectors

Use this fixed valid command input:

```json
{"order_id":"00000000-0000-4000-8000-000000000001","client_order_id":"client-order-001","total":"125.00","item_count":2}
```

Its canonical success output is:

```json
{"order_id":"00000000-0000-4000-8000-000000000001","status":"placed"}
```

After success, the logical persisted row is exactly the four normalized input
values plus `status = placed` and the canonical input fingerprint. Repeating
the same request returns the same output and creates no second row. Reusing
`client-order-001` with any different canonical input returns
`idempotency_conflict` and leaves the row unchanged.

A nonpositive total or `item_count < 1` returns `invalid_order`; a total above
`10000 USD` returns `order_limit_exceeded`. Each failure creates no row and
leaves existing rows unchanged. After process restart, querying the fixed order
ID returns the same success output. Querying an absent ID returns
`order_not_found` without mutation.

### Projection and determinism matrix

| Projection/review artifact | Flagship result |
|---|---|
| Domain review, policy-to-operation table, semantic/realization diff | required |
| RDF semantic graph | required for graph-supported declarations; application-only declarations appear through their stable semantic refs until Graph support is accepted |
| Authority/evidence contract | required for `operation_trace`; no external authority contract |
| OpenTelemetry semantic conventions | required because both operations declare runtime observation |
| Cell environment | `not_applicable(no_cell_declared, "flagship has no Cell")` |
| BPMN | optional review aid |
| CMMN | `not_applicable(no_case_lifecycle, "synchronous order slice")` |
| ArchiMate | optional review aid |

Canonical AST, `ApplicationContract`, review bundle, semantic envelope, and
generated source must be byte-identical for the fixed logical source set,
compiler, profile, providers, and `--created-at`. Runtime trace IDs and timing
remain observed evidence and are excluded from deterministic artifact equality.

**Trade-offs:** The fixture proves meaningful policy, validation, idempotency,
conflict, rollback, persistence, restart, and query behavior, but deliberately
does not claim payment authorization or authentication. Those require later
authority/provider conformance.

**Human decision:** Accepted as amended (Option 1 with the preregistered local
contract above) by the DomainForge repository maintainer on 2026-07-18.

## D10 — Canonicalization boundary

**Question:** Does typed envelope normalization reuse the semantic-pack
canonical JSON primitive or stand separate, and how are existing pack
hashes/signatures protected?

**Why it matters:** Semantic packs are already hashed and signed; any change
to their canonical form invalidates existing artifacts. The application
contract needs its own `source_set_hash`/`semantic_closure_hash` (spec §10.1).

**Repository evidence:** `semantic_pack/canonical_json.rs:7` (`canonical_json`)
is the existing primitive; `semantic_pack/diagnostics.rs:82` shows pack-level
canonical-name invariants already enforced against it.

**Options considered:**
1. Separate typed normalization for the application envelope (may call
   `canonical_json` internally as a byte-serialization helper), producing new
   hash fields that never feed into pack hashes; compatibility tests assert
   existing pack fixtures' hashes/signatures are byte-identical before and
   after Milestone 0.
2. Extend the pack canonical form to embed application contracts (versioned,
   compatibility-preserving), changing pack content hashes for models that
   use the new declarations.
3. One shared canonicalizer for both, refactored now.

**Recommended option:** Option 1. It is the only option whose blast radius on
existing signed packs is provably zero, which is what the plan's constraint
("unless D10 explicitly approves a versioned compatibility-preserving
alternative") asks the human to protect.

**Trade-offs:** Two normalization paths must be kept consistent by tests;
Option 2 is more unified but forces a pack schema version bump and
re-signing story in Milestone 0, coupling the language gate to pack
infrastructure.

**Human decision:** Accepted as recommended (Option 1) by the DomainForge
repository maintainer on 2026-07-18.

## Coherence statement

The accepted set (D1 Option 5, D2 Option 1, D3 Option 2 amended, D4 Option 1
amended, D5 Option 4, D6 Option 1 amended, D7 Option 1 amended, D8 Option 1
amended, D9 Option 1 amended, D10 Option 1) is internally consistent:

- D1 separates typed persisted state from identityless operation records;
- D2 operations reference those records and D3 resolves all references once;
- D4's policy context uses D5 types and D6 failure/effect rules;
- D9 uses only the closed v0.1 values accepted in D4–D6;
- D7 publishes the D3 model without breaking additive AST v3 consumers;
- D8 preserves existing parses through contextual syntax and collision tests;
- D10 canonicalizes the D3/D7 application envelope without changing semantic
  pack hashes or signatures.

ADR-013 may choose exact surface spelling and Rust field names only where those
choices do not alter an accepted semantic rule above. A semantic change requires
returning to Human Gate A.

## Gate outcome and next action

1. Human Gate A is closed: D1–D10 and the D3/D5/D6 amendments were accepted by
   the DomainForge repository maintainer on 2026-07-18 with no change to the
   Proposed v0.2 release boundary.
2. Human Gate B is closed: ADR-013 and its normative reference were ratified on
   2026-07-18.
3. Task 3 writes the Milestone 0 language code plan before compiler work starts.
