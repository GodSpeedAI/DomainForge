# ArchiMate 3.0 Model Projection

## Overview
Projects roles, entities, resources, flows, and authority policies into a single XSD-valid ArchiMate Model Exchange File, restricted to the declared business-layer v1 subset plus motivation requirements.

## Interface
- `[mod.rs:OUTPUT_FILE]` — the single emitted artifact's relative path, `model.xml`.
- `[mod.rs:emit]` — builds the IR, self-checks it, renders it, and writes it to an `ArtifactSink`.
- `[mod.rs:project_archimate_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:ElemKind]` — the ArchiMate element types minted (`BusinessRole`, `BusinessObject`, `BusinessProcess`, `BusinessService`, `Requirement`), each with an `archimate_type` `xsi:type` value and `is_business_layer`.
- `[ir.rs:RelKind]` — the ArchiMate relationship types minted, each with an `archimate_type` `xsi:type` value.
- `[ir.rs:ALLOWED_RELATIONS]` — the curated legal `(source-kind, relation-kind, target-kind)` triple table.
- `[ir.rs:relation_allowed]` — membership test against that table.
- `[ir.rs:ArchitectureIR::from_graph]` / `[ir.rs:ArchitectureIR::validate_references]` — IR construction and its reference-integrity check.
- `[xml.rs:render]` — the XML renderer, reusing the BPMN XML writer.

## Invariants
- `[ir.rs:Relation::build]` — a relation triple absent from `ALLOWED_RELATIONS` cannot be constructed; spec legality is enforced by construction, not by post-hoc validation.
- `[mod.rs:emit]` — `ArchitectureIR::validate_references` runs before rendering and fails the projection on a dangling relationship endpoint, view `elementRef`, endpoint-kind mismatch, or a relation no longer in the allowed table.
- `[mod.rs:with_provenance]` — a provenance XML comment is inserted in the rendered document with `--` neutralized (illegal inside XML comments).
- `[ir.rs:ElemKind::is_business_layer]` — `Requirement` is excluded from the auto-generated business-layer view.
- `[ir.rs:ArchitectureIR::from_graph]` — all collections are sorted, so output is byte-identical for a fixed `created_at`; random flow UUIDs never reach the file (process ids derive from deterministic endpoint ids).
- Per-stakeholder views are a documented v1 non-goal; only one auto-generated business-layer view is emitted.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
