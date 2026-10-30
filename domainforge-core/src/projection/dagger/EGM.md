# Dagger Activation Module Projection

## Overview
Projects a SEA model into a minimal runnable Dagger module — a `dagger.json` manifest plus a single-file Python module (`main.py`, `pyproject.toml`) — so `dagger call` exposes one function per `Flow`.

## Interface
- `[mod.rs:MANIFEST]` — emitted path `dagger.json`.
- `[mod.rs:MODULE_SRC]` — emitted path `main.py` (the Python module source).
- `[mod.rs:PYPROJECT]` — emitted path `pyproject.toml`.
- `[mod.rs:DAGGER_ENGINE_VERSION]` — the pinned Dagger engine version (`v0.21.4`) stamped into the manifest.
- `[mod.rs:emit]` — writes the three artifacts into an `ArtifactSink` and returns their relative paths.
- `[mod.rs:project_dagger_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.

## Invariants
- `[mod.rs:emit]` — exactly three artifacts, emitted in order manifest → module source → pyproject.
- `[mod.rs:emit]` — the module is named after `model_namespace`, pascal-cased into the object-type class name; a multi-namespace model is a hard error.
- `[mod.rs:emit]` — each `Flow` becomes one `@dagger.function` named `run_<resource>_<from>_<to>`.
- `[mod.rs:build_manifest]` — manifest keys emit in sorted order because `serde_json::Map` is BTreeMap-backed, so output is byte-identical run-to-run.
- The projection emits only these three files; `dagger develop --sdk=python` generates the SDK layer on top.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
