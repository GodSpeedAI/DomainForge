# DomainForge Semantic Adapters Implementation Spec

Status: Draft v0.3 — implementation contract

Scope: v0.3 implements semantic-pack generation and validation in DomainForge `sea-core`, CLI pack commands, and `domainforge-lsp` loading/diagnostics. SWE_SEED, SEA-Forge, GodSpeed-Agent, and Context Kernel receive a stable consumer contract, but are not required for v0.3 conformance except for one smoke fixture that proves the JSON result can be consumed without LSP internals.

Purpose: Build a deterministic, review-gated, signed semantic-pack system so `.sea` source files, `.sea` tests, and policy artifacts can be validated against an approved organization/domain vocabulary without creating custom customer binaries. v0.3 closes remaining implementer-blocking ambiguities around deprecated-policy behavior, signature payload verification, alias conflict detection, unsigned fixture bypasses, meaning-version enforcement, canonical array ordering, expected-hash pins, and review amendment cost.

Owner: GodSpeed AI / DomainForge maintainers

## 0. Implementation Frame

This is a build spec, not a concept note. A conforming v0.3 implementation must produce code, commands, fixtures, and passing tests.

The v0.3 build target is:

```text
.sea source files
  -> sea-core parser / graph
  -> pre-pack source coherence checks
  -> candidate semantic pack
  -> review manifest gate
  -> approved semantic pack
  -> deterministic pack hash
  -> Ed25519 signature
  -> CLI validation
  -> LSP diagnostics
```

The implementation is complete only when this command sequence works against committed fixtures:

```bash
domainforge pack build \
  --source fixtures/org/acme/domain/**/*.sea \
  --review fixtures/org/acme/review.semantic-review.jsonl \
  --out fixtures/org/acme/.domainforge/packs/acme.procurement.semantic-pack.json

 domainforge pack sign \
  --pack fixtures/org/acme/.domainforge/packs/acme.procurement.semantic-pack.json \
  --key path/to/your-ed25519-private.pem

 domainforge pack verify \
  --pack fixtures/org/acme/.domainforge/packs/acme.procurement.semantic-pack.json \
  --pubkey path/to/your-ed25519-public.pem

 domainforge pack validate \
  --pack fixtures/org/acme/.domainforge/packs/acme.procurement.semantic-pack.json \
  --mode strict \
  fixtures/org/acme/tests/*.sea
```

The LSP is complete only when opening the same fixture files produces the same diagnostic `code`, `semantic_truth`, and primary range as CLI strict validation.

## 1. Problem Statement

DomainForge already provides generic `.sea` syntax and graph construction. That does not prove a model is semantically aligned with one organization's declared business world. A `.sea` file can be syntactically valid while inventing an alien term, using a deprecated alias, mixing units, or referencing a relation that the organization does not recognize.

The system must close four implementation gaps:

1. **Organization vocabulary boundary** — tests and policies must not silently introduce terms outside the approved semantic model.
2. **Bootstrapping boundary** — a pack must not become authority merely because it was generated from `.sea`; source model coherence must be checked before pack approval.
3. **Three-valued truth boundary** — `unknown` must remain distinct from `invalid`, even when configured as an error severity in CI.
4. **Artifact authority boundary** — strict-mode packs must be hash-stable, review-gated, and signed before they can govern CI or runtime claims.

## 2. v0.3 Goals and Non-Goals

### 2.1 Goals

- Add `SemanticPack`, `SemanticPackBuilder`, `SemanticPackValidator`, `PackSet`, `SemanticDiagnostic`, and review/signature support to `sea-core` or the nearest DomainForge core crate.
- Add CLI commands: `pack build`, `pack validate`, `pack inspect`, `pack diff`, `pack sign`, and `pack verify`.
- Extend `domainforge-lsp` so workspace config can load a semantic pack and publish semantic diagnostics in addition to parse diagnostics.
- Require deterministic canonical JSON and deterministic output ordering for pack hashes.
- Require review manifests for approved packs.
- Require `semantic_truth` on every semantic diagnostic.
- Implement deterministic alias resolution over the full candidate set, not over ordered shortcut tiers.
- Implement pack-set merge identity for multi-pack validation.
- Provide committed fixtures that prove valid, invalid, unknown, deprecated, ambiguous, unit mismatch, relation mismatch, unsigned-pack, and pack-version mismatch behavior.

### 2.2 Non-Goals

- No customer-specific LSP binary.
- No embedding model as source of truth.
- No full ontology governance workflow UI.
- No automatic business-meaning approval from syntax alone.
- No Context Kernel ingestion enforcement in v0.3 beyond a reusable mapping result schema and one consumer smoke test.
- No runtime SEA-Forge authority integration as a v0.3 completion requirement.

## 3. Repository-Level Work Items

### 3.1 DomainForge core crate

Add these modules under the core DomainForge crate that owns parsing and graph construction. If the crate is named `sea-core`, use these paths:

```text
sea-core/src/semantic_pack/mod.rs
sea-core/src/semantic_pack/schema.rs
sea-core/src/semantic_pack/builder.rs
sea-core/src/semantic_pack/review.rs
sea-core/src/semantic_pack/validator.rs
sea-core/src/semantic_pack/resolver.rs
sea-core/src/semantic_pack/pack_set.rs
sea-core/src/semantic_pack/canonical_json.rs
sea-core/src/semantic_pack/signing.rs
sea-core/src/semantic_pack/diagnostics.rs
sea-core/src/semantic_pack/diff.rs
sea-core/tests/semantic_pack_build.rs
sea-core/tests/semantic_pack_validate.rs
sea-core/tests/semantic_pack_alias_resolution.rs
sea-core/tests/semantic_pack_three_valued.rs
sea-core/tests/semantic_pack_pack_set.rs
```

If the real crate layout differs, keep the module boundaries and public API names. Do not move pack semantics into the LSP crate.

### 3.2 DomainForge CLI

Add these CLI subcommands wherever the DomainForge CLI currently lives:

```text
domainforge pack build
domainforge pack validate
domainforge pack inspect
domainforge pack diff
domainforge pack sign
domainforge pack verify
```

CLI commands must call `sea-core` APIs. They must not reimplement parsing, lookup, alias resolution, pack merging, or diagnostic classification.

### 3.3 domainforge-lsp

Add these modules:

```text
src/semantic_config.rs
src/semantic_pack_loader.rs
src/semantic_diagnostics.rs
src/semantic_completion.rs
```

Modify these existing surfaces:

```text
src/backend.rs
src/completion.rs
src/hover/*
package.json configuration contribution, if VS Code extension config is present
```

The current LSP backend must remain a protocol adapter. It should load configuration, watch pack files, call `sea-core`, and translate `SemanticDiagnostic` to LSP diagnostics.

## 4. Core Data Structures

### 4.1 SemanticPack

Rust shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticPack {
    pub schema_version: String,
    pub pack_id: String,
    pub org_id: String,
    pub domain_id: String,
    pub pack_version: String,
    pub meaning_version: String,
    pub meaning_fingerprint: String,
    pub source_graph_hash: String,
    pub build_config_hash: String,
    pub review_manifest_hash: String,
    pub created_at: String,
    pub generator: GeneratorInfo,
    pub trust: PackTrust,
    pub concepts: Vec<ConceptDef>,
    pub relations: Vec<RelationDef>,
    pub metrics: Vec<MetricDef>,
    pub dimensions: Vec<DimensionDef>,
    pub units: Vec<UnitDef>,
    pub aliases: Vec<AliasDef>,
    pub mapping_rules: Vec<MappingRuleDef>,
    pub compatibility: CompatibilityInfo,
}
```

Rules:

- `schema_version` for v0.3 must be `0.3`.
- `pack_id` format: `<org_id>/<domain_id>/<pack_version>`.
- `meaning_version` is a manually managed SemVer string using `MAJOR.MINOR.PATCH`. It MUST be supplied explicitly by `pack build --meaning-version <semver>` and MUST NOT be inferred from `pack_version`.
- `meaning_fingerprint` is computed as `sha256(canonical_json(sorted_definition_records))`, where each record contains `subject_type`, `subject_id`, `definition_hash`, `status`, and `decision_ref` for every definition-bearing object.
- If a previous pack is supplied during build or diff, any changed `meaning_fingerprint` with an unchanged `meaning_version` MUST emit `meaning_version_not_bumped` and fail approved builds.
- If no previous pack is supplied, the builder MUST emit `meaning_version_baseline_missing` as a warning for candidate builds and as an error for approved builds unless `--allow-first-approved-version` is provided.
- `--allow-first-approved-version` is a bootstrap exception, not normal build behavior:
  - It MUST be accepted only as an explicit CLI flag or explicit test-harness API parameter.
  - It MUST NOT be loadable from workspace config, repository config, environment variables, LSP settings, semantic pack metadata, or downstream adapter config.
  - It MUST be valid only when no previously approved pack exists for the same `org_id` + `domain_id` pair in the configured pack registry, output directory, or explicit `--previous-pack` chain.
  - It MUST emit an audit sidecar field `first_approved_version_bypass_used=true` with `caller`, `pack_id`, `org_id`, `domain_id`, `meaning_version`, `review_manifest_hash`, and `reason`.
  - It MUST fail if an existing approved pack for the same `org_id` + `domain_id` is discoverable.
  - It MUST NOT be accepted by SEA-Forge, SWE_SEED, GodSpeed-Agent, Context Kernel, or production adapter configuration.
- `source_graph_hash`, `build_config_hash`, and `review_manifest_hash` are computed from canonical bytes.
- `trust` must record whether the pack is unsigned, signed, and approved.
- All vectors must follow the canonical array-ordering table in Section 6.1.1 before serialization.

### 4.2 PackTrust

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackTrust {
    pub approval_state: ApprovalState,
    pub signature_state: SignatureState,
    pub signed_by: Option<String>,
    pub signature_alg: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Candidate,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureState {
    Unsigned,
    Signed,
    InvalidSignature,
}
```

Strict CI may accept only:

```text
approval_state = approved
signature_state = signed
```

unless `--allow-unsigned-for-test-fixtures` is explicitly passed.

Unsigned fixture bypass rule:

- `allow_unsigned_test_fixtures` MUST NOT be loadable from workspace config, repository config, environment variables, LSP settings, or semantic pack metadata.
- The bypass MAY be provided only as an explicit CLI flag or test-harness API parameter in the same process invocation that consumes the fixture pack.
- When used, the validation result MUST include `unsigned_fixture_bypass_used=true`, `bypass_reason`, `caller`, and `pack_path`.
- The bypass MUST be rejected when `approval_state=approved` and the pack path is outside an allowed fixture root.
- The default allowed fixture roots are `fixtures/`, `tests/fixtures/`, and implementation-defined test temp directories.
- SEA-Forge, GodSpeed-Agent, SWE_SEED production adapters, and CI strict mode for non-fixture paths MUST treat the bypass as non-conforming.

### 4.3 ConceptDef

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptDef {
    pub id: String,
    pub canonical_name: String,
    pub kind: ConceptKind,
    pub status: ConceptStatus,
    pub definition: ConceptDefinition,
    pub owner: String,
    pub source_refs: Vec<SourceRef>,
    pub examples: Vec<String>,
    pub counterexamples: Vec<String>,
    pub allowed_predicates: Vec<String>,
    pub valid_contexts: Vec<String>,
}
```

`definition` is required. This is not optional in v0.3 because definitional drift cannot be controlled if the concept has no declared meaning.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptDefinition {
    pub text: String,
    pub definition_hash: String,
    pub decision_ref: String,
}
```

`definition_hash` is computed from normalized `text`, examples, counterexamples, and status. It must change when meaning changes.

### 4.4 AliasDef

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasDef {
    pub alias: String,
    pub normalized_alias: String,
    pub target_concept_id: String,
    pub status: AliasStatus,
    pub confidence: Option<DecimalString>,
    pub decision_ref: String,
    pub source_ref: SourceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AliasStatus {
    Approved,
    Deprecated,
    Ambiguous,
    Blocked,
}
```

Do not model alias status as a lookup tier. Resolution must collect all aliases with the same normalized key and decide over the complete candidate set.

### 4.5 SemanticTruth and DiagnosticSeverity

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTruth {
    Valid,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}
```

`SemanticTruth` is the epistemic result. `DiagnosticSeverity` is presentation/gating policy.

A diagnostic must never omit `semantic_truth`.

### 4.6 SemanticDiagnostic

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub code: SemanticDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub semantic_truth: SemanticTruth,
    pub message: String,
    pub source_ref: SourceRef,
    pub pack_ref: PackRef,
    pub suggestions: Vec<Suggestion>,
    pub recoverability_hint: String,
}
```

Required diagnostic codes:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiagnosticCode {
    UnknownConcept,
    DeprecatedConcept,
    AmbiguousConcept,
    AmbiguousAlias,
    AliasConflict,
    RejectedConcept,
    ProposedConcept,
    InvalidRelation,
    InvalidPredicate,
    UnknownMetric,
    UnknownDimension,
    UnknownUnit,
    UnitMismatch,
    DimensionMismatch,
    MissingDefinition,
    MissingOwner,
    DuplicateCanonicalName,
    DuplicateConceptId,
    UnreviewedConcept,
    PackUnavailable,
    PackSchemaMismatch,
    PackVersionMismatch,
    PackHashMismatch,
    PackUnsigned,
    PackSignatureInvalid,
    PackSetConflict,
    MappingRequired,
    MeaningVersionNotBumped,
    MeaningVersionBaselineMissing,
    AmbiguousAliasGroup,
}
```

### 4.7 SemanticValidationResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticValidationResult {
    pub status: SemanticValidationStatus,
    pub diagnostics: Vec<SemanticDiagnostic>,
    pub pack_set_ref: PackSetRef,
    pub input_hash: String,
    pub validation_mode: ValidationMode,
    pub expected_hashes: Vec<ExpectedHashCheck>,
    pub unsigned_fixture_bypass_used: bool,
    pub first_approved_version_bypass_used: bool,
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticValidationStatus {
    Passed,
    Failed,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedHashCheck {
    pub path: String,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackSetRef {
    pub merged_pack_hash: String,
    pub pack_refs: Vec<PackRef>,
}
```

`SemanticValidationResult` is the stable handoff object for CLI JSON, LSP tests, and downstream smoke consumers. Consumers MUST NOT infer validation outcome from CLI exit code alone.

Status computation:

- `blocked` when validation cannot run because parsing failed, required pack load failed, expected hash pin failed, required signature verification failed, pack-set conflict failed preflight, or required config is invalid.
- `failed` when validation runs and any diagnostic meets or exceeds the severity threshold for the current mode, regardless of `semantic_truth`. Example: `DeprecatedPolicy::ErrorInStrict` produces `semantic_truth=valid`, `severity=error`, CLI exit 1, and `status=failed`.
- `unknown` when validation runs, no severity-threshold failure exists, and at least one diagnostic has `semantic_truth=unknown`.
- `passed` when validation runs and all diagnostics are below the mode threshold and no diagnostic has `semantic_truth=unknown`.

Severity thresholds:

- LSP warn mode: diagnostics do not set `status=failed` unless pack preflight is blocked; documents can have warnings while the workspace semantic state remains usable.
- CLI warn mode: `severity=error` sets `status=failed`; warning/info/hint do not.
- CLI strict mode: `severity=error` sets `status=failed`; warning/info/hint do not unless a specific policy says strict warnings are fatal.

## 5. Bootstrapping and Review Gate

### 5.1 Source model coherence must run before pack build

`SemanticPackBuilder` must run pre-pack checks against source `.sea` graph data before producing an approved pack.

Pre-pack checks:

| Check | Diagnostic | Truth | Blocks approved pack |
|---|---|---:|---:|
| Duplicate concept ID | `duplicate_concept_id` | invalid | yes |
| Duplicate normalized canonical name targeting different IDs | `duplicate_canonical_name` | invalid | yes |
| Active concept without definition | `missing_definition` | unknown | yes |
| Active concept without owner | `missing_owner` | unknown | yes |
| Alias target missing | `unknown_concept` | invalid | yes |
| Deprecated alias without replacement | `deprecated_concept` | unknown | yes in strict |
| Relation endpoint missing | `invalid_relation` | invalid | yes |
| Metric references unknown dimension/unit | `unknown_dimension` or `unknown_unit` | unknown | yes |
| Concept introduced without review decision | `unreviewed_concept` | unknown | yes |
| Any alias key resolves to more than one distinct target concept across non-blocked alias statuses, except explicitly ambiguous-only alias groups | `alias_conflict` or `ambiguous_alias` | invalid or unknown | yes for conflict; no for explicit ambiguous-only groups |

Alias conflict rule:

- `AliasStatus::Blocked` entries are excluded from conflict detection.
- `AliasStatus::Approved` and `AliasStatus::Deprecated` entries are authority-bearing candidates. If the same normalized alias key targets more than one distinct concept through any approved/deprecated combination, the builder MUST emit `alias_conflict` and block approved builds.
- `AliasStatus::Ambiguous` entries are allowed to target multiple concepts only when every non-blocked entry for that normalized alias key is also `Ambiguous` and all entries share the same `decision_ref` or review decision group. In that case the builder MUST emit a build warning `ambiguous_alias_group` and validation MUST return `ambiguous_alias` with `semantic_truth=unknown`.
- Mixed ambiguous + approved/deprecated entries for the same normalized alias key MUST emit `alias_conflict` and block approved builds. Do not let an intentional ambiguity hide an active/deprecated authority path.

### 5.2 Review manifest is required for approved packs

A source model can generate a candidate pack without review. It cannot generate an approved pack without a review manifest.

Review file path convention:

```text
.domainforge/review/<org_id>.<domain_id>.semantic-review.jsonl
```

Review JSONL record:

```json
{
  "decision_id": "rev_2026_0001",
  "subject_type": "concept",
  "subject_id": "supplier",
  "decision": "approve",
  "rationale": "Supplier is the canonical procurement party providing goods or services.",
  "reviewer": "domain-owner@example.com",
  "reviewed_at": "2026-06-07T00:00:00Z",
  "definition_hash": "sha256:..."
}
```

Allowed `decision` values:

```text
approve
reject
deprecate
merge_into
split_required
needs_definition
needs_owner
minor_amendment_no_semantic_change
```

Build behavior:

- `pack build --approval candidate` does not require review decisions.
- `pack build --approval approved` requires matching review decisions for every active concept, relation, metric, dimension, unit, alias, and mapping rule.
- If the `definition_hash` in the review manifest does not match the current concept definition, the build must emit `unreviewed_concept` and fail approved mode unless a reviewer records `minor_amendment_no_semantic_change` that includes both `previous_definition_hash` and `new_definition_hash`.
- `minor_amendment_no_semantic_change` still requires a reviewer, rationale, timestamp, and decision ID. It is a lighter review path, not an automatic bypass.
- Any accepted minor amendment changes `review_manifest_hash` and therefore requires a new pack signature.

Operational cost note:

- Definition text, examples, counterexamples, and status transitions are authority-bearing. Even typo-only edits or `proposed -> active`, `active -> deprecated`, and `deprecated -> rejected` transitions can change `definition_hash` or `meaning_fingerprint` and force review/version handling.
- Implementations SHOULD provide tooling that summarizes definition-hash, status, and meaning-fingerprint changes before requiring re-review.
- Operators SHOULD expect review churn when definitions or concept statuses are edited. This is intentional: the pack is an authority artifact, not a casual cache.

### 5.3 Pack authority states

| State | How created | Can power LSP | Can power CI strict | Can power SEA-Forge authority |
|---|---|---:|---:|---:|
| candidate unsigned | `pack build --approval candidate` | yes, warning | no | no |
| approved unsigned | `pack build --approval approved` | yes, warning | no unless explicit fixture override | no |
| approved signed | `pack sign` after approved build | yes | yes | yes, if accepted by runtime trust config |
| rejected | review rejected | no | no | no |

## 6. Deterministic Serialization and Hashing

### 6.1 Canonical JSON

`canonical_json.rs` must implement canonical JSON serialization:

- UTF-8 output.
- Object keys sorted lexicographically.
- No insignificant whitespace.
- Timestamps normalized to RFC3339 UTC.
- Decimal strings preserved as strings, not binary floats.
- Arrays MUST follow the explicit ordering table below. Do not use generic "sort arrays if order is not semantic" behavior.

#### 6.1.1 Array ordering table

| Field | Ordering rule | Rationale |
|---|---|---|
| `concepts` | sort by `id` ascending | stable pack hash |
| `relations` | sort by `id` ascending, then predicate | stable pack hash |
| `metrics` | sort by `id` ascending | stable pack hash |
| `dimensions` | sort by `id` ascending | stable pack hash |
| `units` | sort by `id` ascending, then symbol | stable pack hash |
| `aliases` | sort by normalized alias key, then `target_concept_id`, then status | stable resolver input |
| `mapping_rules` | sort by `id` ascending | stable mapping behavior |
| `ConceptDefinition.examples` | normalize strings and sort lexicographically | examples are an unordered evidence set in v0.3 |
| `ConceptDefinition.counterexamples` | normalize strings and sort lexicographically | counterexamples are an unordered evidence set in v0.3 |
| `source_refs` | sort by URI, start byte, end byte | source order is not semantic |
| `suggestions` in diagnostics | sort by rank descending, then label ascending | deterministic UX and tests |
| `diagnostics` | sort by URI, start byte, code, message | deterministic CLI JSON |

If a future field is order-semantic, the schema MUST name it and canonicalization MUST preserve insertion order for that field. v0.3 has no order-semantic arrays in the pack hash path.

Do not hash pretty-printed JSON.

### 6.2 Pack hash

Pack hash input excludes `trust.signature` and `trust.signature_state` so the pack can be signed without changing the content hash.

```text
pack_content_hash = sha256(canonical_json(pack_without_signature_fields))
```

The emitted sidecar must contain:

```json
{
  "pack_id": "acme/procurement/0.1.0",
  "pack_content_hash": "sha256:...",
  "source_graph_hash": "sha256:...",
  "build_config_hash": "sha256:...",
  "review_manifest_hash": "sha256:...",
  "meaning_fingerprint": "sha256:..."
}
```

### 6.3 Determinism test

Required test:

```text
Build the same fixture pack 20 times.
Assert identical pack_content_hash every time.
Assert identical bytes for canonical JSON every time.
```

## 7. Signing

### 7.1 Signature algorithm

v0.3 must implement Ed25519 detached signing.

Command:

```bash
domainforge pack sign --pack <path> --key <private-key.pem>
```

Output:

```text
<pack>.sig
```

Signature payload:

```text
DOMAINFORGE_SEMANTIC_PACK_V0_3\n<pack_content_hash>\n<pack_id>\n<schema_version>\n```

### 7.2 Verification

Command:

```bash
domainforge pack verify --pack <path> --pubkey <public-key.pem>
```

Exit behavior:

| Case | Exit code | Diagnostic |
|---|---:|---|
| signature valid | 0 | none |
| signature missing | 3 | `pack_unsigned` |
| signature invalid | 4 | `pack_signature_invalid` |
| pack bytes changed after signing | 4 | `pack_hash_mismatch` |

LSP default behavior:

- Unsigned pack: warning diagnostic on workspace root and semantic features still load in `warn` mode.
- Invalid signature: semantic features disabled, diagnostic emitted.

CI strict behavior:

- Unsigned or invalid signature fails unless `--allow-unsigned-for-test-fixtures` is present.

## 8. Alias and Concept Resolution Algorithm

### 8.1 Normalization

Lookup normalization:

```text
normalize_lookup_key(s):
  unicode NFC
  trim leading/trailing whitespace
  collapse internal ASCII whitespace to single space
  case fold
```

Canonical IDs must match `[A-Za-z0-9._:/-]+`.

Canonical names preserve authored case.

### 8.2 Resolver input

```rust
pub struct ResolveRequest<'a> {
    pub raw_text: &'a str,
    pub expected_kind: Option<ConceptKind>,
    pub source_ref: SourceRef,
}
```

### 8.3 Resolver algorithm

The resolver must evaluate the full candidate set.

```text
1. key = normalize_lookup_key(raw_text)
2. candidates = []
3. Add exact concept ID matches.
4. Add canonical name matches.
5. Add alias matches.
6. Add mapping-rule matches when enabled for this context.
7. Remove candidates whose kind conflicts with expected_kind, but record them for suggestions.
8. If no candidates remain: return UnknownConcept, truth=unknown.
9. If any candidate has status=rejected or alias status=blocked: return RejectedConcept or AliasConflict, truth=invalid.
10. Group remaining candidates by target_concept_id.
11. If group count > 1: return AmbiguousAlias or AmbiguousConcept, truth=unknown.
12. Let winner be the single target concept.
13. If winner.status=proposed: return ProposedConcept, truth=unknown.
14. If winner.status=deprecated OR lookup matched deprecated alias: return DeprecatedConcept, truth=valid with warning severity in LSP warn mode; truth=valid with error severity only if config says deprecated terms block strict mode.
15. If winner.status=active and no conflicts: return valid.
```

Important: severity can change by mode; `semantic_truth` does not change.

### 8.4 Required alias conflict fixture

Fixture pack:

```json
{
  "concepts": [
    {"id":"supplier", "canonical_name":"Supplier", "status":"active"},
    {"id":"external_partner", "canonical_name":"ExternalPartner", "status":"active"}
  ],
  "aliases": [
    {"alias":"Vendor", "target_concept_id":"supplier", "status":"approved"},
    {"alias":"Vendor", "target_concept_id":"external_partner", "status":"deprecated"}
  ]
}
```

Expected result for `Vendor`:

```text
code = ambiguous_alias
semantic_truth = unknown
severity = error in strict, warning in LSP warn mode
```

It must not silently resolve to `supplier` because approved aliases are checked before deprecated aliases.

## 9. Three-Valued Semantic Behavior

### 9.1 Truth table

| Case | `semantic_truth` | Typical severity in LSP warn | Typical severity in CI strict |
|---|---|---|---|
| Known active concept | valid | none | none |
| Known deprecated alias | valid | warning | error or warning by config |
| Unknown concept | unknown | warning | error |
| Proposed concept | unknown | info/warning | error |
| Ambiguous alias | unknown | warning | error |
| Rejected concept | invalid | error | error |
| Invalid relation | invalid | error | error |
| Unit mismatch | invalid | error | error |
| Missing pack | unknown | warning | error |

### 9.2 Required invariant

Configuration may map `unknown` to error severity. It must not convert `semantic_truth=unknown` into `semantic_truth=invalid`.

Required unit test:

```text
Given unknown concept "Vendor"
When validate --mode strict --unknown-concept-policy error
Then diagnostic.semantic_truth == unknown
And diagnostic.severity == error
```

## 10. PackSet and Multi-Pack Merge

### 10.1 PackSet structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackSet {
    pub packs: Vec<PackRef>,
    pub precedence: Vec<String>,
    pub merged_pack_hash: String,
    pub conflicts: Vec<PackConflict>,
}
```

`PackRef`:

```rust
pub struct PackRef {
    pub pack_id: String,
    pub pack_content_hash: String,
    pub path_or_uri: String,
    pub priority: i32,
}
```

### 10.2 Merge order

Canonical merge order:

```text
1. explicit priority ascending
2. pack_id ascending
3. pack_content_hash ascending
```

The order of `semantic.packs` in config must not change validation results unless explicit priority differs.

### 10.3 PackSet hash

```text
merged_pack_hash = sha256(canonical_json({pack_refs_sorted, precedence_rules, conflict_policy}))
```

Diagnostics must report both:

```text
pack_ref.pack_content_hash
pack_set_ref.merged_pack_hash
```

### 10.4 Conflict policy

Default conflict policy is `error_on_conflict`.

Conflicts:

- Same concept ID, different definition hash.
- Same normalized canonical name, different concept ID.
- Same alias key, different target concept ID.
- Same relation predicate/endpoints with incompatible cardinality.
- Same unit symbol with incompatible dimension.

In LSP warn mode, conflicts disable semantic validation for affected lookup keys and emit `pack_set_conflict`.

In CI strict mode, conflicts fail validation before file validation begins.

## 11. Semantic Validation API

### 11.1 Public API

```rust
pub fn build_semantic_pack(input: PackBuildInput) -> Result<PackBuildOutput, SemanticPackError>;

pub fn validate_semantic_pack(pack: &SemanticPack) -> Result<Vec<SemanticDiagnostic>, SemanticPackError>;

pub fn validate_graph_with_pack(
    graph: &sea_core::Graph,
    source: &str,
    pack_set: &PackSet,
    options: ValidationOptions,
) -> SemanticValidationResult;

pub fn resolve_concept(
    request: ResolveRequest,
    pack_set: &PackSet,
    options: ResolveOptions,
) -> ResolveResult;
```

### 11.2 ValidationOptions

```rust
pub struct ValidationOptions {
    pub mode: ValidationMode,
    pub unknown_concept_policy: UnknownConceptPolicy,
    pub deprecated_policy: DeprecatedPolicy,
    pub require_signed_pack: bool,
    pub allow_unsigned_test_fixtures: bool,
}

pub enum ValidationMode {
    Off,
    Warn,
    Strict,
}

pub enum UnknownConceptPolicy {
    Ignore,
    Warning,
    Error,
}

pub enum DeprecatedPolicy {
    Allow,
    Warn,
    ErrorInStrict,
    ErrorAlways,
}
```

Deprecated policy semantics:

| Policy | LSP warn behavior | CLI warn behavior | CLI strict behavior | `semantic_truth` |
|---|---|---|---|---|
| `Allow` | no diagnostic | no diagnostic | no diagnostic | `valid` |
| `Warn` | warning | warning | warning, exit 0 unless other errors exist | `valid` |
| `ErrorInStrict` | warning | warning | error, exit 1 | `valid` |
| `ErrorAlways` | error | error, exit 1 | error, exit 1 | `valid` |

Default:

```text
LSP warn mode: DeprecatedPolicy::Warn
CLI warn mode: DeprecatedPolicy::Warn
CLI strict mode: DeprecatedPolicy::ErrorInStrict
```

No option is allowed to suppress `semantic_truth`.

### 11.3 Validation result status

`SemanticValidationResult.status` MUST be computed by the rules in Section 4.7. Status tracks whether the run is usable for the active validation mode, not only epistemic truth. `semantic_truth` remains the per-diagnostic epistemic value; `severity` and mode determine whether the validation run passes, fails, becomes unknown, or is blocked.

A run with only deprecated diagnostics under `DeprecatedPolicy::ErrorInStrict` MUST return:

```text
status = failed
semantic_truth = valid on each deprecated diagnostic
severity = error on each deprecated diagnostic
CLI exit code = 1
```

A run with only unresolved terms configured as warnings MUST return:

```text
status = unknown
semantic_truth = unknown on unresolved diagnostics
severity = warning
CLI exit code = 0 in warn mode, 1 in strict mode if policy upgrades unknowns to error
```

### 11.4 Exit codes

CLI exit codes:

| Code | Meaning |
|---:|---|
| 0 | passed |
| 1 | semantic validation failed |
| 2 | parse error |
| 3 | pack unavailable/unsigned when required |
| 4 | signature/hash failure |
| 5 | config/schema error |
| 6 | internal error |

## 12. CLI Commands

### 12.1 `pack build`

```bash
domainforge pack build \
  --source <glob-or-path>... \
  --domain <domain_id> \
  --org <org_id> \
  --version <pack_version> \
  --meaning-version <semver> \
  --previous-pack <pack-json> \
  --review <review-jsonl> \
  --approval candidate|approved \
  --out <pack-json>
```

Behavior:

- Parses source files with core parser.
- Runs pre-pack checks.
- If `--approval approved`, requires review decisions.
- Emits canonical pack JSON and hash sidecar.
- Computes `meaning_fingerprint`.
- Fails approved build when `--previous-pack` is missing unless `--allow-first-approved-version` is present.
- Fails approved build when `meaning_fingerprint` changed and `meaning_version` did not increase relative to `--previous-pack`.
- Fails if output would be nondeterministic.

### 12.2 `pack validate`

```bash
domainforge pack validate \
  --pack <pack-json> \
  --mode off|warn|strict \
  --deprecated-policy allow|warn|error-in-strict|error-always \
  --require-signature \
  --expected-hash <sha256:...> \
  --format human|json|jsonl \
  <input.sea>...
```

Behavior:

- Parses `.sea` inputs.
- Loads and verifies pack by configured trust options.
- Emits `SemanticValidationResult`.
- In strict mode, exits nonzero on severity `error`.

### 12.3 `pack inspect`

```bash
domainforge pack inspect --pack <pack-json> --format human|json
```

Must show:

```text
pack_id
schema_version
pack_version
meaning_version
approval_state
signature_state
source_graph_hash
review_manifest_hash
concept count by kind/status
alias count by status
relation count
metric/unit/dimension count
```

### 12.4 `pack diff`

```bash
domainforge pack diff --old <pack-json> --new <pack-json> --format human|json
```

Diff classifications:

```text
additive
definitional_change
deprecating
breaking
governance_critical
signature_only
```

Breaking changes:

- Removed active concept.
- Changed concept definition hash without meaning-version bump.
- Changed `meaning_fingerprint` without increasing `meaning_version`.
- Removed approved alias without deprecation window.
- Changed unit dimension for existing unit symbol.
- Changed relation cardinality incompatibly.

### 12.5 `pack sign` / `pack verify`

Specified in Section 7.

## 13. LSP Implementation

### 13.1 Config extension

Add semantic config to `DomainForgeConfig`:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainForgeConfig {
    #[serde(default)]
    pub formatting: FormattingConfig,
    #[serde(default)]
    pub semantic: SemanticConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub packs: Vec<SemanticPackConfig>,
    #[serde(default = "default_lsp_validation_mode")]
    pub validation_mode: ValidationMode,
    #[serde(default = "default_lsp_unknown_policy")]
    pub unknown_concept_policy: UnknownConceptPolicy,
    #[serde(default = "default_lsp_deprecated_policy")]
    pub deprecated_policy: DeprecatedPolicy,
    #[serde(default)]
    pub require_signature: bool,
}
```

`SemanticPackConfig`:

```rust
pub struct SemanticPackConfig {
    pub path: String,
    #[serde(default)]
    pub priority: i32,
    pub expected_hash: Option<String>,
}
```

`expected_hash` is a pin, not advisory metadata.

- If `expected_hash` is present and does not equal the loaded pack `pack_content_hash`, the pack MUST NOT be loaded.
- LSP warn mode MUST emit workspace diagnostic `pack_hash_mismatch`, keep the last-known-good matching pack if available, and otherwise degrade to syntax-only validation with `SemanticValidationResult.status=blocked` for semantic validation.
- CI strict mode MUST fail before validating input files with `SemanticValidationResult.status=blocked` and CLI exit code `4`.
- Multi-pack mode MUST exclude the mismatched pack from `PackSet`; it MUST NOT silently continue with an unpinned replacement.
- The validation result MUST include `expected_hash`, `actual_hash`, and `path` in `expected_hashes` for auditability.

### 13.2 Backend state changes

Add to `Backend`:

```rust
semantic_pack_set: RwLock<Option<PackSet>>,
semantic_pack_errors: RwLock<Vec<SemanticDiagnostic>>,
```

On initialize and config change:

```text
1. Read semantic config.
2. Load packs.
3. Verify schema/hash/signature according to mode.
4. Build PackSet.
5. Cache last-known-good PackSet.
6. Publish workspace-level diagnostic if semantic pack unavailable.
```

### 13.3 validate_document behavior

Current parse-only behavior must become layered validation:

```text
1. Parse document using sea-core.
2. If parse fails, publish parse diagnostic and stop semantic validation for that document.
3. If parse succeeds and semantic.enabled=false, publish no semantic diagnostics.
4. If parse succeeds and pack unavailable in warn mode, publish pack unavailable diagnostic.
5. If parse succeeds and pack active, call validate_graph_with_pack.
6. Convert SemanticDiagnostic[] to LSP Diagnostic[].
7. Preserve semantic code in Diagnostic.code.
8. Preserve semantic_truth and pack hash in Diagnostic.data.
```

LSP diagnostic `data` payload:

```json
{
  "domainforge": {
    "semantic_code": "unknown_concept",
    "semantic_truth": "unknown",
    "pack_id": "acme/procurement/0.1.0",
    "pack_hash": "sha256:...",
    "merged_pack_hash": "sha256:..."
  }
}
```

### 13.4 Completion behavior

If semantic pack is active:

- Completion for concept positions must include active concepts matching expected kind.
- Deprecated aliases may appear only when `semantic.includeDeprecatedCompletion=true`; default false.
- Proposed and rejected concepts must not appear as normal completion suggestions.

### 13.5 Hover behavior

If symbol resolves to pack concept:

Show:

```text
canonical_name
kind
status
definition text
owner
pack_id
meaning_version
source refs
```

If no pack is active, existing hover behavior remains.

## 14. Source References and Ranges

Semantic diagnostics must point to the smallest useful range:

- Unknown concept: the string literal or identifier that failed resolution.
- Invalid relation: the predicate span if available, else the relation declaration span.
- Unit mismatch: the unit literal span.
- Pack-level errors: workspace root URI or virtual URI `domainforge://semantic-pack/<pack_id>`.

`SourceRef` shape:

```rust
pub struct SourceRef {
    pub uri: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}
```

## 15. Pack Version Lifecycle

### 15.1 Concept lifecycle

Concept statuses:

```text
active
proposed
deprecated
rejected
external_only
```

Rules:

- `proposed` validates as `semantic_truth=unknown`.
- `rejected` validates as `semantic_truth=invalid`.
- `deprecated` validates as `semantic_truth=valid` with diagnostic severity based on mode.
- Removed concepts are breaking unless previous pack marked them deprecated and the deprecation window has expired.

### 15.2 Pack lifecycle

Pack states:

```text
candidate
approved_unsigned
approved_signed
retired
revoked
```

Pack compatibility fields:

```rust
pub struct CompatibilityInfo {
    pub domainforge_min_version: String,
    pub domainforge_max_version: Option<String>,
    pub lsp_min_version: Option<String>,
    pub deprecated_after: Option<String>,
    pub retired_after: Option<String>,
    pub replaces_pack_ids: Vec<String>,
}
```

CI strict must fail on `retired` or `revoked` pack unless `--allow-retired-pack` is explicitly passed for migration tests.

## 16. Context Kernel Consumer Contract

Context Kernel integration is not a v0.3 completion requirement, but the pack must expose a mapping API contract.

Mapping statuses:

```text
known
mapped
ambiguous
unknown
proposed
rejected
```

Only `known` and `mapped` are approved for handoff into tests, policies, authority checks, or settlement claims.

`ambiguous`, `unknown`, and `proposed` must produce `semantic_truth=unknown`.

`rejected` must produce `semantic_truth=invalid`.

Consumer result shape:

```json
{
  "term": "vendor",
  "status": "mapped",
  "semantic_truth": "valid",
  "canonical_concept_id": "supplier",
  "pack_id": "acme/procurement/0.1.0",
  "pack_hash": "sha256:...",
  "evidence": {
    "mapping_rule_id": "map_vendor_supplier",
    "decision_ref": "rev_2026_0007"
  }
}
```

No runtime consumer may treat `unknown` as approved merely because the LSP used warning severity.

## 17. Fixtures

Commit fixtures under:

```text
fixtures/semantic_packs/acme_procurement/
```

Required files:

```text
domain/entities.sea
domain/resources.sea
domain/relations.sea
domain/metrics.sea
review/acme.procurement.semantic-review.jsonl
packs/expected/acme.procurement.semantic-pack.json
tests/valid_purchase_policy.sea
tests/unknown_vendor_policy.sea
tests/deprecated_vendor_alias.sea
tests/ambiguous_vendor_alias.sea
tests/invalid_relation.sea
tests/unit_mismatch.sea
tests/proposed_concept.sea
tests/rejected_concept.sea
```

Minimum domain concepts:

```text
Supplier
PurchaseOrder
MaterialLot
InspectionRecord
CorrectiveAction
InventoryItem
CashRegister
```

Minimum aliases:

```text
Vendor -> Supplier, deprecated
PO -> PurchaseOrder, approved
Provider -> Supplier, ambiguous if ExternalPartner fixture enabled
```

## 18. Required Tests

### 18.1 Core unit tests

- `build_is_deterministic_across_repeated_runs`
- `approved_pack_requires_review_manifest`
- `definition_hash_must_match_review_manifest`
- `missing_definition_blocks_approved_pack`
- `missing_owner_blocks_approved_pack`
- `duplicate_canonical_name_is_invalid`
- `alias_resolution_uses_full_candidate_set`
- `approved_and_deprecated_alias_same_key_blocks_approved_build`
- `approved_and_deprecated_alias_same_key_is_ambiguous`
- `deprecated_policy_error_in_strict_preserves_valid_truth`
- `unknown_truth_survives_error_severity`
- `rejected_concept_is_invalid_not_unknown`
- `pack_hash_excludes_signature_fields`
- `signature_verifies_content_hash`
- `signature_payload_pack_id_must_match_pack_field`
- `signature_payload_schema_version_must_match_pack_field`
- `tampered_pack_fails_signature_verification`
- `pack_set_hash_is_independent_of_config_array_order`
- `pack_set_conflict_blocks_strict_mode`
- `meaning_fingerprint_change_requires_meaning_version_bump`
- `canonical_json_sorts_examples_and_counterexamples`
- `expected_hash_mismatch_blocks_pack_load`
- `unsigned_fixture_bypass_is_cli_only_and_audited`

### 18.2 CLI tests

- `pack_build_writes_pack_and_hash_sidecar`
- `pack_validate_strict_fails_unknown_concept`
- `pack_validate_warn_returns_zero_with_warning`
- `pack_validate_json_output_matches_schema`
- `pack_diff_classifies_definition_change`
- `pack_verify_fails_unsigned_when_required`
- `pack_validate_expected_hash_mismatch_exits_four`
- `pack_build_approved_requires_previous_pack_or_first_version_override`

### 18.3 LSP tests

- `lsp_loads_pack_from_workspace_config`
- `lsp_reports_same_code_as_cli_for_unknown_concept`
- `lsp_reports_semantic_truth_in_diagnostic_data`
- `lsp_keeps_last_known_good_pack_after_invalid_reload`
- `lsp_expected_hash_mismatch_degrades_to_last_known_good_or_syntax_only`
- `lsp_completion_uses_active_pack_concepts`
- `lsp_hover_shows_pack_definition`

### 18.4 Consumer smoke test

One non-LSP consumer test must parse CLI JSON and reject a failing result:

```text
Given pack validate returns status=failed with code=unknown_concept
When consumer reads JSON result
Then consumer refuses proof/completion claim
And records pack_id + pack_hash
```

This can live in a small test harness if SWE_SEED or SEA-Forge integration is not ready.

## 19. Acceptance Criteria

v0.3 is accepted only when all of these are true:

1. `sea-core` exposes the semantic pack API listed in Section 11.
2. CLI pack commands exist and pass fixture tests.
3. Approved pack build fails without review manifest.
4. Approved signed pack verifies successfully.
5. Tampered pack fails verification.
6. Unknown concepts produce `semantic_truth=unknown` in all modes.
7. Invalid relations produce `semantic_truth=invalid`.
8. Alias conflict fixture produces `ambiguous_alias`, not silent resolution.
9. Multi-pack config order does not alter merged pack hash or diagnostics.
10. LSP emits semantic diagnostics with same code and truth as CLI for the committed fixtures.
11. LSP does not duplicate semantic validation logic outside `sea-core`.
12. One consumer smoke test proves downstream tools can use CLI JSON or shared schema without depending on LSP internals.
13. DeprecatedPolicy is defined and covered by unit tests.
14. Expected-hash mismatch prevents pack load in LSP and fails CI strict.
15. Unsigned fixture bypass is CLI/test-harness only and is recorded in validation output.
16. Signature verification fails when payload pack ID or schema version do not match pack fields.
17. Meaning-fingerprint changes require meaning-version increase when a previous pack is supplied.
18. Canonical JSON array ordering is covered by tests for examples and counterexamples.
19. Documentation includes command examples, config examples, diagnostic codes, trust states, and migration behavior.

## 20. Non-Conformance Conditions

Do not mark v0.3 complete if any are true:

- Pack generation works only in the LSP.
- The same fixture produces different diagnostic codes in CLI and LSP.
- `semantic_truth` is optional or missing from diagnostics.
- Unknown concepts are converted to invalid internally instead of only changing severity.
- Approved packs can be built without review decisions.
- Strict CI accepts unsigned packs by default.
- Alias resolution uses first-match precedence over full candidate-set evaluation.
- Pack hash changes when the same pack is signed.
- PackSet validation depends on config array order without explicit priority.
- Downstream consumers must link to `domainforge-lsp` to validate semantics.
- `DeprecatedPolicy` is left implementation-defined.
- `expected_hash` mismatch is treated as advisory.
- `allow_unsigned_test_fixtures` can be set from workspace config or omitted from evidence output.
- Approved packs can contain alias keys that resolve to multiple target concepts.
- Signature verification ignores signed payload `pack_id` or `schema_version`.
- Definition hashes change without either re-review, minor-amendment review, or meaning-version enforcement.

## 21. Implementation Sequence

### Phase 1 — Core schema and deterministic pack builder

Deliverables:

```text
SemanticPack schema
canonical JSON
pack hash
pre-pack checks
candidate pack build
approved pack review gate
builder tests
```

Proof command:

```bash
cargo test -p sea-core semantic_pack_build
```

### Phase 2 — Resolver and validator

Deliverables:

```text
full candidate-set resolver
SemanticTruth-required diagnostics
relation/metric/unit validation
PackSet merge and conflict handling
validator tests
```

Proof command:

```bash
cargo test -p sea-core semantic_pack_validate semantic_pack_alias_resolution semantic_pack_three_valued semantic_pack_pack_set
```

### Phase 3 — CLI

Deliverables:

```text
pack build
pack validate
pack inspect
pack diff
pack sign
pack verify
JSON output schema
CLI integration tests
```

Proof command:

```bash
cargo test -p domainforge-cli pack_
```

Adjust crate name to the actual CLI package.

### Phase 4 — LSP

Deliverables:

```text
semantic config
pack loader
last-known-good pack cache
semantic diagnostics
pack-driven completion
pack-driven hover
LSP integration tests
```

Proof command:

```bash
cargo test -p domainforge-lsp semantic_
```

### Phase 5 — Consumer smoke

Deliverables:

```text
machine-readable validation result consumed outside LSP
proof/completion refusal on semantic failure
pack id/hash recorded
```

Proof command:

```bash
cargo test semantic_pack_consumer_smoke
```

## 22. Configuration Examples

### 22.1 Workspace config

`.domainforge/config.json`:

```json
{
  "semantic": {
    "enabled": true,
    "validationMode": "warn",
    "unknownConceptPolicy": "warning",
    "deprecatedPolicy": "warn",
    "requireSignature": false,
    "packs": [
      {
        "path": ".domainforge/packs/acme.procurement.semantic-pack.json",
        "priority": 10,
        "expectedHash": "sha256:example"
      }
    ]
  }
}
```

### 22.2 CI config

```bash
domainforge pack validate \
  --pack .domainforge/packs/acme.procurement.semantic-pack.json \
  --mode strict \
  --deprecated-policy error-in-strict \
  --require-signature \
  --expected-hash sha256:<pinned-pack-content-hash> \
  --format json \
  domain/**/*.sea tests/**/*.sea
```

## 23. Documentation Requirements

Add docs:

```text
docs/semantic-packs.md
docs/semantic-pack-review.md
docs/semantic-pack-signing.md
docs/lsp-semantic-adapters.md
docs/diagnostics.md
```

Documentation must include:

- What a semantic pack is.
- What it is not.
- Candidate vs approved vs signed pack states.
- Review manifest format.
- Alias resolution rules.
- Three-valued truth behavior.
- CLI examples.
- LSP config examples.
- Pack version migration rules.
- Diagnostic code table.

## 24. Open Questions Deferred Past v0.3

These are intentionally outside v0.3:

- Remote pack registry.
- Pack marketplace.
- Embedding-assisted mapping suggestions.
- Full Context Kernel ingestion enforcement.
- SEA-Forge runtime policy-gateway enforcement using pack signatures.
- UI for domain review workflows.
- Cross-organization semantic federation.

Do not block v0.3 on these.

## 25. v0.3 Definition of Done

v0.3 is done when a developer can clone the repos, run the fixture command sequence in Section 0, open fixture files in an LSP-compatible editor, and see the same semantic truth values and diagnostic codes that CI sees.

The settlement target is not “we described semantic adapters.”

The settlement target is:

```text
A signed, review-gated semantic pack changes CLI and LSP behavior deterministically,
blocks out-of-domain vocabulary in strict mode,
preserves unknown vs invalid semantics,
and can be consumed by at least one non-LSP test harness.
```
