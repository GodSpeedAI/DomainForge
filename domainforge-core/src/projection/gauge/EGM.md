# Gauge Verification Spec Projection

## Overview
Projects a SEA `Graph` into a Gauge `.spec` Markdown file where each `Flow` becomes an executable test scenario with Given/When/Then steps that a test author wires implementations to.

## Interface
- `[mod.rs:emit]` — emits the Gauge spec into an `ArtifactSink`; returns the one emitted relative path.
- `[mod.rs:project_gauge_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.

## Invariants
- `[mod.rs:emit]` — output filename is `<sanitize_filename(namespace)>.spec`; the spec heading is `# <Namespace> flows`.
- `[mod.rs:build_spec]` — exactly one `## ` scenario per `Flow`; an empty model emits zero scenarios plus a `No flows declared` comment.
- `[mod.rs:build_spec]` — scenario headings are deduplicated through `NameRegistrar` because Gauge requires unique scenario names per spec file.
- `[mod.rs:build_spec]` — step text is escaped so Gauge's reserved `"`, `<`, `>` never appear raw; hostile names cannot corrupt step-parameter quoting.
- `[mod.rs:build_spec]` — flow resolution goes through `collect_flows`, so dangling references are a hard error.
- `[mod.rs:slug]` — heading dedup reuses the kernel `crate::projection::ids::slug` rather than a private reimplementation.
- Output is byte-identical run-to-run for a fixed `created_at` (flows sorted by `(resource, from, to)`).

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write`: filesystem write under the output root, or insertion into an in-memory map.
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
