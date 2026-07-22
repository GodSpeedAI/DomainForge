# ADR-014: Application Review and Approval Contract

**Status:** Proposed
**Date:** 2026-07-19
**Deciders:** DomainForge Architecture Team

> **Ratification record** _(fill in only after explicit human acceptance)_:
> - Ratifier:
> - Approval date:

## Context

ADR-013 and Milestone 0 provide the canonical semantic envelope and typed
Application Contract. Milestone 1 must turn those machine artifacts into a
human-reviewable, diffable, explicitly approved semantic boundary before any
provider or generator work begins.

The governing generator plan deliberately left two decisions open: the exact
approval-capture commands/artifact paths, and the legal command-result status
values. Leaving either choice to an implementation agent would make approval
replay, invalidation, CLI compatibility, and later generation arguments
ambiguous.

If accepted, this ADR is a normative amendment to the generator specification:
its exact commands, status enum, approval arguments, diagnostic namespace, and
milestone ownership supersede conflicting examples in specification §§8.4,
14, 16, and 19. The specification MUST be reconciled in the same acceptance
commit; both contradictory forms MUST NOT remain normative.

## Decision

### 1. Existing Milestone 0 documents remain authoritative

`CanonicalSemanticEnvelopeDocument` remains the normalized semantic source and
`ApplicationContractDocument` remains the Application IR for v1. Milestone 1
MUST NOT create a second operation/application model.

Milestone 1 adds a serializable `DomainIrDocument` derived only from the
canonical envelope. It contains stable declaration IDs, logical origins, typed
semantic payloads, and resolved references needed for review. It MUST NOT use
or extend the legacy heuristic `projection::domain::ir::DomainIr`, whose
inferred fields and no-op policy hooks are nonconforming for application
generation.

### 2. MODEL means a filesystem SEA entry file

For `application` CLI commands, positional `MODEL` is the entry `.sea` file.
The loader:

- uses `--source-root ROOT` when supplied;
- otherwise uses the discovered `.sea-registry.toml` root, falling back to the
  entry file's parent directory;
- assigns `/`-separated, root-relative logical module IDs;
- follows relative imports within the root and namespace imports through the
  explicitly supplied or discovered registry;
- uses built-in `std:*` modules without copying them into the source map;
- never follows symlinks and rejects entries/imports outside the root;
- enforces the Milestone 0 byte, module-count, and import-depth budgets before
  semantic resolution.

`--registry PATH` overrides discovery. `--semantic-pack PATH` is repeatable;
packs are validated and passed to the existing pack-aware envelope API in
lexical canonical-pack-ID order. No directory-wide best-effort scan is
permitted.

### 3. Exact Milestone 1 CLI

```text
domainforge application inspect MODEL [--source-root ROOT] [--registry PATH] [--semantic-pack PACK]... [--json]

domainforge application review MODEL --out REVIEW.json [--review-input REVIEW_INPUT.json] [--source-root ROOT] [--registry PATH] [--semantic-pack PACK]... [--against PREVIOUS_REVIEW.json --approval PREVIOUS_APPROVAL.json] [--json]

domainforge application diff semantic MODEL --against PREVIOUS_REVIEW.json --approval PREVIOUS_APPROVAL.json [--review-input REVIEW_INPUT.json] [--source-root ROOT] [--registry PATH] [--semantic-pack PACK]... [--json]

domainforge application approve semantic REVIEW.json --out APPROVAL.json --approver-label LABEL --scope SCOPE --statement STATEMENT --approved-at RFC3339 [--json]
```

`inspect` and `diff` never write files. `review` and `approve semantic` write
exactly the single path supplied by `--out`, using create-new semantics; an
existing path is an error. Milestone 3 owns multi-file generated-tree staging,
replacement, and recovery.

Human mode prints the one-page summary first. JSON mode prints exactly one
`CommandResultDocument` to stdout and no incidental prose. Diagnostics go to
stderr only in human mode and are embedded in the JSON result only in JSON
mode.

For `review`, `--against` and `--approval` are a dependent pair: supplying
either without the other is a CLI error. The approval MUST validate against
that review and MUST represent its accepted semantic closure. Semantic diff is
always against the last approved review; comparing arbitrary unapproved drafts
is outside the approval workflow and may be done by lower-level library tooling
only.

The later realization commands are reserved now so their shape cannot drift:

```text
domainforge application review realization MODEL OUT --providers BINDING --profile PROFILE --semantic-approval SEMANTIC_APPROVAL --out REVIEW.json

domainforge application approve realization REVIEW.json --semantic-approval SEMANTIC_APPROVAL --out APPROVAL.json --approver-label LABEL --scope SCOPE --statement STATEMENT --approved-at RFC3339

domainforge application diff realization MODEL OUT --providers BINDING --profile PROFILE --semantic-approval SEMANTIC_APPROVAL --realization-approval REALIZATION_APPROVAL [--json]
```

Planning and generation consume two explicit arguments:

```text
--semantic-approval SEMANTIC_APPROVAL.json
--realization-approval REALIZATION_APPROVAL.json
```

The realization approval binds the semantic approval hash, but both documents
are supplied and validated. DomainForge never guesses a companion artifact by
filename or directory convention.

Milestone 1 `inspect` intentionally has no profile argument because it reports
declared meaning only. Profile-owned defaults first enter realization review in
Milestone 2. This supersedes the earlier specification example that passed
`--profile rust-local-v1` to semantic inspection.

### 4. Fixed artifact names and schemas

The caller chooses paths, but examples and skills use these names:

```text
.domainforge/review/domain-review.json
.domainforge/approvals/semantic-approval.json
.domainforge/review/realization-review.json       # Milestone 2
.domainforge/approvals/realization-approval.json  # Milestone 2
```

Milestone 1 adds strict schemas:

```text
schemas/domain-ir-v1.schema.json
schemas/application-inspection-v1.schema.json
schemas/domain-review-input-v1.schema.json
schemas/domain-review-bundle-v1.schema.json
schemas/semantic-diff-v1.schema.json
schemas/semantic-approval-v1.schema.json
schemas/command-result-v1.schema.json
```

Every persisted document denies unknown fields and carries `schema_version`,
`producer`, `inputs`, and `self_hash`. Canonical JSON and `sha256:` framing are
the existing Milestone 0 implementations, not a CLI reimplementation.

`ApplicationInspectionDocument` is the strict, non-persisted semantic result
placed in `CommandResultDocument.result` by `inspect`. It contains canonical
input and closure hashes, `readiness` (`ready`, `invalid`, or `unsupported`),
sorted missing-semantic diagnostics, unsupported distinctions, deterministic
review questions, projection applicability, and the Domain/Application IR
artifact descriptors that a review would bind. It contains no approval state.

`DomainReviewInputDocument` is a strict persisted human-decision input with
schema version, producer, inputs containing the exact semantic closure hash,
`decisions`, and `self_hash`. Each decision binds a deterministic question ID
and is either an explicit assumption for a
`consequential_nonblocking` question or an acknowledgement for an
`informational` question, with bounded human text and rationale. It cannot
resolve or downgrade a `blocking` question; blocking questions are removed only
by changing SEA or another authoritative semantic input and rebuilding the
closure. A decision for an unknown question, wrong classification, stale
closure, duplicate question, or blocking question is `APR001`/`APR002`.

### 5. Domain review bundle

`DomainReviewBundleDocument` binds its exact semantic closure and application
contract and contains:

- `decision_summary`: bounded outcome/scope, actor/access, operation, stored
  information, policy, external-effect, assumption, omission, and change counts;
- `actors_and_access`;
- `use_cases` with intent, inputs, output, state/effect, policies, failures,
  idempotency, concurrency, evidence, and lifecycle;
- `stored_information` with explicit retention status (`declared`,
  `unresolved`, or `not_applicable`), never an invented duration;
- `policy_operation_matrix`;
- `external_interactions` and evidence obligations;
- `questions`, each classified `blocking`, `consequential_nonblocking`, or
  `informational`, with effect-if-unresolved and state `unresolved`, `assumed`,
  or `acknowledged`;
- `assumptions`, which may settle only consequential-nonblocking questions and
  come only from a valid `DomainReviewInputDocument`;
- `unsupported_distinctions` and projection applicability/residue;
- optional `semantic_diff` against the supplied previous review;
- embedded canonical `domain_ir` and `application_contract` snapshots so a
  later diff never depends on an unrecorded checkout;
- `review_bundle_hash`, computed over the canonical bundle payload only;
- document `self_hash`, computed over the complete document with only
  `self_hash` omitted, therefore binding `review_bundle_hash` without a cycle.

The wire framing is exact:

```text
DomainReviewBundleDocument {
  schema_version, producer, inputs,
  review_bundle_hash, self_hash,
  bundle: DomainReviewBundle
}
```

`review_bundle_hash = sha256(canonical_json(bundle))`. No document metadata or
hash field is in that preimage. `self_hash` is SHA-256 over canonical JSON of
the complete document with only `self_hash` omitted; it therefore binds schema,
producer, inputs, `review_bundle_hash`, and `bundle`.

The human renderer shows only `decision_summary`, blocking/consequential
questions, assumptions, unsupported distinctions, and change counts initially.
It prints stable JSON Pointer links into the review document for details.

Approval is impossible while a blocking question is unresolved. DomainForge
does not silently convert a consequential question into an assumption; the
assumption must be supplied by a current `DomainReviewInputDocument`.

### 6. Semantic diff

`SemanticDiffDocument` compares the newly resolved envelope/review with the
embedded snapshot in the last approved review supplied by `--against`. The
mandatory `--approval` must validate and bind that exact review. Entries use
stable `ConceptId` or
`ApplicationSymbolId`, a closed JSON Pointer field path, old/new canonical
values or hashes, and one of:

```text
nonsemantic | additive | definitional | behavioral | access_control |
data_shape | state_effect | removal | interpretation
```

`nonsemantic` entries do not change `semantic_closure_hash`. Every other class
is consequential and invalidates the previous semantic approval. Entries sort
by this closed severity rank, then typed subject ID, then field path:

```text
interpretation(0), removal(1), access_control(2), state_effect(3),
behavioral(4), data_shape(5), definitional(6), additive(7), nonsemantic(8)
```

If the approval/review pair
does not validate, diff fails with stale/invalid-artifact diagnostics rather
than presenting a misleading baseline.

### 7. Semantic approval

`SemanticApprovalDocument` contains:

- `schema_version = "domainforge-semantic-approval/v1"`;
- producer identity;
- inputs: `semantic_closure_hash`, `domain_review_bundle_hash`,
  `domain_review_document_self_hash`, and
  `application_contract_schema_version`;
- `approval`: `approver_label`, canonical RFC3339 UTC `approved_at`, `scope`,
  and the exact `statement`;
- `self_hash`.

The review bundle supplies this exact statement template in
`approval_statement_template`:

```text
I approve the domain meaning in semantic closure {{semantic_closure_hash}} and domain review bundle {{review_bundle_hash}} for scope {{scope}}.
```

All three literal tokens remain in the persisted bundle, so no hash appears in
its own preimage. `approve semantic` validates the review document, then
substitutes its exact `semantic_closure_hash`, `review_bundle_hash`, and the
validated scope into the three tokens. It validates `--scope` as NFC, nonempty,
at most 256 UTF-8 bytes, with no control characters, and succeeds only when
`--statement` byte-for-byte equals the resulting string.
The review has no unresolved blocking questions; all schemas/hashes validate;
`--approver-label` is NFC, nonempty, at most 128 bytes, with no control
characters; and `--approved-at` is a valid RFC3339 timestamp normalized to UTC.
DomainForge records a human assertion but does not claim or infer human
identity.

Validation always recomputes hashes and revalidates the review chain. A changed
semantic closure, review bundle payload, review document metadata/self-hash,
contract schema version, statement, scope, or approval metadata makes the
approval a different artifact. A semantic approval is stale when its bound
inputs do not match the current review.

### 8. Command-result contract

The closed status enum is:

```text
ok | generated | verified | invalid | unsupported | blocked | failed | incomplete
```

`generated` is included in `domainforge-command-result/v1`, resolving the
existing spec contradiction and superseding the shorter enum in specification
§14. Legal successful statuses are command-specific:

| Command family | Legal success status |
|---|---|
| inspect, review, diff, approve, provider metadata, clean | `ok` |
| plan/dry-run | `ok` |
| generate without complete proof | `generated` or `incomplete` |
| verify | `verified` or `incomplete` |
| status | the currently derived status |

Failures use `invalid`, `unsupported`, `blocked`, `failed`, or `incomplete` as
defined by the generator specification. The exact mapping is:

| Status | Exit | Meaning |
|---|---:|---|
| `ok`, `generated`, `verified` | 0 | Command-specific successful result. |
| `invalid` | 2 | Authored input, persisted artifact, or approval chain is invalid. |
| `unsupported` | 3 | A required semantic/provider capability is unsupported. |
| `blocked` | 4 | A required environment, permission, tool, or readable input is unavailable. |
| `failed` | 5 | Generation or a required proof assertion executed and failed. |
| `incomplete` | 6 | Output/evidence exists but required work was skipped or residue remains. |
| `failed` | 70 | Internal invariant, serialization, or unexpected compiler failure; the diagnostic code distinguishes this from exit 5. |

Milestone 1 cannot normally produce exit 5 because it performs neither
generation nor proof. Unexpected internal failures use status `failed`, exit
70, and a bounded internal diagnostic.

The result document contains `schema_version`, `status`, sorted diagnostics,
canonical input hashes, artifact descriptors, `result`, optional proof summary,
`recoverable`, and sorted `next_actions`. `result` is required and nullable:
successful inspect, review, semantic-diff, and approval commands place their
strict typed result document there; commands whose durable result is only an
artifact descriptor use `null`. Failure results use `null` unless a bounded,
schema-valid partial inspection is explicitly defined. It is an execution
response, not a persisted approval-chain input, so it has no `self_hash`.

### 9. Diagnostic namespace and M1 classifications

ADR-013 already owns `APP001–APP015`. Milestone 1 MUST NOT reuse those codes
for approval lifecycle failures. Existing `APP` diagnostics remain embedded
when source resolution or application semantics are invalid. Review and
approval orchestration uses this closed range:

| Code | Reason | Status / exit |
|---|---|---|
| `APR001` | invalid or hash-inconsistent review/diff/approval artifact | `invalid` / 2 |
| `APR002` | unresolved blocking review question | `invalid` / 2 |
| `APR003` | stale semantic approval or mismatched prior review/approval | `invalid` / 2 |
| `APR004` | approval label, scope, statement, or timestamp invalid | `invalid` / 2 |
| `APR005` | requested output already exists or parent is not a directory | `invalid` / 2 |
| `APR006` | MODEL/source-root/registry path escapes, is a symlink, or maps to an invalid logical ID | `invalid` / 2 |
| `APR007` | required source, registry, pack, or prior artifact cannot be read because the environment denies access | `blocked` / 4 |
| `APR008` | review, diff, question, diagnostic, or rendered-output resource ceiling exceeded | `invalid` / 2 |
| `APR009` | unescaped control/terminal sequence or unsafe human-rendering value | `invalid` / 2 |
| `APR010` | internal invariant or serialization failure | `failed` / 70 |

Missing files and malformed user paths are `invalid`; an existing readable
path that becomes unavailable because of permissions or a transient I/O
environment is `blocked`. Missing or contradictory required meaning retains the
applicable `APP` diagnostic and status `invalid`. A syntactically and
semantically valid closure that explicitly requires a declaration/capability
outside the current accepted Application/Domain IR or review implementation is
`unsupported` / 3.

Diagnostics sort by severity (`error`, `warning`, `info`), code, logical source
ID, source start/end, artifact path, JSON Pointer, typed subject ID, then
message, all ascending bytewise within a rank. When diagnostics imply multiple
top-level results, precedence is internal `failed`/70, `invalid`/2,
`unsupported`/3, `blocked`/4, execution `failed`/5, then `incomplete`/6. The
result MUST retain every safely reportable diagnostic even when a higher-
precedence status determines the exit code.

### 10. Public core and binding boundary

Pure construction, validation, semantic diff, and human-summary rendering live
in `domainforge-core/src/application/` and expose canonical JSON functions from
Rust. Python, TypeScript, and WASM bindings are byte-parity twins for pure
inspection/review/diff/validation. Filesystem loading, create-new output, human
terminal rendering, and exit-code selection remain CLI-only.

Approval construction is a pure core function requiring every human-supplied
field as an explicit argument. A skill or binding cannot call a parameterless
“approve current” operation.

## Alternatives rejected

- **One generic approval file or one `--approval` argument:** obscures whether
  semantic or operational consequences were approved and weakens stale-chain
  validation.
- **Approval inferred from a label or `yes`:** does not bind the reviewed
  context or exact human statement.
- **Store approval beside a guessed bundle path:** makes artifact discovery an
  ambient filesystem convention and permits replay against the wrong bundle.
- **Reuse legacy projection Domain IR:** it infers application meaning and
  contains no strict persisted schema.
- **Let inspect write a review directory:** crosses the Milestone 3 planned-
  write boundary and makes a supposedly cheap readiness check mutating.
- **Omit `generated` from the machine enum:** contradicts the required
  generation lifecycle and exit table.

## Compatibility and migration

The change is additive: `application` is a new top-level CLI group and all
schemas are new v1 documents. Existing commands and the Milestone 0 JSON
boundaries remain unchanged. Later milestones may add subcommands and optional
arguments but MUST NOT change the meanings fixed here.

Upon acceptance, the generator specification's “first implementation” means
the complete initial v0.1 delivery through Milestone 5, not Milestone 1 alone.
`doctor`, provider-aware dry-run planning, generated README/environment/sample
artifacts, realization/migration/file-plan diff, and skills remain assigned to
their explicit later milestones. Milestone 1 delivers only semantic
inspect/review/diff/approval.

## Security and limits

MODEL, imported sources, semantic packs, prior reviews, approvals, CLI strings,
and output paths are untrusted. Boundary validation enforces existing source
budgets, bounded diagnostics, strict schemas, hash chains, root confinement,
no symlink following, create-new output, maximum review/diff sizes, and no
unbounded echo of source values. Review text escapes control characters and
terminal sequences. No secret-bearing field exists in these schemas.

## Tests required before acceptance of Milestone 1

- equivalent-source fixtures produce byte-identical Domain IR, review bundle,
  and semantic approval inputs;
- every consequential Application Contract field change produces a typed diff
  and invalidates approval;
- formatting/comment-only changes remain nonsemantic;
- missing semantics and unresolved blocking questions prevent approval;
- tampering with every persisted field fails closed;
- prior review/approval mismatch is rejected;
- CLI JSON stdout contains one valid result and human diagnostics do not leak
  into it;
- all command/status/exit-code combinations are table-tested;
- path escape, symlink, oversized input, control character, and existing-output
  tests fail safely;
- Rust, Python, TypeScript, and WASM pure JSON boundaries are byte-identical.

## Disconfirmation criteria

Revisit this decision if a real workflow demonstrates that two explicit
approval documents cannot be managed safely, a single review bundle cannot
support progressive disclosure without multi-file writes, or the filesystem
MODEL loader cannot reproduce the source-map closure exactly across supported
platforms.
