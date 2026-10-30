# Projection-Target Gates

## Overview
One bash gate per projection target plus an `all.sh` orchestrator. Each gate
projects a committed fixture through the real CLI (`cargo run -p domainforge-core
--features cli -- project --format <target>`) into a `mktemp -d` directory and
then asserts the emitted artifact is structurally and/or toolchain valid.

## Interface
- `[all.sh:set -euo pipefail]` — orchestrator entrypoint: runs
  `scripts/verify/domainforge-baseline.sh`, validates
  `fixtures/projection_cell/basic/model.sea` via `validate`, then invokes the 14
  per-target gates in order (cloudevents, asyncapi, devbox, dagger, cedar, gauge,
  alloy, tla, roundtrip-cell, domain-python, domain-typescript, domain-rust, rego,
  openshell) and ends with `all projection-target gates OK`.
- `[alloy.sh:project --format alloy]` — emits `procurement.als`; asserts module
  decl, abstract `Principal`/`Resource` sigs, `Flow.resource/from/to` multiplicity,
  per-entity `extends Principal`, per-resource `extends Resource`, one `fact
  flow_*` per flow, and a `run { } for N` command.
- `[asyncapi.sh:project --format asyncapi]` — emits `asyncapi.yaml`; asserts
  `asyncapi == "3.0.0"`, `info.version`, non-empty `channels`/`operations`/
  `components`, `$ref` messages, and both `send` and `receive` actions.
- `[cedar.sh:project --format cedar]` — emits `schema.cedarschema.json` +
  `policies.cedar`; asserts the `procurement` namespace, `Issue<Resource>` action
  names, populated `appliesTo.principalTypes`/`resourceTypes`, a `to` annotation,
  one `permit` per action, and type-scoped permits.
- `[cloudevents.sh:project --format cloudevents]` — emits strict-JSONL
  `events.jsonl`; asserts `specversion == "1.0"` plus non-empty `id`/`source`/
  `type`/`time` and RFC 3339 `time` on every line.
- `[dagger.sh:project --format dagger]` — emits `dagger.json`, `main.py`,
  `pyproject.toml`; asserts module/field shape, a PascalCase `@dagger.object_type`
  class, `run_*` identifiers, hatchling backend; then runs
  `python3 -m py_compile main.py`.
- `[devbox.sh:project --format devbox]` — emits `devbox.json`; asserts
  `packages`/`env`/`shell` shape and the `DOMAINFORGE_*` manifest env vars.
- `[domain-python.sh:project --format domain-python]` — emits a Python DDD/CQRS
  package; runs structural greps, `python3 -m compileall`, optional
  `mypy --strict`, and `python3 -m unittest discover`.
- `[domain-rust.sh:project --format domain-rust]` — emits a zero-dependency crate;
  runs structural greps, `cargo check --offline`, `cargo test --offline`.
- `[domain-typescript.sh:project --format domain-typescript]` — emits a TS package;
  runs structural greps and `tsc --noEmit -p .` when a compiler is found.
- `[gauge.sh:project --format gauge]` — emits `procurement.spec`; asserts one H1
  heading, `Issue <r> from <f> to <t>` scenario headings, quoted step params, and
  Given/When/Then beats.
- `[openshell.sh:project --format openshell]` — projects
  `fixtures/cell_env/repair-agent/model.sea` (with and without
  `--overrides domainforge.cell.toml`), asserts byte-determinism, then requires an
  `openshell-prover` containment result of `within_boundary`.
- `[rego.sh:project --format rego]` — projects the repair-agent fixture twice,
  asserts byte-determinism, then runs `opa check --strict`, `opa fmt --fail`,
  `opa test`, `opa build --bundle`.
- `[roundtrip-cell.sh:roundtrip_cell_tests]` — runs the CALM export→import
  round-trip test rather than a one-way projection.
- `[tla.sh:project --format tla]` — emits `procurement.tla` + `procurement.cfg`;
  asserts module/`EXTENDS`/`Init`/`Spec`/`TypeInvariant`/per-flow `Transfer*`
  actions and the `CASE ... OTHER` (never `OTHERWISE`) form, then optionally runs
  SANY + TLC.
- `[regenerate-fixtures.sh:set -euo pipefail]` — deliberate fixture-regeneration
  entrypoint; currently a documented no-op (gates render into temp dirs).
- Assets (not source, no EGM contract): `./contracts/` — 17 per-target YAML
  conformance contracts (`target`, `status`, `preserves`, `prohibited_losses`,
  `native`, `gate_optional`) consumed by `scripts/prove/check-contracts.sh` and
  `scripts/prove/projections.sh`.

## Invariants
- `[all.sh:FIXTURE]` — every gate in `all.sh` except `openshell.sh` and `rego.sh`
  operates on the single fixture `fixtures/projection_cell/basic/model.sea`; the
  two toolchain gates use `fixtures/cell_env/repair-agent/model.sea`.
- `[alloy.sh:python3]` — the `run` scope must be `>= 2` (`>=` the flow-fact count),
  so a scope too small to fire every fact fails the gate.
- `[asyncapi.sh:cargo test]` — the structure check alone is insufficient; the
  document must also validate against the vendored `schemas/asyncapi/3.0.0.json`
  via `asyncapi_spec_validation_tests`.
- `[cedar.sh:permit_count]` — permit count must equal action count exactly, and
  each permit must scope `principal is procurement::` and `resource is
  procurement::` (unconstrained permits fail).
- `[cloudevents.sh:json.loads]` — every non-empty line must parse as JSON with no
  comment header, so a `#`-prefixed header is a failure, not something to skip.
- `[devbox.sh:banner]` — the `shell.init_hook` banner must reference
  `$DOMAINFORGE_NAMESPACE` and must not contain the raw namespace value, which
  prevents shell-interpolating a model-controlled string.
- `[gauge.sh:steps]` — step lines must contain no raw `<` or `>` (reserved by
  Gauge dynamic params) and must number at least `3 *` the scenario count.
- `[openshell.sh:PROVER_VERSION]` — the gate fails closed: a missing
  `openshell-prover` or a version other than `openshell-prover 0.1.3` exits
  nonzero rather than skipping.
- `[rego.sh:OPA_VERSION]` — the gate fails closed: a missing `opa` or a version
  other than `1.21.1` exits nonzero rather than skipping.
- `[tla.sh:OTHERWISE]` — the generated `Init` CASE must use `OTHER` and must not
  contain `OTHERWISE` or a discriminant-less `[] = ` arm.
- `[domain-typescript.sh:dependencies]` — the emitted `package.json` must not
  declare a `dependencies` block (the generated package is dependency-free).
- `[openshell.sh:diff -r]` — two runs at the same `--created-at` (with, and again
  without, `--overrides`) must be byte-identical, so non-deterministic key order or
  a timestamp fails the gate.

## Side Effects
- `[all.sh]` — recursively executes `domainforge-baseline.sh` plus 14 gate
  scripts, each spawning `cargo run`/`cargo test` builds.
- `[alloy.sh:OUT]` — creates a `mktemp -d` output directory and removes it via
  `trap 'rm -rf "$OUT"' EXIT`; the same pattern appears in `asyncapi.sh`,
  `cedar.sh`, `cloudevents.sh`, `dagger.sh`, `devbox.sh`, `domain-python.sh`,
  `domain-rust.sh`, `domain-typescript.sh`, `gauge.sh`, `rego.sh`,
  `roundtrip-cell.sh`, and `tla.sh`.
- `[openshell.sh:OUT1]` — creates four temp output directories (`OUT1`–`OUT4`) and
  removes all of them on exit.
- `[domain-python.sh:python3]` — spawns `python3` with an inline heredoc validator,
  plus `python3 -m compileall` and `python3 -m unittest discover`; `dagger.sh`
  spawns `python3 -m py_compile`.
- `[domain-rust.sh:PATH]` — exports a modified `PATH` (resolved toolchain bin dir
  prepended) into this process to bypass per-directory `mise` shims.
- `[openshell.sh:openshell-prover check]` — invokes the external prover process
  twice and parses its JSON report.
- `[rego.sh:opa build]` — writes `bundle.tar.gz` into the temp output dir and
  invokes the external `opa` binary four times.
- `[tla.sh:java]` — spawns `java -cp "$TLA2TOOLS_JAR" tla2sany.SANY` and
  `tlc2.TLC` when both Java and the jar are available.

## Verification
- Run: `bash scripts/verify/projection-targets/all.sh`
- Run (single target): `bash scripts/verify/projection-targets/tla.sh`
- Run (contract parity): `bash scripts/prove/check-contracts.sh`
