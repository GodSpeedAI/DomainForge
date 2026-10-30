# AsyncAPI 3.0 Projection

## Overview
Projects a SEA `Graph` into a single AsyncAPI 3.0 YAML document describing the event-driven contract — channels, send/receive operations, messages, and payload schemas — derived from entities, flows, and resources. It is the contract companion to the CloudEvents envelope projection, not a broker.

## Interface
- `[mod.rs:OUTPUT_FILE]` — the single emitted artifact's relative path, `asyncapi.yaml`.
- `[mod.rs:ASYNCAPI_VERSION]` — the targeted document version, `3.0.0`.
- `[mod.rs:emit]` — builds and serializes the document into an `ArtifactSink`; returns the emitted relative paths.
- `[mod.rs:project_asyncapi_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.

## Invariants
- `[mod.rs:emit]` — exactly one artifact (`asyncapi.yaml`) is emitted; a provenance comment header is prepended as text because the YAML serializer emits no document comment.
- `[mod.rs:emit]` — YAML is produced through `crate::projection::yaml::to_yaml`, which disables block scalars so every string stays a single scalar and ambiguous scalars are quoted.
- `[mod.rs:build_spec]` — the emitted document validates against the vendored official AsyncAPI 3.0.0 schema (`schemas/asyncapi/3.0.0.json`).
- Every operation carries a `#/channels/...` `$ref` and a `#/components/messages/...` `$ref`.
- Output is byte-identical run-to-run for a fixed model (sorted maps; caller-supplied timestamp only).

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write`: filesystem write under the output root, or insertion into an in-memory map.
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
