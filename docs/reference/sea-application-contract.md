# SEA Application Contract — normative reference (v0.1, Proposed)

Companion to `docs/specs/ADR-013-sea-application-contract.md` (Status:
Proposed). Nothing in this document is implemented until ADR-013 is Accepted;
all grammar, types, and fixtures below are the exact proposal. Accepted D1–D10
decisions are recorded in
`.agents/reports/2026-07-18-application-contract-decision-packet.md`; this
reference chooses only surface spelling and Rust field names within those
decisions.

Normative keywords MUST, MUST NOT, and MAY follow RFC 2119.

## 1. Terminology and v0.1 scope

- **Application Contract** — the resolved, typed model of entities, records,
  enums, and operations that decides `generation_ready` (spec §5.1).
- **Entity (typed)** — an existing `entity` declaration extended with a typed
  state body containing exactly one aggregate-key field; owns domain identity
  and persisted state.
- **Record** — a new first-class, identityless, flat declaration for operation
  input, output, failure-detail, and value data. Operation input and output
  MUST be records, never entities.
- **Enum** — a new closed declaration with stable member identifiers and
  explicit wire strings.
- **Operation** — a new first-class declaration carrying the complete §5.1
  machine contract. Flows remain process narrative; an operation MUST NOT be
  derived from a flow, and v0.1 defines no operation-to-flow reference.
- **ResolvedModuleSet** — the single resolver output (module IDs, source
  hashes, import graph, visibility, alias bindings, qualified symbol IDs,
  origins, one symbol table) from which both Graph and the
  `ApplicationContract` are built.
- **ApplicationSymbolId** — stable ID for application-only declarations,
  derived from normalized namespace, declaration kind, and name. Existing SEA
  concepts keep `ConceptId`.

v0.1 scope: one aggregate per operation, synchronous local request/response,
`precondition` policy enforcement only, `public` or `policy_governed` access.
Out of scope: see §12.

## 2. Proposed grammar (Pest)

Additions to `domainforge-core/grammar/sea.pest`. `identifier`, `name`,
`string_literal`, `number`, `version`, `in_keyword`, and `WHITESPACE` are the
existing rules.

`declaration_inner` gains three alternatives, inserted after `unit_decl`:

```pest
declaration_inner = { dimension_decl | unit_decl | record_decl | enum_decl |
    operation_decl | entity_decl | resource_decl | flow_decl | pattern_decl |
    role_decl | relation_decl | instance_decl | policy_decl |
    concept_change_decl | metric_decl | mapping_decl | projection_decl |
    cell_decl | system_dependency_decl | runtime_decl | tool_decl |
    dependency_set_decl | service_decl | mount_decl | endpoint_decl |
    network_flow_decl | credential_decl }
```

`entity_decl` gains one optional trailing body (all existing forms unchanged):

```pest
entity_decl = {
    ^"entity" ~ name ~ (^"v" ~ version)? ~ entity_annotation* ~
    (in_keyword ~ identifier)? ~ entity_body?
}
entity_body = { "{" ~ entity_field* ~ "}" }
entity_field = { key_marker? ~ field_decl }
key_marker = { ^"key" }
```

New declarations and shared field/type rules:

```pest
record_decl = { ^"record" ~ identifier ~ record_body }
record_body = { "{" ~ field_decl* ~ "}" }

enum_decl = { ^"enum" ~ identifier ~ "{" ~ enum_member ~ ("," ~ enum_member)* ~ ","? ~ "}" }
enum_member = { identifier ~ "=" ~ string_literal }

field_decl = { identifier ~ ":" ~ field_type ~ optional_marker? ~ field_constraints? }
optional_marker = { ^"optional" }

field_type = { list_type | quantity_type | ref_type | scalar_type | named_type }
scalar_type = @{ (^"string" | ^"int" | ^"decimal" | ^"bool" | ^"timestamp" | ^"uuid") ~ !(LETTER | NUMBER | "_") }
quantity_type = { ^"quantity" ~ "<" ~ identifier ~ ">" }
ref_type = { ^"ref" ~ "<" ~ identifier ~ ">" }
list_type = { ^"list" ~ "<" ~ element_type ~ ">" }
element_type = { quantity_type | ref_type | scalar_type | named_type }
named_type = { identifier }

field_constraints = { "(" ~ field_constraint ~ ("," ~ field_constraint)* ~ ")" }
field_constraint = {
    (^"min_length" ~ number) | (^"max_length" ~ number) |
    (^"min_items" ~ number) | (^"max_items" ~ number) |
    (^"min" ~ number) | (^"max" ~ number) |
    (^"pattern" ~ string_literal)
}

operation_decl = { ^"operation" ~ identifier ~ "{" ~ operation_body ~ "}" }
operation_body = {
    intent_clause ~ direction_clause ~ actor_clause ~ access_clause ~
    input_clause ~ output_clause ~ state_clause ~ effect_clause ~
    transaction_clause ~ failure_clause+ ~ idempotency_clause ~
    concurrency_clause ~ evidence_clause ~ lifecycle_clause
}
intent_clause = { ^"intent" ~ string_literal }
direction_clause = { ^"direction" ~ direction_kind }
direction_kind = { ^"inbound" | ^"outbound" | ^"internal" }
actor_clause = { ^"actor" ~ (^"anonymous" | identifier) }
access_clause = { ^"access" ~ (policy_governed_clause | ^"public") }
policy_governed_clause = { ^"policy_governed" ~ ^"by" ~ policy_binding ~ ("," ~ policy_binding)* }
policy_binding = { identifier ~ ^"at" ~ enforcement_point ~ ^"fails" ~ ^"with" ~ identifier }
enforcement_point = { ^"precondition" | ^"invariant" | ^"postcondition" }
input_clause = { ^"input" ~ identifier }
output_clause = { ^"output" ~ identifier }
state_clause = { ^"state" ~ identifier }
effect_clause = { ^"effect" ~ effect_kind ~ identifier }
effect_kind = { ^"creates" | ^"mutates" | ^"reads" }
transaction_clause = { ^"transaction" ~ (^"single_aggregate" | ^"read_only") }
failure_clause = { ^"failure" ~ identifier ~ string_literal ~ (^"details" ~ identifier)? }
idempotency_clause = { ^"idempotency" ~ (idempotency_keyed | ^"inherent" | not_applicable_value) }
idempotency_keyed = { ^"keyed_by" ~ identifier }
concurrency_clause = { ^"concurrency" ~ (concurrency_unique | concurrency_optimistic | ^"read_snapshot" | not_applicable_value) }
concurrency_unique = { ^"unique_key" ~ identifier }
concurrency_optimistic = { ^"optimistic_version" ~ identifier }
evidence_clause = { ^"evidence" ~ ^"operation_trace" }
lifecycle_clause = { ^"lifecycle" ~ ^"synchronous_request_response" }
not_applicable_value = { ^"not_applicable" ~ "(" ~ na_reason ~ "," ~ string_literal ~ ")" }
na_reason = { ^"read_only" | ^"no_shared_state" | ^"no_external_lifecycle" }

application_decl_prefix = { (^"record" | ^"enum" | ^"operation") ~ identifier ~ "{" }
```

`resource_decl` changes only its units-branch lookahead so that the bare words
remain valid identifiers (D8 contextual-token rule):

```pest
resource_decl = {
    ^"resource" ~ name ~ resource_annotation* ~ (
        (identifier ~ in_keyword ~ identifier) |
        (in_keyword ~ identifier) |
        ( !(declaration_keyword | application_decl_prefix) ~ identifier )
    )?
}
```

`record`, `enum`, and `operation` MUST NOT be added to `declaration_keyword`.
Operation clauses appear in exactly the order given; the formatter emits that
order canonically. `enforcement_point` parses `invariant` and `postcondition`
so they are reserved words in this position, but validation rejects them
(APP007) because v0.1 supports `precondition` only.

Reference-by-identifier rule: `actor`, `ref<...>`, `quantity<...>`, `state`,
and `effect` targets are identifiers resolved by exact text equality against
the declared name of the target role, entity, or unit. A quoted-name
declaration referenced this way MUST have an identifier-shaped name
(`(LETTER | "_") (LETTER | NUMBER | "_")*`); otherwise APP002.

## 3. Proposed SEA for the D9 flagship

The complete proposed sources appear in §11 as the two proving fixtures. The
command operation is `place_order` (creates `Order`, policy-governed,
idempotent by `client_order_id`); the query is `get_order_status` (public,
reads `Order`).

## 4. Field-by-field semantics of `generation_ready`

An operation is `generation_ready` only when every row below is satisfied
after resolution and validation. Every value set is closed; anything outside
it makes the operation not generation-ready.

| Item | Clause | Semantics |
|---|---|---|
| Stable identity | `operation <id>` | `ApplicationSymbolId` from normalized namespace + `operation` + name; unique in the closure. |
| Intent | `intent "<text>"` | Nonempty plain-language purpose; carried verbatim into review artifacts. |
| Direction | `direction` | `inbound` only in v0.1; `outbound`/`internal` parse but fail APP001 as unsupported-in-v0.1. |
| Actor / access | `actor`, `access` | `anonymous` is legal only with `access public`. A named actor MUST resolve to a `role` and is required with `policy_governed`. `authenticated` does not exist in v0.1. |
| Input / output | `input`, `output` | Each MUST resolve to a `record` in the closure; entities are rejected (APP006). |
| State / effect | `state`, `effect` | `state` MUST resolve to an entity with a typed body; the effect target MUST be that same entity (APP011). Exactly one effect kind. |
| Transaction | `transaction` | `creates`/`mutates` require `single_aggregate`; `reads` requires `read_only` (APP001 on mismatch). |
| Failures | `failure` | One or more; stable lower-snake-case code unique in the closure, nonempty meaning, optional flat `details` record. Every validation, policy, missing-state, idempotency, and concurrency path maps to one declared code. Transport status is adapter mapping, not SEA meaning. |
| Idempotency | `idempotency` | Writes: `keyed_by <input-field>` where the field is required and scalar. Same key + same canonical input fingerprint returns the stored result with no new effect; same key + different fingerprint returns the declared idempotency-conflict failure. Reads: `inherent`. |
| Concurrency | `concurrency` | `creates` → `unique_key <input-field>`; `mutates` → `optimistic_version <input-field>` (required `int` field); `reads` → `read_snapshot`. Referenced fields MUST be required scalars of the stated type. |
| Evidence | `evidence operation_trace` | Mandatory on every inbound v0.1 operation; produces a structured record with semantic operation ID, outcome code, and correlation ID. No payload or secret is evidence by default. |
| Lifecycle | `lifecycle synchronous_request_response` | Mandatory on every inbound v0.1 operation; background, streaming, and externally managed lifecycles are unsupported. |
| Not applicable | `not_applicable(<reason>, "<text>")` | Legal only where effect and direction prove the reason: `read_only` only on `reads`; `no_shared_state` only when the operation touches no persisted entity; `no_external_lifecycle` only on non-inbound direction. A nonempty explanation alone is insufficient (APP010). |

Policy bindings (`policy_governed by <policy> at precondition fails with
<code>`): each referenced policy MUST exist, MUST be a constraint policy whose
expression evaluates from the typed context (operation ID, actor role,
normalized input record, pre-operation aggregate state when one exists), and
each binding names one declared failure code. A bare identifier in a policy
expression resolves to the input-record field of that name; comparing a
quantity field to a bare number compares in the field's declared unit.
Missing facts, `UNKNOWN`, evaluator errors, and false obligations fail closed
to the bound failure code. A policy no operation references remains valid.

## 5. Validation invariants and diagnostic codes

Diagnostics are stable and machine-readable: `APPNNN code_slug`.

| Code | Slug | Invariant violated |
|---|---|---|
| APP001 | missing_application_semantics | Any §4 consequential item absent, unsupported-in-v0.1, or inconsistent (direction, transaction pairing, missing mandatory clause). Message lists operation/source ref, each missing field, why it matters, and the smallest supported remediation. |
| APP002 | unknown_type_reference | `named_type`, `ref`, `quantity` unit, `input`, `output`, `state`, `actor`, or `details` does not resolve in the symbol table, or the target's quoted name is not identifier-shaped. |
| APP003 | invalid_record_shape | Record/entity field uses a nested record, nested list, map-like, union-like, or otherwise non-closed type. |
| APP004 | duplicate_field | Duplicate field name inside one record or entity body. |
| APP005 | invalid_aggregate_key | Typed entity body has zero or multiple `key` fields, or the key type is not `uuid`/`string`, or a key field is `optional`. |
| APP006 | operation_type_mismatch | Operation `input`/`output` resolves to an entity or enum instead of a record, or a `details` target is not a record. |
| APP007 | policy_scope_error | Dangling policy reference, non-constraint policy bound, expression not evaluable from the typed context, or enforcement point `invariant`/`postcondition`. |
| APP008 | invalid_failure_declaration | Failure code not lower-snake-case, duplicate in the resolved closure, or a mapped failure path (validation, policy, missing state, idempotency, concurrency) has no declared code. |
| APP009 | invalid_strategy_reference | `keyed_by`/`unique_key`/`optimistic_version` field missing, optional, non-scalar, or of the wrong type; strategy/effect pairing outside the §4 table. |
| APP010 | invalid_not_applicable | `not_applicable` reason not proven by the operation's effect and direction. |
| APP011 | effect_state_mismatch | Effect target differs from the declared state entity, or the state entity lacks a typed body. |
| APP012 | constraint_error | Constraint applied to an incompatible type (`min`/`max` on non-numeric/non-quantity, `min_length`/`max_length` on non-string, `min_items`/`max_items` on non-list), `min` greater than `max`, negative length/item bound, or `pattern` not resolving to a declared SEA `Pattern`. |
| APP013 | enum_error | Duplicate enum member identifier or duplicate wire string within one enum. |
| APP014 | closure_symbol_collision | Two non-equivalent declarations share one normalized qualified ID; also import cycles and unresolved aliases surfaced through resolution. |
| APP015 | contract_schema_violation | Contract JSON fails `domainforge-application-contract/v1` validation, including any unknown field. |

All diagnostics carry the source module logical ID, byte span, and the stable
semantic ID of the offending declaration.

## 6. Canonicalization rules

- **Closure order (sets):** modules by logical module ID (bytewise ascending),
  then declarations by kind in the fixed order `enum`, `record`, `entity`,
  `operation`, then by stable semantic ID (bytewise ascending).
- **Sequences:** authored order is preserved only for record/entity field
  lists, enum member lists, operation failure lists, and policy-binding
  lists — these are declared sequences.
- **Defaults:** canonical JSON serializes every field explicitly; `optional`
  serializes as `"optional": false` when unmarked. Absent optional values are
  omitted from canonical value fingerprints, never encoded as JSON `null`.
- **Scalars:** strings are NFC-normalized; `int` is a signed 64-bit integer;
  `decimal` canonical form is a string with no exponent, no leading zeros, no
  trailing fractional zeros, and `"0"` for zero (so an authored `"125.00"`
  fingerprints as `"125"`); `timestamp` is RFC 3339 UTC with a trailing `Z`;
  `uuid` is lowercase hyphenated; `bool` is `true`/`false`.
- **Quantities:** normalized through the referenced unit's declared dimension
  and base-unit factor before fingerprinting; the canonical value is the
  base-unit decimal in canonical decimal form plus the base unit symbol.
- **Hashes:** `source_set_hash` is SHA-256 over the closure-ordered sequence
  of (logical module ID, source bytes) pairs; `semantic_closure_hash` is
  SHA-256 over the canonical JSON bytes of the resolved `ApplicationContract`
  excluding both hash fields. Byte serialization MAY reuse
  `semantic_pack::canonical_json` internally, but no application hash feeds
  into semantic-pack content hashes, and existing pack fixtures' hashes and
  signatures MUST remain byte-identical before and after Milestone 0 (D10).
- **Canonical input fingerprint** (idempotency): SHA-256 over the canonical
  JSON of the normalized input record, fields ordered by declared sequence.

## 7. Import, namespace, identity, and duplicate semantics

Exactly the D3 pipeline: `ResolvedModuleSet` is the only resolver output.
Named imports bind only the named exported symbols; wildcard imports remain
alias-qualified; unexported imported declarations are not visible. Import
cycles, unresolved aliases, and two non-equivalent declarations with the same
normalized qualified ID are errors (APP014); equivalent duplicate imports
collapse to one symbol. The namespace is the file-header `@namespace` value,
normalized to NFC lowercase; the qualified ID is
`<namespace>.<kind>.<declared-name-text>`. Application declarations are
exportable and importable with the existing `export` and `import` syntax
unchanged. Graph and `ApplicationContract` both consume the symbol table and
perform no independent name or import resolution.

## 8. Proposed internal Rust types

New module `domainforge-core/src/application/` (all types
`#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`):

```rust
pub struct ApplicationContract {
    pub schema_version: String,            // "domainforge-application-contract/v1"
    pub source_set_hash: String,           // hex SHA-256
    pub semantic_closure_hash: String,     // hex SHA-256
    pub enums: Vec<EnumContract>,
    pub records: Vec<RecordContract>,
    pub entities: Vec<EntityContract>,
    pub operations: Vec<OperationContract>,
}

pub struct ApplicationSymbolId(pub String); // "<ns>.<kind>.<name>"

pub enum ScalarType { String, Int, Decimal, Bool, Timestamp, Uuid }

pub enum FieldType {
    Scalar(ScalarType),
    Quantity { unit: String },
    EntityRef { entity: ApplicationSymbolId },
    Enum { r#enum: ApplicationSymbolId },
    List { element: Box<FieldType> },      // element is never List
}

pub enum FieldConstraint {
    Min(rust_decimal::Decimal),
    Max(rust_decimal::Decimal),
    MinLength(u32),
    MaxLength(u32),
    MinItems(u32),
    MaxItems(u32),
    Pattern { pattern: String },           // declared SEA Pattern name
}

pub struct FieldContract {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub constraints: Vec<FieldConstraint>,
}

pub struct EnumMember { pub name: String, pub wire: String }
pub struct EnumContract { pub id: ApplicationSymbolId, pub name: String, pub members: Vec<EnumMember> }
pub struct RecordContract { pub id: ApplicationSymbolId, pub name: String, pub fields: Vec<FieldContract> }
pub struct EntityContract {
    pub id: ApplicationSymbolId,
    pub name: String,
    pub concept_id: String,                // language-level ConceptId of the entity
    pub key_field: String,
    pub fields: Vec<FieldContract>,
}

pub enum Direction { Inbound, Outbound, Internal }
pub enum EnforcementPoint { Precondition }
pub struct PolicyBinding {
    pub policy_concept_id: String,
    pub enforcement_point: EnforcementPoint,
    pub failure_code: String,
}
pub enum AccessMode { Public, PolicyGoverned { bindings: Vec<PolicyBinding> } }
pub enum EffectKind { Creates, Mutates, Reads }
pub enum TransactionBoundary { SingleAggregate, ReadOnly }
pub struct FailureContract {
    pub code: String,
    pub meaning: String,
    pub details: Option<ApplicationSymbolId>,
}
pub enum NaReason { ReadOnly, NoSharedState, NoExternalLifecycle }
pub struct NotApplicable { pub reason: NaReason, pub explanation: String }
pub enum IdempotencyStrategy { KeyedBy { field: String }, Inherent, NotApplicable(NotApplicable) }
pub enum ConcurrencyStrategy {
    UniqueKey { field: String },
    OptimisticVersion { field: String },
    ReadSnapshot,
    NotApplicable(NotApplicable),
}
pub enum EvidenceKind { OperationTrace }
pub enum LifecycleKind { SynchronousRequestResponse }

pub struct OperationContract {
    pub id: ApplicationSymbolId,
    pub name: String,
    pub intent: String,
    pub direction: Direction,
    pub actor: Option<String>,             // None encodes anonymous
    pub access: AccessMode,
    pub input: ApplicationSymbolId,
    pub output: ApplicationSymbolId,
    pub state: ApplicationSymbolId,
    pub effect: EffectKind,
    pub transaction: TransactionBoundary,
    pub failures: Vec<FailureContract>,
    pub idempotency: IdempotencyStrategy,
    pub concurrency: ConcurrencyStrategy,
    pub evidence: EvidenceKind,
    pub lifecycle: LifecycleKind,
}
```

Resolver output (in `domainforge-core/src/module/resolver.rs`):

```rust
pub struct ResolvedModuleSet {
    pub modules: Vec<ResolvedModule>,      // closure order
    pub import_graph: Vec<(String, String)>, // (importer, imported) logical IDs
    pub symbols: Vec<ResolvedSymbol>,      // the single symbol table
}
pub struct ResolvedModule {
    pub logical_id: String,
    pub source_hash: String,               // hex SHA-256 of source bytes
}
pub struct ResolvedSymbol {
    pub qualified_id: String,
    pub kind: String,
    pub exported: bool,
    pub origin_module: String,
    pub aliases: Vec<String>,
}
```

AST additions in `domainforge-core/src/parser/ast.rs`: declaration variants
`Record(RecordDecl)`, `Enum(EnumDecl)`, `Operation(OperationDecl)` and an
`Option<EntityBody>` field on the existing entity node, each mirroring the §2
grammar one-to-one, with additive serialized twins in
`domainforge-core/src/parser/ast_schema.rs` and
`schemas/ast-v3.schema.json`.

## 9. Public boundary and binding obligations

Public surfaces (D7, accepted):

- Serialized AST v3 with the additive declaration variants; existing valid
  documents remain valid and serialize unchanged; no top-level discriminator
  is added to `{metadata, declarations}`; any non-additive change requires an
  explicit ast-v4 decision.
- `ApplicationContract` canonical JSON under schema ID
  `domainforge-application-contract/v1`
  (`schemas/application-contract-v1.schema.json`), with required
  `schema_version` and unknown-field rejection (APP015).
- Rust: `ApplicationContract` and every §8 constituent type, plus canonical
  JSON serialization, are public API of `domainforge-core`.
- Python, TypeScript, and WASM bindings each expose exactly:
  `parse_to_ast_json(source: str) -> str` and
  `resolve_application_contract_json(entry_logical_path: str, sources_json: str) -> str`,
  where `sources_json` is a JSON object mapping logical paths to SEA source,
  so import behavior is identical on native and WASM targets. v0.1 bindings
  return the stable contract JSON; typed wrappers MAY be added later without
  changing the v1 JSON contract.

`ResolvedModuleSet` and the validation internals remain private Rust in v0.1.

## 10. Backward compatibility and formatter rules

- All new tokens are contextual; none joins `declaration_keyword`. Lookahead
  extension is confined to `resource_decl` via `application_decl_prefix`.
- An entity without a typed body produces exactly the same AST and canonical
  formatter output as before this change.
- The compatibility oracle is equality of formatter and AST output from the
  compiler before and after the change over the existing corpus — not byte
  equality between authored and formatted source.
- Every new token receives a collision fixture in every current identifier,
  name, namespace, unit, alias, and annotation-key position
  (`fixtures/application_generation/compat/keyword-collision.sea`).
- Canonical formatting of new declarations: four-space indentation inside
  bodies; one field or clause per line; clause order exactly as §2; a single
  space around `:`, `=`, and between tokens; enum members separated by
  trailing-comma-free newline-joined `,`; formatted output MUST re-parse to an
  identical AST (round-trip law).
- If any token cannot satisfy the corpus and collision tests, it is
  reclassified breaking per the ADR-013 Migration path contingency before
  implementation proceeds.

## 11. Proving fixtures (fenced; not created until ADR-013 is Accepted)

### 11.1 `fixtures/application_generation/flagship/command-write.sea`

```sea
@namespace "flagship.orders"
@version "0.1.0"

dimension "Currency"
unit "USD" of "Currency" factor 1 base "USD"

role "Customer"

policy order_total_within_limit per Constraint Obligation priority 5 as: total <= 10000

export enum OrderStatus {
    placed = "placed"
}

export entity "Order" {
    key order_id: uuid
    client_order_id: string (min_length 1, max_length 64)
    total: quantity<USD> (min 0.01)
    item_count: int (min 1)
    status: OrderStatus
}

record PlaceOrderInput {
    order_id: uuid
    client_order_id: string (min_length 1, max_length 64)
    total: quantity<USD> (min 0.01)
    item_count: int (min 1)
}

record PlaceOrderOutput {
    order_id: uuid
    status: OrderStatus
}

operation place_order {
    intent "persist one valid order exactly once"
    direction inbound
    actor Customer
    access policy_governed by order_total_within_limit at precondition fails with order_limit_exceeded
    input PlaceOrderInput
    output PlaceOrderOutput
    state Order
    effect creates Order
    transaction single_aggregate
    failure invalid_order "input failed record validation"
    failure order_limit_exceeded "order total exceeds 10000 USD"
    failure idempotency_conflict "client_order_id reused with different canonical input"
    idempotency keyed_by client_order_id
    concurrency unique_key client_order_id
    evidence operation_trace
    lifecycle synchronous_request_response
}
```

D9 requires a strictly positive total; v0.1 `min` is inclusive, so the
flagship pins positivity as `min 0.01` in USD. The D9 limit ("at most 10000
USD") is the policy expression `total <= 10000`, evaluated in the field's
declared unit.

### 11.2 `fixtures/application_generation/flagship/query-read.sea`

```sea
@namespace "flagship.orders"
@version "0.1.0"

import { Order, OrderStatus } from "./command-write.sea"

record GetOrderStatusInput {
    order_id: uuid
}

record GetOrderStatusOutput {
    order_id: uuid
    status: OrderStatus
}

operation get_order_status {
    intent "return the current non-sensitive status for one opaque ID"
    direction inbound
    actor anonymous
    access public
    input GetOrderStatusInput
    output GetOrderStatusOutput
    state Order
    effect reads Order
    transaction read_only
    failure order_not_found "no order exists for the given order_id"
    idempotency inherent
    concurrency read_snapshot
    evidence operation_trace
    lifecycle synchronous_request_response
}
```

Behavioral vectors, persisted-row expectations, restart semantics, and the
projection/determinism matrix for these fixtures are fixed verbatim by packet
D9 and are binding on the Milestone 0 tests.

## 12. Out of scope for v0.1

- `authenticated` access, authentication, and external authority contracts;
- `invariant` and `postcondition` enforcement points (reserved, rejected);
- `outbound`/`internal` operations, `emits` effects, and messaging;
- multi-aggregate or provider-selected transaction boundaries;
- nested records, nested lists, maps, unions, and open type strings;
- background, streaming, and externally managed lifecycles;
- payment authorization in the flagship domain;
- typed mutable binding wrapper hierarchies (JSON parity only);
- embedding application contracts into semantic-pack canonical form or
  changing pack hashes/signatures in any way.

## 13. Traceability matrix (spec §5.1 → contract)

Fixture line numbers count from line 1 of each fenced fixture in §11.

| Requirement | SEA syntax | AST/Rust field | Validation rule | APP diagnostic | D9 fixture line |
|---|---|---|---|---|---|
| Stable operation identity and intent | `operation place_order { intent "..." }` | `OperationContract.id`, `.name`, `.intent` | ID unique in closure; intent nonempty | APP001, APP014 | command-write.sea 35–36 |
| Direction | `direction inbound` | `OperationContract.direction` | `inbound` only in v0.1 | APP001 | command-write.sea 37 |
| Actor/role and access mode | `actor Customer` / `actor anonymous`; `access policy_governed by ...` / `access public` | `OperationContract.actor`, `.access` | anonymous only with public; named actor resolves to a role; policy_governed requires named actor | APP001, APP002 | command-write.sea 38–39; query-read.sea 18–19 |
| Typed input/output records, fields, constraints, cardinality | `record` declarations; `input PlaceOrderInput` / `output PlaceOrderOutput` | `RecordContract.fields`, `FieldContract`, `FieldType`, `FieldConstraint`; `OperationContract.input`, `.output` | input/output resolve to records; flat closed types; constraint/type pairing | APP002, APP003, APP004, APP006, APP012 | command-write.sea 23–33, 40–41 |
| Affected aggregate/state and effect kind | `state Order`, `effect creates Order` | `OperationContract.state`, `.effect`; `EntityContract.key_field` | state has typed body and one key; effect target equals state | APP005, APP011 | command-write.sea 15–21, 42–43 |
| Applicable policies and enforcement point | `policy_governed by order_total_within_limit at precondition fails with order_limit_exceeded` | `AccessMode::PolicyGoverned`, `PolicyBinding` | constraint policy evaluable from typed context; precondition only; fail closed to bound code | APP007 | command-write.sea 39 |
| Transaction boundary | `transaction single_aggregate` / `transaction read_only` | `OperationContract.transaction` | pairing fixed by effect kind | APP001 | command-write.sea 44; query-read.sea 24 |
| Expected canonical failures | `failure <code> "<meaning>"` | `OperationContract.failures`, `FailureContract` | lower-snake-case codes unique in closure; every failure path mapped | APP008 | command-write.sea 45–47; query-read.sea 25 |
| Idempotency/concurrency behavior | `idempotency keyed_by client_order_id`, `concurrency unique_key client_order_id` / `inherent`, `read_snapshot` | `OperationContract.idempotency`, `.concurrency`; `NotApplicable` | strategy/effect pairing; referenced fields required scalars; not_applicable proven by effect/direction | APP009, APP010 | command-write.sea 48–49; query-read.sea 26–27 |
| Evidence/observation obligations | `evidence operation_trace` | `OperationContract.evidence` | mandatory on inbound v0.1 operations | APP001 | command-write.sea 50; query-read.sea 28 |
| Externally visible lifecycle | `lifecycle synchronous_request_response` | `OperationContract.lifecycle` | mandatory on inbound v0.1 operations | APP001 | command-write.sea 51; query-read.sea 29 |
