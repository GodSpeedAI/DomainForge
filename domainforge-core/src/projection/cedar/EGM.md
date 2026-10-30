# Cedar Schema + Policies Projection

## Overview
Projects a SEA `Graph` into a Cedar schema document and a Cedar policy file — one scoped `permit` per action minted from the model's declared flows.

## Interface
- `[mod.rs:SCHEMA_FILE]` — emitted path `schema.cedarschema.json` (entity types + actions).
- `[mod.rs:POLICIES_FILE]` — emitted path `policies.cedar` (one scoped `permit` per action).
- `[mod.rs:emit]` — writes both artifacts into an `ArtifactSink` and returns their relative paths.
- `[mod.rs:project_cedar_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[mod.rs:cedar_ident]` — re-export of the kernel `ident` sanitizer for Cedar identifiers.

## Invariants
- `[mod.rs:emit]` — both artifacts are always emitted together (schema, then policies), in that order.
- `[mod.rs:build_schema]` — the schema is strict JSON (no comment headers, so it parses directly), with the namespace and every entity/resource type run through `NameRegistrar`; `type_names` is sorted and deduped and `serde_json::Map` is BTreeMap-backed, so key emission is sorted for byte-identical runs.
- `[mod.rs:build_schema]` — `entityTypes` is the union of entities and resources, because both are Cedar entity types.
- `[mod.rs:build_policies]` — each policy is a scoped `permit` restricted to `principal is <ns>::<from>`, `action == <ns>::Action::"Issue<resource>"`, `resource is <ns>::<resource>` — never an unconstrained `principal, resource`.
- `[mod.rs:build_policies]` — `(resource, from)` pairs are sorted and deduped, so duplicate flows do not duplicate permits.
- `[mod.rs:build_policies]` — the `.cedar` file carries a `//` provenance and scope note (Cedar policy files allow line comments) stating that SEA `policy` expressions are NOT projected: this is a permissive baseline to be tightened with `when`/`unless` clauses.
- `[mod.rs:emit]` / `[mod.rs:build_policies]` — namespace and flow resolution go through `model_namespace` / `collect_flows`, so multi-namespace models and dangling flow references are hard errors.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
