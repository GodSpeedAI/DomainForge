# 🏛️ DomainForge

## A semantic IR for organizational meaning

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)

Compilers solved this problem decades ago: one intermediate representation,
many targets. Write the meaning once, generate the artifacts.

Your organization never got that layer. Your business rules live in a
requirements doc, a Python service, a TypeScript frontend, an architecture
diagram, and a compliance spreadsheet — five copies, five owners, zero
guarantees they agree. When the rule changes, you update some of them.

That's not governance. That's folklore with version numbers.

DomainForge is the missing IR. You declare your domain once — the actors, the
things that move between them, the rules that govern the movement — in a small
language called SEA that an analyst can read. Then you project it: into Python,
TypeScript, Rust, Protobuf, architecture diagrams, formal proofs, policy
engines, telemetry schemas. Deterministically. From one source.

Not business rules in folklore. Executable domain meaning.

---

## Show, don't tell: one model, in full

Here is a real procurement domain. This is the actual fixture the test suite
runs against — not a mockup:

```sea
@namespace "procurement"
@version "1.0.0"

// WHO acts
Entity "Buyer" in procurement
Entity "Supplier" in procurement
Entity "Approver" in procurement

// WHAT moves
Resource "PurchaseOrder" units in procurement
Resource "Payment" USD in procurement

// HOW it moves
Flow "PurchaseOrder" from "Buyer" to "Supplier" quantity 1
Flow "Payment" from "Buyer" to "Supplier" quantity 100

// The rule everyone "knows" — now written down where machines can see it
policy require_approval per Constraint Obligation priority 1
  @rationale "every order must be approved before payment"
  as:
    1 = 1

// What "healthy" means, as a measurable claim
Metric "order_count" as:
  count(f in flows where f.resource = "PurchaseOrder": f.quantity)
  @threshold 100
  @unit "orders"
```

Read it out loud. A buyer sends purchase orders to a supplier, payment follows,
every order needs approval first. Your compliance lead just understood your
system without a meeting.

---

## Projection one: a Python domain layer

One command:

```bash
domainforge project --format domain-python procurement.sea ./out
```

You get a complete, typed, tested Python package — commands, events,
aggregates, ports:

```text
src/procurement_domain/
├── domain/
│   ├── aggregates.py     # PurchaseOrder, Payment — with the policy check wired in
│   ├── commands.py       # TransferPurchaseOrderFromBuyerToSupplier, ...
│   ├── events.py         # PurchaseOrderTransferredFromBuyerToSupplier, ...
│   ├── roles.py
│   └── value_objects.py
├── ports/                # repositories, bus, read model — your adapters plug in here
└── tests/
```

Look at what happened to the policy. It didn't stay prose. It became a method
call that runs before every transfer:

```python
@dataclass
class Payment:
    """Aggregate root for resource 'Payment' (unit: USD)."""

    def transfer_to_supplier(self, quantity, issued_by, occurred_at):
        """Transfer to Supplier. every order must be approved before payment"""
        self.check_require_approval()
        return PaymentTransferredFromBuyerToSupplier(self.id, quantity, occurred_at)
```

That is generated output, verbatim. The rule your team used to enforce by
remembering it is now a call site the code cannot skip. The generated package
passes `mypy --strict` — and the equivalent TypeScript and Rust projections
pass `tsc --noEmit` and `cargo check` (that's checked in CI, not asserted in
marketing).

---

## Projection two: logs that know what they mean

Here's the pain, in one log line. Without a shared model, your telemetry looks
like this:

```text
INFO payment-svc: transfer ok user=blake amt=100 dest=acct-9
```

Six months later, someone asks: *Was that USD? Which business flow was that?
Which version of the approval rule was in force?* And the honest answer is:
nobody can tell from the logs. The knowledge lived in whoever wrote that line.

Now project the same model into an OpenTelemetry semantic-convention registry:

```bash
domainforge project --format otel-semconv procurement.sea ./out
```

You get generated attribute constants (Python, TypeScript, and Rust) that name
every entity and flow in your domain:

```python
# Generated — do not edit by hand.
PROCUREMENT_FLOW_4A86B32298A6C064_RESOURCE = "procurement.flow.4a86b32298a6c064.resource"
PROCUREMENT_FLOW_4A86B32298A6C064_SOURCE   = "procurement.flow.4a86b32298a6c064.source"
DOMAINFORGE_MODEL_HASH                     = "domainforge.model.hash"
```

So the same event now lands in your traces as:

```text
procurement.flow.4a86b32298a6c064.resource = "Payment"
procurement.flow.4a86b32298a6c064.source   = "Buyer"
procurement.flow.4a86b32298a6c064.target   = "Supplier"
domainforge.model.hash                     = "9196b638edc99b1f"
```

Same event, different epistemic status. The first log line is a string someone
typed. The second is a claim joined to a model: this span describes *that*
flow, between *those* entities, under *this exact version* of the rules —
the model hash is a deterministic fingerprint of the source. When an auditor
asks "which rules governed this transaction," you query for it instead of
scheduling archaeology.

Trust is not a vibe. It is an evidence trail.

---

## Same source, every audience

Every team that asks "can you express this in *our* format?" is asking you to
hand-maintain another copy of the truth. Projection ends that. The one `.sea`
file above also projects to:

| Someone asks for... | You run `--format` | What comes out |
| --- | --- | --- |
| Working code | `domain-python`, `domain-typescript`, `domain-rust` | Typed DDD/CQRS domain layers |
| API contracts | `protobuf`, `asyncapi`, `cloudevents` | Schemas and event definitions |
| Architecture review | `calm`, `archimate`, `bpmn`, `cmmn` | Architecture-as-code and process diagrams |
| Formal assurance | `tla`, `alloy`, `lean` | Machine-checkable specs (TLA+ is model-checked with TLC in CI) |
| Access policy | `cedar` | Cedar schema + policies scoped to your declared flows |
| Observability | `otel-semconv` | The telemetry registry you just saw |
| Knowledge graph | `rdf`, `kg` | RDF/OWL, Turtle, JSON-LD |
| AI/ML pipelines | `baml`, `dspy`, `zenml`, `ai-*` | Typed AI capabilities and training datasets |
| Test suites | `gauge` | One scenario per flow, Given/When/Then |

Projections are deterministic: same model, same flags, byte-identical output.
That's not a slogan — it's an assertion the repo tests against itself (two
isolated runs, `diff -r`).

---

## Drift, detected instead of discovered

The quiet failure mode of every "source of truth" is that it stops being true
and nobody notices. DomainForge makes the model's meaning a fingerprint:

```bash
domainforge pack build --source procurement.sea ... --out pack.json
# → meaning_fingerprint: sha256:458f9541...
```

Build it twice, same source: same fingerprint. Rename one entity and diff:

```bash
domainforge pack diff pack_a.json pack_b.json
# → typed diff, classified "breaking", nonzero exit — wire it into CI
```

Semantic change becomes a build failure instead of a production surprise.

---

## Receipts

Claims in this README are classified and tested. The repo proves itself:

```bash
just prove
```

runs the whole proof suite — language checks, determinism, every projection
gate, round-trips, drift detection — and writes a machine-readable evidence
pack to `evidence/latest/`. Every public claim is cataloged in
[PROOFS.md](PROOFS.md) as **proven**, **partial**, **planned**, or **blocked**,
with the exact command that would falsify it. Where a projection is validated
by its ecosystem's own toolchain (TLA+ via SANY/TLC, AsyncAPI via the official
schema, generated code via `mypy`/`tsc`/`cargo`), PROOFS.md says so. Where the
check is structural only (Cedar, Gauge, Alloy), it says that too.

Completion without proof is theater. We'd rather show you the gaps.

---

## Start in two minutes

```bash
# Get the CLI
git clone https://github.com/GodSpeedAI/DomainForge && cd DomainForge
cargo install --path domainforge-core --features cli

# Steal the model from this README
cp fixtures/projection_cell/basic/model.sea my-domain.sea

# See your domain become code
domainforge validate my-domain.sea
domainforge project --format domain-python my-domain.sea ./out
ls out/src/procurement_domain/domain/
```

Then swap in your own entities and flows. The fastest way to learn SEA is to
rename things in a working model and re-validate.

Library bindings, if you want the graph in-process instead of the CLI:

```bash
pip install domainforge        # Python
npm install domainforge        # TypeScript / Node
cargo add domainforge-core     # Rust
```

The same Rust core drives all of them (and WASM for the browser). Projections
are CLI-only today.

---

## Where it fits

DomainForge is standalone Apache 2.0 infrastructure. It does not require any
particular cloud, agent runtime, or governance platform. Use it as the
semantic layer under your services, your architecture repo, your policy
engine, or your agents.

Within the broader GodSpeed stack, it is the first layer:

```text
DomainForge defines the domain.
SEA-Forge governs the work.
SWE_SEED proves the change.
GodSpeed-Agent compounds the capability.
```

That stack is optional. DomainForge stands on its own.

---

## Documentation

- [Getting started & SEA language guide](docs/index.md)
- [Projection families](docs/projection-families.md)
- [Projection target status](docs/projection-target-implementation-status.md) — the honest per-target validator table
- [Project domain code (SEA → DDD mapping)](docs/how-tos/project-domain-code.md)
- [PROOFS.md](PROOFS.md) — every claim, classified, with its falsifying command

## Contributing

Issues and PRs welcome. Run `just prove` before submitting — if the evidence
pack passes, your change didn't break a claim.

## License

Apache 2.0 — see [LICENSE](LICENSE).
