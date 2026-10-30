# Cell Environment Projection

## Overview
Projects a `.sea` model into a Cell IR and then into the per-component artifacts of a reproducible environment: {Devbox, Mise, dependency sets, sandbox/network, authority, evidence, `cell.lock`}. All semantics are decided once in `CellIr::build`; renderers only translate the IR.

## Interface
- `[ir.rs:CellIr]` — the complete cell environment IR.
- `[ir.rs:CellIr::build]` — the single place cell semantics are decided; reads the parsed `Ast` (not `Graph`) because none of the cell declaration kinds are consumed by other projections.
- `[ir.rs:Origin]` — provenance of a cell value: `profile`, `sea`, or `override` (lowercase-serialized).
- `[ir.rs:CellIdentity]` / `[ir.rs:PlatformSpec]` / `[ir.rs:ResourceSpec]` — identity, platform, and CPU/memory/disk specs.
- `[ir.rs:SystemDependencyIr]` / `[ir.rs:RuntimeIr]` / `[ir.rs:ToolIr]` / `[ir.rs:DependencySetIr]` — OS packages, language runtimes, CLIs, and native dependency sets.
- `[ir.rs:ServiceIr]` / `[ir.rs:MountIr]` / `[ir.rs:EndpointIr]` / `[ir.rs:NetworkFlowIr]` — services, mount points, network endpoints, and declared network flows.
- `[ir.rs:CredentialIr]` / `[ir.rs:PolicyRefIr]` / `[ir.rs:EvidenceProbeIr]` / `[ir.rs:UnsafeOverrideRecord]` — credentials, policy references, evidence probes, and recorded unsafe exceptions.
- `[mod.rs:CellComponents]` — which component directories `--only` selects; defaults to all six enabled.
- `[mod.rs:CellComponents::parse_only]` — parses the comma-separated `--only` value, failing closed on unknown component names.
- `[mod.rs:emit]` — builds the IR, renders every selected component, adds `semantic/cell-ir.json`, `semantic/canonical-model.json`, `cell.lock`, and `README.md`, and writes through an `ArtifactSink`.
- `[mod.rs:project_cell_in_memory]` — binding surface: source + overrides + `--only` in, `BTreeMap<path, content>` out, no filesystem.
- `[profiles.rs:CellProfileData]` / `[profiles.rs:PinnedTool]` / `[profiles.rs:SystemDep]` / `[profiles.rs:DependencyContract]` — the built-in profile data model.
- `[profiles.rs:lookup]` — profile lookup by id; unknown ids return `None`, which callers turn into `CELL001`.
- `[profiles.rs:KNOWN_PROFILES]` — the four known profile ids: `python-agent-v1`, `typescript-agent-v1`, `rust-agent-v1`, `polyglot-agent-v1`.
- `[overrides.rs:CellOverrides]` — the explicit override document.
- `[overrides.rs:OVERRIDES_SCHEMA]` — the override schema id, `domainforge-cell-overrides/v1`.
- `[overrides.rs:DevboxOverrides]` / `[overrides.rs:MiseOverrides]` / `[overrides.rs:DependencySetOverride]` / `[overrides.rs:ResourceOverrides]` / `[overrides.rs:NetworkOverrides]` / `[overrides.rs:EndpointOverride]` / `[overrides.rs:EvidenceOverrides]` / `[overrides.rs:UnsafeOverrides]` — the per-section override types.
- `[overrides.rs:parse]` — parses an override document.
- `[overrides.rs:validate_unsafe_overrides]` — validates `[unsafe_overrides]` completeness (CELL015) and expiry (CELL016).
- `[devbox.rs:render]` — OS packages and native libraries only.
- `[mise.rs:render]` — language runtimes, toolchains, package managers, and developer CLIs only.
- `[deps.rs:render]` — dependency-set file references plus lockfile hashing.
- `[sandbox.rs:render]` — conservative-by-default isolation template.
- `[network.rs:render]` — endpoints and allow-rules derived only from declared `NetworkFlow`s.
- `[authority.rs:render]` — mutation/credential contracts using the `Allow`/`Deny`/`Escalate` vocabulary.
- `[evidence.rs:render]` — environment-proof probes and the environment-proof event schema.
- `[lock.rs:build]` — composes `cell.lock` from every other emitted file's content.
- `[lock.rs:sha256_hex]` — the shared sha256 hex helper.


## Invariants
- `[lock.rs]` (module doc) — every emitted file is hashed and bound to one `cell.lock`; `emit` fails closed if a renderer produced a file the lock did not capture, so there is no code path producing an unhashed artifact.
- `[mod.rs:emit]` — `cell.lock` is computed from the `files` map before lock/README insertion, so every other emitted file is captured by construction.
- `[mod.rs:CellComponents]` — the IR and `cell.lock` always cover every category regardless of the `--only` filter; `--only` is a fast-iteration UX convenience, not a semantic subset.
- `[overrides.rs:validate_unsafe_overrides]` — an enabled `[unsafe_overrides]` requires non-empty `ticket`, `authority`, `rationale`, and `expires_at` (CELL015); `expires_at` must be RFC 3339 and strictly after `created_at` (CELL016).
- `[deps.rs]` (module doc) — the declared `@lockfile` is resolved canonically and rejected if absolute, containing `..`, escaping `source_dir` via symlink, not a regular file, missing, or above the 16 MiB ceiling (CELL022).
- `[deps.rs:render]` — in-memory callers pass `source_dir = None` and lockfile hashing is skipped (`lockfile_sha256` is null), matching the filesystem-free binding surface.
- `[devbox.rs:render]` — runtimes and tools never appear in Devbox output; `[mise.rs:render]` — OS packages never appear in Mise output.
- `[network.rs]` (module doc) — "no declared route, no packet": reachability stays distinct from authorization.
- `[authority.rs:mutation_decision]` — any mutation value other than `deny`/`allow` maps to `Escalate`.
- `[profiles.rs:lookup]` — the profile set is a closed enumeration of `KNOWN_PROFILES`.
- `[sandbox.rs:render]` — this repo only generates the template; runtime activation is an external runner's job.

## Side Effects
- `[mod.rs:emit]` — writes every artifact through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — canonicalizes the `.sea` source for `cell.lock` hashing and substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.
- `[deps.rs:render]` — reads the referenced lockfile from disk when `source_dir` is `Some` (bounded at 16 MiB).
- `[lock.rs:build]` — hashes each emitted file's bytes into the lock document.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
