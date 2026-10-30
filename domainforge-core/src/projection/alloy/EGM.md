# Alloy Model Projection

## Overview
Projects a SEA `Graph` into a single Alloy (`.als`) formal model — signatures for entities/resources plus one relational fact per `Flow` — so the architecture structure can be checked with the Alloy Analyzer.

## Interface
- `[mod.rs:emit]` — emits the Alloy model into an `ArtifactSink`; returns the one emitted relative path.
- `[mod.rs:project_alloy_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.

## Invariants
- `[mod.rs:emit]` — output filename is `<sanitize_filename(namespace)>.als`, so one model yields exactly one artifact.
- `[mod.rs:build_model]` — entities and resources are sorted and deduped; signature names are minted through `NameRegistrar` so sanitization collisions get a hash suffix.
- `[mod.rs:build_model]` — the namespace is registered as an `ident`, so `module supply chain` becomes `module supply_chain` (Alloy requires a valid module identifier).
- `[mod.rs:build_model]` — flow resolution goes through `collect_flows`, so a dangling flow reference is a hard error rather than a silently dropped fact.
- `[mod.rs:build_model]` — the `run {} for N` scope is `max(3, flow_count)`, so multi-flow models remain satisfiable instead of reporting a false inconsistency.
- `[mod.rs:build_model]` — output is byte-identical run-to-run for a fixed `created_at` (sorted signatures, flows sorted by `(resource, from, to)`).

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write`: filesystem write under the output root, or insertion into an in-memory map.
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
