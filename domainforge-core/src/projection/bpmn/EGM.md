# BPMN 2.0 Process Projection

## Overview
Projects flows and their branching into a single XSD-valid, non-executable BPMN 2.0 XML process: lanes from roles, tasks from participating entities, parallel gateways from flow fan-out/fan-in, sequence flows, and data objects from resources.

## Interface
- `[mod.rs:OUTPUT_FILE]` — the single emitted artifact's relative path, `model.bpmn`.
- `[mod.rs:emit]` — builds the IR, self-checks it, renders it, and writes it to an `ArtifactSink`.
- `[mod.rs:project_bpmn_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:NodeKind]` — flow-node kinds: `StartEvent`, `EndEvent`, `Task`, `GatewayDiverging`, `GatewayConverging`.
- `[ir.rs:Node]`, `[ir.rs:SequenceFlow]`, `[ir.rs:Lane]`, `[ir.rs:DataObject]` — the process elements; `Node` holds ids of the sequence flows touching it.
- `[ir.rs:ProcessIR::from_graph]` / `[ir.rs:ProcessIR::validate_references]` — IR construction and reference-integrity check.
- `[xml.rs:render]` — XML renderer for `ProcessIR`.

## Invariants
- `[ir.rs:ProcessIR::from_graph]` — a task exists per entity that participates in a flow (source or target); a lane exists per role owning at least one participating entity, and an entity's first (sorted) role owns its task.
- `[ir.rs:ProcessIR::from_graph]` — every split/join is a parallel (AND) gateway; exclusive gateways are not emitted because flows carry no branch predicate.
- `[ir.rs:ProcessIR::from_graph]` — ids are minted through `element_id` under the `bpmn` family and prefixed with an alpha tag so each is a legal XML `NCName`.
- `[mod.rs:emit]` — `validate_references` fails the projection on a dangling IDREF before anything is written.
- `[mod.rs:with_provenance]` — a provenance XML comment is inserted directly after the XML declaration line; `--` (illegal inside XML comments) is broken up and newlines are stripped from the interpolated values.
- All collections are built and emitted in sorted (`BTreeMap`/`BTreeSet`) order, so output is byte-identical for a fixed `created_at`.
- BPMNDI visual layout is a documented v1 non-goal.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
