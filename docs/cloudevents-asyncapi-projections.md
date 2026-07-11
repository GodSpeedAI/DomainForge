# CloudEvents & AsyncAPI Projections

Two first-class projection targets turn a `.sea` architecture model into the
interoperability contracts an event-driven system needs. **Neither is a
runtime broker** — neither ships messages, routes events, or runs at
message time. Both are *contracts* other tools consume.

## The core distinction

- **CloudEvents** ([cloudevents.io](https://cloudevents.io)) standardizes the
  **event envelope** — a single, transport-neutral wrapper that says "this is
  an event, here is its id, source, type, and time." A broker, sink, function,
  or tracer that speaks CloudEvents can ingest the event without knowing the
  producer's internals. DomainForge emits one CloudEvents 1.0 envelope per
  `Flow` as `events.jsonl`.

- **AsyncAPI** ([asyncapi.com](https://www.asyncapi.com)) standardizes the
  **event-driven API contract** — the description of *what channels exist,
  who publishes to them, who subscribes to them, and what the messages look
  like.* It is the OpenAPI-for-messaging: a single document a broker, client
  generator, and doc generator all read. DomainForge emits an
  AsyncAPI **3.0.0** document as `asyncapi.yaml`, validated against the
  official AsyncAPI 3.0 schema (`schemas/asyncapi/3.0.0.json`).

They are complementary, not competing: **AsyncAPI describes the channels and
the messages that flow over them; CloudEvents describes the envelope each
individual message carries.** A typical pipeline uses both — AsyncAPI as the
top-level contract, with each message payload itself a CloudEvent.

## Concept mapping (SEA → standard)

The AsyncAPI/CloudEvents specifications speak in terms of *Event*, *Actor*,
*Capability*, *Policy*, *Evidence*. SEA does not have all of these as
first-class primitives. The projection maps each to its real SEA equivalent
and documents the alias:

| Spec concept | SEA primitive mapped | Role in the projections |
| --- | --- | --- |
| **Event** | `Flow` | A flow (resource moving from→to) *is* the event. Each flow → one CloudEvents envelope and one AsyncAPI channel + send/receive operations. |
| **Actor** | `Entity` | Entities (Buyer, Supplier, …) are the actors. The `from` entity is the **producer** (AsyncAPI `send` operation, CloudEvents `source`); the `to` entity is the **consumer** (AsyncAPI `receive` operation, CloudEvents `subject`). |
| **Capability** | `Resource` | "Capability" has no SEA primitive. Where it would appear, it is represented by `Resource`, which becomes the message **payload schema** (AsyncAPI `components.schemas`). |
| **Evidence** | `Resource` | Same as Capability — no SEA primitive; represented by `Resource`. |
| **Policy** | `Policy` | Policies do not map to a structural AsyncAPI/CloudEvents field. They are recorded as documentation — the governing policy names are appended to the AsyncAPI `info.description`. |
| **Flow** | `Flow` | The unit of work; maps directly. |

> If you need `Capability` or `Evidence` as distinct first-class primitives,
> add them to the DSL first (grammar → AST → primitive → bindings, per
> `AGENTS.md`); this projection maps whatever the DSL exposes.

## What each projection emits

### CloudEvents (`--format cloudevents`)

- **Output:** `events.jsonl` — one CloudEvents 1.0 envelope per line, sorted
  by a deterministic id.
- **Required fields (always present):** `specversion` (`"1.0"`), `id`,
  `source` (`/<namespace>/<from>`), `type`
  (`<namespace>.<resource>.issued`), plus `subject` (the `to` entity),
  `time`, `datacontenttype`, and a `data` object carrying `resource`,
  `quantity`, `from`, `to`.
- **Determinism:** ids are minted from stable flow content (not random
  per-parse UUIDs); output is byte-identical run-to-run for a fixed
  `created_at`.

### AsyncAPI (`--format asyncapi`)

- **Output:** `asyncapi.yaml` — an AsyncAPI **3.0.0** document.
- **Structure:** `channels` (one per `(resource, from)` flow-group, with an
  `address`); `operations` (a `send` operation per producer entity + a
  `receive` operation per consumer entity — these *are* the producers and
  consumers); `components.messages` (per channel) referencing
  `components.schemas` (the `Resource` payload schema).
- **Validation:** `tests/asyncapi_spec_validation_tests.rs` loads the official
  AsyncAPI 3.0 schema and asserts every projected document validates. The
  `asyncapi.sh` gate runs that test.

## Why neither is a broker

A broker (Kafka, NATS, RabbitMQ, EventBridge) moves bytes at runtime. These
projections produce *descriptions* — artifacts you commit, review, diff, and
hand to other tools:

- The CloudEvents envelope lets any CloudEvents-aware consumer parse an event
  your system emits, without coupling to your producer's code.
- The AsyncAPI document lets a client/broker/doc generator learn your
  channels, producers, and consumers by reading one file.

If you later wire a real broker, you point it at the channels the AsyncAPI
document declares and have your producers emit the CloudEvents envelopes the
CloudEvents projection specifies. DomainForge stops at the contract.

## See also

- `docs/projection-target-implementation-status.md` — status of all nine
  projection targets.
- `domainforge-core/src/projection/cloudevents/mod.rs`,
  `domainforge-core/src/projection/asyncapi/mod.rs` — implementations.
- `schemas/asyncapi/3.0.0.json` — vendored official AsyncAPI 3.0 schema
  (provenance in `schemas/asyncapi/VENDORED.md`).
