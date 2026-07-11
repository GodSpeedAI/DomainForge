# Projection Target Implementation Status

Tracks the eight event/authority/verification/activation projection targets
added around the existing `calm`, `kg`, and `protobuf`/`proto` projections.

A target is **implemented** only when its per-target gate script and
`scripts/verify/projection-targets/all.sh` both pass. No status below is
advanced ahead of a passing gate.

## Toolchain verification (ADR-011 §1)

Each gate validates output with the real ecosystem toolchain where feasible.
Tools that require heavy daemons or credentials are noted with the lighter
check that substitutes for them:

| Target | Real toolchain step | Notes |
|---|---|---|
| TLA+ | SANY parse + TLC model-check (pinned `tla2tools.jar` v1.8.0) | Runs in CI via `verify-projection-targets`. Proves the spec parses, all actions are satisfiable, and the TypeInvariant holds. |
| Dagger | `python3 -m py_compile main.py` | Catches escaping bugs (a `"`/`\` in a name → SyntaxError). Full `dagger develop` needs the daemon; deferred to containerized CI. |
| CloudEvents | Strict JSONL parse + RFC 3339 `time` validation | Every line parses as JSON (no comment header); `time` checked against RFC 3339. |
| AsyncAPI | Official vendored 3.0.0 JSON Schema validation | `tests/asyncapi_spec_validation_tests.rs`; the model citizen pattern the others copy. |
| Cedar | Strict JSON schema parse + H6 scoped-permit structural check | Schema parses as strict JSON (no comment header); each permit scoped to principal/resource type. `cedar validate-schema` deferred (needs the Cedar CLI). |
| Devbox | JSONC structural check (JSONC allows `//` comments) | Devbox parses hujson/JSONC — the `//` header is correct here. |
| Gauge | Structural check (H1, M2: angle-bracket sanitization) | `gauge validate` deferred (needs the Gauge CLI). |
| Alloy | Structural check (M3: scope scales with flow count) | Alloy CLI parse deferred (needs the Alloy CLI). |

| Target | CLI `--format` | Gate script | Status |
|---|---|---|---|
| CloudEvents | `cloudevents` | `scripts/verify/projection-targets/cloudevents.sh` | Implemented |
| AsyncAPI | `asyncapi` | `scripts/verify/projection-targets/asyncapi.sh` | Implemented (3.0.0, YAML, spec-validated) |
| Devbox | `devbox` | `scripts/verify/projection-targets/devbox.sh` | Implemented |
| Dagger | `dagger` | `scripts/verify/projection-targets/dagger.sh` | Implemented |
| Cedar | `cedar` | `scripts/verify/projection-targets/cedar.sh` | Implemented (permissive baseline — see below) |
| Gauge | `gauge` | `scripts/verify/projection-targets/gauge.sh` | Implemented |
| Alloy | `alloy` | `scripts/verify/projection-targets/alloy.sh` | Implemented |
| TLA+ | `tla` | `scripts/verify/projection-targets/tla.sh` | Implemented (SANY+TLC verified) |
| Roundtrip cell | — | `scripts/verify/projection-targets/roundtrip-cell.sh` | Implemented (structural primitives only) |
| Domain Python | `domain-python` | `scripts/verify/projection-targets/domain-python.sh` | Implemented (`compileall` + `mypy --strict` + `unittest`) |
| Domain TypeScript | `domain-typescript` | `scripts/verify/projection-targets/domain-typescript.sh` | Implemented (`tsc --noEmit` strict) |
| Domain Rust | `domain-rust` | `scripts/verify/projection-targets/domain-rust.sh` | Implemented (`cargo check` + `cargo test`, zero-dep) |

### Code targets — flow-annotation plumbing

The domain-code targets are the first consumers of flow `@cqrs` annotations
(`@cqrs { "kind": "command" }` / `"event"`). The parser copies flow
annotations into the `Graph` and `ResolvedFlow` (Task 1), which the Domain
IR reads to switch between command-only / event-only / command+event minting.

### Code targets — full fixture consumption

Unlike the eight event/authority/verification/activation targets (which read
entities/resources/flows only), the three domain-code targets consume the
**entire** fixture: entities, resources, flows, roles, policies, metrics,
patterns, relations, and instances. See
[Project Domain Code](how-tos/project-domain-code.md) for the SEA→DDD mapping.

### Cedar authority scope note

The `policies.cedar` file is a **permissive baseline**: one `permit` per
Action, scoped to the flow's source entity type (`principal is <From>`) and
resource type (`resource is <Resource>`). It does not project SEA `policy`
expressions as Cedar `forbid`/`when` clauses. A Cedar engine loaded with
this baseline authorizes the model's declared flows, not its full obligation
set. Projecting SEA `policy` obligations (e.g. `require_approval`) into Cedar
policy clauses is future work.

## Existing projections (pre-date this plan)

| Target | CLI `--format` | Notes |
|---|---|---|
| FINOS CALM | `calm` | JSON architecture-as-code |
| Knowledge Graph | `kg` | RDF/Turtle or RDF/XML (selected by output extension) |
| Protocol Buffers | `protobuf` / `proto` | `.proto` with optional gRPC services |

## Fixture

All targets project from `fixtures/projection_cell/basic/model.sea`, a
single-file flat SEA model exercising entities, resources, roles, flows, a
pattern, a relation, an instance, a policy, and a metric. The eight
event/authority/verification/activation targets read entities, resources, and
flows only (AsyncAPI also reads policy names as prose); roles, patterns,
relations, instances, and metrics are unconsumed by those targets. The three
**code targets** (`domain-python`/`domain-typescript`/`domain-rust`) consume
the entire fixture — see [Project Domain Code](how-tos/project-domain-code.md).

## Bindings

None of the projection families (existing or new) are exposed through the
Python/TypeScript/WASM bindings today; projection export is CLI-only. New
targets follow the same convention and add bindings only when a concrete
downstream consumer requires in-memory access.
