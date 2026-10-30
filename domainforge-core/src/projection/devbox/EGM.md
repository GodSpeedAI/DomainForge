# Devbox Activation Environment Projection

## Overview
Projects a SEA model into a `devbox.json` that pre-loads the domain manifest (namespace, entities, resources, flows) as environment variables and prints a banner on shell entry, so `devbox shell` lands in an environment that knows which architecture it hosts.

## Interface
- `[mod.rs:OUTPUT_FILE]` — the single emitted artifact's relative path, `devbox.json`.
- `[mod.rs:emit]` — emits the manifest into an `ArtifactSink`; returns the emitted relative path.
- `[mod.rs:project_devbox_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.

## Invariants
- `[mod.rs:build_spec]` — `packages` is intentionally empty: the SEA model declares what exists in a domain, not the toolchain needed to build it.
- `[mod.rs:build_spec]` — the emitted document also carries a `shell.init_hook` banner and a `shell.scripts.domainforge-context` helper, both reading the exported env vars.
- `[mod.rs:build_spec]` — environment variables are `DOMAINFORGE_MODEL`, `DOMAINFORGE_NAMESPACE`, `DOMAINFORGE_ENTITIES`, `DOMAINFORGE_RESOURCES`, `DOMAINFORGE_FLOWS`; entities/resources are sorted, flow strings are `resource:from->to`.
- `[mod.rs:build_spec]` — the shell-init banner reads `$DOMAINFORGE_*` variables instead of interpolating the namespace into a shell command, so a hostile namespace cannot inject shell.
- `[mod.rs:build_spec]` — namespace resolution uses `model_namespace` (multi-namespace is a hard error) and flows use `collect_flows` (loud dangling-reference policy).
- `[mod.rs:build_spec]` — `serde_json::Map` is BTreeMap-backed, so env keys emit sorted and output is byte-identical run-to-run.
- `[mod.rs:emit]` — the emitted `devbox.json` is JSONC: a leading `//` provenance comment block is prepended as text before the pretty-printed JSON, because Devbox parses the leading comment.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`; the timestamp is not carried into `build_spec`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
