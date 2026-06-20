# Semantic Pack Review Process

Every semantic pack that transitions from `candidate` to `approved` must pass a review gate. This document describes the review manifest format, the allowed decisions, and the checks the build system performs.

## Why Review Is Required

Semantic packs define the vocabulary that downstream tools---the LSP, CI validators, code generators---treat as authoritative. An unreviewed pack that introduces a misspelled concept name or an ambiguous alias can silently corrupt generated code, produce misleading diagnostics, or break compatibility with existing consumers.

Review is the **bootstrapping boundary**: it is the moment when a human confirms that the vocabulary is intentional, the definitions are correct, and the ownership is assigned. The build system enforces that every active concept in an approved pack has a matching review record.

## Review Manifest Format

Review records are stored as JSONL (one JSON object per line). Each record captures a single review decision:

```jsonl
{"decision_id":"rev-001","subject_type":"concept","subject_id":"supplier","decision":"approve","rationale":"Approved after stakeholder review","reviewer":"alice@acme.com","reviewed_at":"2025-06-01T10:00:00Z","definition_hash":"sha256:a1b2c3...","previous_definition_hash":null,"new_definition_hash":null}
{"decision_id":"rev-002","subject_type":"concept","subject_id":"warehouse","decision":"approve","rationale":"Matches supply chain taxonomy","reviewer":"bob@acme.com","reviewed_at":"2025-06-01T11:00:00Z","definition_hash":"sha256:d4e5f6...","previous_definition_hash":null,"new_definition_hash":null}
```

### Fields

| Field                        | Type   | Description                                                        |
|------------------------------|--------|--------------------------------------------------------------------|
| `decision_id`                | string | Unique identifier for this review decision.                        |
| `subject_type`               | string | What is being reviewed. Currently `concept`.                       |
| `subject_id`                 | string | The `id` of the concept being reviewed.                            |
| `decision`                   | string | The review decision. See below for allowed values.                 |
| `rationale`                  | string | Human-readable explanation for the decision.                       |
| `reviewer`                   | string | Identifier for the reviewer (email, username, or system ID).       |
| `reviewed_at`                | string | RFC 3339 timestamp of when the review occurred.                    |
| `definition_hash`            | string | The definition hash at the time of review.                         |
| `previous_definition_hash`   | string | (Optional) The hash before a minor amendment.                      |
| `new_definition_hash`        | string | (Optional) The hash after a minor amendment.                       |

## Allowed Decision Values

| Decision                              | Effect                                                                 |
|---------------------------------------|------------------------------------------------------------------------|
| `approve`                             | The concept is approved for inclusion in the pack.                     |
| `reject`                              | The concept must not appear in an approved pack.                       |
| `deprecate`                           | The concept is approved but marked for future removal.                 |
| `merge_into`                          | The concept should be merged into another concept.                     |
| `split_required`                      | The concept covers too many meanings and must be split.                |
| `needs_definition`                    | The concept lacks a definition and must be defined before approval.    |
| `needs_owner`                         | The concept lacks an owner and must have one assigned.                 |
| `minor_amendment_no_semantic_change`  | A cosmetic change (formatting, typo fix) that does not alter semantics. |

Only `approve` and `minor_amendment_no_semantic_change` count as passing reviews for the purpose of review coverage.

## Definition Hash Matching

When building an approved pack, the builder validates that each active concept's `definition_hash` matches the hash recorded in the review manifest. This ensures that no concept has been modified after review without a corresponding review record.

The builder checks hashes in order:

1. **Match**: The current definition hash matches the reviewed hash. Pass.
2. **Minor Amendment**: The decision is `minor_amendment_no_semantic_change` and the `new_definition_hash` matches the current hash. Pass.
3. **Mismatch**: The current hash differs from the reviewed hash and no minor amendment covers the transition. The build emits an `unreviewed_concept` diagnostic.
4. **No Review**: No review record exists for this concept. The build emits an `unreviewed_concept` diagnostic.

### What Happens on Mismatch

A hash mismatch blocks the approved build. The error message includes both the reviewed hash and the current hash:

```
[ERROR] unreviewed_concept: Definition hash mismatch for 'supplier': reviewed='sha256:a1b2c3...', current='sha256:x9y8z7...'
  hint: Re-review or record minor_amendment_no_semantic_change
```

To resolve a mismatch:

- **If the change is cosmetic** (whitespace, formatting, typo in examples): add a `minor_amendment_no_semantic_change` review record with `previous_definition_hash` set to the old hash and `new_definition_hash` set to the current hash.
- **If the change is semantic** (definition text, status, meaning): add a full `approve` review record with the new definition hash.

## Minor Amendment Path

The minor amendment path exists because small formatting changes should not require a full re-review cycle. The rules are:

1. A minor amendment must still have a `decision_id`, a `reviewer`, and a `reviewed_at` timestamp.
2. The amendment must include both `previous_definition_hash` and `new_definition_hash`.
3. The amendment does not reset the `meaning_version`---it is explicitly a non-semantic change.

Example minor amendment record:

```jsonl
{"decision_id":"rev-042","subject_type":"concept","subject_id":"supplier","decision":"minor_amendment_no_semantic_change","rationale":"Fixed typo in counterexample","reviewer":"alice@acme.com","reviewed_at":"2025-06-05T14:00:00Z","definition_hash":"sha256:a1b2c3...","previous_definition_hash":"sha256:a1b2c3...","new_definition_hash":"sha256:m3n4o5..."}
```

## Operational Cost

Any change to a concept's `text`, `examples`, `counterexamples`, or `status` changes the `definition_hash`. This means:

- **Fixing a typo in an example** changes the hash and forces re-review (or a minor amendment).
- **Adding a counterexample** changes the hash and forces re-review.
- **Changing status** (e.g., `active` to `deprecated`) changes the hash.

This is intentional. The hash covers the full definition surface to prevent silent semantic drift. Teams should expect that most changes to active concepts require a review record.

## CLI Usage

### Building an Approved Pack with Review

```bash
domainforge pack build \
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

The `--review` flag points to the JSONL file containing review records. The builder reads all records, validates coverage for active concepts, and checks definition hash matches.

### Review Coverage Errors

If any active concept lacks a passing review, the build fails:

```
[ERROR] unreviewed_concept: Concept 'new_concept' lacks review approval
  hint: Add review decision for this concept
```

### Definition Hash Mismatch Errors

If a concept's definition changed after review without a new review record:

```
[ERROR] unreviewed_concept: Definition hash mismatch for 'supplier': reviewed='sha256:a1b2c3...', current='sha256:x9y8z7...'
  hint: Re-review or record minor_amendment_no_semantic_change
```

## Review Manifest Hash

The builder computes a deterministic hash of the review manifest itself (`review_manifest_hash`). This hash is included in the pack so downstream consumers can verify that the review records used during the build have not been tampered with. The hash is computed by sorting review records by `decision_id` and hashing the canonical JSON representation.

## See Also

- [Semantic Packs](semantic-packs.md) for the overall pack system.
- [Semantic Pack Signing and Verification](semantic-pack-signing.md) for the signing layer that runs after review.
- [Semantic Diagnostic Codes](diagnostics.md) for `unreviewed_concept` and related codes.
