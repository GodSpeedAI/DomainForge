# OpenTelemetry SemConv Projection

## Overview
Projects domain entities, flows, and policy-decision points into an OTel semantic-convention registry (YAML) plus generated attribute-constant files (Rust / Python / TypeScript), all rendered from one IR so the four artifacts can never disagree.

## Interface
- `[mod.rs:emit]` — builds `TelemetryIR` and writes `registry/telemetry.yaml`, `constants/attributes.rs`, `constants/attributes.py`, `constants/attributes.ts` into an `ArtifactSink`; returns the emitted paths.
- `[mod.rs:project_otel_semconv_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:FAMILY]` — the `element_id` family tag, `otel`.
- `[ir.rs:VENDOR_NS]` — the DomainForge vendor namespace prefix, `domainforge`.
- `[ir.rs:RESERVED_NAMESPACES]` — the OTel-reserved namespace list the guard rejects.
- `[ir.rs:TelemetryIR::from_graph]` — graph → telemetry IR (resource group, per-entity group, per-flow group, per-policy group).
- `[ir.rs:TelemetryIR::all_attributes]` — the deduplicated, key-sorted attribute set that every renderer consumes.
- `[ir.rs:Attribute]` / `[ir.rs:AttributeGroup]` / `[ir.rs:SpanSuggestion]` / `[ir.rs:AttrType]` / `[ir.rs:SpanKind]` — the IR element types.
- `[ir.rs:validate_attribute_key]` — the reserved-namespace guard.
- `[yaml.rs:render]` — the SemConv registry renderer.
- `[constants.rs:render_rust]` / `[constants.rs:render_python]` / `[constants.rs:render_typescript]` — the three attribute-constant renderers.

## Invariants
- `[ir.rs:validate_attribute_key]` — an attribute key is accepted only when prefixed with `<model-ns>.` or `domainforge.`; every other prefix is an `Err`, so the IR can never hold an attribute that collides with an upstream OTel semantic convention.
- `[ir.rs:TelemetryIR::all_attributes]` — a single producer for the YAML registry and all three constant files, so the attribute sets cannot diverge.
- `[constants.rs]` (module doc) — the three language renderers differ only in surface syntax and identifier casing; there is no per-language business logic.
- `[ir.rs:model_namespace]` — the single attribute-namespace prefix is the lexicographically first non-empty declared namespace, sanitized to one segment; an empty model falls back to `VENDOR_NS`.
- `[ir.rs:TelemetryIR::from_graph]` — element ids are minted via `element_id` under the `otel` family, and all collections are sorted so output is byte-identical for a fixed `created_at`.
- The resource attribute group always carries the correlation keys `domainforge.model.ref`, `domainforge.model.hash`, `domainforge.element.id`.

## Side Effects
- `[mod.rs:emit]` — writes four artifacts through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
