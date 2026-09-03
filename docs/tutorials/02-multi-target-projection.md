# Tutorial: Generating Multi-Target Projections

In this tutorial, you will take the `logistics.sea` model created in [Tutorial 1](01-first-sea-model.md) and project it into Python domain code, BPMN 2.0 process XML, and an RDF knowledge graph.

---

## Prerequisites
- Completed [Tutorial 1: Authoring Your First SEA Model](01-first-sea-model.md).
- A valid `logistics.sea` file in your current working directory.

---

## 1. Project to a Python Domain Package

Run the following command to project your model into a Domain-Driven Design (DDD) Python package:

```bash
domainforge project --format domain-python logistics.sea ./out-python
```

### Inspect the Generated Artifacts
Look inside `./out-python`:
```text
out-python/
└── src/
    └── logistics_domain/
        ├── domain/
        │   ├── aggregates.py
        │   ├── commands.py
        │   └── events.py
        └── ports/
            ├── repositories.py
            └── message_bus.py
```

Open `out-python/src/logistics_domain/domain/aggregates.py`. Notice that:
- `Package` has been generated as a dataclass aggregate root.
- The transfer flow has been generated as a method `transfer_to_fulfillment_center()`.
- The `daily_dispatch_limit` policy rule has been wired in directly as an assertion call before the transfer event is emitted!

---

## 2. Project to BPMN 2.0 Process XML

Now project the same model into business process XML for your enterprise architecture team:

```bash
domainforge project --format bpmn logistics.sea ./out-bpmn
```

### Inspect the Generated Artifacts
Look inside `./out-bpmn`:
```text
out-bpmn/
└── process.bpmn
```

Open `process.bpmn` in any standard BPMN editor (such as Camunda Modeler) or inspect the XML tags:
- A `bpmn:collaboration` contains participants `Warehouse` and `FulfillmentCenter`.
- A `bpmn:sequenceFlow` connects the transfer task with a deterministic ID generated via `projection::ids`.

---

## 3. Project to an RDF/OWL Knowledge Graph

Project the model into semantic web standards (Turtle, JSON-LD, and OWL ontology):

```bash
domainforge project --format rdf logistics.sea ./out-rdf
```

### Inspect the Generated Artifacts
```text
out-rdf/
├── model.ttl         # W3C Turtle triples
├── model.jsonld      # Linked Data JSON-LD
└── ontology.owl.ttl  # OWL 2 Web Ontology Language axioms
```

Open `out-rdf/model.ttl` to see that:
- Entities are declared as `sea:Entity`.
- Flows are declared as `sea:Flow` with explicit `sea:source`, `sea:target`, and `sea:quantity` properties.

---

## 4. Test Byte-Level Determinism

DomainForge guarantees bit-for-bit identical output when given the same `--created-at` timestamp. Test this by running two isolated projections:

```bash
domainforge project --format bpmn --created-at "2026-01-01T00:00:00Z" logistics.sea ./run1
domainforge project --format bpmn --created-at "2026-01-01T00:00:00Z" logistics.sea ./run2

diff -r ./run1 ./run2
```
The `diff` command prints nothing and exits with code 0. Both outputs are completely identical!

---

## Next Steps
- Learn how to govern organizational vocabulary in [Tutorial: Building & Signing Semantic Packs](03-building-signing-packs.md).
- Explore the complete list of 17+ targets in the [Projections Subsystem Guide](../subsystems/projections-engine.md).
