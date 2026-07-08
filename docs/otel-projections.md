# OpenTelemetry SemConv Projection (`--format otel-semconv`)

DomainForge projects a `.sea` model into an **OpenTelemetry semantic-convention
registry** plus generated **attribute-key constant files** — the
runtime-observation operator of the projection family. This is the traceability
keystone: it is what lets *runtime* telemetry point back at *model* identity.

```bash
domainforge project --format otel-semconv domain/model.sea out/
```

The same artifacts are available from the language bindings as a path → content
map (no filesystem access needed):

```python
artifacts = json.loads(graph.export_otel_semconv(created_at="2026-07-02T00:00:00+00:00"))
```

```ts
const artifacts = JSON.parse(graph.exportOtelSemconv(undefined, '2026-07-02T00:00:00+00:00'));
```

## What gets generated

A directory containing:

| Path | Contents |
| --- | --- |
| `registry/telemetry.yaml` | An OTel semantic-convention registry: attribute groups + span-kind suggestions. |
| `constants/attributes.rs` | `pub const NAME: &str = "key";` for every attribute. |
| `constants/attributes.py` | `NAME: str = "key"` for every attribute. |
| `constants/attributes.ts` | `export const NAME = "key";` for every attribute. |

The mapping from model concepts to telemetry:

| Registry group | Derived from |
| --- | --- |
| resource attribute group (`registry.domainforge.resource`) | the correlation keys carried on the OTel Resource (see below) |
| entity attribute group (`registry.<ns>.entity.<name>`) | every declared entity — its deterministic element id and human name |
| flow attribute group (`registry.<ns>.flow.<hash>`) | every declared flow — resource / source / target / quantity attributes |
| flow span suggestion (`span.<ns>.flow.<hash>`, `span_kind: internal`) | every declared flow — the suggested span for that transfer |
| policy attribute group (`registry.<ns>.policy.<name>`) | every authority policy — a decision point emitting a `decision` and its `modality` |

## The correlation story

The whole point of this projection is a provable path from a runtime span back
to the model that predicted it:

```
runtime span attribute  ──►  deterministic element id  ──►  model hash
  domainforge.element.id       (domainforge element_id)      domainforge.model.hash
```

Three correlation attributes live on the OTel **Resource** (so every span and
metric produced by a service inherits them):

- `domainforge.model.ref` — the provenance label of the source model.
- `domainforge.model.hash` — the deterministic content hash of the projected
  model. Telemetry carrying this value is provably about *this exact model*.
- `domainforge.element.id` — the deterministic `element_id` (from
  `domainforge-core/src/projection/ids.rs`) of the specific domain element a
  span/metric describes. It is the join key: given a span, look up
  `domainforge.element.id` in the registry (every entity/flow attribute group
  documents the element id it corresponds to) to recover the exact entity or
  flow, then `domainforge.model.hash` pins the model version.

Because element ids and the model hash are content-derived and routed through
the shared `projection::ids` kernel, the same model always yields the same
correlation values — telemetry stays joinable across rebuilds.

## Reserved-namespace guard

Domain attributes are namespaced under the **model's own namespace**
(`<ns>.entity.…`, `<ns>.flow.…`, `<ns>.policy.…`). The projection **refuses**,
at IR-construction time, to mint any attribute outside the two allowed
namespaces:

- the model namespace (`<ns>.*`), and
- the DomainForge vendor namespace (`domainforge.*`, for correlation keys).

Any attempt to emit an attribute under an OpenTelemetry-reserved or well-known
namespace — `otel.*`, `service.*`, `telemetry.*`, `http.*`, `rpc.*`, `db.*`,
`k8s.*`, … — returns an `Err` and the projection fails. This is validation *by
construction*: the IR can never hold an attribute that would collide with an
upstream semantic convention and silently shadow it. The guard is unit-tested
(`reserved_namespace_attribute_is_rejected`) and enforced in every language
binding, since they all render from the same IR.

## Single producer

The registry YAML and all three constant files are rendered from one
`TelemetryIR` (`domainforge-core/src/projection/otel/ir.rs`). The renderers
(`yaml.rs`, `constants.rs`) differ only in surface syntax and identifier casing
— there is no per-language business logic. A test
(`registry_and_constants_agree_on_attribute_set`) parses the emitted files and
asserts the attribute set is identical across the YAML and the `.rs`/`.py`/`.ts`
constants, so they can never drift.

## Determinism

Output is byte-deterministic for a fixed `--created-at`: every collection is
built and emitted in sorted (`BTreeMap`/`BTreeSet`/`Vec::sort`) order, all ids
flow through `projection::ids`, and no wall-clock or random data enters file
contents. Projecting the same model twice yields identical bytes (asserted by
`output_is_byte_deterministic`).

## Validation gate

The plan's preferred gate is OpenTelemetry's official `weaver registry check`.
`weaver` is a heavy binary whose registry schema has churned across releases;
for the small, closed registry subset this projection emits, the CI job
(`verify-otel`) **substitutes a JSON-Schema validation** of the emitted YAML
against `schemas/otel/semconv_registry.schema.json` (a DomainForge-authored
Draft-07 schema — see `schemas/otel/VENDORED.md`). The same job also re-asserts
the registry↔constants agreement and byte-determinism. Bumping to real `weaver`
later is a drop-in replacement of that one CI step; the JSON-Schema gate remains
the fast local check.

Validate locally:

```bash
python3 -c "import json,yaml,jsonschema; \
jsonschema.validate(yaml.safe_load(open('out/registry/telemetry.yaml')), \
json.load(open('schemas/otel/semconv_registry.schema.json')))"
```

## Non-goals (v1)

- **No metric/log conventions yet** — only attribute groups and span-kind
  suggestions. Metric instruments derived from resources/flows are a follow-up.
- **No executable instrumentation** — the projection emits *conventions and
  constants*, not tracer/meter wiring. Wiring is the runtime's job; the
  constants make it a compile-time-checked lookup.
- **Registry, not a full SemConv model** — the emitted shape is the fixed subset
  DomainForge produces (see the schema), not the entire upstream SemConv schema.
