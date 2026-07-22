# Cell environment projections

## Scope

`--format cell` projects a `.sea` `Cell` declaration into a hermetic agent
execution environment: Devbox (OS packages), Mise (runtimes/toolchains),
a dependency-set contract referencing the native manifest/lockfile,
sandbox/network isolation contracts, SEA-Forge-style authority/evidence
contracts, and a composed `cell.lock` binding every artifact to one
declared world. See ADR-012 for why the language gained ten new
declarations to express this, and its disclosed deviation from ADR-011
(this projection's IR is built from the parsed AST, not `Graph` — none of
the ten declarations are consumed by any other projection today).

Activation family (like `--format devbox` / `--format dagger` — see
`docs/projection-families.md`): this repository generates templates;
runtime activation (actually booting an isolated sandbox) is an external
runner's job, out of scope here. Neither "CubeSandbox" nor "SEA-Forge" are
external systems this repository integrates with — the sandbox/network and
authority/evidence outputs are self-contained, schema-versioned JSON
contracts this repository defines; external runtimes are consumers.

## Normative SEA → Cell IR mapping

| SEA element | Cell IR | Realization |
|---|---|---|
| `@namespace`/`@version`/`@owner` | `CellIdentity` | all outputs + `cell.lock` |
| `Cell` + `@profile` | identity, platform, resources, profile defaults | sandbox `template.json`/`resources.json` |
| `SystemDependency` | `SystemDependencyIr` | `devbox/devbox.json` packages |
| `Runtime` / `Tool` | `RuntimeIr` / `ToolIr` | `mise/mise.toml` `[tools]` |
| `DependencySet` | `DependencySetIr` | `dependencies/dependency-sets.json` + `mise` install task |
| `Service` | `ServiceIr` | devbox package (if versioned) |
| `Mount` | `MountIr` | `sandbox/mounts.json` |
| `Endpoint` / `NetworkFlow` | `EndpointIr` / `NetworkFlowIr` | `sandbox/network.json` (default deny, allow-list from declared flows) |
| `Credential` | `CredentialIr` | `authority/credential-contracts.json` (scope/provider/delivery only, never a value) |
| `Policy` (existing category, reused) | `PolicyRefIr` | `authority/*.json` decisions |
| `Metric` (existing category, reused) | `EvidenceProbeIr` | `evidence/environment-proof-contract.json` + `mise` `prove-environment` task |

`Mapping`/`Projection`/`ConceptChange` (existing categories) are not yet
wired into the cell projection — see "Known limitations."

## Profile definitions

Four versioned profiles, defined as plain data in
`domainforge-core/src/projection/cell/profiles.rs` (not renderer
conditionals):

- `python-agent-v1` — git/ca-certificates/openssl/pkg-config/curl;
  python 3.13, uv 0.9.8, just 1.43.0; `pyproject.toml`/`uv.lock`/
  `uv sync --frozen`.
- `typescript-agent-v1` — git/ca-certificates/openssl/curl; node 24,
  pnpm 9.15.0, just 1.43.0; `package.json`/`pnpm-lock.yaml`/
  `pnpm install --frozen-lockfile`.
- `rust-agent-v1` — git/ca-certificates/openssl/pkg-config/clang/lld;
  rust 1.92.0, cargo-nextest 0.9.100, just 1.43.0; `Cargo.toml`/
  `Cargo.lock`/`cargo fetch --locked`.
- `polyglot-agent-v1` — deterministic union of the three above (deduped by
  name; version conflicts resolved to the higher pinned version). Has no
  default dependency contract — a polyglot cell must declare each
  `DependencySet` explicitly.

An unknown `@profile` on `Cell` fails closed (`CELL001`).

## Ownership boundaries

```text
.sea owns declared meaning.
Profiles (versioned data in profiles.rs) own opinionated defaults.
Cell IR (ir.rs) owns normalized projection semantics — decided exactly once.
Renderers (devbox.rs, mise.rs, deps.rs, sandbox.rs, network.rs,
  authority.rs, evidence.rs, lock.rs) own target-native expression only.
Devbox/Mise/native package managers own concrete resolution (outside this repo).
The sandbox runtime owns isolation/lifecycle (outside this repo; consumes
  our contracts).
cell.lock owns composed projection identity.
```

## Override precedence and safe/unsafe rules

Merge order: `profile defaults -> explicit SEA declarations -> safe overrides`.
An explicit SEA declaration for the same (category, name) replaces the
profile default. Overrides come from an optional `domainforge.cell.toml`
(`--overrides <path>`), schema `domainforge-cell-overrides/v1`,
`#[serde(deny_unknown_fields)]` at every level (unknown keys fail).

Safe overrides may: tighten a version, select an approved concrete
package (`[devbox.packages]`), replace a dependency-set install command,
reduce `[resources]`, retarget a *declared* endpoint's host/port, and add
evidence probes. Safe overrides may not: weaken a declared version, raise
a resource ceiling above its declared value, or reference anything not
already declared in `.sea`.

An `[unsafe_overrides]` table (`enabled`, `ticket`, `authority`,
`rationale`, `expires_at` — all required together) is the only way past a
safe-override rejection. It is validated for completeness and expiry, then
recorded in `cell.lock` (`overrides.unsafe = true`) and in
`authority/dependency-mutation-policy.json` — never silently accepted.

### Error codes

| Code | Meaning |
|---|---|
| CELL001 | Cell missing `@profile`, or `@profile` names an unknown profile |
| CELL002 | no `Cell` declaration found |
| CELL003 | more than one `Cell` declaration (v1 supports exactly one per file) |
| CELL004 | a numeric `Cell` annotation (`@cpu`, `@memory_mb`, `@disk_mb`, `@timeout_seconds`) is not a non-negative integer |
| CELL005 | duplicate declaration within a category, or a required annotation is missing |
| CELL010 | override document fails to parse, has the wrong `schema`, or an unknown key |
| CELL011 | `[mise.tools]` override weakens a declared version |
| CELL012 | `[resources]` override raises a value above its declared ceiling |
| CELL013 | an override references an undeclared object (tool, dependency set, endpoint) |
| CELL015 | `[unsafe_overrides]` is enabled but missing required fields |
| CELL016 | `[unsafe_overrides].expires_at` has already passed |
| CELL020 | a `NetworkFlow`'s `from`/`to` references an unknown Cell/Service/Endpoint |
| CELL021 | an `Endpoint` declares a wildcard host (`*` or `0.0.0.0`) without an unsafe override |

## Determinism requirements

For a fixed `.sea` source, override file, and `--created-at`, output is
byte-identical across runs: every collection in `CellIr` is sorted before
rendering, every JSON document serializes through `serde_json`'s
`BTreeMap`-backed `Map`, and `cell.lock`'s file-hash map is computed from
the exact set of files about to be written (so there is no code path that
can emit a file the lock doesn't cover). `model.hash` in `cell.lock` hashes
the canonically *formatted* source, so whitespace-only edits don't churn
the lock.

## Activation mode

Devbox and Mise outputs are directly consumable by their respective CLIs.
`sandbox/*.json` are templates for an external sandbox runtime to consume;
this repository does not activate or boot anything.

## Validation commands

```bash
domainforge project --format cell \
  --overrides fixtures/cell_env/repair-agent/domainforge.cell.toml \
  --created-at 2026-07-11T00:00:00Z \
  fixtures/cell_env/repair-agent/model.sea out/cell
```

Run twice with the same `--created-at` and `diff -r` the two output
directories to verify determinism (see `scripts/verify/cell.sh`).
`cargo test -p domainforge-core --test cell_projection_tests --features
cli` exercises the flagship fixture end to end plus every `CELL0xx`
failure path.

## Generated tree

```text
out/cell/
├── semantic/
│   ├── cell-ir.json
│   └── canonical-model.json
├── devbox/
│   ├── devbox.json
│   └── projection-manifest.json
├── mise/
│   ├── mise.toml
│   └── projection-manifest.json
├── dependencies/
│   └── dependency-sets.json        (omitted if no DependencySet is declared)
├── sandbox/
│   ├── template.json
│   ├── resources.json
│   ├── mounts.json
│   ├── network.json
│   └── lifecycle.json
├── authority/
│   ├── dependency-mutation-policy.json
│   ├── runtime-upgrade-policy.json
│   └── credential-contracts.json
├── evidence/
│   ├── environment-proof-contract.json
│   └── event-schema.json
├── cell.lock
└── README.md
```

`--only devbox,mise,...` writes just the selected component directories
(plus the semantic/IR files and `cell.lock`, which always cover every
category regardless of the filter — it's a fast-iteration convenience, not
a semantic subset).

## Known limitations

- **Graph deviation (ADR-012)**: the IR is built from the parsed `Ast`, not
  `Graph`, deviating from ADR-011's "one IR module built from the graph"
  rule. Disclosed and justified in ADR-012; revisit if a second consumer
  of these declarations appears.
- **No `devbox.lock` generation**: requires network access and the
  `devbox` binary; out of scope for this projection. `scripts/verify/
  cell.sh` runs `devbox generate` against the emitted manifest only when
  the binary is present on `PATH`.
- **No settlement contract**: the plan's original `evidence/
  settlement-contract.json` has no substrate in this repository (no
  settlement system exists to contract against) and was dropped.
- **One `Cell` per file**: a v1 restriction (`CELL003`); multi-cell files
  are not yet supported.
- **Declaration-granular origin tracking, not span-granular**: `Origin`
  (`Profile`/`Sea`/`Override`) is recorded per IR item, not per source
  span — the parsed `Graph`/`Ast` in this codebase does not retain spans
  through to projection time.
- **`Mapping`/`Projection`/`ConceptChange` are not yet consumed** by this
  projection (they exist in the grammar for other projections already).

## Versioning policy

Additive grammar changes bump nothing (schema `ast-v3.schema.json` stays
at v3 — new `oneOf` variants are backward compatible). Changes to
generated *content* meaning bump the relevant output schema
(`domainforge-cell-*/v1` — see each renderer) and require a golden-fixture
update plus a changelog entry. Formatting-only changes to generated bytes
must be identified as such and do not require a schema bump.
