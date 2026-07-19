# Semantic Diagnostic Codes

DomainForge emits structured diagnostics when validating .sea source files against semantic packs. Each diagnostic carries a code, a three-valued semantic truth, a severity, and contextual information. This reference documents all 30 diagnostic codes, their truth values, and how they behave in different validation modes.

## Three-Valued Semantic Truth

Every diagnostic has a `semantic_truth` field that captures the resolution outcome independently of severity:

| Truth      | Meaning                                                          |
|------------|------------------------------------------------------------------|
| `valid`    | The concept resolved successfully. No semantic issue.            |
| `invalid`  | The concept is definitively wrong (rejected, blocked).           |
| `unknown`  | Insufficient information to confirm or deny (missing, ambiguous, proposed). |

`semantic_truth` is **independent** of `DiagnosticSeverity`. An `unknown` truth can map to `Error` severity in strict mode, but the truth value stays `unknown`---not `invalid`. This distinction matters for tooling that uses the truth value for control flow (e.g., the LSP uses `unknown` to offer "add to pack" suggestions, while `invalid` triggers error highlighting).

## Complete Diagnostic Code Table

### Concept Resolution

| Code                    | Semantic Truth | LSP Warn Severity | CI Strict Severity | Description                                    |
|-------------------------|----------------|-------------------|--------------------|------------------------------------------------|
| `unknown_concept`       | unknown        | Warning           | Error              | Term does not match any concept, alias, or mapping rule. |
| `deprecated_concept`    | valid          | Warning           | Varies (policy)    | Concept resolved but has `deprecated` status.  |
| `ambiguous_concept`     | unknown        | Warning           | Error              | Term matches multiple active concepts.          |
| `ambiguous_alias`       | unknown        | Warning           | Error              | Alias matches only `ambiguous`-status entries across multiple targets. |
| `alias_conflict`        | invalid        | Error             | Error              | Alias is `blocked` or has conflicting approved/deprecated entries targeting different concepts. |
| `rejected_concept`      | invalid        | Error             | Error              | Concept resolved but has `rejected` status.    |
| `proposed_concept`      | unknown        | Warning           | Error              | Concept resolved but has `proposed` status.    |

### Alias Resolution

| Code                    | Semantic Truth | LSP Warn Severity | CI Strict Severity | Description                                    |
|-------------------------|----------------|-------------------|--------------------|------------------------------------------------|
| `ambiguous_alias_group` | unknown        | Warning           | Error              | Multiple aliases for the same key are all in `ambiguous` status. |

### Pack Integrity

| Code                        | Semantic Truth | LSP Warn Severity | CI Strict Severity | Description                                    |
|-----------------------------|----------------|-------------------|--------------------|------------------------------------------------|
| `pack_unavailable`          | unknown        | Error             | Error              | No pack could be loaded for validation.         |
| `pack_schema_mismatch`      | invalid        | Error             | Error              | Pack `schema_version` is not `0.3`.             |
| `pack_version_mismatch`     | invalid        | Warning           | Error              | Pack version is incompatible with tooling.      |
| `pack_hash_mismatch`        | invalid        | Error             | Error              | Pack content hash does not match expected hash.  |
| `pack_unsigned`             | invalid        | Warning           | Error              | Pack is unsigned but signature is required.      |
| `pack_signature_invalid`    | invalid        | Error             | Error              | Pack signature verification failed.              |
| `pack_set_conflict`         | invalid        | Error             | Error              | Multiple packs have conflicting definitions.     |

### Relation and Metric Validation

| Code                    | Semantic Truth | LSP Warn Severity | CI Strict Severity | Description                                    |
|-------------------------|----------------|-------------------|--------------------|------------------------------------------------|
| `invalid_relation`      | invalid        | Error             | Error              | Relation references a concept that does not exist in the pack. |
| `invalid_predicate`     | invalid        | Error             | Error              | Predicate is not in the concept's `allowed_predicates` list. |
| `unknown_metric`        | unknown        | Warning           | Error              | Metric ID is not defined in the pack.           |
| `unknown_dimension`     | unknown        | Warning           | Error              | Dimension ID referenced by a metric is not defined. |
| `unknown_unit`          | unknown        | Warning           | Error              | Unit ID referenced by a metric is not defined.  |
| `unit_mismatch`         | invalid        | Error             | Error              | Unit is incompatible with the dimension.        |
| `dimension_mismatch`    | invalid        | Error             | Error              | Dimension does not match the expected dimension. |

### Governance

| Code                                | Semantic Truth | LSP Warn Severity | CI Strict Severity | Description                                    |
|-------------------------------------|----------------|-------------------|--------------------|------------------------------------------------|
| `missing_definition`                | unknown        | Warning           | Warning            | Active concept has no definition text.          |
| `missing_owner`                     | unknown        | Warning           | Warning            | Active concept has no owner assigned.           |
| `duplicate_canonical_name`          | invalid        | Error             | Error              | Two concepts share the same canonical name.     |
| `duplicate_concept_id`              | invalid        | Error             | Error              | Two concepts share the same ID.                 |
| `unreviewed_concept`                | unknown        | Error             | Error              | Active concept lacks a passing review record or has a definition hash mismatch. |
| `meaning_version_not_bumped`        | invalid        | Error             | Error              | `meaning_fingerprint` changed but `meaning_version` was not increased. |
| `meaning_version_baseline_missing`  | unknown        | Error             | Error              | Approved build has no previous pack for version comparison. |
| `mapping_required`                  | unknown        | Warning           | Error              | Term must be mapped to an approved concept.     |

## DeprecatedPolicy Behavior

The `deprecated_concept` diagnostic severity is controlled by the `DeprecatedPolicy` setting:

| Policy            | Warn Mode Severity | Strict Mode Severity | Description                                 |
|-------------------|--------------------|----------------------|---------------------------------------------|
| `Allow`           | Hint               | Hint                 | Deprecated concepts are silently accepted.  |
| `Warn`            | Warning            | Warning              | Deprecated concepts produce warnings in all modes. |
| `ErrorInStrict`   | Warning            | Error                | Warnings in warn mode, errors in strict mode. |
| `ErrorAlways`     | Error              | Error                | Deprecated concepts always produce errors.  |

This policy is orthogonal to the `semantic_truth` value---a deprecated concept always has truth `valid` because it resolved successfully. The severity controls whether the resolution result is treated as actionable.

## How semantic_truth Is Independent of DiagnosticSeverity

The separation between truth and severity serves distinct consumers:

- **LSP**: Uses `semantic_truth` to decide what quick-fix actions to offer. `unknown` suggests "add to pack." `invalid` suggests "remove or fix reference." `valid` with `deprecated` suggests "update to replacement."
- **CI**: Uses `severity` to decide pass/fail. A `Warning` in strict mode is still a failure if the policy maps it to `Error`.
- **Code generators**: Use `semantic_truth` to decide whether to include a concept in generated output. `invalid` concepts are excluded. `unknown` concepts may be included with a TODO comment.

Example: A term that resolves to a `proposed` concept has truth `unknown` and severity `Warning` in warn mode. In strict mode, the severity becomes `Error`, but the truth stays `unknown`. The CI fails (severity = Error), but the code generator can still include the concept with a caveat (truth = unknown).

## LSP Diagnostic Data Payload

The DomainForge LSP attaches structured data to each diagnostic via the LSP `data` field:

```json
{
  "domainforge.semantic_code": "unknown_concept",
  "domainforge.semantic_truth": "unknown",
  "domainforge.pack_id": "acme/logistics/1.1.0",
  "domainforge.pack_content_hash": "sha256:abc123...",
  "domainforge.recoverability_hint": "Add the concept to the semantic pack",
  "domainforge.suggestions": [
    { "label": "supplier", "rank": 10 },
    { "label": "warehouse", "rank": 5 }
  ]
}
```

### Payload Fields

| Field                              | Type     | Description                                              |
|------------------------------------|----------|----------------------------------------------------------|
| `domainforge.semantic_code`        | string   | The `SemanticDiagnosticCode` value.                      |
| `domainforge.semantic_truth`       | string   | The three-valued truth (`valid`, `invalid`, `unknown`).  |
| `domainforge.pack_id`              | string   | The pack that produced this diagnostic.                  |
| `domainforge.pack_content_hash`    | string   | Content hash of the pack at validation time.             |
| `domainforge.recoverability_hint`  | string   | Human-readable suggestion for fixing the issue.          |
| `domainforge.suggestions`          | array    | Ranked list of concept suggestions (for `unknown` results). |

## Validation Modes

| Mode      | Behavior                                                                 |
|-----------|--------------------------------------------------------------------------|
| `off`     | No semantic validation is performed. Pack is not loaded.                 |
| `warn`    | Diagnostics are reported as warnings. Unknown concepts produce warnings. |
| `strict`  | Unknown concepts are errors. Deprecated concepts may be errors (per policy). Unsigned packs are errors if `require_signed_pack` is true. |

## Validation Options

The `ValidationOptions` structure controls validation behavior:

```json
{
  "mode": "strict",
  "unknown_concept_policy": "error",
  "deprecated_policy": "error_in_strict",
  "require_signed_pack": true,
  "allow_unsigned_test_fixtures": false
}
```

| Field                          | Type     | Default   | Description                                          |
|--------------------------------|----------|-----------|------------------------------------------------------|
| `mode`                         | string   | `warn`    | Validation mode (`off`, `warn`, `strict`).           |
| `unknown_concept_policy`       | string   | `warning` | How to handle unknown concepts (`ignore`, `warning`, `error`). |
| `deprecated_policy`            | string   | `warn`    | How to handle deprecated concepts. See table above.  |
| `require_signed_pack`          | boolean  | `false`   | Whether to require a signed pack.                    |
| `allow_unsigned_test_fixtures` | boolean  | `false`   | Bypass signature requirement for test fixture packs. |

## Application Contract Diagnostics (APP001–APP015)

Application contract resolution emits its own stable registry, separate from
the E-code and semantic-pack tables above. See
[error-codes.md](reference/error-codes.md#app001-app015-application-contract-diagnostics)
for the full table. Key properties:

- Wire form is `{code, slug, severity, message, context}`; `severity` is
  always `"error"` and blocks contract emission.
- `context` carries `reason`, `expected`, `actual`, `remediation`, and source
  evidence (`logical_module_id`, `line`, `column`); inapplicable values are
  omitted, never serialized as empty strings.
- APP014 always includes one closed `reason` value; APP015 instead identifies
  the artifact `document_kind`, `schema_version`, and failing JSON Pointer or
  hash field.
- Diagnostics are sorted by logical module ID, line, column, then code, so
  output is deterministic for one input source set.

## See Also

- [Semantic Packs](semantic-packs.md) for the overall pack system.
- [Semantic Pack Review Process](semantic-pack-review.md) for governance codes.
- [Semantic Pack Signing and Verification](semantic-pack-signing.md) for signature-related codes.
- [Three-Valued Logic](explanations/three-valued-logic.md) for the theoretical background.
