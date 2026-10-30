# OpenShell Sandbox Policy Projection

## Overview
Projects the Cell IR into an NVIDIA OpenShell sandbox policy plus the parameters OpenShell keeps outside the policy (`sandbox.json`, `mapping-report.json`, `openshell.lock`). This is an execution-policy view of the same `CellIr` the `cell` target renders — there is no second reading of the `.sea` source.

## Interface
- `[mod.rs:OPENSHELL_VERSION]` — the targeted OpenShell schema version, `0.1.3`.
- `[mod.rs:PROJECTION_VERSION]` — the projection identity tag, `domainforge-openshell/v1`.
- `[mod.rs:SANDBOX_SCHEMA]` / `[mod.rs:MAPPING_SCHEMA]` / `[mod.rs:LOCK_SCHEMA]` — the per-artifact schema ids (`domainforge.openshell.{sandbox,mapping,lock}/v1`).
- `[mod.rs:POLICY_FILE]` — `policy.yaml`.
- `[mod.rs:SANDBOX_FILE]` — `sandbox.json`.
- `[mod.rs:MAPPING_FILE]` — `mapping-report.json`.
- `[mod.rs:LOCK_FILE]` — `openshell.lock`.
- `[mod.rs:OpenShellProjectionError]` — typed errors: `Unrepresentable` (OSH001), `Cell` (OSH002), `Write` (OSH003).
- `[mod.rs:emit]` — builds the `CellIr`, projects it, and writes the four artifacts into an `ArtifactSink`.
- `[mod.rs:project_openshell_in_memory]` — binding surface: source + overrides in, `BTreeMap<path, content>` out, no filesystem.
- `[mod.rs:project]` — the `CellIr` → OpenShell `Projection` lowering.

## Invariants
- `[mod.rs]` (module doc) — anything the Cell requires that OpenShell cannot enforce becomes a typed `Unrepresentable` error or an entry under `obligations` in `sandbox.json`; nothing is omitted.
- `[mod.rs]` (module doc) — no rule is emitted in audit mode or with a wildcard.
- `[mod.rs:BASELINE_READ_ONLY]` — the baseline read-only path list must be a superset of OpenShell's supervisor-enriched baseline (`/usr`, `/lib`, `/etc`, `/app`, `/var/log`, `/proc`, `/dev/urandom`, …), so the projected Cell boundary contains the enforced policy; the superset test in `tests/openshell_projection_tests.rs` pins it and must be re-checked on every OpenShell bump.
- `[mod.rs:EXECUTABLE_DIRS]` — install paths are exact (no globs) because OpenShell binds network rules to binary identity and the prover cannot reason about globs.
- `[mod.rs:emit]` — `openshell.lock` records `policy_digest` and a sha256 per emitted file, plus a canonical-source model hash, so the lock covers every artifact.
- `[mod.rs:emit]` — `openshell.lock` is the only artifact carrying `created_at`; the others are deterministic.

## Side Effects
- `[mod.rs:emit]` — writes four artifacts through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.
- `[mod.rs:project_openshell_in_memory]` — parses the `.sea` source and the optional overrides document before projecting.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
