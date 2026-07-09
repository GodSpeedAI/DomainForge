# Projection Target Implementation Status

Tracks the eight event/authority/verification/activation projection targets
added around the existing `calm`, `kg`, and `protobuf`/`proto` projections.

A target is **implemented** only when its per-target gate script and
`scripts/verify/projection-targets/all.sh` both pass. No status below is
advanced ahead of a passing gate.

| Target | CLI `--format` | Gate script | Status |
|---|---|---|---|
| CloudEvents | `cloudevents` | `scripts/verify/projection-targets/cloudevents.sh` | Implemented |
| AsyncAPI | `asyncapi` | `scripts/verify/projection-targets/asyncapi.sh` | Implemented |
| Devbox | `devbox` | `scripts/verify/projection-targets/devbox.sh` | Implemented |
| Dagger | `dagger` | `scripts/verify/projection-targets/dagger.sh` | Implemented |
| Cedar | `cedar` | `scripts/verify/projection-targets/cedar.sh` | Implemented |
| Gauge | `gauge` | `scripts/verify/projection-targets/gauge.sh` | Planned |
| Alloy | `alloy` | `scripts/verify/projection-targets/alloy.sh` | Planned |
| TLA+ | `tla` | `scripts/verify/projection-targets/tla.sh` | Planned |
| Roundtrip cell | — | `scripts/verify/projection-targets/roundtrip-cell.sh` | Planned |

## Existing projections (pre-date this plan)

| Target | CLI `--format` | Notes |
|---|---|---|
| FINOS CALM | `calm` | JSON architecture-as-code |
| Knowledge Graph | `kg` | RDF/Turtle or RDF/XML (selected by output extension) |
| Protocol Buffers | `protobuf` / `proto` | `.proto` with optional gRPC services |

## Fixture

All targets project from `fixtures/projection_cell/basic/model.sea`, a
single-file flat SEA model exercising entities, resources, roles, flows, a
pattern, a relation, an instance, a policy, and a metric.

## Bindings

None of the projection families (existing or new) are exposed through the
Python/TypeScript/WASM bindings today; projection export is CLI-only. New
targets follow the same convention and add bindings only when a concrete
downstream consumer requires in-memory access.
