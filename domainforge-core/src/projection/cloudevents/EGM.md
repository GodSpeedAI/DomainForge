# CloudEvents 1.0 Projection

## Overview
Projects each SEA `Flow` (a resource moving from one entity to another) into a CloudEvents 1.0 envelope so an architecture model can be consumed directly by event-driven runtimes.

## Interface
- `[mod.rs:OUTPUT_FILE]` — the single emitted artifact's relative path, `events.jsonl`.
- `[mod.rs:SPEC_VERSION]` — the targeted CloudEvents spec version, `1.0`.
- `[mod.rs:emit]` — emits the JSONL stream into an `ArtifactSink`; returns the emitted relative path.
- `[mod.rs:project_cloudevents_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.

## Invariants
- `[mod.rs:emit]` — `events.jsonl` is strict JSON Lines: every line is a valid JSON document and there is no `#` comment header, so provenance never occupies line 1.
- `[mod.rs:build_events]` — each event `id` is minted by `element_id("cloudevents", ..)` from the flow's stable content (source entity, resource, target entity, quantity); random per-parse flow UUIDs never reach output.
- `[mod.rs:build_events]` — `source` and `type` fragments are `slug`-encoded, so raw spaces never appear in either attribute.
- `[mod.rs:build_events]` — events are sorted by deterministic id, making output byte-identical run-to-run for a fixed `created_at`.
- `[mod.rs:build_events]` — flow resolution goes through `collect_flows`; dangling references are a hard error.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write`: filesystem write under the output root, or insertion into an in-memory map.
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
