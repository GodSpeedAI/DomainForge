# SEA Application Contract — normative reference (v0.1, Accepted)

Companion to `docs/specs/ADR-013-sea-application-contract.md` (Status:
Accepted). Acceptance authorizes Milestone 0 implementation; it does not claim
the grammar, types, or fixtures below already exist. Accepted D1–D10
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
  input, output, and value data. Operation input and output
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

field_decl = { identifier ~ ":" ~ field_type ~ optional_marker? ~ field_constraints? ~ field_default? }
optional_marker = { ^"optional" }
field_default = { ^"default" ~ literal }

field_type = { list_type | quantity_type | ref_type | scalar_type | named_type }
scalar_type = @{ (^"string" | ^"int" | ^"decimal" | ^"bool" | ^"timestamp" | ^"uuid") ~ !(LETTER | NUMBER | "_") }
quantity_type = { ^"quantity" ~ "<" ~ symbol_ref ~ ">" }
ref_type = { ^"ref" ~ "<" ~ symbol_ref ~ ">" }
list_type = { ^"list" ~ "<" ~ element_type ~ ">" }
element_type = { quantity_type | ref_type | scalar_type | named_type }
named_type = { symbol_ref }
symbol_ref = { identifier ~ ("." ~ identifier)? }

field_constraints = { "(" ~ field_constraint ~ ("," ~ field_constraint)* ~ ")" }
field_constraint = {
    (^"min_length" ~ unsigned_integer) | (^"max_length" ~ unsigned_integer) |
    (^"min_items" ~ unsigned_integer) | (^"max_items" ~ unsigned_integer) |
    (^"exclusive_min" ~ number) | (^"exclusive_max" ~ number) |
    (^"min" ~ number) | (^"max" ~ number) |
    (^"pattern" ~ symbol_ref)
}
unsigned_integer = @{ ASCII_DIGIT+ }

operation_decl = { ^"operation" ~ identifier ~ "{" ~ operation_body ~ "}" }
operation_body = { operation_clause* }
operation_clause = {
    intent_clause | direction_clause | actor_clause | access_clause |
    input_clause | output_clause | state_clause | effect_clause |
    transaction_clause | failure_clause | idempotency_clause |
    concurrency_clause | evidence_clause | lifecycle_clause
}
intent_clause = { ^"intent" ~ string_literal }
direction_clause = { ^"direction" ~ direction_kind }
direction_kind = { ^"inbound" | ^"outbound" | ^"internal" }
actor_clause = { ^"actor" ~ (^"anonymous" | symbol_ref) }
access_clause = { ^"access" ~ (policy_governed_clause | ^"public") }
policy_governed_clause = { ^"policy_governed" ~ ^"by" ~ policy_binding ~ ("," ~ policy_binding)* }
policy_binding = { symbol_ref ~ ^"at" ~ enforcement_point ~ ^"fails" ~ ^"with" ~ identifier }
enforcement_point = { ^"precondition" | ^"invariant" | ^"postcondition" }
input_clause = { ^"input" ~ symbol_ref }
output_clause = { ^"output" ~ symbol_ref }
state_clause = { ^"state" ~ symbol_ref }
effect_clause = { ^"effect" ~ effect_kind ~ symbol_ref }
effect_kind = { ^"creates" | ^"mutates" | ^"reads" }
transaction_clause = { ^"transaction" ~ (^"single_aggregate" | ^"read_only") }
failure_clause = {
    ^"failure" ~ identifier ~ ^"for" ~ failure_kind ~
    ("," ~ failure_kind)* ~ string_literal
}
failure_kind = {
    ^"input_validation" | ^"policy" | ^"missing_state" |
    ^"idempotency_conflict" | ^"concurrency_conflict"
}
idempotency_clause = { ^"idempotency" ~ (idempotency_keyed | ^"inherent" | not_applicable_value) }
idempotency_keyed = { ^"keyed_by" ~ identifier }
concurrency_clause = { ^"concurrency" ~ (concurrency_unique | concurrency_optimistic | ^"read_snapshot") }
concurrency_unique = { ^"unique_key" ~ identifier }
concurrency_optimistic = { ^"optimistic_version" ~ identifier }
evidence_clause = { ^"evidence" ~ ^"operation_trace" }
lifecycle_clause = { ^"lifecycle" ~ ^"synchronous_request_response" }
not_applicable_value = { ^"not_applicable" ~ "(" ~ na_reason ~ "," ~ string_literal ~ ")" }
na_reason = { ^"read_only" }

application_decl_prefix = { (^"record" | ^"enum" | ^"operation") ~ identifier ~ "{" }
application_role_ref = { ^"role" ~ "<" ~ symbol_ref ~ ">" }
```

Insert `application_role_ref` into the existing `primary_expr` immediately
before `member_access`. `role<Customer>` and `role<alias.Customer>` are thereby
contextual role-reference expressions; a bare `role` remains an identifier.

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
The permissive `operation_clause*` production deliberately preserves a partial
operation AST so validation can emit APP001 for a missing, duplicate, or
out-of-order clause. A valid operation has exactly one of every non-failure
clause, one or more failures, and the order shown in `operation_clause`; the
formatter emits that order canonically. `enforcement_point` parses `invariant`
and `postcondition` so they are reserved words in this position, but validation
rejects them (APP007) because v0.1 supports `precondition` only.

Reference rule: every `symbol_ref` is either an unqualified identifier or
`<import-alias>.<identifier>`. An unqualified reference resolves in the local
module and its named-import bindings; an alias-qualified reference resolves
only through that wildcard alias. A quoted-name declaration referenced this
way MUST have an identifier-shaped name (`(LETTER | "_") (LETTER | NUMBER |
"_")*`); otherwise APP002. Resolution returns the target's stable ID; exact
text is never used as the post-resolution identity.

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
| Intent | `intent "<text>"` | Nonempty plain-language purpose; carried as decoded authored text into review artifacts and NFC-normalized in canonical artifacts. |
| Direction | `direction` | `inbound` only in v0.1; `outbound`/`internal` parse but fail APP001 as unsupported-in-v0.1. |
| Actor / access | `actor`, `access` | `anonymous` is legal only with `access public`. A named actor MUST resolve to a `role` and is required with `policy_governed`. `authenticated` does not exist in v0.1. |
| Input / output | `input`, `output` | Each MUST resolve to a `record` in the closure; entities are rejected (APP006). |
| State / effect | `state`, `effect` | `state` MUST resolve to an entity with a typed body; the effect target MUST be that same entity (APP011). Exactly one effect kind. |
| Transaction | `transaction` | `creates`/`mutates` require `single_aggregate`; `reads` requires `read_only` (APP001 on mismatch). |
| Failures | `failure <code> for <kind>...` | One or more; stable lower-snake-case code unique in the closure, one or more closed failure kinds, and nonempty meaning. Every validation, policy, missing-state, idempotency, and concurrency path maps by kind or policy binding to exactly one declared code. A kind may appear on only one failure in an operation. Structured failure details are outside v0.1; transport status is adapter mapping, not SEA meaning. |
| Idempotency | `idempotency` | Writes: `keyed_by <input-field>` where the field is required and scalar. Same key + same canonical input fingerprint returns the stored result with no new effect; same key + different fingerprint returns the declared idempotency-conflict failure. Reads: `inherent`. |
| Concurrency | `concurrency` | `creates` → `unique_key <input-field>`; `mutates` → `optimistic_version <input-field>` (required `int` field); `reads` → `read_snapshot`. Referenced fields MUST be required scalars of the stated type. |
| Evidence | `evidence operation_trace` | Mandatory on every inbound v0.1 operation; produces a structured record with semantic operation ID, outcome code, and correlation ID. No payload or secret is evidence by default. |
| Lifecycle | `lifecycle synchronous_request_response` | Mandatory on every inbound v0.1 operation; background, streaming, and externally managed lifecycles are unsupported. |
| Not applicable | `idempotency not_applicable(read_only, "<text>")` | Legal only on a `reads` operation. Runtime lowering is identical to `idempotency inherent`, while the canonical contract preserves the explicit reason and explanation for review. No other v0.1 concern has a reachable not-applicable state (APP010). |

State and output lowering are closed v0.1 language semantics:

- A required, non-key entity field of scalar, quantity, or enum type MAY
  declare `default <literal>`; record fields, optional fields, keys, refs, and
  lists MUST NOT. The literal is normalized, type-checked, and validated against
  every field constraint during contract construction. For an enum it is the
  declared wire string.
- `creates` uses this precedence for every state field: a present same-named
  input value, then an explicit entity default, then absence only when the state
  field is optional. Every required state field needs a required input source or
  a default; an optional input may source a required state field only when a
  default covers its absence. Any impossible path is APP011; providers never
  synthesize a value.
- `mutates` locates state using a same-named input matching the entity key,
  preserves the key, and replaces each other same-named state field supplied by
  the input. An input field that is neither a state field nor named by an
  idempotency/concurrency strategy is APP011.
- `reads` locates state using a same-named required input matching the entity
  key and performs no mutation.
- Every input field that maps to state and every output field that projects
  state MUST have the same field type, optionality, and canonical constraint set
  as its same-named entity field. All input values are validated before policy
  evaluation/effect and map to the input-validation failure on violation.
  Output is the post-state projection in record declaration order. Computed,
  renamed, or external output fields are outside v0.1 and fail APP011.

Policy bindings (`policy_governed by <policy> at precondition fails with
<code>`): each referenced policy MUST exist, MUST be a constraint policy whose
expression evaluates from the typed context (operation ID, actor role,
normalized input record, pre-operation aggregate state when one exists), and
each binding names one declared failure code. A bare identifier in a policy
expression resolves to the input-record field of that name; comparing a
quantity field to a bare number compares in the field's declared unit.
Missing facts, `UNKNOWN`, evaluator errors, and false obligations fail closed
to the bound failure code. A policy no operation references remains valid.

The v0.1 executable policy subset is closed. It accepts `Literal`,
`QuantityLiteral`, `RoleReference`, `Variable`, `MemberAccess`, `Binary`, and `Unary` expression
nodes. `Variable(name)` and `input.name` resolve only a declared input field;
`state.name` resolves only a declared state field and is `UNKNOWN` when no
pre-state exists; `actor.role` is the bound role `ConceptId` and is `UNKNOWN`
for anonymous callers. `role<symbol_ref>` resolves at contract construction to
that role's exact `ConceptId`; it is legal only as the other operand of
`actor.role = ...` or `actor.role != ...`. An unresolved, unexported, aliased-to-
the-wrong-kind, or otherwise misplaced role reference is APP007. No other
member-access object is legal. Binary operators
are `And`, `Or`, `Equal`, `NotEqual`, `GreaterThan`, `LessThan`,
`GreaterThanOrEqual`, `LessThanOrEqual`, `Plus`, `Minus`, `Multiply`, and
`Divide`; unary operators are `Not` and `Negate`. Boolean operators use the
existing three-valued truth tables. Numeric operations require identical scalar
types, except a bare numeric literal compared with a quantity adopts that
field's unit; division by zero, overflow, loss of decimal precision, invalid
coercion, `GroupBy`, `Cast`, `Quantifier`, `Aggregation`,
`AggregationComprehension`, temporal nodes/operators, string operators, and
`HasRole` fail APP007 during contract construction.

Strategy execution is also fixed. `unique_key f` requires same-named compatible
required scalar fields in input and state and enforces durable uniqueness on
the state field. `optimistic_version f` requires same-named required `int`
fields in input and state; the input is the expected persisted version, a
successful mutation atomically stores `f + 1`, and mismatch returns the
concurrency-conflict failure. A create of an entity containing that version
field MUST obtain its initial value from input or an explicit entity default;
the provider supplies no implicit zero. `read_snapshot` reads one transactionally
consistent aggregate snapshot. For a keyed create, idempotency lookup happens
before uniqueness: same key/same fingerprint replays success, same key/different
fingerprint returns the idempotency-conflict failure, and a new idempotency key
that violates `unique_key` returns the concurrency-conflict failure. The two
kinds MAY map to one declared code, as the D9 command does.

For bounds, at most one of `min`/`exclusive_min` and at most one of
`max`/`exclusive_max` may occur. Normalize quantity bounds to the field's base
unit before comparison. A lower value below its upper value is valid; equal
values are valid only when both bounds are inclusive (`min` plus `max`), which
denotes a singleton. Equality with either exclusive bound and every lower value
above its upper value are APP012.

## 5. Validation invariants and diagnostic codes

Diagnostics are stable and machine-readable: `APPNNN code_slug`.
Every APP diagnostic has `Error` severity in interactive and strict modes and
blocks contract emission. Its structured context includes `reason`, `expected`,
`actual`, and `remediation`; inapplicable values are omitted, not serialized as
empty strings. `remediation` is the smallest authored-source or artifact-chain
change that can satisfy the invariant without weakening it. APP014 additionally
includes its closed reason value; APP015 includes `document_kind`,
`schema_version`, and the failing JSON Pointer or hash field.

| Code | Slug | Invariant violated |
|---|---|---|
| APP001 | missing_application_semantics | Any §4 consequential item absent, duplicated, out of canonical order, unsupported-in-v0.1, or inconsistent (direction, transaction pairing, mandatory clause cardinality). Message lists operation/source ref, each defect, why it matters, and the smallest supported remediation. |
| APP002 | unknown_contract_reference | `named_type`, `ref`, `quantity` unit, `pattern`, `input`, `output`, `state`, or `actor` does not resolve in the symbol table, resolves to the wrong declaration kind where APP006/APP007/APP011 do not apply, or the target's quoted name is not identifier-shaped. |
| APP003 | invalid_record_shape | Record/entity field uses a nested record, nested list, map-like, union-like, or otherwise non-closed type; a forbidden field declares a default; or a default does not satisfy its entity field type and constraints. |
| APP004 | duplicate_field | Duplicate field name inside one record or entity body. |
| APP005 | invalid_aggregate_key | Typed entity body has zero or multiple `key` fields, the key type is not `uuid`/`string`, or a key field is `optional` or has a default. |
| APP006 | operation_type_mismatch | Operation `input`/`output` resolves to an entity or enum instead of a record. |
| APP007 | policy_scope_error | Dangling policy reference, non-constraint policy bound, expression not evaluable from the typed context, or enforcement point `invariant`/`postcondition`. |
| APP008 | invalid_failure_declaration | Failure code not lower-snake-case, duplicate in the resolved closure, failure kind duplicated within the operation, or a mapped failure path (validation, policy, missing state, idempotency, concurrency) has zero or multiple declared codes. |
| APP009 | invalid_strategy_reference | `keyed_by`/`unique_key`/`optimistic_version` input field is missing, optional, non-scalar, or the wrong type; its required same-named state field is missing or incompatible; or strategy/effect pairing falls outside §4. |
| APP010 | invalid_not_applicable | `not_applicable` reason not proven by the operation's effect and direction. |
| APP011 | effect_state_mismatch | Effect target differs from state, state lacks a typed body, key lookup is absent/incompatible, create cannot construct every required state field, mutate contains an unmapped field, or output cannot project by same-name compatible state fields. |
| APP012 | constraint_error | Constraint applied to an incompatible type; more than one lower-bound or upper-bound member; an empty interval under the inclusivity rules; duplicate other constraint kind; fractional/overflowing length or item bound; decimal precision above 28 digits or scale outside 0–28; arithmetic requiring rounding; or `pattern` not resolving to a declared SEA `Pattern`. |
| APP013 | enum_error | Duplicate enum member identifier or duplicate wire string within one enum. |
| APP014 | closure_resolution_error | Closure resolution fails. Diagnostic context MUST identify one reason: `symbol_collision`, `import_cycle`, `unresolved_specifier`, `unresolved_alias`, or `not_exported`. Distinct declarations with one qualified ID are `symbol_collision`; repeated reachability of the same origin is not. |
| APP015 | artifact_schema_violation | An Application Contract or Canonical Semantic Envelope document fails its declared schema, required metadata/hash validation, or canonical-envelope invariants, including unknown fields, source spans, or an unstripped `export` wrapper. |

APP001–APP014 carry the source module logical ID, line, and column. They also
carry the stable semantic ID when one has been assigned; pre-identity resolver
failures instead carry the import or declaration field path. APP015 identifies
the artifact path/document kind and failing JSON Pointer or hash field instead
of inventing a SEA coordinate. This matches the current AST's `Spanned<T>`
evidence; byte spans are not claimed until the parser retains them.

## 6. Canonicalization rules

- **Closure order (sets):** modules by logical module ID (bytewise ascending),
  then semantic declarations by this fixed kind rank: `dimension`, `unit`,
  `record`, `enum`, `operation`, `entity`, `resource`, `flow`, `pattern`,
  `role`, `relation`, `instance`, `policy`, `concept_change`, `metric`;
  declarations of one kind sort by canonical declaration ID (bytewise
  ascending). Mapping/projection and Cell-environment declarations are
  realization/configuration inputs, not semantic declarations; their hashes
  belong to the later resolution/plan lock and MUST NOT enter this envelope.
- **Sequences:** authored order is preserved only for record/entity field
  lists, enum member lists, operation failure lists, and policy-binding
  lists — these are declared sequences.
- **Defaults:** canonical JSON serializes every required field and every
  schema-defined default explicitly; `optional` serializes as
  `"optional": false` when unmarked. Absent `Option<T>` values are omitted,
  never encoded as JSON `null`.
- **Scalars:** strings are NFC-normalized; `int` is a signed 64-bit integer;
  `decimal` canonical form is a string with no exponent, no leading zeros, no
  trailing fractional zeros, and `"0"` for zero (so an authored `"125.00"`
  fingerprints as `"125"`). Authored and computed decimals MUST fit 28
  significant digits and scale 0–28; arithmetic that would overflow or require
  rounding fails APP012. `timestamp` is RFC 3339 UTC with a trailing `Z`;
  `uuid` is lowercase hyphenated; `bool` is `true`/`false`.
- **Quantities:** normalized through the referenced unit's declared dimension
  and base-unit factor before fingerprinting; the canonical value is the
  base-unit decimal in canonical decimal form plus the base unit symbol.
- **Hash representation:** every hash in this contract is lowercase SHA-256
  with the required `sha256:` prefix.
- **Source-set hash:** construct canonical JSON
  `{"schema_version":"domainforge-source-set/v1","sources":[...]}`. Each
  source entry is `{"logical_id":"...","source_utf8":"..."}`; logical IDs
  are NFC-normalized and sorted bytewise, source text is the exact decoded
  UTF-8 content, and duplicate logical IDs are rejected. `source_set_hash` is
  SHA-256 over those canonical JSON bytes. This framing prevents pair-boundary
  ambiguity.
- **Semantic-closure hash:** construct the `CanonicalSemanticEnvelope` defined
  in §8 with `self_hash` and `semantic_closure_hash` absent. It contains the
  all normalized resolved semantic declarations in the closure (including
  policies, roles, units, patterns, and entities whether or not an operation
  references them), normalized
  application contract, import graph and source content hashes, namespace
  bindings, semantic-pack IDs/content hashes, language schema version,
  canonicalization version, and compiler interpretation version.
  `semantic_closure_hash` is SHA-256 over its canonical JSON bytes. Comments,
  formatting, absolute paths, timestamps, and semantically irrelevant source
  order are excluded.
- **Artifact self-hash:** for each persisted contract/envelope, compute
  `self_hash` over canonical JSON with only `self_hash` absent; `inputs` records
  the hashes consumed to build it. Validation of schema, inputs, and self-hash
  precedes use.
- Byte serialization MAY reuse `semantic_pack::canonical_json` without changing
  that helper's semantics. No application hash feeds semantic-pack content
  hashes, and existing pack fixture hashes/signatures MUST remain byte-identical
  before and after Milestone 0 (D10).
- **Semantic-pack-set hash:** construct canonical JSON
  `{"schema_version":"domainforge-semantic-pack-set/v1","packs":[...]}`
  from `{pack_id, content_hash}` entries sorted by `pack_id`, reject duplicate
  IDs, and SHA-256 those bytes with the standard prefix. The empty set has one
  fixed golden vector; it is never represented by an empty string.
- **Canonical input fingerprint** (idempotency): `sha256:` plus SHA-256 over
  canonical JSON of the normalized input record. Object keys sort bytewise
  after NFC normalization; declared field order remains contract meaning but
  does not affect the fingerprint of an otherwise identical JSON object.

## 7. Import, namespace, identity, and duplicate semantics

`ResolvedModuleSet` is the only resolver output. Import syntax remains
`import ... from "<specifier>"`, with these exact specifier rules:

- a specifier beginning `./` or `../` is a logical-path import resolved against
  the importing module's logical directory;
- logical paths use `/`, preserve case, normalize NFC and `.` segments, reject
  `\`, absolute paths, empty segments, and any `..` that escapes the source-map
  root, and must match exactly one `sources_json` key after normalization;
- `std` and `std:*` retain current built-in resolution;
- every other specifier is a namespace import. Native resolution uses the
  existing `NamespaceRegistry`; in-memory resolution requires exactly one
  source-map module whose declared namespace equals the specifier's exact
  decoded, case-sensitive bytes.

Named imports bind only named exported symbols. Wildcard imports remain
alias-qualified and are referenced as `<alias>.<name>` through `symbol_ref`.
Unexported imported declarations are not visible. Import cycles, unresolved
aliases/specifiers, and two distinct declarations with the same qualified ID
are APP014 errors. The same declaration origin reached through a diamond import
collapses to one symbol; two separately authored but textually equivalent
declarations never collapse.

Existing concept identity and reference matching preserve the exact decoded,
case-sensitive namespace/name bytes because current
`ConceptId::from_concept(namespace, name)` performs no Unicode normalization.
NFC-normalizing those inputs would be a breaking change and is forbidden here.
New application declaration names and their references are NFC-normalized and
case-preserving; application-only qualified IDs are
`<nfc-namespace>.<kind>.<nfc-declared-name>`. Graph and `ApplicationContract`
both consume the same typed resolved declarations and perform no independent
name or import resolution. Canonical display strings use NFC, but an existing
opaque `ConceptId` remains authoritative when two legacy spellings normalize to
the same display text.

## 8. Proposed internal Rust types

New module `domainforge-core/src/application/`. Public serialized structs use
`#[serde(deny_unknown_fields)]`; discriminated enums use adjacent tagging
`#[serde(tag = "kind", content = "data", rename_all = "snake_case")]` so unit,
newtype, tuple, and struct variants all serialize; newtype IDs use
`#[serde(transparent)]`; field names remain `snake_case`. These attributes and
the generated JSON schema, not Serde defaults, define the public wire shape.

```rust
pub struct ApplicationSymbolId(pub String); // record/enum/operation only

pub struct OriginRef {
    pub logical_module_id: String,
    pub line: usize,
    pub column: usize,
}

pub struct ProducerIdentity {
    pub name: String,                       // "domainforge-core"
    pub version: String,
}

pub struct ApplicationContractInputs {
    pub source_set_hash: String,            // "sha256:<lowercase-hex>"
    pub semantic_pack_set_hash: String,
    pub language_schema_version: String,
    pub interpretation_version: String,
}

pub struct ApplicationContractDocument {
    pub schema_version: String,             // "domainforge-application-contract/v1"
    pub producer: ProducerIdentity,
    pub inputs: ApplicationContractInputs,
    pub self_hash: String,
    pub semantic_closure_hash: String,
    pub contract: ApplicationContract,
}

pub struct ApplicationContract {
    pub enums: Vec<EnumContract>,
    pub records: Vec<RecordContract>,
    pub entities: Vec<EntityContract>,
    pub operations: Vec<OperationContract>,
}

pub enum ScalarType { String, Int, Decimal, Bool, Timestamp, Uuid }

pub enum FieldType {
    Scalar { scalar: ScalarType },
    Quantity { unit: ConceptId },
    EntityRef { entity: ConceptId },
    Enum { symbol: ApplicationSymbolId },
    List { element: Box<FieldType> },       // element is never List
}

pub enum FieldConstraint {
    Min { value: rust_decimal::Decimal },
    Max { value: rust_decimal::Decimal },
    ExclusiveMin { value: rust_decimal::Decimal },
    ExclusiveMax { value: rust_decimal::Decimal },
    MinLength { value: u32 },
    MaxLength { value: u32 },
    MinItems { value: u32 },
    MaxItems { value: u32 },
    Pattern { pattern: ConceptId },
}

pub struct FieldContract {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub constraints: Vec<FieldConstraint>,
    pub default: Option<TypedValue>,         // entity fields only
}

pub struct EnumMember { pub name: String, pub wire: String }
pub struct EnumContract { pub id: ApplicationSymbolId, pub name: String, pub members: Vec<EnumMember> }
pub struct RecordContract { pub id: ApplicationSymbolId, pub name: String, pub fields: Vec<FieldContract> }
pub struct EntityContract {
    pub concept_id: ConceptId,
    pub name: String,
    pub key_field: String,
    pub fields: Vec<FieldContract>,
}

pub enum Direction { Inbound, Outbound, Internal }
pub enum ActorRef { Anonymous, Role { role: ConceptId } }
pub enum EnforcementPoint { Precondition }
pub struct PolicyBinding {
    pub policy: ConceptId,
    pub enforcement_point: EnforcementPoint,
    pub failure_code: String,
}
pub enum AccessMode { Public, PolicyGoverned { bindings: Vec<PolicyBinding> } }
pub enum EffectKind { Creates, Mutates, Reads }
pub enum TransactionBoundary { SingleAggregate, ReadOnly }
pub enum FailureKind { InputValidation, Policy, MissingState, IdempotencyConflict, ConcurrencyConflict }
pub struct FailureContract {
    pub code: String,
    pub kinds: Vec<FailureKind>,
    pub meaning: String,
}
pub struct NotApplicable { pub explanation: String } // reason is always read_only in v0.1
pub enum IdempotencyStrategy { KeyedBy { field: String }, Inherent, NotApplicable(NotApplicable) }
pub enum ConcurrencyStrategy { UniqueKey { field: String }, OptimisticVersion { field: String }, ReadSnapshot }
pub enum EvidenceKind { OperationTrace }
pub enum LifecycleKind { SynchronousRequestResponse }

pub struct OperationContract {
    pub id: ApplicationSymbolId,
    pub name: String,
    pub intent: String,
    pub direction: Direction,
    pub actor: ActorRef,
    pub access: AccessMode,
    pub input: ApplicationSymbolId,
    pub output: ApplicationSymbolId,
    pub state: ConceptId,
    pub effect: EffectKind,
    pub transaction: TransactionBoundary,
    pub failures: Vec<FailureContract>,
    pub idempotency: IdempotencyStrategy,
    pub concurrency: ConcurrencyStrategy,
    pub evidence: EvidenceKind,
    pub lifecycle: LifecycleKind,
}

pub struct ApplicationPolicyContext {
    pub operation: ApplicationSymbolId,
    pub actor: ActorRef,
    pub input: indexmap::IndexMap<String, TypedValue>,
    pub pre_state: Option<indexmap::IndexMap<String, TypedValue>>,
}

pub enum TypedValue {
    String(String), Int(i64), Decimal(rust_decimal::Decimal), Bool(bool),
    Timestamp(chrono::DateTime<chrono::Utc>), Uuid(uuid::Uuid),
    Quantity { base_value: rust_decimal::Decimal, unit: ConceptId },
    EntityRef { entity: ConceptId, key: Box<TypedValue> },
    Enum { symbol: ApplicationSymbolId, wire: String },
    List(Vec<TypedValue>),
}
```

Application preconditions are evaluated by a new application-owned evaluator
that consumes the existing policy `Expression` plus `ApplicationPolicyContext`.
It MUST NOT mutate or synthesize Graph instances. It returns the existing
three-valued result; false, `UNKNOWN`, or evaluation error maps fail-closed to
the binding's declared failure code.

Resolver output (in `domainforge-core/src/module/resolver.rs`) retains the typed
declaration payload needed by both consumers:

```rust
pub struct ResolvedModuleSet {
    pub modules: Vec<ResolvedModule>,
    pub import_graph: Vec<ImportEdge>,
    pub symbols: indexmap::IndexMap<String, ResolvedSymbol>,
    pub aliases: Vec<AliasBinding>,
}
pub struct ResolvedModule {
    pub logical_id: String,
    pub source_hash: String,                // "sha256:<lowercase-hex>"
    pub ast: crate::parser::ast::Ast,
}
pub struct ImportEdge { pub importer: String, pub imported: String }
pub struct AliasBinding { pub importer: String, pub alias: String, pub target: String }
pub enum ResolvedDeclarationKind { ExistingConcept, Record, Enum, Operation }
pub struct ResolvedSymbol {
    pub qualified_id: String,
    pub kind: ResolvedDeclarationKind,
    pub exported: bool,
    pub origin: OriginRef,
    pub module_index: usize,
    pub declaration_index: usize,
}
```

`module_index` and `declaration_index` address the immutable typed AST payload;
all reference edges are resolved once to `ConceptId` or `ApplicationSymbolId`
before Graph or `ApplicationContract` construction. `OriginRef` remains resolver
and diagnostic evidence; it is deliberately absent from persisted contract
types so line movement and formatting cannot change semantic hashes.

The persisted canonical envelope required by the governing spec is:

```rust
pub struct CanonicalSemanticEnvelopeDocument {
    pub schema_version: String,             // "domainforge-semantic-envelope/v1"
    pub producer: ProducerIdentity,
    pub inputs: ApplicationContractInputs,
    pub self_hash: String,
    pub semantic_closure_hash: String,
    pub envelope: CanonicalSemanticEnvelope,
}
pub struct CanonicalSemanticEnvelope {
    pub language_schema_version: String,
    pub canonicalization_version: String,
    pub compiler_interpretation_version: String,
    pub modules: Vec<CanonicalModuleRef>,
    pub import_graph: Vec<ImportEdge>,
    pub namespace_bindings: Vec<CanonicalNamespaceBinding>,
    pub semantic_packs: Vec<CanonicalSemanticPackRef>,
    pub semantic_declarations: Vec<CanonicalSemanticDeclaration>,
    pub resolved_references: Vec<CanonicalResolvedReference>,
    pub application_contract: ApplicationContract,
}
pub struct CanonicalModuleRef {
    pub logical_id: String,
    pub semantic_content_hash: String,
    pub namespace: String,
}
pub struct CanonicalNamespaceBinding { pub namespace: String, pub logical_id: String }
pub struct CanonicalSemanticPackRef { pub pack_id: String, pub content_hash: String }
pub struct CanonicalSemanticDeclaration {
    pub id: CanonicalDeclarationId,
    pub logical_module_id: String,
    pub declaration: CanonicalSemanticPayload,
}
pub enum CanonicalDeclarationId {
    Concept { id: ConceptId },
    Application { id: ApplicationSymbolId },
    Occurrence { kind: String, content_hash: String, duplicate_index: u32 },
}
pub enum CanonicalReferenceTarget {
    Declaration { id: CanonicalDeclarationId },
    Namespace { exact_name: String, logical_module_id: String },
}
pub struct CanonicalResolvedReference {
    pub source: CanonicalDeclarationId,
    pub field_path: String,
    pub target: CanonicalReferenceTarget,
}

pub enum CanonicalSemanticPayload {
    Dimension(CanonicalDimensionDecl), Unit(CanonicalUnitDecl),
    Record(RecordContract), Enum(EnumContract), Operation(OperationContract),
    Entity(CanonicalEntityDecl), Resource(CanonicalResourceDecl),
    Flow(CanonicalFlowDecl), Pattern(CanonicalPatternDecl),
    Role(CanonicalRoleDecl), Relation(CanonicalRelationDecl),
    Instance(CanonicalInstanceDecl), Policy(CanonicalPolicyDecl),
    ConceptChange(CanonicalConceptChangeDecl), Metric(CanonicalMetricDecl),
}
```

The fifteen dedicated `Canonical*Decl` payload structs mirror the semantic
fields of their corresponding non-`Export` `ast_schema::AstNode` variant; they
do not embed `AstNode`. Conversion is one exhaustive `match` with no wildcard,
so adding an AST variant forces an explicit classification. Each reference
field is replaced by `CanonicalReferenceTarget`, every absent schema default is
inserted, maps become duplicate-rejecting key-sorted `IndexMap`s, and authored
sequences remain vectors. The exact reference fields are: unit dimension/base,
entity domain and typed field types/constraints, resource unit/domain, flow
resource/from/to, role domain, relation subject/object/via-flow, instance
entity type, policy/metric expression unit and semantic member references, and
concept-change target. Names, versions, predicates, regexes, literal values,
policy metadata, metric metadata, annotations, quantities, and expressions
remain concrete normalized values, not opaque JSON. `Record`, `Enum`, and
`Operation` reuse their normalized contract structs; `CanonicalEntityDecl`
contains the existing entity metadata plus `Option<EntityContract>`.

Flow declarations, whose current Graph IDs are run-random, use `Occurrence`:
hash their canonical payload without the ID, group equal hashes, and assign
`duplicate_index` from zero within that sorted equal-payload group. Instances
retain the existing deterministic
`ConceptId::from_concept(namespace, format!("{entity_type}:{name}"))`; a parity
test compares the canonical ID with Graph. All other existing declarations
retain `ConceptId`; application declarations use `ApplicationSymbolId`. The envelope validator rejects export wrappers,
source positions, raw unresolved reference strings, and unclassified AST
variants rather than silently accepting them.

A module's `semantic_content_hash` hashes only its normalized semantic
declarations, not raw source; raw bytes belong to `inputs.source_set_hash`.
Each `field_path` is a canonical, schema-defined path; the unit reference on
the `total` field is `fields.total.field_type.unit`. It is never a source
coordinate. `resolved_references` duplicates the typed reference edges for
indexing and audit; the canonical payload itself also stores the resolved IDs.
This representation excludes Graph's run-random occurrence IDs and keeps
comments/formatting out of semantic hashes.

AST additions in `domainforge-core/src/parser/ast.rs`: declaration variants
`Record(RecordDecl)`, `Enum(EnumDecl)`, `Operation(OperationDecl)` and an
`Option<EntityBody>` field on the existing entity node, each mirroring the §2
grammar one-to-one, with additive serialized twins in
`domainforge-core/src/parser/ast_schema.rs` and
`schemas/ast-v3.schema.json`. `FieldDecl` has
`default: Option<Expression>` restricted by validation to the `literal` grammar
branch. `OperationDecl` stores `clauses: Vec<OperationClause>` in source order;
`OperationClause` has one variant for each `operation_clause` alternative, and
`Failure(FailureDecl)` may repeat. It MUST NOT deserialize directly into
`OperationContract`; resolution validates clause cardinality/order first. This
partial-AST shape is what makes APP001 available for structurally incomplete
operations. Existing expression AST/schema enums gain the additive
`RoleReference { role: String }` variant; resolution replaces that authored
`symbol_ref` text with the target role `ConceptId` in canonical policy payloads.

## 9. Public boundary and binding obligations

Public surfaces (D7, accepted):

- Serialized AST v3 with the additive declaration variants; existing valid
  documents remain valid and serialize unchanged; no top-level discriminator
  is added to `{metadata, declarations}`; any non-additive change requires an
  explicit ast-v4 decision.
- `ApplicationContractDocument` canonical JSON under schema ID
  `domainforge-application-contract/v1`
  (`schemas/application-contract-v1.schema.json`), with required
  `schema_version`, `producer`, `inputs`, `self_hash`, and unknown-field
  rejection (APP015). `CanonicalSemanticEnvelopeDocument` uses
  `domainforge-semantic-envelope/v1` and the same artifact rules.
- Rust: both document types, `ApplicationContract`,
  `CanonicalSemanticEnvelope`, and their serialized constituent types, plus
  strict validation and canonical JSON serialization, are public API of
  `domainforge-core`. Resolver-only types remain crate-private.
- Python preserves the existing static
  `Graph.parse_to_ast_json(source: str) -> str` and adds
  `Graph.resolve_application_contract_json(entry_logical_path: str,
  sources_json: str) -> str`.
- TypeScript adds static `Graph.parseToAstJson(source: string): string` and
  `Graph.resolveApplicationContractJson(entryLogicalPath: string,
  sourcesJson: string): string` to its napi `Graph` class.
- WASM preserves static `Graph.parseToAstJson(source: string): string` and adds
  `Graph.resolveApplicationContractJson(entryLogicalPath: string,
  sourcesJson: string): string`.
- `sources_json` is a strict JSON object mapping normalized logical paths to SEA
  UTF-8 source strings under §7. All targets return identical compact canonical
  `ApplicationContractDocument` JSON. Typed mutable binding wrappers remain out
  of v0.1.

`ResolvedModuleSet` and the validation internals remain private Rust in v0.1.

## 10. Backward compatibility and formatter rules

- All new tokens are contextual; none joins `declaration_keyword`. Lookahead
  extension is confined to `resource_decl` via `application_decl_prefix`.
- An entity without a typed body produces exactly the same AST and canonical
  formatter output as before this change.
- Enums are closed in v0.1 and expose no unknown-member fallback. Adding,
  removing, renaming, or changing the wire value of a member is therefore a
  breaking contract change. A future language version MAY make addition
  compatible only by adding explicit unknown-member handling first.
- The compatibility oracle is equality of formatter and AST output from the
  compiler before and after the change over the existing corpus — not byte
  equality between authored and formatted source.
- Every new token receives a collision fixture in every current identifier,
  name, namespace, unit, alias, and annotation-key position
  (`fixtures/application_generation/compat/keyword-collision.sea`).
- Canonical formatting of new declarations: four-space indentation inside
  bodies; one field or clause per line; clause order exactly as §2; no space
  before and one space after `:`; one space around `=`; enum members appear one
  per line with a comma after every member except the last; a field default
  follows constraints as ` default <canonical-literal>`; formatted output MUST
  re-parse to an identical AST (round-trip law).
- If any token cannot satisfy the corpus and collision tests, it is
  reclassified breaking per the ADR-013 Migration path contingency before
  implementation proceeds.

## 11. Proving fixtures (fenced; to be created during Milestone 0)

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
    total: quantity<USD> (exclusive_min 0)
    item_count: int (min 1)
    status: OrderStatus default "placed"
}

record PlaceOrderInput {
    order_id: uuid
    client_order_id: string (min_length 1, max_length 64)
    total: quantity<USD> (exclusive_min 0)
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
    failure invalid_order for input_validation "input failed record validation"
    failure order_limit_exceeded for policy "order total exceeds 10000 USD"
    failure idempotency_conflict for idempotency_conflict, concurrency_conflict "client_order_id reused with different canonical input"
    idempotency keyed_by client_order_id
    concurrency unique_key client_order_id
    evidence operation_trace
    lifecycle synchronous_request_response
}
```

D9 requires a strictly positive total, expressed without assuming a currency
scale as `exclusive_min 0`. The D9 limit ("at most 10000 USD") is the policy
expression `total <= 10000`, evaluated in the field's declared unit. The
explicit entity default supplies `status = placed`; create and output lowering
then follow the same-name rules in §4 without renderer inference.

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
    failure order_not_found for missing_state "no order exists for the given order_id"
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
- structured failure-detail payloads;
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
| Typed input/output records, fields, constraints, cardinality | `record` declarations; `input PlaceOrderInput` / `output PlaceOrderOutput` | `RecordContract.fields`, `FieldContract`, `FieldType`, `FieldConstraint`; `FieldContract.default`; `OperationContract.input`, `.output` | input/output resolve to records; flat closed types; constraint/type/default validation | APP002, APP003, APP004, APP006, APP012 | command-write.sea 23–33, 40–41 |
| Affected aggregate/state and effect kind | `state Order`, `effect creates Order` | `OperationContract.state`, `.effect`; `EntityContract.key_field`, `.fields` | state has one typed key; effect target equals state; same-name input/defaults construct state and post-state projects output | APP005, APP011 | command-write.sea 15–21, 42–43 |
| Applicable policies and enforcement point | `policy_governed by order_total_within_limit at precondition fails with order_limit_exceeded` | `AccessMode::PolicyGoverned`, `PolicyBinding` | constraint policy evaluable from typed context; precondition only; fail closed to bound code | APP007 | command-write.sea 39 |
| Transaction boundary | `transaction single_aggregate` / `transaction read_only` | `OperationContract.transaction` | pairing fixed by effect kind | APP001 | command-write.sea 44; query-read.sea 24 |
| Expected canonical failures | `failure <code> for <kind>... "<meaning>"` | `OperationContract.failures`, `FailureContract` | lower-snake-case codes unique in closure; each failure kind maps to exactly one code; policy binding maps to a declared policy-kind failure | APP008 | command-write.sea 45–47; query-read.sea 25 |
| Idempotency/concurrency behavior | `idempotency keyed_by client_order_id`, `concurrency unique_key client_order_id` / `inherent`, `read_snapshot` | `OperationContract.idempotency`, `.concurrency`; `NotApplicable` | strategy/effect pairing; referenced fields required scalars; not_applicable proven by effect/direction | APP009, APP010 | command-write.sea 48–49; query-read.sea 26–27 |
| Evidence/observation obligations | `evidence operation_trace` | `OperationContract.evidence` | mandatory on inbound v0.1 operations | APP001 | command-write.sea 50; query-read.sea 28 |
| Externally visible lifecycle | `lifecycle synchronous_request_response` | `OperationContract.lifecycle` | mandatory on inbound v0.1 operations | APP001 | command-write.sea 51; query-read.sea 29 |
