# OpenShell projection

## Scope

`--format openshell` projects the Cell IR (the same IR the `cell` projection
uses) into a policy for NVIDIA OpenShell v0.1.3, schema `version: 1`. It is a
second view of `CellIr::build`; there is no second parser.

The projection emits a policy document and sandbox contracts. It does not
start an OpenShell sandbox, does not run the gateway, and does not establish
runtime isolation. Several obligations are listed for the executor to
discharge rather than discharged here (see "Obligations").

The Cell IR and its normative SEA mapping are described in
`docs/cell-environment-projections.md`.

## Usage

```bash
domainforge project --format openshell \
  --overrides fixtures/cell_env/repair-agent/domainforge.cell.toml \
  --created-at 2026-07-11T00:00:00Z \
  fixtures/cell_env/repair-agent/model.sea out/openshell
```

`--overrides` (optional) takes the same `domainforge.cell.toml` document as
the `cell` projection. `--created-at` is RFC 3339.

Generated tree:

```text
out/openshell/
├── policy.yaml
├── sandbox.json
├── mapping-report.json
└── openshell.lock
```

- `policy.yaml`: the OpenShell policy. It is also the boundary against which
  the gateway's effective policy is checked. It is serialized with
  `serde-saphyr` through the shared `projection::yaml::to_yaml` helper (the same
  options as the AsyncAPI projection: block scalars disabled, YAML 1.1-safe
  quoting). Its bytes, and therefore `policy_digest`, change only if that
  serializer or the policy structure changes.
- `sandbox.json` (schema `domainforge.openshell.sandbox/v1`): `vm_shape`
  (`vcpus`, `mem_mib`, `overlay_disk_mib`), `container_limits`,
  `exec_timeout_seconds`, `workspaces`, `dependency_sets`, `evidence_probes`,
  `credentials`, `obligations`.
- `mapping-report.json` (schema `domainforge.openshell.mapping/v1`): one entry
  per rule, with `target`, `source` (the Cell element id), `class`, and
  `phase`.
- `openshell.lock` (schema `domainforge.openshell.lock/v1`): `cell_id`, the
  model hash (identical to the hash in `cell.lock`), the policy digest, file
  hashes, and the pinned OpenShell version.

For a fixed model, overrides, and `--created-at`, the output is
byte-identical across runs.

## Mapping

| Cell element | OpenShell target | Class | Phase |
|---|---|---|---|
| Mount | `policy.yaml#/filesystem_policy/read_write/<path>`; the target is relocated under `/sandbox`, and the host source is uploaded, never mounted | adapter | startup |
| Profile filesystem defaults | `policy.yaml#/filesystem_policy/read_only/<path>` and `.../read_write/<path>` | baseline | startup |
| Landlock requirement | `policy.yaml#/landlock` (`compatibility: hard_requirement`) | enforced | startup |
| `runtime_user` (numeric id) | `policy.yaml#/process` (`run_as_user` passes through) | enforced | startup |
| `runtime_user` (named non-root user) | `policy.yaml#/process` (`run_as_user: sandbox`) | adapter | startup |
| `NetworkFlow` | one `policy.yaml#/network_policies/<rule>` with `enforcement: enforce` | enforced | live |
| CPU, memory, disk | `sandbox.json#/vm_shape/<field>` (the gateway's `[openshell.drivers.vm]` table holds the values; the VM driver has no per-sandbox limits) | enforced | gateway |
| `timeout_seconds` | `sandbox.json#/exec_timeout_seconds` (per-exec timeout) | enforced | exec |

Network rules are built per `NetworkFlow`:

| Flow operation | `access` | HTTP methods |
|---|---|---|
| `read` | `read-only` | GET, HEAD, OPTIONS |
| `write` | `read-write` | GET, HEAD, OPTIONS, POST, PUT, PATCH |
| `connect` | (`tcp` endpoint only) | — |

Each rule's `binaries` are the exact executable paths of the Cell's declared
Tools and Runtimes under `/usr/bin` and `/usr/local/bin`. Rules exist only for
declared flows.

### Static and dynamic rules

- Filesystem, Landlock, and process settings are fixed at sandbox startup.
- Network rules can be updated on a running sandbox.

The `phase` field in `mapping-report.json` records which applies to each rule.

## Refusals

A refusal fails the projection with a typed error,
`OSH001 unrepresentable: <element> requires <requirement>: <reason>`.

| Refused input | Requirement |
|---|---|
| `network_default` other than `deny` | default-deny network |
| `runtime_user` root | non-root runtime user |
| Read-only mounts | OpenShell's public interface has no host mounts |
| Mount targets that are not clean absolute paths | clean absolute targets |
| Flows whose source is not the Cell | source is the Cell |
| Credential-bound flows | a verified provider mapping (none exists yet) |
| Wildcard hosts | exact hosts |
| Flows when the Cell declares no Tool or Runtime | a declared executable to bind |
| Unsupported protocols or operations | supported protocol and operation |
| Companion Services | none |
| CPU above 255 | `vcpus` fits in a u8 |

Other codes:

- `OSH002 cell: CELL0xx ...` passes Cell IR errors through unchanged.
- `OSH003 write` is a write error.

## Obligations

The projection lists these obligations in `sandbox.json#/obligations` for the
executor to discharge. They are listed, not dropped.

- Architecture check (probe).
- Protected dependency manifests and lockfiles, with the Cell's mutation
  decision.
- Policy references, evaluated through the Rego projection
  (`docs/rego-projection.md`).
- Image contents, through evidence probes.

Credentials are emitted as contracts with `delivered: false`. No secret
material is written.

## Verification

`scripts/verify/projection-targets/openshell.sh` requires `openshell-prover`
0.1.3 on `PATH` and fails if it is missing or reports another version. It runs:

1. Two projections without overrides, compared with `diff -r`.
2. Two projections with `--overrides`, compared with `diff -r`.
3. `openshell-prover check policy.yaml --boundary policy.yaml --output json`
   in each output directory, requiring `"result": "within_boundary"`.
4. `cargo test -p domainforge-core --features cli --test openshell_projection_tests -- --include-ignored`
   (10 tests; two of them require `openshell-prover` 0.1.3 on `PATH`).

With the real prover, the projected RepairAgent policy is `within_boundary`
against itself, with coverage `filesystem`, `landlock`, `network_l4`,
`network_rest`, and `process`. Three weakened variants each give a
non-`within_boundary` result, which consumers treat as failures:

| Variant | Result |
|---|---|
| Undeclared host | `exceeds_boundary` |
| `access: read-write` | `exceeds_boundary` |
| `landlock: best_effort` | `exceeds_boundary` |
| `enforcement: audit` | `unsupported` |
| Unknown key | `error` |

The prover checks policy containment only. It does not establish runtime
isolation.

## Known limitations

- **No live sandbox**: no OpenShell sandbox is started by DomainForge's gates.
  Enforcement at runtime is not exercised in this repository.
- **Containment is not a runtime security proof**: a `within_boundary` result
  shows that the candidate policy is contained in the boundary policy. It
  does not show what the sandbox does.
- **Credential-bound flows are refused**: there is no verified provider
  mapping yet, so a flow bound to a credential does not project.
- **Read-only mounts are refused**: OpenShell's public interface has no host
  mounts.
- **Obligations are not discharged**: architecture probes, dependency
  mutation decisions, policy evaluation, and image evidence are left to the
  executor.
- **Resource limits are gateway configuration**: CPU, memory, and disk values
  are emitted as a VM shape for the gateway's `[openshell.drivers.vm]` table,
  because the VM driver has no per-sandbox limits.
