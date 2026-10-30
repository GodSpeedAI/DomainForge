# TLA+ Specification Projection

## Overview
Projects a SEA architecture into a TLA+ spec plus TLC config describing flows as a state machine — a `holder` function over flow-bearing resources with one `Transfer<R>` action per flow — so behavior can be model-checked.

## Interface
- `[mod.rs:emit]` — emits `<ns>.tla` and `<ns>.cfg` into an `ArtifactSink`; returns both relative paths.
- `[mod.rs:project_tla_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[mod.rs:build_tla]` — renders the spec module.
- `[mod.rs:build_cfg]` — renders the TLC config (CONSTANT assignments, SPECIFICATION, INVARIANT bindings).

## Invariants
- `[mod.rs:emit]` — output is exactly two files, both named from `sanitize_filename(namespace)` (`<base>.tla`, `<base>.cfg`).
- `[mod.rs:build_tla]` — the module header is `MODULE <ns>` and each entity and resource becomes a `CONSTANT`; `NameRegistrar` assigns entities first then resources so identifiers are deterministic, and collision-getting names receive a hash suffix so the spec never has duplicate `CONSTANT`s.
- `[mod.rs:build_tla]` — an empty model emits `CONSTANTS ModelValue` rather than an empty constant list.
- `[mod.rs:build_tla]` — the movable resources are the deduped, sorted set of flow resources, so `VARIABLES holder` ranges over exactly the flow-bearing resources.
- `[mod.rs:build_tla]` — `Init` starts every flow-bearing resource held by its first flow's source; each flow yields a `Transfer<R>` guarded on `holder[R] = E1`; `Next` is the disjunction of transfers and `Spec == Init /\ [][Next]_holder`.
- `[mod.rs:build_tla]` — `TypeInvariant` keeps `holder` mapping resources into the entity set.
- `[mod.rs:emit]` — flow and namespace resolution go through `collect_flows` / `model_namespace`, so dangling references and multi-namespace models are hard errors.
- Sorted output ⇒ byte-identical for a fixed `created_at`.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
