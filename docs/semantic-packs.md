# Semantic Packs

A **semantic pack** is a deterministic, review-gated, signed JSON artifact that defines an organization's approved vocabulary of business concepts, relations, metrics, dimensions, units, aliases, and mapping rules.

Semantic packs act as a contract between the people who define business terminology and the tools that consume it. Every concept in a pack carries a human-reviewed definition, an owner, and a cryptographic hash of its semantic content. Downstream consumers---the DomainForge LSP, CI validators, and the SEA-Forge pipeline---resolve .sea source terms against the pack and emit structured diagnostics when terms are unknown, ambiguous, deprecated, or rejected.

## What a Semantic Pack Is

A semantic pack captures:

- **Concepts**: the core nouns of your domain---entities, resources, roles, flows, policies, metrics, dimensions, and units.
- **Relations**: typed predicates between concepts (e.g., `processes`, `owned_by`).
- **Metrics and dimensions**: observability definitions with unit and dimension bindings.
- **Aliases**: alternative names mapped to canonical concepts, each with a status (`approved`, `deprecated`, `ambiguous`, `blocked`).
- **Mapping rules**: cross-domain or cross-system term mappings with confidence scores.

Each pack is a self-contained JSON document with a stable content hash, a meaning fingerprint, and a review manifest hash. Two builds from identical inputs produce bit-for-bit identical output.

## What a Semantic Pack Is Not

- Not an embedding model or vector similarity index.
- Not a customer-specific LSP binary---the LSP loads packs dynamically.
- Not an ontology governance workflow UI---packs are the artifact that governance tooling produces.

## Pack Authority States

Every pack carries a `trust` object with two orthogonal dimensions: **approval state** and **signature state**.

| Approval State      | Signature State | What It Powers                                      |
|---------------------|-----------------|-----------------------------------------------------|
| `candidate`         | `unsigned`      | Local development, IDE exploration                  |
| `approved`          | `unsigned`      | CI warn mode, LSP with warnings, pre-release checks |
| `approved`          | `signed`        | CI strict mode, SEA-Forge production pipeline       |
| `rejected`          | any             | Nothing---rejected packs block downstream use       |

An approved-but-unsigned pack allows teams to iterate on definitions while still enforcing review coverage. Moving to `signed` is the final gate before production use.

## Three-Valued Semantic Truth

DomainForge uses three-valued logic for concept resolution:

| Truth Value | Meaning                                                        |
|-------------|----------------------------------------------------------------|
| `valid`     | The term resolved to an active, reviewed concept.              |
| `invalid`   | The term resolved to a rejected concept or a blocked alias.    |
| `unknown`   | The term could not be resolved, or resolved to a proposed concept, or is ambiguous. |

`unknown` stays distinct from `invalid` even when mapped to an error severity in strict mode. An `unknown` result means "we lack enough information to confirm or deny"---the tooling should encourage the user to add a definition or resolve the ambiguity rather than treat it as a hard rejection.

This separation lets the LSP offer "add to pack" quick-fix suggestions for `unknown` terms while still surfacing `invalid` terms as genuine errors.

## Pack Lifecycle

```
candidate --> approved_unsigned --> approved_signed --> retired/revoked
                  |                                      ^
                  +---- rejected ------------------------+
```

1. **Candidate**: Built from .sea source files. No review required. Suitable for local exploration.
2. **Approved (unsigned)**: All active concepts have matching review records with `approve` or `minor_amendment_no_semantic_change` decisions. Definition hashes are verified. Meaning version has been bumped if the fingerprint changed.
3. **Approved (signed)**: An Ed25519 detached signature covers the pack content hash. The signing payload includes the content hash, pack ID, and schema version.
4. **Retired/Revoked**: Listed in `replaces_pack_ids` of a successor pack. Downstream tools should treat retired packs as deprecated and redirect to the replacement.

A pack never transitions backward. Once approved, it cannot return to candidate. Once signed, the signature can only be invalidated (tampered) or superseded by a new signed version.

## Compatibility and Versioning

Each pack carries several versioning fields:

| Field                  | Purpose                                                                 |
|------------------------|-------------------------------------------------------------------------|
| `schema_version`       | The pack JSON schema version. Currently `0.3`.                          |
| `pack_version`         | Semver for the pack artifact itself (e.g., `1.2.0`).                    |
| `meaning_version`      | Semver for the semantic content. Must be bumped when `meaning_fingerprint` changes. |
| `meaning_fingerprint`  | SHA-256 hash of all concept definition records. Changes when any concept definition, status, or decision_ref changes. |
| `replaces_pack_ids`    | List of pack IDs that this pack supersedes.                             |

The build system enforces that `meaning_version` increases whenever `meaning_fingerprint` changes. A pack that changes meaning without bumping the version fails the build with `meaning_version_not_bumped`.

## CLI Examples

### Building a Pack

Build a candidate pack from .sea source files:

```bash
sea pack build \
  --source "models/**/*.sea" \
  --org acme \
  --domain logistics \
  --version 1.0.0 \
  --meaning-version 1.0.0 \
  --approval candidate \
  --out packs/acme-logistics-1.0.0.json
```

Build an approved pack with review records:

```bash
sea pack build \
  --source "models/**/*.sea" \
  --org acme \
  --domain logistics \
  --version 1.1.0 \
  --meaning-version 1.1.0 \
  --approval approved \
  --review reviews/logistics-review.jsonl \
  --previous-pack packs/acme-logistics-1.0.0.json \
  --out packs/acme-logistics-1.1.0.json
```

For the first approved pack in a domain, use `--allow-first-approved-version`:

```bash
sea pack build \
  --source "models/**/*.sea" \
  --org acme \
  --domain logistics \
  --version 1.0.0 \
  --meaning-version 1.0.0 \
  --approval approved \
  --review reviews/logistics-review.jsonl \
  --allow-first-approved-version \
  --out packs/acme-logistics-1.0.0.json
```

### Validating Against a Pack

Validate .sea files against a pack in warn mode:

```bash
sea pack validate \
  --pack packs/acme-logistics-1.1.0.json \
  --mode warn \
  models/payment.sea models/shipping.sea
```

Validate in strict mode with signature required:

```bash
sea pack validate \
  --pack packs/acme-logistics-1.1.0.json \
  --mode strict \
  --require-signature \
  --expected-hash sha256:abc123... \
  models/**/*.sea
```

### Inspecting a Pack

```bash
sea pack inspect --pack packs/acme-logistics-1.1.0.json

# JSON output for tooling
sea pack inspect --pack packs/acme-logistics-1.1.0.json --format json
```

### Diffing Two Packs

```bash
sea pack diff \
  --old packs/acme-logistics-1.0.0.json \
  --new packs/acme-logistics-1.1.0.json

# JSON output
sea pack diff \
  --old packs/acme-logistics-1.0.0.json \
  --new packs/acme-logistics-1.1.0.json \
  --format json
```

Diff classifications:

| Classification         | Meaning                                                |
|------------------------|--------------------------------------------------------|
| `additive`             | New concept added.                                     |
| `definitional_change`  | Definition hash changed (non-breaking).                |
| `deprecating`          | Concept status changed to deprecated.                  |
| `breaking`             | Active concept removed, or meaning changed without version bump. |
| `governance_critical`  | Status transition that is not deprecated or rejected.  |
| `signature_only`       | Only the signature changed, semantic content is identical. |

### Signing and Verifying

```bash
# Sign a pack
sea pack sign packs/acme-logistics-1.1.0.json --key keys/acme-private.pem --out packs/acme-logistics-1.1.0-signed.json

# Verify a pack signature
sea pack verify packs/acme-logistics-1.1.0-signed.json --key keys/acme-public.pem
```

## Workspace Configuration

Add a `.domainforge/config.json` to your project root to configure pack resolution:

```json
{
  "semantic_packs": [
    {
      "path": "packs/acme-logistics-1.1.0.json",
      "priority": 0
    },
    {
      "path": "packs/acme-shared-2.0.0.json",
      "priority": 1
    }
  ],
  "validation_mode": "warn",
  "deprecated_policy": "warn",
  "require_signed_packs": false
}
```

The LSP reads this configuration on startup and resolves terms against the configured packs in priority order.

## CI Strict Mode

For CI pipelines that enforce semantic correctness:

```bash
#!/bin/bash
set -euo pipefail

# Build and sign the pack
sea pack build \
  --source "models/**/*.sea" \
  --org "$ORG" \
  --domain "$DOMAIN" \
  --version "$VERSION" \
  --meaning-version "$MEANING_VERSION" \
  --approval approved \
  --review "reviews/review.jsonl" \
  --previous-pack "packs/previous.json" \
  --out "packs/current-unsigned.json"

sea pack sign "packs/current-unsigned.json" \
  --key "$SIGNING_KEY_PATH" \
  --out "packs/current.json"

# Validate all source files in strict mode
sea pack validate \
  --pack "packs/current.json" \
  --mode strict \
  --require-signature \
  --expected-hash "$EXPECTED_HASH" \
  models/**/*.sea
```

Exit codes for pack validation:

| Code | Meaning                          |
|------|----------------------------------|
| 0    | Validation passed.               |
| 1    | Semantic validation failed.      |
| 2    | Parse error in input file.       |
| 3    | Pack unsigned when required.     |
| 4    | Signature verification failed.   |

## Consumer Contract

Downstream tools that consume semantic packs must follow these rules:

1. **Never modify a pack in place.** Treat packs as immutable artifacts. To change vocabulary, build a new pack version.
2. **Always verify the signature before trusting content.** Unsigned packs are acceptable in development; signed packs are required in production.
3. **Respect the three-valued truth.** Map `unknown` to actionable suggestions, not hard errors, unless in strict mode.
4. **Check `meaning_version` before `pack_version`.** A meaning change without a version bump is a bug, not a feature.
5. **Use `normalize_lookup_key` for term matching.** The normalization algorithm (NFC, case-fold, whitespace collapse) is the canonical equality function for lookup keys.
6. **Honor `replaces_pack_ids`.** When loading multiple packs, redirect references to superseded packs to the replacement.

### Consumers

- **Context Kernel**: Uses packs to build context windows for LLM-assisted architecture modeling.
- **SWE_SEED**: Uses packs to validate that generated code references approved business concepts.
- **SEA-Forge Pipeline**: Uses signed packs as the authoritative vocabulary for production code generation.
- **DomainForge LSP**: Loads packs at workspace initialization and provides real-time diagnostics.

## See Also

- [Semantic Pack Review Process](semantic-pack-review.md)
- [Semantic Pack Signing and Verification](semantic-pack-signing.md)
- [Semantic Diagnostic Codes](diagnostics.md)
- [Semantic Modeling Concepts](explanations/semantic-modeling-concepts.md)
