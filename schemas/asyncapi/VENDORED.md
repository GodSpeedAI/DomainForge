# Vendored AsyncAPI 3.0.0 Schema

Pinned copy of the official AsyncAPI 3.0.0 JSON Schema, consumed by the
AsyncAPI projection's spec-validation test
(`domainforge-core/tests/asyncapi_spec_validation_tests.rs`) and the
`scripts/verify/projection-targets/asyncapi.sh` gate.

| File | Role |
| --- | --- |
| `3.0.0.json` | AsyncAPI 3.0.0 document schema (JSON Schema draft-07, self-contained) |

**Provenance:** Fetched verbatim from the canonical AsyncAPI spec-json-schemas
repository (`asyncapi/spec-json-schemas`, `schemas/3.0.0.json`) via the jsDelivr
GH mirror. Unmodified. The AsyncAPI 3.0.0 schema is stable.

The schema uses internal `$id`-rooted references (e.g.
`http://asyncapi.com/definitions/3.0.0/…`); the `jsonschema` crate resolves
these in-document at compile time, so no network access is needed at
validation time.

To update: replace `3.0.0.json` with a newer pinned copy from the
`asyncapi/spec-json-schemas` repository, bump
`ASYNCAPI_VERSION` in `domainforge-core/src/projection/asyncapi/mod.rs`
if the major version changes, and re-run
`cargo test -p domainforge-core --features cli --test asyncapi_spec_validation_tests`
plus the `asyncapi.sh` gate.
