# DomainForge Conversational Application Generator

Status: Proposed v0.2 (adversarially revised)

Owner: GodSpeed AI / DomainForge maintainers

Scope: a human-in-the-loop authoring workflow and a deterministic DomainForge
compiler path that can generate and prove a bounded, runnable application from
approved SEA semantics and approved implementation bindings.

This document replaces the v0.1.1 proposal. It keeps the intended product
outcome but removes claims that the current grammar, Domain IR, artifact sink,
provider system, and proof harness cannot yet support.

## 0. Normative language and reading order

The terms MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, MAY, and OPTIONAL have
the meanings defined by RFC 2119.

Read Sections 1–5 before implementing anything. They define the current
codebase boundary, the product outcome, the first conformance slice, and the
semantic gate that prevents DomainForge from inventing application behavior.

## 1. Outcome and product promise

### 1.1 Job to be done

A domain expert describes a need in ordinary language. An agent helps the expert
produce a small, valid, human-reviewable SEA model. The expert approves what the
model means. The agent then helps select implementation bindings from a bounded
catalog. The expert approves the operational consequences. DomainForge, not the
agent, generates and verifies the application.

The product promise is:

> The agent helps a domain expert declare the application; DomainForge builds
> and proves the supported parts and reports every unsupported part.

The promise is satisfied only when the person can:

1. understand what the application will do and will not do;
2. correct the model without reading generated code;
3. see implementation consequences before generation;
4. reproduce generation from checked-in inputs and locks;
5. run a real end-to-end use case;
6. distinguish verified behavior from skipped, blocked, or unsupported behavior;
7. regenerate safely after a change and inspect the semantic and artifact diff.

### 1.2 Success measures

The initial user experience MUST meet these measures in moderated fixture runs:

- a domain reviewer can identify the actors, use cases, information stored,
  policies applied, external effects, assumptions, and omissions from the review
  bundle without reading Rust, IR JSON, or provider YAML;
- every approval screen fits a progressive-disclosure flow: a one-page decision
  summary links to detailed artifacts instead of presenting an undifferentiated
  compiler dump;
- the workflow never converts an unresolved consequential question into a
  generated default;
- a reviewer can answer “what changed since I last approved?” from a generated
  semantic or realization diff;
- a developer can reproduce a verified local application using commands printed
  in the handoff, without relying on the original agent conversation;
- an operator can identify runtime configuration and secret references without
  editing generated source.

Numeric model-cost and correction thresholds belong to a pre-registered
benchmark, not to v0.1 conformance. Section 20 defines that later claim.

### 1.3 Non-goals

The system is not:

- an autonomous coding agent;
- permission to emit an adapter when no supported semantic or provider mapping
  exists;
- a hidden low-code runtime or a compile-time dependency on SEA-Forge;
- a public package recommender or dynamic plugin marketplace;
- a production deployment engine;
- a database upgrade planner for populated databases;
- a graphical application/UI generator in the first release;
- proof that a human approval is cryptographically attributable to a person;
- a claim of general application coverage.

Generated organizational projections remain useful review artifacts. They do
not prove runtime behavior unless a conformance test connects the projection to
the generated behavior.

## 2. Repository-grounded baseline

Implementation MUST begin from these verified repository facts and re-check
them when work starts:

| Area | Current fact | Evidence | Consequence |
|---|---|---|---|
| Workspace | The Rust workspace contains only `domainforge-core`; Python and TypeScript are binding/package surfaces, not workspace crates. | `Cargo.toml`, `domainforge-core/Cargo.toml` | Prefer modules in the existing crate until isolation has a measured benefit. |
| CLI | `domainforge` exposes `parse`, `validate`, `import`, `project`, `format`, `test`, `validate-kg`, `normalize`, `registry`, `authority`, and `pack`. | `domainforge-core/src/cli/mod.rs`, `src/bin/domainforge.rs` | Application orchestration is a new command group; do not pretend it already exists. |
| Projection pattern | ADR-011 requires one target IR, pure renderer, `ArtifactSink`, in-memory surface, proving fixture, determinism test, and native verification. It also rejects a dynamic plugin framework without a concrete third-party need. | `docs/specs/ADR-011-operator-backed-projection-families.md` | Adapter inspection follows this shape. Application orchestration may compose projections but must not bypass them. |
| Domain IR | `DomainIr` maps SEA elements into DDD/CQRS code. It infers non-identity entity fields from instances as strings, creates generic repository/bus/read-model ports, and applies every policy as a no-op guard to every aggregate method. It is not currently a serialized, hashed public contract. | `domainforge-core/src/projection/domain/ir.rs`, `domain/{rust,python,typescript}.rs` | It is insufficient as the sole source for HTTP DTOs, persistence schemas, policy placement, transactions, idempotency, or lifecycle behavior. |
| Grammar | SEA models entities, resources, flows, instances, policies, metrics, mappings, projections, and Cell environment declarations. It has no canonical typed application/use-case contract. | `domainforge-core/grammar/sea.pest`, `parser/ast.rs` | A grammar/AST/Graph/IR settlement is a prerequisite, not an emitter detail. |
| Cell boundary | Cell declarations are parsed from AST because graph construction drops required environment context. Cell already owns environment, dependencies, network, credentials, authority/evidence, unsafe overrides, and `cell.lock`. | `projection/cell/`, `cli/project.rs` | Application generation must compose with Cell and must not reinterpret Cell declarations from an incomplete Graph. |
| Artifact writes | `ArtifactSink` writes UTF-8 strings to a directory or `BTreeMap` and delegates path validation. It does not itself reject duplicate writes, enforce a predeclared plan, stage atomically, preserve modes, or support binary artifacts. | `projection/sink.rs` | The application pipeline needs a stricter planned/staged sink before it can claim safe regeneration. |
| Hashes | The repository has canonical JSON and SHA-256 helpers for semantic packs, projection IDs based on deterministic xxh64, several family-specific model hashes, and a Cell source/file lock. There is no single canonical semantic-closure hash. | `semantic_pack/canonical_json.rs`, `projection/ids.rs`, `projection/cell/lock.rs`, `projection/ai_learning/mod.rs` | This feature must define one application-specific hash contract and must not relabel an existing family hash as universal. |
| Trust | Semantic-pack signing and registry trust exist. Provider catalogs do not. | `src/semantic_pack`, `src/registry`, `docs/semantic-pack-signing.md` | Reuse primitives where appropriate, but keep semantic packs and provider packs as distinct types and namespaces. |
| Security | Untrusted SEA input is data; parser/import/archive limits and no-process-execution rules are documented. | `docs/reference/security-model.md` | Provider bindings must preserve this boundary. |
| Generated artifacts | Only fixtures/goldens may be checked in as generated output. | `docs/reference/generated-artifacts-policy.md` | Normal application bundles are outputs; source inputs, locks, schemas, and test fixtures are authoritative. |

Current facts are evidence, not permanent design constraints. A change to a
fact in this table MUST update this section and the affected acceptance tests.

## 3. Adversarial findings and required resolutions

The prior proposal contained the following blocking gaps. Each resolution is
normative for this revision.

| Severity | Finding / debt | Required resolution |
|---|---|---|
| Critical | The proposal derived rich API, persistence, transaction, authority, failure, and lifecycle semantics from a Domain IR that does not contain them. | Add the Semantic Expressiveness Gate in Section 5. No provider or emitter may guess missing semantics. |
| Critical | Provider packs mixed declarative metadata with lowering rules, templates, and emitters, creating an arbitrary-code execution boundary that contradicted the threat model. | v0.1 provider descriptors are data-only and may reference only compiled, allowlisted emitter IDs. External executable packs are roadmap. |
| Critical | `ArtifactSink` was credited with collision, planned-write, and atomicity properties it does not have. | Add `PlannedArtifactSink` semantics and stage/commit rules in Section 13 before complete-app generation. |
| Critical | “Every policy guards every method” and no-op policy hooks could expose unsafe generated endpoints while appearing governed. | An operation is generatable only when applicable policies and enforcement mode are explicit and proven; otherwise it is blocked or generated without that operation. |
| Required | v0.1 bundled grammar work, two production profiles, many provider families, a generic dependency solver, skills, three domains, production integrations, and a research benchmark. | Use staged conformance levels. v0.1 proves one local vertical slice; provider substitution and production move later. |
| Required | The source/semantic hash ignored import closure, namespace resolution, semantic-pack inputs, compiler interpretation, and canonicalization version. | Define `semantic_closure_hash` over a typed canonical semantic envelope in Section 10. |
| Required | Approval manifests were replayable labels and did not bind the complete reviewed context. | Bind semantic approval to the semantic closure and review bundle; bind realization approval to requirements, binding, profile, provider catalog snapshot, and review bundle. Revalidate immediately before plan and commit. |
| Required | “Run state is derived from files” did not define how forged, partial, or stale files are rejected. | Every derived artifact includes schema version, input hashes, producer version, and self-hash; status is derived only after validation of the chain. |
| Required | Interrupted-run cleanup could discard evidence or race with an active writer. | Use an owner-token staging lease. Never auto-delete a live or unverifiable staging directory; provide inspect/clean commands. |
| Required | Proof commands could be pack-authored shell, leak secrets, run without bounds, or mutate external systems. | Commands come from compiled profiles/providers, use tokenized argv, explicit environment allowlists, redaction, time/output/resource limits, and disposable resources. |
| Required | Byte-identical proof records were conflated with inherently variable runtime observations. | The proof plan is deterministic; the proof record is an observation with normalized stable fields and explicitly variable timing/log attachments. |
| Required | Generated-file pruning was automatic and destructive. | Default regeneration previews orphaned files and refuses drift. Deletion requires `--prune` and succeeds only for files whose prior hash still matches. |
| Required | Native dependency lock generation, offline regeneration, and provider resolution were conflated. | v0.1 profiles pin contributions; native lockfiles are produced by a declared toolchain step, captured in the application lock, and may require a populated cache. Offline capability is reported, not assumed. |
| Required | Existing Cell endpoints/network declarations were treated as if ordinary domain flows implied network access. | Network access comes from the selected profile, provider bindings, and explicit Cell declarations; a domain `Flow` alone never opens a network path. |
| Required | All existing CLI commands were implicitly required to gain one JSON envelope. | Stable JSON envelopes and exit codes apply to new `application` and `provider` commands. Existing commands evolve separately. |
| Required | “Complete application” had no bounded behavioral definition. | Section 4 defines the exact v0.1 path and proof. |
| Required | Review UX listed artifacts but not decision burden, omission handling, or change review. | Section 8 defines progressive summaries, consequential-question gates, approval language, and diffs. |
| Required | A generic provider/dependency solver was specified before a second real provider established the problem shape. | v0.1 uses exact provider selection and deterministic validation. Ranking/composition become required only at substitution conformance. |
| Required | The generated schema migration promise could be mistaken for safe upgrades. | v0.1 creates fresh-database migrations only and emits a blocking deployment residue on persistent-schema change. |
| Improvement | Users lacked a cheap readiness check before approval/generation. | Add `application inspect`, `application diff`, `application doctor`, and `--dry-run`/plan previews. |
| Improvement | Handoffs lacked a first-use path. | Generate a redacted `.env.example`, exact run/test commands, health check, and copyable sample request/response. |
| Improvement | Provider rejection explanations were secondary. | Resolution output records selected, rejected, and unavailable options with plain-language reasons. |
| Improvement | The spec did not cap input, artifacts, logs, or proof work. | Section 17 requires configurable ceilings and bounded capture. |

## 4. Conformance levels and release boundary

### 4.1 Foundation conformance

Foundation conformance proves the compiler has enough explicit meaning to plan
an application. It MUST deliver:

- the grammar/AST/Graph/API-contract settlement in Section 5;
- a canonical semantic envelope and `semantic_closure_hash`;
- serializable, schema-versioned Domain/Application/Adapter IR inspection;
- actionable diagnostics for missing application semantics;
- domain review and semantic diff artifacts;
- no concrete adapter or complete-application claim.

### 4.2 v0.1 local application conformance

v0.1 supports exactly one reference profile, `rust-local-v1`:

- one SEA namespace and import closure;
- explicit typed application records and explicit use-case operations;
- synchronous JSON/HTTP inbound transport through a compiled Rust HTTP emitter;
- one SQLite database through a compiled Rust persistence emitter;
- fresh-database migrations;
- one transaction boundary per declared write use case;
- structured local tracing/logging;
- generated Cell/environment output where the model contains a valid Cell;
- no runtime secrets for the default fixture;
- generated unit tests and a real HTTP-to-SQLite integration test;
- persistence verified across process restart;
- deterministic plan and source regeneration;
- no handwritten edits inside generated zones.

The flagship fixture MUST contain at least:

- one command-style write use case;
- one query-style read use case;
- typed request validation;
- one explicitly scoped policy or an explicit declaration that the operation is
  public and unguarded;
- one expected conflict or invalid-input path;
- a health endpoint supplied by the profile, clearly marked as profile-owned.

“Complete” in v0.1 means complete for that declared slice. It does not mean
authentication, a browser UI, messaging, cloud deployment, multi-database
transactions, arbitrary external APIs, or production database upgrades.

The profile's projection matrix is explicit:

| Projection/review artifact | `rust-local-v1` status |
|---|---|
| domain review, policy-to-operation table, semantic/realization diff | required |
| RDF semantic graph | required for graph-supported declarations |
| authority/evidence contracts | required when an operation declares authority/evidence |
| OpenTelemetry semantic conventions | required when runtime observation obligations are declared; otherwise optional |
| Cell environment | required when a valid Cell is declared; otherwise not applicable |
| BPMN | optional review aid; becomes required only in a later profile that supports ordered-process execution |
| CMMN | not applicable to the v0.1 application slice |
| ArchiMate | optional review aid |

The output manifest records `required`, `optional`, or `not_applicable` and the
reason for every known projection. A missing required projection prevents a
verified claim; an optional projection failure remains visible in residue.

### 4.3 Substitution conformance (v0.2 target)

Substitution conformance adds a second relational provider (PostgreSQL) for the
same Adapter IR and proves:

- no SEA semantic change;
- provider and operational review changes are visible;
- both providers pass one family conformance suite;
- selection is deterministic from a fixed catalog snapshot;
- incompatible requirements produce a provider gap, not handwritten fallback.

Only at this level are provider ranking, version-range resolution, and generic
dependency conflict resolution REQUIRED.

### 4.4 Production-profile conformance (later)

A production Rust profile may add PostgreSQL, external messaging, outbox,
OpenTelemetry export, identity, secret providers, and disposable-container fault
tests. Each capability requires its own provider conformance evidence. Merely
listing NATS, OIDC, or a cloud service in a profile does not satisfy this level.

### 4.5 General capability claim (research, later)

Three domains, modest-model comparisons, provider reuse, and direct coding-agent
baselines are benchmark claims under Section 20. They do not gate v0.1 and MUST
not appear in product copy before the pre-registered benchmark passes.

## 5. Semantic Expressiveness Gate

### 5.1 Rule

DomainForge MUST NOT generate an application operation unless canonical SEA
meaning explicitly determines every consequential item needed by that operation.
Provider bindings choose realization; they do not supply missing domain meaning.

An operation is `generation_ready` only when the compiler can determine:

- stable operation identity and plain-language intent;
- inbound, outbound, or internal direction;
- actor/role and whether access is public, authenticated, or policy-governed;
- typed input and output records, required/optional fields, constraints, and
  cardinality;
- affected aggregate/state and effect kind;
- applicable policies and their enforcement point;
- transaction boundary;
- expected canonical failures;
- idempotency/concurrency behavior when duplicate or concurrent execution can
  cause a harmful effect;
- evidence/observation obligations;
- externally visible lifecycle behavior when relevant.

If any consequential field is absent, the compiler MUST emit
`APP001 missing_application_semantics` with the operation/source ref, missing
fields, why each matters, and the smallest supported remediation. It MUST NOT
fill these fields from provider defaults, method names, fixture instances, or
language-model guesses.

### 5.2 Required language settlement

Before Foundation conformance, maintainers MUST approve a grammar-first ADR and
language spec that introduces or reuses SEA syntax for a minimal Application
Contract. The exact syntax is intentionally not invented in this umbrella spec.
The settlement MUST update, in this order:

1. `domainforge-core/grammar/sea.pest`;
2. AST types and parser construction in `domainforge-core/src/parser/ast.rs`;
3. graph/primitives or an explicitly AST-owned application-contract IR;
4. formatter, canonical semantic envelope, JSON schemas, diagnostics, fixtures,
   and parser tests;
5. Python, TypeScript, and WASM public bindings when the new contract is public;
6. Domain IR/application-contract lowering and all affected projections.

The ADR MUST settle at least typed fields, use cases/operations, operation-policy
scope, failure declarations, state/effect references, and backward compatibility.
Generic annotations are not acceptable unless their keys, types, defaults, and
validation become a versioned language contract.

### 5.3 Current Domain IR debt

Application generation MUST NOT parse generated Rust/Python/TypeScript ports.
Instead, the canonical lowering must expose typed `PortIr` / `OperationIr`
structures shared by renderers. The work MUST also address these current debts:

- instance examples cannot define an entity schema;
- all inferred fields cannot remain `str` for application conformance;
- policy guards cannot be universal no-op hooks;
- repository `get/save`, buses, and read-model methods need stable semantic refs;
- Domain/Application IR persisted artifacts need schema versions,
  serialization, validation, origin refs, and canonical hashes.

Existing domain renderers MAY retain compatibility output, but the complete
application path MUST reject an operation whose semantics are only heuristic.

### 5.4 Adapter-field source matrix

Every Adapter IR field has one authoritative source. This prevents a profile or
provider from smuggling in missing domain meaning.

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

Any field absent from this matrix is forbidden until the language ADR assigns
it an authoritative source and adds a missing-semantics diagnostic.

## 6. Canonical ownership and architecture

```text
Human owns domain truth and approval.
Agent skills own elicitation and translation into constrained inputs.
.sea and its resolved import/semantic-pack closure own declared meaning.
Application profile owns versioned non-domain defaults and proof policy.
Provider binding owns per-application realization choices and safe configuration.
Compiled provider emitter owns trusted technology-specific implementation logic.
DomainForge owns validation, canonical IRs, resolution, planning, generation,
locking, proof classification, and review artifacts.
Generated files are projections, never a competing source of truth.
SEA-Forge may consume runtime authority/evidence contracts but is not required
to compile or generate an application.
```

Required dependency direction:

```text
SEA source closure
  -> canonical semantic envelope
  -> Domain IR + Application Contract IR
  -> Adapter IR + capability requirements
  -> provider binding + fixed catalog snapshot
  -> capability resolution
  -> realization plan
  -> planned/staged emitters
  -> application lock
  -> proof plan
  -> proof record and completion classification
```

Emitters MUST consume the realization plan or typed provider-specific units.
They MUST NOT read SEA independently, choose providers, change semantics, or add
unplanned dependencies.

## 7. Core artifacts

Every persisted artifact MUST deny unknown fields for its supported major schema
version, use canonical JSON for hashing, carry `schema_version`, `producer`,
`inputs`, and `self_hash`, and validate before it contributes to run state.

### 7.1 CanonicalSemanticEnvelope

Contains the normalized, import-resolved semantic model, language/grammar schema
version, semantic-pack set and hashes, namespace resolution, source origins, and
compiler semantic-version identity. It is the input to
`semantic_closure_hash`.

Comments, source formatting, absolute checkout paths, and timestamps MUST NOT
affect the semantic closure. A change to resolved meaning, imports, namespace
resolution, semantic-pack content, or interpretation version MUST affect it.
The envelope records a separate `source_set_hash` over logical source paths and
bytes for provenance. That hash does not invalidate semantic approval unless the
bytes change the generated semantic review; harmless comments and formatting
remain separate from meaning.

### 7.2 DomainReviewBundle

Contains:

- one-page outcome and scope summary;
- actors/roles and explicit public access;
- use cases with inputs, outcomes, side effects, and failures;
- stored information and retention assumptions;
- policies mapped to operations;
- external interactions and evidence obligations;
- assumptions, unsupported distinctions, and consequential unresolved questions;
- applicable BPMN/CMMN/ArchiMate/RDF projections labelled as review aids;
- change summary against the last approved semantic closure;
- `review_bundle_hash`.

### 7.3 AdapterIr

Adapter IR is provider-neutral and includes stable semantic/origin refs,
direction, family requirement, operations, typed contracts, effects,
transaction/concurrency/idempotency requirements, failures, authority, evidence,
observability, and lifecycle requirements.

It MUST NOT contain a vendor, SDK, package, endpoint address, credential,
generated-code fragment, or provider configuration. An architecture test MUST
scan typed fields and serialized fixtures for forbidden provider terms.

### 7.4 ProviderDescriptor and ProviderBinding

For v0.1, a provider descriptor is data-only and contains:

- typed ID, exact version, family, target, maturity, and content hash;
- compatible compiler/IR/profile versions;
- capability claims and unsupported capabilities;
- exact dependency contributions;
- configuration schema ref;
- compiled emitter ID and compiled conformance-suite ID;
- evidence refs and revocation state.

A provider binding contains stable instance IDs, exact provider refs, bound port
refs, schema-valid configuration, environment refs, and secret references. It
MUST NOT contain source, SQL, shell, templates, dependency overrides, arbitrary
paths, new policies, or capability claims.

### 7.5 CapabilityResolution

Records a decision for every required and forbidden capability, the selected
provider, native or named synthesized realization, evidence ref, rejected
alternatives, unmet requirements, and exact catalog snapshot hash.

v0.1 permits exact selection only. A synthesis rule must be compiled,
allowlisted, named, and covered by conformance tests. “Best effort” and silent
degradation are forbidden.

### 7.6 ApplicationRealizationPlan

The plan is the only generation handoff. It lists, before any write:

- every text artifact path, owner, class, emitter, and inputs;
- exact dependency contributions;
- generated and extension zones;
- fresh-database migrations;
- environment/network requirements;
- deterministic proof steps;
- expected residue;
- all upstream hashes.

v0.1 supports UTF-8 text artifacts only because the current `ArtifactSink`
surface is text-only. Binary artifacts require a sink and lock schema revision.

### 7.7 ApplicationLock

The lock covers every generated file’s path, SHA-256 content hash, ownership,
emitter identity, relevant executable bit if later supported, native lockfiles,
and all plan/provider/profile/compiler hashes. It excludes build caches,
integration databases, runtime logs, and secrets.

### 7.8 ProofPlan and ProofRecord

`ProofPlan` is deterministic and hashable. `ProofRecord` is an observation. It
records environment/tool versions, redacted argv, bounded output refs, start/end
times, exit status, stable assertions, skipped steps, cleanup status, residue,
and classification.

The immutable layers are distinct:

1. the resolution/plan lock binds compiler inputs, catalog, providers,
   dependencies, toolchain constraints, and planned paths;
2. `ApplicationLock` binds emitted content, including the native dependency
   lockfile;
3. `EvidenceManifest` binds the proof plan, proof record, redacted attachments,
   environment fingerprint, and application-lock hash.

No layer contains itself or hashes a downstream layer. Proof can therefore be
rerun against unchanged application content without circular locking.

Variable timing, ports, container IDs, and runtime logs are excluded from
determinism comparisons but preserved as redacted evidence where useful.

## 8. Conversational workflow and approval UX

### 8.1 SEA authoring skill

The repository MUST provide a SEA authoring skill that:

- treats conversation and uploaded documents as untrusted domain evidence;
- elicits outcome, actors, terms, state, use cases, information, rules,
  exceptions, authority, evidence, measures, and unknowns;
- distinguishes domain meaning from technology choices;
- writes the smallest model that preserves consequential distinctions;
- runs the real formatter, parser, validator, semantic-envelope builder, and
  review projections after each material change;
- maps diagnostics back to the affected human distinction;
- never invents syntax or removes a requirement to make validation pass;
- presents no more than the one-page decision summary first, with links to
  detailed review artifacts;
- labels every question as `blocking`, `consequential_nonblocking`, or
  `informational` and explains the effect of leaving it unresolved;
- refuses semantic approval while any blocking question remains;
- records explicit assumptions only for nonblocking matters;
- asks the human to confirm the current review bundle, not merely “looks good?”;
- cannot approve on the human’s behalf.

If the CLI is unavailable, the skill may write a draft but MUST label it
`unvalidated` and provide exact commands. It MUST NOT proceed to binding.

### 8.2 Provider binding skill

The provider binding skill MUST:

- require a current semantic approval and generated requirements;
- use only descriptors returned by DomainForge’s fixed catalog snapshot;
- normally write only provider-binding YAML;
- show services, storage location, network access, secrets, data durability,
  consistency, operational burden, local/production differences, and known
  limits in plain language;
- show why alternatives were rejected or unavailable;
- run schema validation, exact capability resolution, dependency validation,
  and plan preview before approval;
- stop with a provider-gap report when no supported option exists;
- never search the public package ecosystem, create a provider pack, inject a
  dependency, or embed code/commands/secrets.

### 8.3 Approval contracts

Semantic and realization approvals are separate.

Semantic approval binds:

- `semantic_closure_hash`;
- `domain_review_bundle_hash`;
- application-contract schema version;
- approver-supplied label, timestamp, scope, and exact statement.

Realization approval binds:

- semantic approval hash;
- Adapter IR and requirement hashes;
- provider-binding hash;
- profile hash;
- provider catalog snapshot and selected provider hashes;
- resolver-policy hash and capability-resolution hash;
- requested proof profile and operational consequences;
- realization review hash;
- approver-supplied label, timestamp, scope, and exact statement.

Approval is evidence of a review action, not identity proof. The CLI computes
all hashes. Skills may invoke approval capture only after an explicit human
statement. Any bound hash change makes the approval stale.

DomainForge MUST revalidate the approval chain immediately before planning and
again before staged output commits, preventing time-of-check/time-of-use drift.

### 8.4 Low-effort, high-reward UX requirements

The first implementation MUST include:

- `application inspect`: readiness, missing semantics, and unsupported features;
- `application diff`: semantic, realization, migration-risk, and file-plan diff;
- `application doctor`: toolchain/cache/provider prerequisites with remediation;
- `application plan --dry-run`: no output-tree mutation;
- generated README with exact regenerate/build/test/run commands;
- redacted `.env.example` containing references/placeholders only;
- health-check and copyable sample request/response;
- compact success summary with artifact count, proof classification, residue,
  and next permitted action;
- stable diagnostic codes and source paths suitable for agent repair.

## 9. Provider and trust model

### 9.1 v0.1 boundary

v0.1 providers are statically known compiled emitters selected by declarative
descriptors. Descriptor paths cannot escape their pack root. A descriptor cannot
name an arbitrary executable, template engine, dynamic library, URL, or script.

The catalog hashes each descriptor and every referenced declarative file into a
canonical provider content hash. Built-in provider trust is inherited from the
verified DomainForge release containing the compiled emitter. If external packs
are later admitted, they MUST reuse or extend semantic-pack canonical hashing,
signature verification, trusted-key configuration, and revocation rather than
inventing an unsigned parallel mechanism.

This design follows ADR-011’s current “no plugin framework” decision. A future
external-provider ABI requires a separate threat model, sandbox, signature and
revocation design, resource limits, compatibility policy, and ADR.

### 9.2 Catalog and maturity

Use a new `ProviderCatalog` type; do not overload the existing semantic-pack
registry or `projection::registry` names.

Maturity states are:

```text
candidate -> schema_valid -> conformance_green -> integration_proven
          -> substitution_proven -> production_qualified
any state -> revoked
```

Only `integration_proven` built-in providers may satisfy v0.1. A profile states
its minimum maturity. Revocation always wins.

### 9.3 Configuration and secrets

Provider configuration schemas deny unknown fields. Only fields explicitly
typed as a closed `SecretRef` accept `secret://` or `env://`; those fields have
no literal-string variant. DomainForge does not claim it can recognize every
credential embedded in unrelated free text, so provider schemas MUST avoid
unconstrained text wherever a secret could plausibly appear. Secret values never
enter hashes, locks, generated source, review bundles, logs, or proof records.

Child processes receive an allowlisted environment built by the proof harness.
DomainForge redacts configured secret values and common credential patterns from
captured output, caps captured bytes, and stores a truncation marker and complete
output hash when truncation occurs.

### 9.4 Network and external effects

Generation is offline by default. Catalog installation or refresh is a separate
explicit action that produces a snapshot hash.

Integration proof uses local disposable resources only. It MUST NOT connect to
production or a non-disposable database/broker. The harness requires an explicit
disposable-resource marker and refuses suspicious endpoints under governed
profiles.

Generated Cell network access is derived from explicit Cell declarations plus
selected provider/profile requirements. A SEA domain `Flow` alone never grants
network access.

## 10. Canonicalization and determinism

### 10.1 Hashes

All security/approval/lock hashes use SHA-256 with a `sha256:` prefix. Existing
xxh64 projection IDs remain valid identifiers but MUST NOT be used as security or
integrity hashes.

`semantic_closure_hash` is SHA-256 over canonical JSON containing:

- schema/version discriminator;
- resolved, normalized semantic declarations and application contracts;
- import graph with logical paths and content hashes;
- namespace resolution;
- semantic-pack IDs and content hashes;
- interpretation/normalization version.

Absolute paths, comments, source order that is semantically irrelevant,
formatting, and timestamps are excluded. Order-sensitive domain constructs
remain ordered. Canonicalization MUST be implemented once and cross-checked by
equivalent-source fixtures.

Canonical rules are explicit:

- strings use Unicode NFC; identifiers then apply their declared case/slug rule;
- object keys sort by UTF-8 byte order after normalization;
- maps reject duplicate normalized keys;
- collections declared as sets sort by stable typed ID and reject duplicates;
- sequences whose order affects behavior retain authored semantic order;
- defaults are inserted by the schema-versioned canonicalizer before hashing;
- numbers and durations use one normalized, lossless representation;
- YAML comments, anchors, aliases, key order, and scalar spelling do not survive
  typed decoding;
- each collection field's schema metadata declares `set` or `sequence`.

Golden vectors MUST cover Unicode equivalence, duplicate-after-normalizing, map
order, set order, sequence order, defaults, durations, imports, and comments.

### 10.2 Deterministic and observed outputs

For fixed semantic closure, binding, profile, catalog snapshot, compiler,
emitter versions, dependency inputs, and `--created-at`, these MUST be identical:

- IR JSON;
- requirements and resolution;
- realization plan and proof plan;
- generated UTF-8 source/config/migrations/tests;
- application lock.

Native `Cargo.lock` is part of deterministic output only when generated from the
same pinned toolchain, manifest, registry/cache snapshot, and resolver mode. If
that environment cannot be reproduced, the run is `blocked_environment`, not
nondeterministically “verified.”

Runtime observations are not byte-deterministic. Proof compares declared stable
assertions and canonicalized result fields.

## 11. Resolution and planning

v0.1 resolution is deliberately simple:

1. derive normalized requirements;
2. load the exact catalog snapshot selected by the profile/binding;
3. validate exact provider IDs, versions, hashes, target, maturity, and
   compatibility;
4. map every required and forbidden capability;
5. validate exact dependency contributions against the pinned profile set;
6. emit selected/rejected decisions and gaps in stable lexical order.

There is no heuristic package search, floating “latest,” or generic SAT solver
in v0.1. When more than one conforming provider exists, substitution conformance
adds versioned ranking with this order: current lock, organization/profile
preference, native coverage, lower synthesis burden, higher maturity, lower
operational burden, lexical tie-break.

Every dependency contribution records ecosystem, package, exact version,
registry/source identity, checksum or immutable revision, enabled features,
target constraints, license expression, advisory-policy result, and resolver
version. Container/system dependencies also record an immutable image digest or
repository/package identity. Native verification uses locked/frozen mode.
Missing cache/network access is `blocked_environment`; a yanked, revoked,
checksum-mismatched, disallowed-license, or policy-rejected dependency is an
invalid resolution, not an automatic upgrade.

The realization plan MUST be complete and validated before any generated-tree
write. It lists every path and emitter. An emitter that returns a different path
or undeclared dependency fails with `APP030 unplanned_artifact`.

## 12. Generated application contract

For `rust-local-v1`, DomainForge generates:

- existing compatible domain code, corrected typed ports, and explicit policy
  enforcement hooks;
- application services connecting declared operations to domain effects;
- HTTP routes and DTO mappings only for declared inbound operations;
- SQLite repositories only for declared persistence ports;
- fresh-database migrations and schema mappings;
- composition root and typed configuration;
- canonical error mapping that does not expose internals;
- structured tracing with semantic operation refs;
- unit, contract, integration, restart-persistence, and smoke tests;
- README, `.env.example`, sample requests, Cell output when applicable, and
  application lock.

Profile-owned behavior such as `/health` MUST be labelled profile-owned and must
not be mistaken for domain meaning.

Generated source headers SHOULD contain a short semantic/plan reference. Full
provider/compiler provenance belongs in the manifest and lock, avoiding noisy
churn and oversized headers in every file.

No operation is emitted with a no-op authorization or policy guard. If a policy
cannot be compiled or connected to a proven runtime evaluator, the affected
operation is blocked and appears in residue.

## 13. Safe writes, regeneration, and extension points

### 13.1 PlannedArtifactSink

Before v0.1 completion, extend or wrap `ArtifactSink` with a planned sink that:

- accepts only normalized relative UTF-8 paths declared in the plan;
- rejects absolute paths, `..`, symlink escapes, reserved paths, normalized
  collisions, case-fold collisions on case-insensitive targets, and duplicate
  writes, even when bytes match;
- accepts UTF-8 text only for v0.1 and normalizes line endings as declared;
- hashes bytes as written;
- fails if any planned artifact is missing or any unplanned artifact appears;
- never follows a pre-existing symlink in staging or output traversal.

### 13.2 Staging and commit

Generation uses a sibling staging directory on the same filesystem and an
atomically created owner lease containing a random run token, process metadata,
start time, and target output path. A second writer fails closed.

DomainForge MUST NOT automatically delete an unverifiable or apparently live
staging directory. `application status` explains it; `application clean` removes
it only after ownership/liveness checks or explicit user confirmation.

For a new output root, commit is one same-filesystem rename. For an existing
locked root, DomainForge first verifies every generated file against the prior
lock, produces a diff, stages the replacement, and uses a documented two-rename
swap with recovery markers. Cross-filesystem output fails preflight.

### 13.3 Regeneration and pruning

Manual edits in generated zones fail with `APP031 generated_zone_drift`.
Extension files live outside generated zones and are never overwritten.

Files in the prior lock but absent from the new plan appear in the diff. Default
regeneration refuses to remove them. `--prune` may delete only files whose bytes
still match the prior lock. Modified or unknown files are preserved and block
commit. `--force` MUST NOT bypass path, trust, secret, approval, or drift checks;
any future reset command must make destructive scope explicit.

Persistent-schema change emits `deployment_migration_required` residue and
blocks a production-upgrade claim. Fresh local fixture databases may be rebuilt.

## 14. CLI and machine contract

Application generation is orchestration, not a pure projection. Use a new
top-level group rather than overloading all behavior into `project`:

```text
domainforge application inspect MODEL --profile rust-local-v1 --json
domainforge application plan MODEL --providers PROVIDERS --profile rust-local-v1 --dry-run --json
domainforge application generate MODEL OUT --providers PROVIDERS --profile rust-local-v1 --approval APPROVAL --created-at TIMESTAMP --json
domainforge application verify OUT --level integration --json
domainforge application diff MODEL OUT --providers PROVIDERS --profile rust-local-v1 --json
domainforge application status OUT --json
domainforge application doctor --profile rust-local-v1 --json
domainforge application clean OUT
domainforge provider list --profile rust-local-v1 --json
domainforge provider inspect PROVIDER_ID --json
domainforge provider validate PATH --json
```

Adapter IR inspection MAY also be exposed as an ADR-011 projection format if it
retains the one-IR/pure-renderer/in-memory shape.

New machine-facing commands return one versioned JSON envelope:

```json
{
  "schema_version": "domainforge-command-result/v1",
  "status": "ok|invalid|unsupported|blocked|failed|incomplete|verified",
  "diagnostics": [],
  "inputs": {},
  "artifacts": [],
  "proof": null,
  "recoverable": true,
  "next_actions": []
}
```

Exit codes MUST distinguish success/verified, invalid input, unsupported/gap,
blocked environment, proof failure, and internal failure:

| Exit | Meaning |
|---:|---|
| `0` | request succeeded; JSON status is `ok`, `generated`, or `verified` as applicable |
| `2` | invalid authored input or artifact chain |
| `3` | unsupported semantic/provider capability |
| `4` | blocked environment or prerequisite |
| `5` | generation or proof assertion failed |
| `6` | incomplete because required proof was skipped |
| `70` | internal compiler error |

Human output goes to stdout, diagnostics to stderr, and `--json` emits no
incidental prose on stdout. Cancellation handles SIGINT/SIGTERM, stops launching
new work, requests child termination, runs bounded cleanup, records cleanup
status, and leaves the previously committed output untouched. Automatic retries
are allowed only for profile-declared transient integration setup, use a fixed
maximum and backoff schedule, and appear as individual proof attempts.

Application orchestration is CLI-only in v0.1 because it performs filesystem and
native-process work. Pure Adapter/Application IR inspection MUST retain
ADR-011's in-memory surface. New public core types require Python, TypeScript,
and WASM binding parity under repository policy; otherwise they remain
`pub(crate)` and the exception is documented in the accepted ADR.

## 15. State and approval invalidation

Run state is derived only from a valid artifact chain:

```text
domain_draft -> domain_valid -> domain_approved
-> binding_valid -> realization_approved -> planned -> generated -> verified
```

Terminal/non-success classifications are:

- `invalid`: authored input or artifact chain violates syntax/schema/semantics;
- `unsupported`: DomainForge lacks a required semantic/provider capability;
- `blocked`: the plan may be valid but a required tool, cache, credential, or
  disposable service is unavailable;
- `failed`: generation or a required behavioral assertion ran and failed;
- `incomplete`: output exists but required proof was skipped or residue remains;
- `verified`: every required proof for the exact lock passed.

A self-declared state file is never authoritative. `application status`
validates schema versions, self-hashes, upstream hashes, generated tree, and
proof chain before reporting state.

Any semantic-closure change invalidates all downstream artifacts. A binding,
profile, catalog, provider, or requirements change invalidates realization
approval and everything after it. Environment repair alone may rerun proof
against the same application lock.

## 16. Failure and recovery

Required diagnostic classes include:

| Code | Class | Behavior |
|---|---|---|
| `APP001` | missing application semantics | Block affected operation before binding. |
| `APP002` | stale semantic approval | Regenerate review/diff and request approval. |
| `APP003` | stale realization approval | Re-resolve, regenerate realization review, request approval. |
| `APP010` | invalid provider binding | Report field/source and permitted values. |
| `APP011` | provider gap | Emit gap report; emit no affected adapter. |
| `APP012` | provider untrusted/revoked | Stop before planning. |
| `APP020` | dependency conflict | Report each contribution and provider provenance. |
| `APP021` | blocked environment | Preserve plan/output; provide doctor remediation. |
| `APP030` | unplanned artifact | Abort staging; no output lock. |
| `APP031` | generated-zone drift | Preserve user bytes; show diff; refuse commit. |
| `APP032` | concurrent/stale staging | Preserve staging; provide status/clean guidance. |
| `APP040` | nondeterministic output | Preserve both trees/hashes; classify failed. |
| `APP041` | proof assertion failed | Preserve redacted bounded evidence; classify failed. |
| `APP042` | proof skipped | Classify incomplete. |
| `APP050` | internal compiler error | Preserve safe diagnostics; never claim partial success. |

Errors include a stable code, concise message, source or semantic ref, blast
radius, secrecy-safe context, and one or more next actions. They never print a
resolved secret or an unbounded untrusted payload.

Recovery patches the owning input or implementation layer. The skills may repair
SEA or provider YAML after presenting the change. They may not weaken meaning,
change a provider pack, bypass proof, or edit generated source.

## 17. Security and resource limits

Untrusted inputs include conversation, documents, SEA, provider bindings,
external catalog metadata, provider responses, generated application requests,
and native-tool output.

Mandatory controls:

- parse all authored input as bounded data;
- deny arbitrary commands, code, templates, paths, dependencies, and unknown
  fields in provider bindings;
- use compiled/allowlisted emitters and proof steps only;
- invoke processes with tokenized argv, fixed executable identities where
  feasible, no shell, explicit working directory, environment allowlist,
  timeout, output cap, and cleanup policy;
- validate output paths against symlink and traversal attacks;
- use SHA-256 for approvals, locks, and provider integrity;
- never persist or hash secret values;
- default deny generated network access except explicit profile/binding/Cell
  requirements;
- apply generated input validation and proven policy/authority checks before
  side effects;
- fail closed on unavailable authority for governed operations;
- prohibit proof against production endpoints;
- record unsafe Cell overrides using the existing ticket/authority/rationale/
  expiry mechanism.

Profiles MUST set ceilings for SEA/source bytes, declarations, provider count,
planned artifact count, bytes per artifact, total output bytes, process count,
per-step/overall timeout, captured output, retries, and disposable-resource
lifetime. Exceeding a limit produces a typed failure before the next trust
boundary. Exact values are versioned profile data and exercised by boundary
tests.

## 18. Proof and acceptance matrix

### 18.1 v0.1 proof levels

```text
plan         schemas + semantic gate + resolution + complete planned paths
static       plan + deterministic regeneration + format + cargo check + unit tests
integration  static + real local HTTP/SQLite/restart flow + provider conformance
```

There is no `none` level that can yield a completion claim. `generate` without
verification returns `generated` or `incomplete`, never `verified`.

### 18.2 Required tests

| Area | Test | Expected evidence |
|---|---|---|
| Semantic gate | Missing typed input, policy scope, or transaction decision | `APP001`; no affected route/adapter in plan. |
| Equivalent source | Formatting/comment/order-only change | Same semantic closure hash where order is irrelevant. |
| Semantic change | Imported declaration or semantic-pack meaning changes | New closure hash; approvals stale. |
| Approval TOCTOU | Input changes between preflight and commit | Commit blocked; old output untouched. |
| Adapter neutrality | Serialized Adapter IR scanned for provider terms | Architecture test passes. |
| Provider trust | Revoked/hash-mismatched descriptor | Resolution blocked. |
| Binding safety | Unknown key, plaintext secret, command/template field | Schema/security diagnostic; no writes. |
| Planned sink | traversal, symlink, case collision, duplicate, unplanned write | Staging fails with no committed lock. |
| Determinism | same fixed inputs/toolchain/cache snapshot twice | Identical plan, source tree, native lock, application lock. |
| Happy path | generated write then read over HTTP | Real SQLite assertions and proof record. |
| Restart | stop/restart then read prior data | Durable result persists. |
| Validation | malformed request | Canonical client error; no side effect. |
| Policy | guarded operation allowed/denied | Correct effect/no effect and policy evidence. |
| Transaction | injected failure during write | No partial durable state. |
| Drift | manually edit generated file | Regeneration refuses and preserves bytes. |
| Prune | unchanged orphan with/without `--prune` | Preview/refusal by default; safe deletion only with flag. |
| Concurrency | two writers target one output | one lease wins; other fails safely. |
| Interrupted run | killed staging run | committed output unchanged; status/clean recover. |
| Proof environment | unavailable tool/cache | `blocked`, not failed or verified. |
| Secret redaction | child echoes supplied test secret | record redacted; secret absent from tree/logs. |
| Limits | oversized source/artifact/output/log | bounded typed failure. |
| Skill approval | model attempts self-approval | workflow stops at valid/unapproved. |
| Review UX | semantic change after approval | one-page “changed since approval” summary generated. |

Integration tests that require a behavior cannot be replaced by mocks. Skipped
required tests classify `incomplete`. Test resources are disposable and cleanup
status is recorded even after failure.

### 18.3 Verification commands

Add commands following repository conventions only when the implementation
exists:

```text
just application-rust-verify
just provider-sqlite-rust-verify
just skill-sea-authoring-verify
just skill-provider-binding-verify
```

The main Rust gate remains the repository’s actual command, currently
`cargo test --workspace --features cli`, plus formatting/clippy and applicable
binding suites. A documentation proposal MUST NOT list nonexistent recipes as
current evidence.

## 19. Implementation sequence and stop gates

Each milestone is independently reviewable and MUST stop when its settlement
evidence fails.

### Milestone 0 — Language and architecture settlement

Deliver the application-contract ADR, grammar/AST/Graph ownership, typed
operation/policy contract, canonical semantic envelope, compatibility policy,
and examples reviewed by domain and compiler maintainers.

Stop gate: no Adapter IR or application emitter work until two nontrivial SEA
fixtures can express the v0.1 use cases without generic unvalidated annotations
or fixture-derived types.

### Milestone 1 — Canonical contracts and review

Deliver serializable Domain/Application IR, semantic closure hashing, schemas,
valid/invalid fixtures, `application inspect`, domain review, semantic diff, and
approval capture/validation.

Stop gate: equivalent sources hash equally; consequential changes invalidate
approval; missing semantics produce stable diagnostics.

### Milestone 2 — Provider-neutral planning

Deliver Adapter IR, requirements, data-only provider descriptors/bindings,
built-in catalog, exact resolver, capability gaps, realization review, plan,
and provider-binding skill.

Stop gate: every requirement resolves or is an explicit gap; provider terms do
not leak into Adapter IR; no output application files are written.

### Milestone 3 — Safe emission foundation

Deliver planned/staged sink, locks, lease/recovery, dry-run, diff, drift/prune
behavior, bounded process runner, proof-plan/record schemas, and security tests.

Stop gate: traversal, collision, concurrent writer, interruption, unplanned
write, output drift, secret echo, and resource-limit tests all fail safely.

### Milestone 4 — `rust-local-v1`

Deliver compiled Rust HTTP/SQLite/tracing emitters, application services,
migrations, configuration, tests, README/sample requests, and integration proof.

Stop gate: the flagship application passes HTTP write/read, invalid-input,
policy, rollback, restart, determinism, and clean-regeneration tests with no
handwritten generated-zone edits.

### Milestone 5 — Guided workflow

Finalize SEA authoring and provider binding skills, fixtures/evals, doctor,
progressive review bundles, and an end-to-end tutorial with a real human approval
boundary.

Stop gate: evaluator transcripts show no self-approval, package invention,
hidden consequential defaults, or unvalidated completion claims.

### Milestone 6 — Substitution, then production

Add PostgreSQL substitution and only then generic ranking/dependency resolution.
Add production capabilities one provider and conformance suite at a time.

Stop gate: no production-profile claim until every required provider and
composition passes real integration/fault evidence.

## 20. Capability benchmark contract

The later “modest model” claim MUST pre-register:

- exact model/version/runtime and prompts/skill versions;
- representative participants or role-playing protocol;
- at least three materially different domains;
- direct-generation and stronger-coding-model baselines;
- thresholds for semantic omissions, correction rounds, unsafe inventions,
  tokens, cost, wall time, human review time, handwritten residue, compile,
  integration, recovery, and determinism;
- scoring rubric, blinded review where practical, failure handling, and raw
  transcript/evidence retention;
- a rule that thresholds cannot change after results are inspected.

The claim passes only if the constrained DomainForge workflow reduces model and
review burden while meeting or improving verified outcome quality. One curated
demo, a compiling app with incorrect meaning, mocked integration, or hidden
agent-written code is not evidence.

## 21. Definition of done

v0.1 is done only when:

- [ ] Milestone 0’s grammar-first semantic settlement is accepted and public
      bindings remain in parity where applicable.
- [ ] The semantic expressiveness gate blocks every incomplete operation.
- [ ] Canonical envelope, Domain/Application/Adapter IR, binding, resolution,
      plan, lock, approvals, proof, and residue schemas have positive/negative
      fixtures and canonical hashes.
- [ ] Semantic and realization reviews are readable, separate, diffable, and
      bound to explicit approvals.
- [ ] Providers are data-only descriptors referencing compiled allowlisted
      emitters; no arbitrary provider code or public package search executes.
- [ ] The planned/staged sink proves containment, collision rejection,
      completeness, atomic commit/recovery, drift protection, and safe pruning.
- [ ] `rust-local-v1` meets the exact Section 4.2 scope and no broader scope is
      implied.
- [ ] Real HTTP/SQLite/policy/rollback/restart tests pass.
- [ ] Deterministic plan, source, native lock, and application lock proof passes
      in the pinned environment.
- [ ] Missing tools/cache classify blocked; skipped proof classifies incomplete;
      failed behavior classifies failed; only all-green evidence verifies.
- [ ] Secrets, shell/code injection, path/symlink escape, untrusted provider,
      concurrency, interruption, and resource exhaustion tests pass.
- [ ] Generated output contains exact run/test/regenerate commands, `.env.example`,
      health check, sample request, proof summary, and residue.
- [ ] `.agents/current_state.md` records implementation evidence and
      `.agents/next_steps.md` names only the next two or three actions.

## Appendix A. Conceptual repository placement

Final paths follow the accepted ADR and existing conventions. The initial
low-debt shape is:

```text
domainforge-core/src/
├── projection/domain/            # existing, extended typed Domain IR
├── projection/adapter/           # provider-neutral IR + inspection renderer
├── application/                  # orchestration, plan, lock, proof, status
├── provider/                     # data-only descriptors, catalog, exact resolver
└── cli/{application,provider}.rs

schemas/application/
profiles/rust-local-v1.yaml
providers/<built-in-provider-id>/provider.yaml
fixtures/application_generation/
fixtures/providers/
skills/sea-authoring/
skills/provider-binding/
scripts/verify/                    # follow the repository's actual script layout
```

A crate split requires measured dependency, isolation, compile-time, or
independent-versioning value. A dynamic provider/plugin crate is not justified
by v0.1.

## Appendix B. Minimal provider binding example

This example is illustrative and does not become valid until its schema and
built-in provider descriptors exist.

```yaml
schema_version: domainforge-provider-binding/v1
application_ref: application:qualification
profile: rust-local-v1
catalog_snapshot: sha256:CATALOG_HASH
instances:
  api:
    provider: provider:http.axum.rust@0.1.0
    binds: [port:qualification.commands, port:qualification.queries]
    configuration:
      listen_address: 127.0.0.1
      port: 0
  store:
    provider: provider:relational.sqlite.rust@0.1.0
    binds: [port:qualification.repository]
    configuration:
      database_path: runtime://data/qualification.sqlite3
```

Port `0` requests an ephemeral test port. Production/wildcard listen addresses
are outside `rust-local-v1` and cannot be enabled through an unknown key or
unsafe binding override.

## Appendix C. Required handoff

Every guided run ends with:

```text
Classification and exact scope
What the application does / does not do
Semantic and realization approval hashes
Generated root and application lock
Proof level, passed/failed/skipped stages, and evidence path
Runtime configuration and unresolved secret references
Residue and migration risk
Exact build, test, run, sample-request, regenerate, diff, and doctor commands
Next permitted action
```

The handoff MUST never use “complete,” “safe,” “production-ready,” or “verified”
without the scope and proof classification that make the statement true.
