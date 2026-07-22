# Build the DomainForge Cell Environment Projection

Implement a `--format cell` projection that maps `.sea` into a hermetic agent execution
environment: Devbox (OS packages), Mise (runtimes/tools), ecosystem lockfile contracts,
sandbox/network/authority/evidence JSON contracts, and a composed `cell.lock`.

Do not stop at design documents. Implement grammar, AST, Graph, IR, renderers, validators,
fixtures, golden tests, CLI, CI, and docs. Iterate until all locally runnable checks pass.
Never claim a check passed that was not actually run.

---

# 0. Grounding Facts — Verified Against This Repository

Everything in this section EXISTS today. Reuse it; do not reinvent it.

| Fact | Location |
|---|---|
| Grammar | `domainforge-core/grammar/sea.pest` — flat declarations, per-declaration typed annotation rules (e.g. `entity_annotation`), case-insensitive keywords, `declaration_inner` alternation at line ~34 |
| AST | `domainforge-core/src/parser/ast.rs` (`AstNode` enum, one `parse_*` fn per declaration), `ast_schema.rs`, JSON schema `schemas/ast-v3.schema.json` |
| Formatter | `domainforge-core/src/formatter/printer.rs` (`format_ast`, comment preservation via `CommentedSource`) |
| Pipeline | source → AST → `Graph` (`src/graph/`) → projection. Projections read **Graph**, not AST |
| IR discipline | `src/projection/domain/ir.rs`: `DomainIr::from_graph(&Graph)` decides all semantics once; renderers (`python.rs`/`rust.rs`/`typescript.rs`) are pure IR→text; all `Vec`s sorted |
| Renderer signature | `fn emit(graph: &Graph, model_ref: &str, created_at: Option<String>, sink: &mut ArtifactSink) -> Result<Vec<String>, String>` |
| Output sink | `src/projection/sink.rs`: `ArtifactSink::{Dir, Memory}` (Memory = `BTreeMap<String,String>`), path-traversal safe |
| IDs | `src/projection/ids.rs`: `content_hash`, `element_id(family, parts)`, `slug`, `pascal`, `sanitize_qname`, `NameRegistrar` (collision-safe) |
| CLI | `src/cli/project.rs`: `ProjectFormat` ValueEnum (26 variants incl. `Devbox`, `Dagger` = "Activation" family), flags `--created-at` (RFC 3339), `--recipe`, `--authority-config`, `--output`. **No `--overrides` or `--profile` flag exists** |
| Existing devbox format | `src/projection/devbox/mod.rs` — emits a single `devbox.json` that preloads the domain manifest as env vars. **It is a different feature. Do not modify it.** |
| Authority | `src/authority/` — `AuthorityPack` + `compute_pack_hash`, `AuthorityEnvironmentConfig`, `FinalDecision::{Allow,Deny,Escalate,NotApplicable,Reject}`, `SourceClass`, Ed25519 signing behind `signing` feature, `EvidenceSink` in `trace.rs` |
| Language profiles | `src/parser/profiles.rs`: `ProfileRegistry::global()` with profiles `standard`/`cloud`/`data` — **file-level `@profile` is a language-feature gate, already taken** |
| File-level annotations | `@namespace`, `@version`, `@owner`, `@profile` all exist |
| Existing declarations | Entity, Resource, Flow (resource flows), Pattern, Role, Relation, Dimension, Unit, Policy (`policy name per KIND MODALITY priority N ... as: expr`), Instance, ConceptChange, Metric (`metric "name" as: expr`), Mapping, Projection |
| Determinism convention | `--created-at 2026-07-02T00:00:00+00:00` + run twice + `diff -r` (see CI jobs `verify-rdf` etc. in `.github/workflows/ci.yml`) |
| Proof harness | `justfile` recipes (`rust-test`, `prove`, …); `scripts/prove/{language,canonical,projections,drift,collect-evidence}.sh` |
| Docs layout | per-projection docs are flat `docs/<name>-projections.md`; how-tos in `docs/how-tos/`; ADRs in `docs/specs/ADR-*.md`; registry docs `docs/projection-families.md`, `docs/projection-target-implementation-status.md`. There is **no** `docs/adr/`, `docs/how-to/`, or `docs/reference/projections/` |
| Fixture collision | `fixtures/projection_cell/` **already exists** and belongs to the event/authority projection suite. Do not touch it. This feature uses `fixtures/cell_env/` |
| Deps available | `toml = "0.8"` (parse), `sha2 = "=0.10"`, `xxhash-rust`, `serde`, `chrono` — all already in `domainforge-core/Cargo.toml`. No new dependencies are needed |
| Toolchain | Rust 1.92.0 (`rust-toolchain.toml`), MSRV 1.77, CLI binary gated by `cli` feature |

## Binding decisions (resolve conflicts found during grounding)

1. **CLI surface is one format: `--format cell`.** Do NOT add `--format mise` / `--format cubesandbox`; `--format devbox` is already taken by an unrelated projection. Component subsetting uses a new `--only` flag (Phase 4).
2. **Environment profile is declared on the Cell**, not at file level: `Cell "RepairAgent" @profile "python-agent-v1"`. File-level `@profile` keeps its existing language-feature meaning.
3. **"CubeSandbox" and "SEA-Forge" have zero presence in this repo.** All sandbox/authority/evidence outputs are self-contained JSON contracts whose schemas this repository defines and versions (`schema` field = `domainforge-cell-<x>/v1`). External runtimes are consumers of these files; no integration client code is in scope.
4. **New fixtures live in `fixtures/cell_env/`.**
5. **`devbox.lock` is NOT generated by the projection** (it requires network + the devbox binary). CI may optionally run `devbox generate` where available; otherwise record the unrun command.
6. **All annotation values are string literals** (`@port "443"`), matching the dominant grammar pattern; validation parses/ranges them. No numeric annotation grammar variants.
7. **Source maps are declaration-granular, not span-granular.** The Graph does not retain source spans; each IR item records an `Origin` (see Phase 2). Do not build span plumbing.

---

# 1. Architecture

```text
.sea source
    ↓ parser (sea.pest → AstNode)
    ↓ Graph (new node kinds + accessors)
    ↓ CellIr::from_graph(&Graph, &CellProfileData, Option<CellOverrides>)   ← ALL semantics decided here
    ├── devbox renderer        → devbox/devbox.json + projection-manifest.json
    ├── mise renderer          → mise/mise.toml + projection-manifest.json
    ├── dependency renderer    → dependencies/dependency-sets.json
    ├── sandbox renderer       → sandbox/{template,resources,mounts,network,lifecycle}.json
    ├── network renderer       → sandbox/network.json (endpoints + flows, default deny)
    ├── authority renderer     → authority/{dependency-mutation,runtime-upgrade,credential-contracts}.json
    ├── evidence renderer      → evidence/{environment-proof-contract,event-schema}.json
    └── lock renderer          → cell.lock (sha256 over all emitted files)
```

Ownership boundary (invariant):

```text
.sea owns declared meaning.
Profiles (data in this repo) own opinionated defaults.
Cell IR owns normalized projection semantics — decided exactly once.
Renderers own target-native expression only.
Devbox/Mise/native package managers own concrete resolution (outside this repo).
The sandbox runtime owns isolation/lifecycle (outside this repo; consumes our contracts).
cell.lock owns composed projection identity.
Generated files are never a semantic source.
```

For fixed source + overrides + `--created-at`, output must be byte-identical across runs.

---

# 2. Phase 1 — Grammar, AST, Graph

## 2.1 New declarations (exact authoring syntax)

Flat, following existing grammar conventions (annotations trail the header; whitespace-insensitive; keywords case-insensitive):

```sea
@namespace "godspeed.cells.repair"
@version "1.0.0"
@owner "godspeed-platform"

Cell "RepairAgent"
    @profile "python-agent-v1"
    @architecture "x86_64"
    @network_default "deny"
    @runtime_user "agent"
    @cpu "4"
    @memory_mb "8192"
    @disk_mb "20480"
    @timeout_seconds "1800"

SystemDependency "git" version "2.45"
SystemDependency "pkg-config"

Runtime "python" version "3.13"

Tool "uv" version "0.9"

DependencySet "python-application"
    @ecosystem "python"
    @manifest "pyproject.toml"
    @lockfile "uv.lock"
    @install "uv sync --frozen"
    @mutation "escalate"

Service "postgres" version "16"
    @activation "optional"

Mount "workspace"
    @source "."
    @target "/workspace"
    @mode "read-write"

Endpoint "package-mirror"
    @host "pypi.godspeed.internal"
    @port "443"
    @protocol "https"

NetworkFlow "resolve-python-packages" from "RepairAgent" to "package-mirror"
    @operation "read"

Credential "package-mirror-token"
    @provider "broker"
    @scope "package-mirror:read"
    @delivery "ephemeral"

Policy dependency_files_immutable per Constraint Prohibition priority 5
    @rationale "Agents may not modify dependency manifests or lockfiles without escalation."
    as: true

Metric "environment-proof" as: count(entities) >= 0
```

Note the corrections vs. the earlier draft: `Policy` requires `priority N` when `per` is used
(grammar line ~184); `Metric` puts `as:` before annotations; indentation is cosmetic.

## 2.2 Grammar rules to add (`domainforge-core/grammar/sea.pest`)

Follow the existing per-declaration typed-annotation pattern exactly (model on
`entity_decl`/`entity_annotation`). Add ten rules and their annotation rules, and extend
`declaration_inner`:

```pest
cell_decl              = { ^"cell" ~ name ~ cell_annotation* }
system_dependency_decl = { ^"systemdependency" ~ name ~ (^"version" ~ string_literal)? ~ sysdep_annotation* }
runtime_decl           = { ^"runtime" ~ name ~ ^"version" ~ string_literal ~ runtime_annotation* }
tool_decl              = { ^"tool" ~ name ~ ^"version" ~ string_literal ~ tool_annotation* }
dependency_set_decl    = { ^"dependencyset" ~ name ~ depset_annotation* }
service_decl           = { ^"service" ~ name ~ (^"version" ~ string_literal)? ~ service_annotation* }
mount_decl             = { ^"mount" ~ name ~ mount_annotation* }
endpoint_decl          = { ^"endpoint" ~ name ~ endpoint_annotation* }
network_flow_decl      = { ^"networkflow" ~ name ~ ^"from" ~ string_literal ~ ^"to" ~ string_literal ~ netflow_annotation* }
credential_decl        = { ^"credential" ~ name ~ credential_annotation* }
```

Each `*_annotation` rule enumerates exactly the keys shown in §2.1 (plus `custom_annotation`
if and only if existing declarations allow it — mirror `flow_annotation`). Unknown keys must
be a parse or validation error, not silently ignored.

`Runtime`/`Tool` require `version` in the grammar; `SystemDependency`/`Service` make it
optional (profile default or unpinned-OS-package is acceptable for system deps).

## 2.3 Files that MUST change together (the full checklist)

1. `grammar/sea.pest` — rules above + `declaration_inner` alternation.
2. `src/parser/ast.rs` — ten new `AstNode` variants + `parse_*` functions + the
   `parse_declaration()` match. Fields are plain `String`/`Option<String>`.
3. `src/parser/ast_schema.rs` — schema entries for the new variants.
4. `schemas/ast-v3.schema.json` — additive `oneOf` variants (additive = non-breaking; do not
   create ast-v4).
5. `src/formatter/printer.rs` — canonical formatting for each new node (one annotation per
   line, 4-space indent under the header, matching §2.1).
6. `src/primitives/` + `src/graph/` — new primitive structs and Graph storage with accessors
   following the existing pattern (`graph.all_policies()` → add `all_cells()`,
   `all_system_dependencies()`, `all_runtimes()`, `all_tools()`, `all_dependency_sets()`,
   `all_services()`, `all_mounts()`, `all_endpoints()`, `all_network_flows()`,
   `all_credentials()`).
7. Graph-level validation: duplicate names within a category fail; `NetworkFlow.from` must
   name a declared `Cell` or `Service`; `NetworkFlow.to` must name a declared `Endpoint` or
   `Service`; at most one `Cell` per file (v1 restriction — document it).
8. Bindings check: `just python-test` and `just ts-test` must still pass (bindings expose
   semantic primitives, not AST nodes, so this is verification, not new work).

Do not replace these categories with generic annotations on `Entity`.

---

# 3. Phase 2 — Cell IR (`domainforge-core/src/projection/cell/`)

```text
src/projection/cell/
├── mod.rs          # pub emit(); orchestrates ir → renderers → lock
├── ir.rs           # CellIr + item structs + CellIr::from_graph(...)
├── profiles.rs     # CellProfileData: the four profiles as const data
├── overrides.rs    # CellOverrides: TOML schema + serde + safety validation
├── devbox.rs       # renderer
├── mise.rs         # renderer
├── deps.rs         # renderer
├── sandbox.rs      # renderer (template/resources/mounts/lifecycle)
├── network.rs      # renderer
├── authority.rs    # renderer
├── evidence.rs     # renderer
└── lock.rs         # cell.lock builder
```

## 3.1 IR shape

```rust
pub enum Origin { Profile, Sea, Override }

pub struct CellIr {
    pub identity: CellIdentity,        // namespace, cell name, model version, owner, cell_id = element_id("cell", ...)
    pub profile: ProfileId,            // e.g. ("python-agent-v1", "1.0.0")
    pub platform: PlatformSpec,        // architecture, runtime_user, network_default
    pub system_dependencies: Vec<SystemDependencyIr>, // name, version: Option, origin
    pub runtimes: Vec<RuntimeIr>,
    pub tools: Vec<ToolIr>,
    pub dependency_sets: Vec<DependencySetIr>,        // ecosystem, manifest, lockfile, install, mutation, origin
    pub services: Vec<ServiceIr>,
    pub mounts: Vec<MountIr>,
    pub endpoints: Vec<EndpointIr>,
    pub network_flows: Vec<NetworkFlowIr>,
    pub credentials: Vec<CredentialIr>,
    pub resources: ResourceSpec,       // cpu, memory_mb, disk_mb, timeout_seconds (+ origins)
    pub policies: Vec<PolicyRefIr>,    // name, modality, rationale — from graph.all_policies()
    pub evidence_probes: Vec<EvidenceProbeIr>, // from Metrics + per-tool `--version` probes
}
```

Every item carries: stable id (`element_id("cell", &[category, name])` from
`projection/ids.rs`), its `Origin`, and its source declaration name. Use `NameRegistrar` for
sanitized identifiers. Sort every `Vec` by (category, name) before rendering.

## 3.2 Profiles (data, not conditionals)

`profiles.rs` defines exactly four profiles as plain Rust const/static data (name, version
"1.0.0", default system deps, default tools with pinned versions, default dependency
contract):

- `python-agent-v1`: sysdeps git, ca-certificates, openssl, pkg-config, curl; tools
  python=3.13, uv=0.9.8, just=1.43.0; contract pyproject.toml / uv.lock / `uv sync --frozen`.
- `typescript-agent-v1`: sysdeps git, ca-certificates, openssl, curl; tools node=24,
  pnpm=9.15.0, just=1.43.0; contract package.json / pnpm-lock.yaml /
  `pnpm install --frozen-lockfile`.
- `rust-agent-v1`: sysdeps git, ca-certificates, openssl, pkg-config, clang, lld; tools
  rust=1.92.0, cargo-nextest=0.9.100, just=1.43.0; contract Cargo.toml / Cargo.lock /
  `cargo fetch --locked`.
- `polyglot-agent-v1`: computed deterministic union of the three (dedupe by category+name;
  on version disagreement within the union, highest pinned version wins — this is profile
  data composition, not user-facing merging).

Pins above are the plan's defaults; the implementer may bump to currently supported patch
releases at implementation time — the chosen pins live only in `profiles.rs` and are covered
by golden fixtures.

## 3.3 Merge order and conflict rules

```text
profile defaults → explicit SEA declarations → safe overrides
```

- Explicit SEA declaration with the same (category, name) replaces the profile default and
  records `Origin::Sea`.
- Two explicit SEA declarations for the same (category, name): already a duplicate-name
  Graph validation failure (Phase 1).
- SEA declaration contradicting itself vs. profile is not a conflict (SEA wins); a conflict
  is only possible via overrides (§3.4) and fails closed.
- Missing `@profile` on the Cell: error `CELL001 unknown or missing profile '<x>'; known: python-agent-v1, typescript-agent-v1, rust-agent-v1, polyglot-agent-v1`.

## 3.4 Override TOML (`overrides.rs`)

Optional file passed via `--overrides <path>`. Parse with the existing `toml` crate into
serde structs with `#[serde(deny_unknown_fields)]` on every level (this implements
"unknown override keys fail" for free).

```toml
schema = "domainforge-cell-overrides/v1"

[devbox.packages]        # exact package realization: name -> devbox package string
openssl = "openssl_3"

[mise.tools]             # exact tool version: must be >= and semver-compatible with declared
python = "3.13.5"

[dependency_sets.python-application]
install_command = "uv sync --frozen --no-dev"

[resources]              # may only reduce vs IR values
cpu = 2
memory_mb = 4096
disk_mb = 10240
timeout_seconds = 900

[network.endpoints.package-mirror]   # may only override host/port of a DECLARED endpoint
host = "pypi.mirror.internal"
port = 443

[evidence]               # may only ADD probes
extra_probes = ["python -c 'import ssl'"]
```

Safety rules (enforced in `overrides.rs`, each with an error code):

- `CELL010` schema field missing or not `domainforge-cell-overrides/v1`.
- `CELL011` override tightens check failed: `[mise.tools]` version not a semver refinement of
  the declared constraint (e.g. declared `3.13`, override `3.12.1` → fail).
- `CELL012` `[resources]` value exceeds the IR value (overrides may only reduce).
- `CELL013` override references an undeclared dependency set / endpoint / tool.
- `CELL014` override attempts to remove/disable anything (no removal syntax exists; any
  future key that would remove semantics is rejected by `deny_unknown_fields`).

Unsafe overrides (`[unsafe_overrides]` table with `enabled`, `ticket`, `authority`,
`rationale`, `expires_at` — all mandatory when present):

- `CELL015` incomplete unsafe-override metadata.
- `CELL016` `expires_at` in the past relative to `--created-at` (or wall clock if absent).
- Accepted unsafe overrides are hashed into `cell.lock` (`overrides.unsafe = true`,
  ticket/expiry recorded) and echoed into `authority/dependency-mutation-policy.json` as an
  active exception. They always print a warning to stderr.

---

# 4. Phase 3 — Renderers

All renderers: pure functions `fn render(ir: &CellIr, created_at: &Option<String>) -> Vec<(String, String)>`
(relative path, content), invoked by `cell::emit()` which writes through
`ArtifactSink::Memory` first (see §4.8). Every JSON output is serialized with
`serde_json::to_string_pretty` from `BTreeMap`-backed structures (sorted keys). Every
`projection-manifest.json` records: schema id, cell_id, model version, profile id+version,
`created_at`, and the list of files that component emitted.

## 4.1 Devbox (`devbox.rs`) — OS layer only

Emits `devbox/devbox.json`:

```json
{
  "packages": ["ca-certificates@latest", "curl@latest", "git@2.45", "openssl@3", "pkg-config@latest"],
  "env": { "DOMAINFORGE_CELL": "godspeed.cells.repair.RepairAgent" }
}
```

`SystemDependencyIr.version` → `name@version`, unversioned → `name@latest`. `[devbox.packages]`
overrides replace the package string wholesale. Runtimes/tools never appear here.
`Service` entries with a version map to their versioned package (e.g. `postgresql@16`) and are
additionally listed under a `"cell:services"` key in the projection manifest.

## 4.2 Mise (`mise.rs`) — runtimes/toolchains/tools only

Emits `mise/mise.toml` with deterministic `[tools]` (sorted) and `[tasks]`:

```toml
[tools]
just = "1.43.0"
python = "3.13"
uv = "0.9.8"

[tasks.install]
run = "uv sync --frozen"

[tasks.prove-environment]
run = ["python --version", "uv --version", "uv sync --frozen"]
```

`tasks.install` concatenates each dependency set's install command; `tasks.prove-environment`
is generated from evidence probes (per-tool `--version` + dependency-set frozen installs +
`[evidence].extra_probes`). Application libraries never appear in Mise.

## 4.3 Dependency sets (`deps.rs`)

Emits `dependencies/dependency-sets.json`: for each set — ecosystem, manifest path, lockfile
path, install command, mutation policy (`escalate` | `deny` | `allow`, default `escalate`),
and `lockfile_sha256` when the lockfile exists relative to the `.sea` file's directory
(record `null` + a warning otherwise; missing lockfile is a hard error only when the fixture
declares `@mutation "deny"`). `.sea` never lists individual packages.

## 4.4 Sandbox (`sandbox.rs`)

Emits `sandbox/template.json`, `sandbox/resources.json`, `sandbox/mounts.json`,
`sandbox/lifecycle.json` — all `schema: "domainforge-cell-sandbox/v1"`. Conservative
defaults when the Cell omits them: `network_default=deny`, `runtime_user=agent` (non-root),
`cpu=2`, `memory_mb=4096`, `disk_mb=10240`, `timeout_seconds=1800`, no devices, only declared
mounts, no credentials anywhere in output. This is an Activation-family target
(like Devbox/Dagger per `docs/projection-families.md`): this repo generates templates;
runtime activation is an external runner's job.

## 4.5 Network (`network.rs`)

Emits `sandbox/network.json`: `default: "deny"`, endpoint inventory (host/port/protocol),
and allow-rules derived only from `NetworkFlowIr` (from, to, operation, credential ref by
name only). Validation (in `ir.rs`, not the renderer): `CELL020` flow references unknown
cell/service/endpoint; `CELL021` endpoint host is `*` or `0.0.0.0` without an unsafe
override; `CELL022` credential value literal detected (any `Credential` annotation that
looks like a secret value — the only allowed keys are provider/scope/delivery, so this is
enforced by the grammar; keep the validation as a defense-in-depth check on override input).

## 4.6 Authority (`authority.rs`)

Emits `authority/dependency-mutation-policy.json`, `authority/runtime-upgrade-policy.json`,
`authority/credential-contracts.json` — `schema: "domainforge-cell-authority/v1"`. Reuse
vocabulary from `src/authority/types.rs`: decisions are `Allow`/`Deny`/`Escalate` matching
`FinalDecision`. Mutation policy covers: manifest change, lockfile change, runtime upgrade,
system-dependency change, mount change, network change, credential issuance — each mapped
from `DependencySetIr.mutation` and the Cell's Policies (a `Prohibition` policy whose name
contains a governed noun maps to `Deny`; default is `Escalate`). Credential contracts carry
name, provider, scope, delivery — never values.

## 4.7 Evidence (`evidence.rs`)

Emits `evidence/environment-proof-contract.json` (probes with their expected exit-0
commands, semantic source ids, profile id/version) and `evidence/event-schema.json` (the
shape of an environment-proof event: cell_id, probe id, command, exit code, observed
version, timestamp). `schema: "domainforge-cell-evidence/v1"`. No settlement contract in v1
— there is no substrate for it in this repo; note it in the reference doc's known
limitations.

## 4.8 `cell.lock` (`lock.rs`) and orchestration (`mod.rs`)

`cell::emit()` renders everything into an `ArtifactSink::Memory` map, computes
`sha256(content)` (the `sha2` crate is already a dependency) for every file, builds
`cell.lock`, then flushes all files plus the lock through the caller's sink. This gives
atomic output and free hashing.

```json
{
  "schema": "domainforge-cell-lock/v1",
  "cell_id": "godspeed.cells.repair.RepairAgent",
  "created_at": "2026-07-11T00:00:00+00:00",
  "model": { "version": "1.0.0", "hash": "sha256:<canonical .sea bytes>" },
  "profile": { "id": "python-agent-v1", "version": "1.0.0", "hash": "sha256:<profile data canonical json>" },
  "overrides": { "present": true, "unsafe": false, "hash": "sha256:<override file bytes>" },
  "files": { "devbox/devbox.json": "sha256:...", "mise/mise.toml": "sha256:..." }
}
```

`model.hash` is sha256 of the canonically formatted source (run the formatter first) so
whitespace changes don't churn the lock. Every emitted file appears in `files`, sorted.
`CELL030`: emit fails if any renderer produced a file the lock did not capture.

Also emit `semantic/cell-ir.json` (the serialized IR, sorted) and a generated `README.md`
at the output root listing what was generated and the next commands (`devbox shell`,
`mise install`, `mise run prove-environment`).

Omit component directories that are empty for the profile rather than writing empty files.

---

# 5. Phase 4 — CLI (`src/cli/project.rs`)

1. Add `ProjectFormat::Cell` with `#[value(name = "cell")]` and doc comment
   `/// Activation operator: Cell environment — hermetic agent execution environment (directory output)`.
2. Add flags to `ProjectArgs`:
   - `--overrides <PATH>` (`Option<PathBuf>`) — cell-only; error if used with other formats
     (mirror the existing `--recipe is not used by --format X` guard pattern).
   - `--only <LIST>` (`Option<String>`, comma-separated of
     `devbox,mise,dependencies,sandbox,authority,evidence`) — renders everything internally
     (the IR and lock must always be complete) but writes only the selected component
     directories plus `cell.lock` and `semantic/`. Fast iteration UX; costs one filter loop.
3. Wire `run_cell(&args, &graph)` following `run_devbox` (directory-output check,
   `--recipe` rejection, `--created-at` passthrough, file-count summary line).

Usage:

```bash
domainforge project fixtures/cell_env/repair-agent/model.sea \
  --format cell --output out/cell \
  --overrides fixtures/cell_env/repair-agent/domainforge.cell.toml \
  --created-at 2026-07-11T00:00:00+00:00
```

No `cell-python`/`cell-typescript`/… aliases: the profile is declared in the `.sea` file on
the Cell; aliases would duplicate that with no benefit.

---

# 6. Phase 5 — Fixtures and Tests

## 6.1 Fixtures (`fixtures/cell_env/`)

```text
fixtures/cell_env/
├── repair-agent/                 # flagship end-to-end fixture (see §6.3)
│   ├── model.sea
│   ├── domainforge.cell.toml
│   ├── pyproject.toml            # minimal but real
│   └── uv.lock                   # minimal but real (checked-in bytes; never regenerated by tests)
├── basic-typescript/{model.sea, package.json, pnpm-lock.yaml}
├── basic-rust/{model.sea, Cargo.toml, Cargo.lock}
├── polyglot/model.sea
├── format_stable.sea             # round-trip-clean, exercises every new declaration
├── unsafe-override/{model.sea, domainforge.cell.toml}     # valid, warns, marks lock
└── invalid/
    ├── 01_unknown_profile.sea            → CELL001
    ├── 02_unknown_flow_reference.sea     → CELL020
    ├── 03_duplicate_declaration.sea      → graph duplicate error
    ├── 04_wildcard_endpoint.sea          → CELL021
    ├── 05_resource_increase.toml (+model.sea)   → CELL012
    ├── 06_version_weakening.toml (+model.sea)   → CELL011
    ├── 07_expired_unsafe.toml (+model.sea)      → CELL016
    └── 08_unknown_override_key.toml (+model.sea) → deny_unknown_fields error
```

Do not modify `fixtures/projection_cell/` (different feature) or any existing fixture.

## 6.2 Tests

New file `domainforge-core/tests/cell_projection_tests.rs` plus additions to existing
parser/formatter test files, covering:

- **Parser/AST**: each new declaration parses; each `invalid/` fixture fails with its listed
  error; fields survive AST JSON serialization (validate against `schemas/ast-v3.schema.json`
  the same way existing schema tests do); duplicate names fail.
- **Formatter**: `format_stable.sea` parse → format → parse preserves semantics and is
  idempotent (byte-stable on second format), following `scripts/prove/language.sh`'s
  format-stability convention.
- **IR**: profile expansion for all four profiles; SEA-over-profile precedence with `Origin`
  recorded; polyglot union deterministic; collections sorted; every invalid override fixture
  hits its error code.
- **Renderers**: devbox contains only OS deps; mise contains only runtimes/tools; deps
  output references native lockfiles with correct sha256; sandbox is deny-by-default;
  network contains exactly the declared endpoint; no credential value in any output byte;
  authority decisions use `Allow/Deny/Escalate` vocabulary.
- **Golden/determinism**: run `cell::emit` twice into `ArtifactSink::Memory` with fixed
  `created_at`; assert the two maps are equal and the lock's `files` hashes match
  recomputed sha256 of the map contents.
- **Backward compatibility**: full existing suite must stay green — `just rust-test`,
  `just python-test`, `just ts-test`, `just prove`. Existing DDD/domain-code goldens are
  covered by `just prove`/CI; do not touch their fixtures.

## 6.3 Flagship end-to-end proof (`repair-agent`)

The fixture declares: one Cell (`@profile "python-agent-v1"`), Python runtime, uv + just
tools, git/openssl/pkg-config/ca-certificates, one Python dependency set, one workspace
mount, one package-mirror endpoint, one NetworkFlow, one ephemeral Credential, the
`dependency_files_immutable` Policy, one `environment-proof` Metric. The test asserts the 15
conditions: parses/validates; IR contains every category; devbox has expected sysdeps; mise
has expected runtime+tools; deps reference pyproject.toml + uv.lock; sandbox deny-by-default;
only the declared endpoint in egress; no credential value in output; authority policies cover
mutation + credential issuance; evidence retains semantic ids; cell.lock hashes every file;
repeated generation identical; undeclared endpoint fixture rejected; version-weakening
override rejected; existing projections unchanged.

## 6.4 Native target validation (CI-optional)

Add `scripts/verify/cell.sh`: project the repair-agent fixture twice, `diff -r`, then — only
if the binary exists on PATH — run `mise config` against the emitted `mise.toml` and
`devbox generate` against the emitted `devbox.json`; `uv sync --frozen --check`-style
validation only where `uv` exists. When a tool is absent, print the exact skipped command
and exit 0. Never report a skipped check as passed.

---

# 7. Phase 6 — Docs, CI, Governance

## 7.1 Documentation (real paths for this repo)

- `docs/cell-environment-projections.md` — the projection reference: scope, the normative
  SEA→Cell mapping table (below), profile definitions, ownership boundaries, override
  precedence + safe/unsafe rules + error codes, determinism requirements, activation mode,
  validation commands, generated tree, known limitations (no devbox.lock, no settlement
  contract, single Cell per file, declaration-granular source origins), versioning policy.
- `docs/how-tos/project-a-cell-environment.md` — authoring + CLI walkthrough.
- `docs/reference/sea-language-evolution-policy.md` — the language stability contract (§7.3).
- `docs/specs/ADR-012-cell-environment-declarations.md` — the ADR for this grammar change
  (ADRs live in `docs/specs/`, numbered; 011 is taken).
- `docs/templates/ADR-sea-language-change.md` — reusable template (sections: proposed
  distinction, current limitation, existing mechanisms considered and why they fail,
  cross-target semantics, normative mappings, grammar/AST impact, compatibility, migration,
  fixtures, tests, failure modes, disconfirmation criterion).
- Update `docs/projection-families.md` (Activation family row) and
  `docs/projection-target-implementation-status.md`.

Normative mapping (encode in the reference doc AND assert category coverage in tests):

| SEA element | Cell IR | Realization |
|---|---|---|
| `@namespace`/`@version`/`@owner` | CellIdentity | all outputs + cell.lock |
| `Cell` + `@profile` | identity, platform, resources, profile defaults | sandbox template |
| `SystemDependency` | SystemDependencyIr | devbox packages |
| `Runtime` / `Tool` | RuntimeIr / ToolIr | mise `[tools]` |
| `DependencySet` | DependencySetIr | dependencies/*.json + mise install task |
| `Service` | ServiceIr | devbox package + manifest services list |
| `Mount` | MountIr | sandbox/mounts.json |
| `Endpoint` / `NetworkFlow` | EndpointIr / NetworkFlowIr | sandbox/network.json |
| `Credential` | CredentialIr | authority/credential-contracts.json |
| `Policy` | PolicyRefIr | authority/*.json |
| `Metric` | EvidenceProbeIr | evidence contracts + mise prove task |
| `Projection`/`Mapping` (existing) | renderer overrides via existing `ProjectionRegistry` | out of scope v1; note as limitation |
| `ConceptChange` (existing) | migration metadata passthrough in manifest | manifest field |

## 7.2 CI

In `.github/workflows/ci.yml`, add a `verify-cell` job cloned from `verify-rdf`: build with
`--features cli`, run the repair-agent projection twice with fixed `--created-at`,
`diff -r`, then `bash scripts/verify/cell.sh`. Add a `just cell-verify` recipe and hook the
cell gate into `scripts/prove/projections.sh` so `just prove` covers it.

## 7.3 Grammar-change gate

`scripts/check-sea-language-change.sh`, run as a CI step in the `lint` job:

```text
watched paths: domainforge-core/grammar/*.pest, domainforge-core/src/parser/ast.rs,
               domainforge-core/src/parser/ast_schema.rs, schemas/ast-*.schema.json,
               domainforge-core/src/formatter/printer.rs
```

If `git diff --name-only origin/${GITHUB_BASE_REF:-main}...HEAD` touches a watched path, the
same diff must also touch (a) a `docs/specs/ADR-*.md` file and (b) at least one test file
under `domainforge-core/tests/` or a fixture. Otherwise exit 1 listing exactly which
artifact is missing. Skip silently when not in a PR context (no base ref).
This is deliberately a coarse presence check, not semantic review —
`# ponytail: presence gate; tighten to artifact-content checks if it ever misses a real regression`.

The evolution policy doc states: new technologies enter DomainForge as projection targets,
realization mappings, profiles, adapters, activation modes, or override schemas — never as
SEA syntax by default. Grammar changes require: semantic necessity, cross-target value,
stable semantics, documented normative mapping, backward compatibility, complete
parser/formatter/schema/bindings update, fixture evidence, migration path for breaking
changes, an ADR, and never renderer convenience as justification.

---

# 8. Failure Rules (fail closed)

Fail with a coded, actionable error when: unknown/missing profile (CELL001); version
weakening (CELL011); resource increase via override (CELL012); dangling override reference
(CELL013); unsafe-override metadata incomplete (CELL015) or expired (CELL016); network flow
references unknown object (CELL020); wildcard endpoint without unsafe override (CELL021);
secret-looking literal (CELL022); lock cannot bind an emitted file (CELL030); duplicate
declarations; required lockfile absent under `@mutation "deny"`.

Never silently: pick a newer version than declared; float versions; broaden network access;
drop an IR item in a renderer (assert category counts in `emit()`); rewrite the `.sea`
source; mutate ecosystem lockfiles; change existing SEA semantics.

---

# 9. Suggested Implementation Order

1. Grammar + AST + schema + formatter + parser tests (§2) — keep existing suite green.
2. Primitives + Graph accessors + graph validation (§2.3.6–7).
3. Profiles + IR + merge + validation error codes (§3), unit-tested against fixtures.
4. Overrides TOML (§3.4).
5. Renderers, one at a time, each with its renderer test (§4).
6. Lock + orchestration + CLI (§4.8, §5).
7. Flagship fixture + golden/e2e tests (§6).
8. Docs, ADR, CI job, grammar gate, prove integration (§7).

---

# 10. Completion Evidence

Run and report actual results (never claim unrun checks):

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features
just rust-test
just python-test
just ts-test
just prove
bash scripts/verify/cell.sh          # includes the double-run byte-identity proof
bash scripts/check-sea-language-change.sh
```

Report: files added/changed; exact grammar additions; IR structure; profiles; renderers;
override rules; grammar gate; commands run with results; commands skipped and why; the
generated repair-agent tree; the two-run hash proof; remaining limitations; any deviation
from this plan.

---

# 11. Design Laws (invariants)

```text
SEA categories decide what declarations mean.
Profiles (versioned data in profiles.rs) encode opinionated defaults.
The Cell IR decides semantics once; renderers only translate.
Devbox realizes the OS layer. Mise realizes runtimes and toolchains.
Native lockfiles realize application dependency closure.
Sandbox/network/authority/evidence contracts are repo-defined, schema-versioned JSON;
external runtimes consume them.
cell.lock proves every artifact belongs to one declared world.
New tools become projections, profiles, or overrides before they become language changes.
```
