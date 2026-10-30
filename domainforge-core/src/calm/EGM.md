# calm — FINOS CALM Import/Export

## Overview
Projects a `Graph` into the FINOS CALM architecture-JSON shape and imports CALM
JSON back into a `Graph`. Also offers an SBVR-XMI import path that produces a
`Graph` via `sbvr::SbvrModel`.

## Interface
- `[export.rs:export]` — `export(graph: &Graph) -> Result<Value, String>`; emits the CALM JSON document.
- `[import.rs:import]` — `import(calm_json: Value) -> Result<Graph, String>`; builds a `Graph` from a CALM document.
- `[sbvr_import.rs:import_sbvr_xmi]` — `import_sbvr_xmi(xmi: &str) -> Result<Graph, String>`; SBVR-XMI to `Graph`.
- `[models.rs:CalmModel]` — top-level CALM document (`version`, `metadata`, `nodes`, `relationships`).
- `[models.rs:CalmMetadata]` — `sea:exported` / `sea:version` / `sea:timestamp` metadata fields.
- `[models.rs:CalmNode]` with `[models.rs:NodeType]` — CALM nodes.
- `[models.rs:CalmRelationship]` with `[models.rs:RelationshipType]`, `[models.rs:FlowDetails]`, `[models.rs:Parties]` — CALM relationships and flow detail.
- `[models.rs:CalmModel::new]` — construct an empty model.
- `[mod.rs]` — re-exports `export`, `import`, `import_sbvr_xmi`, `CalmModel`, `CalmNode`, `CalmRelationship`.

## Invariants
- `[export.rs:export]` / `[import.rs:import]` — both are fallible with a `String` error rather than panicking; malformed or unsupported input returns `Err`.
- `[export.rs:export]` — the emitted metadata carries the CALM `sea:exported` / `sea:version` namespaced keys, so consumers can distinguish a DomainForge-produced CALM document.

## Side Effects
- `[export.rs]` — writes a warning to stderr (`eprintln!`) when a flow's referenced resource is missing from the graph, and skips that mapping instead of failing the export.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
