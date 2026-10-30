# Lean 4 Package Projection

## Overview
Emits a self-contained, zero-dependency Lake package from a SEA graph under a two-layer proof discipline: the `DomainForge` library must compile with zero `sorry`, while non-auto-groundable properties become documented `sorry` stubs in a separate `Obligations` library.

## Interface
- `[mod.rs:LEAN_TOOLCHAIN]` — the pinned Lean toolchain (`leanprover/lean4:v4.15.0`) written to `lean-toolchain`.
- `[mod.rs:emit]` — emits the nine-file Lake package (`lean-toolchain`, `lakefile.toml`, `README.md`, `DomainForge.lean`, `DomainForge/{Types,Model,Policies}.lean`, `Obligations.lean`, `Obligations/Stubs.lean`) into an `ArtifactSink`.
- `[mod.rs:project_lean_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[expr.rs:Lowered]` — the lowering result: `Groundable { lean, holds }` or `Deferred { reason }`.
- `[expr.rs:GroundCtx]` — shared ground context: the decimal scale `k` and the scaled flow quantities.
- `[expr.rs:lower_policy]` — lowers a `policy::Expression` into a Lean `Prop` (or defers it).
- `[expr.rs:collect_decimals]` / `[expr.rs:scaled]` — decimal collection and exact integer scaling.
- `[expr.rs:lean_ident]` — Lean identifier sanitizer.
- `[expr.rs:unique_idents]` — collision-free identifier list for a set of names.

## Invariants
- `[expr.rs]` (module doc) — the Rust evaluator evaluates exactly the semantics that get emitted to Lean (same scaled integers, same operators), so a `decide` proof emitted as "holds" is guaranteed to check.
- `[expr.rs:lower_policy]` — an expression that does not lower to a proposition is `Deferred`, never silently treated as groundable.
- `[expr.rs:lower_policy]` — a flow-referencing policy defers when the model declares no flows (`flow_quantities` is `None`).
- `[mod.rs:LEAN_TOOLCHAIN]` — bumping the toolchain requires changing this constant and confirming the `verify-lean` CI job (`lake build`) is green.
- Only the `DomainForge` layer is CI-gated; `Obligations` deliberately contains `sorry` stubs.
- All graph collections iterate in sorted order, so output is byte-identical for a fixed `created_at`.

## Side Effects
- `[mod.rs:emit]` — writes nine artifacts through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
