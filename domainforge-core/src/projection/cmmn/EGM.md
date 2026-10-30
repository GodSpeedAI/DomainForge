# CMMN 1.1 Case Projection

## Overview
Projects policies, roles, and resources into a single XSD-valid CMMN 1.1 XML case: case-file items from resources, case roles from roles, human tasks from entities, milestones from authority policies, and sentries whose entry criteria lower each policy's condition.

## Interface
- `[mod.rs:OUTPUT_FILE]` — the single emitted artifact's relative path, `model.cmmn`.
- `[mod.rs:emit]` — builds the IR, self-checks it, renders it, and writes it to an `ArtifactSink`.
- `[mod.rs:project_cmmn_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:CaseFileItemDef]` / `[ir.rs:CaseFileItem]` — case-file item definition and instance, derived from resources.
- `[ir.rs:Role]` — case role, derived from a SEA role.
- `[ir.rs:HumanTask]` — human task definition with an optional `performer_ref` binding it to a `Role`.
- `[ir.rs:Milestone]` / `[ir.rs:Sentry]` — milestone definitions and the sentries guarding them; `Sentry.condition` is a policy expression rendered as text into the `ifPart` condition.
- `[ir.rs:PlanItemKind]` / `[ir.rs:PlanItem]` — plan-item definitions and their instantiations, carrying `definition_ref` and (for milestones) an `entry_criterion` `(id, sentryRef)` pair.
- `[ir.rs:CaseIR::from_graph]` / `[ir.rs:CaseIR::validate_references]` — IR construction and reference-integrity check.
- `[xml.rs:render]` — XML renderer for `CaseIR`, reusing the BPMN XML writer.

## Invariants
- `[ir.rs:CaseIR::from_graph]` — ids are minted as `<prefix>_<element_id("cmmn", parts)>`.
- `[mod.rs:emit]` — `validate_references` fails the projection on dangling `definitionRef`/`sentryRef`/`performerRef` before writing; the XSD cannot see this integrity.
- `[mod.rs:with_provenance]` — a provenance XML comment is inserted in the rendered document with `--` neutralized (illegal inside XML comments).
- `[ir.rs:CaseIR::from_graph]` — a human task's `performer_ref` names the entity's first (sorted) bound role, when the graph binds one.
- Event listeners (`userEventListener` / `timerEventListener`) are a documented v1 non-goal — the canonical model carries no runtime-event source, so none are invented.
- Every collection is sorted by id/name so output is byte-identical for a fixed `created_at`.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
