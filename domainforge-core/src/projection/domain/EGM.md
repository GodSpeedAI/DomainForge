# Domain Code Projection Family

## Overview
One language-neutral DDD/CQRS IR (`ir.rs`) consumed by three pure renderers (`python.rs`, `typescript.rs`, `rust.rs`): all domain semantics are decided once in `DomainIr::from_graph`, and renderers only translate IR into language syntax.

## Interface
- `[ir.rs:DomainIr::from_graph]` — the single place the SEA → DDD/CQRS mapping is decided; returns the `DomainIr` for a graph.
- `[ir.rs:GuardCheck]` — a no-op guard hook (`check_<slug(policy)>`) plus the `<Pascal>Violation` error it may raise.
- `[ir.rs:MethodIr]` — an aggregate-root method minted from one flow (e.g. `transfer_to_<slug(to)>`).
- `[ir.rs:AggregateIr]` — aggregate root (from a resource) with its repository port and quantity unit.
- `[ir.rs:EntityIr]` — DDD entity (id + name; fields from `instance of`).
- `[ir.rs:CommandIr]` / `[ir.rs:EventIr]` — command and event minted from a flow, per its `@cqrs` kind.
- `[ir.rs:ErrorIr]` / `[ir.rs:QueryIr]` / `[ir.rs:ValueObjectIr]` / `[ir.rs:InstanceIr]` — errors from policies, queries from metrics, value objects from patterns, and instance literals.
- `[python.rs:emit]` — Python renderer; emits `pyproject.toml`, `README.md`, and a `src/<pkg_slug>_domain/` package (`domain/{value_objects,entities,roles,errors,commands,events,aggregates}.py`, `ports/{repositories,bus}.py`, `container.py`, `__init__.py` files).
- `[typescript.rs:emit]` — TypeScript renderer; emits `package.json`, `tsconfig.json`, `README.md`, and `src/{index.ts,container.ts,smoke.ts,domain/*.ts,ports/*.ts}`.
- `[rust.rs:emit]` — Rust renderer; emits `Cargo.toml`, `README.md`, and `src/{lib,value_objects,roles,entities,errors,commands,events,aggregates,ports,container}.rs` plus `tests/smoke.rs`.
- `[python.rs:project_domain_python_in_memory]` / `[typescript.rs:project_domain_typescript_in_memory]` / `[rust.rs:project_domain_rust_in_memory]` — binding surfaces returning `BTreeMap<path, content>`.
- `[ir.rs:DomainIr]` — the root IR carrying namespace, package slug/pascal, roles, relations doc, and the aggregate/entity/command/event/error/query/value-object/instance collections.

## Invariants
- `[mod.rs]` (module doc) — a renderer never calls `graph.*`, never re-derives a name, and never invents a construct the IR does not carry (confirmed: `python.rs`, `typescript.rs`, and `rust.rs` reference only `DomainIr`).
- `[ir.rs:DomainIr::from_graph]` — every minted name is registered through a single `NameRegistrar`, so identifier collisions are resolved once and identically across all three renderers.
- `[ir.rs:DomainIr::from_graph]` — namespace resolution (`model_namespace`) and flow resolution (`collect_flows`) are shared, so dangling references / multi-namespace models are hard errors for every renderer.
- The mapping table in the `ir.rs` module doc is normative: `Entity`→entity, `Resource`→aggregate+repository, `Flow`→command+event+method (varied by `@cqrs`), `Policy`→error+guard, `Metric`→query, `Pattern`→value object, `Role`→enum variant, `Relation`→Role enum doc comment.

## Side Effects
- `[python.rs:emit]`, `[typescript.rs:emit]`, `[rust.rs:emit]` — write through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
