# DomainForge

### Compile domain meaning once, then project it where the work happens.

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)


Your coding agent can change a repository in minutes. It can also change five different representations of the same business rule and leave every one of them looking reasonable.

The payment limit lives in service code. The event schema calls the concept something slightly different. A test fixture encodes yesterday's threshold. An architecture file describes the old flow. The agent instructions explain the rule one more time because the model needs enough context to work.

Then the rule changes.

Every copy has to move together, and there is usually nothing in the repository that can prove they still mean the same thing.

DomainForge gives that meaning a source file.

**DomainForge is a semantic compiler and projection engine for the ****`.sea`**** domain modeling language. It compiles executable domain meaning into a canonical semantic model that can be validated and projected into multiple machine-usable representations.**

SEA stands for **Semantic Executable Abstraction**.

A `.sea` file is a human-readable semantic model of a purposeful system. It describes the consequential structure of the domain: what exists, what moves, how things relate, which rules apply, what can be measured, and how that meaning should project into other systems.

A purposeful system can be a product, business process, service, agent workflow, program, organization, or another coordinated system built to produce repeatable outcomes.

The basic shape is familiar if you have worked with a compiler:

```text
domain knowledge
      |
      v
   .sea source
      |
      v
 parse + resolve + validate
      |
      v
canonical semantic model
      |
      +----------+-----------+-----------+-----------+
      |          |           |           |           |
      v          v           v           v           v
    code       schemas    policy      telemetry   architecture
```

For compiler people: `.sea` is the source language. DomainForge's resolved semantic model is the compiler IR. Projections are targets.

For everyone else: write the domain once, check it, and derive the representations that should agree with it.

---

## The problem got sharper when coding got faster

AI coding tools have made implementation cheap enough that a single developer can produce a surprising amount of software. Parallel agents push that further. One agent can work on an API while another changes the frontend and a third writes tests.

The agents still need to answer the same question:

> What does this system mean?

Most repositories make them infer the answer from code, tickets, docs, schemas, prompts, tests, and whatever context happened to be loaded this turn.

That works until two sources disagree.

The failure can be subtle. Each artifact may compile. Each agent may finish its task. The PR can look clean. The inconsistency only becomes visible later, when a rule behaves differently across services, telemetry cannot explain what happened, an architecture diagram no longer describes the software, or the next agent learns from the wrong copy.

More generation creates more surfaces where meaning can drift.

DomainForge moves the shared meaning into a form that humans can read and machines can validate.

---

## SEA in one minute

Here is a small payment domain:

```sea
@namespace "payments"
@version "1.0.0"
@owner "payments-team"

Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"

Entity "Buyer"
Entity "Supplier"

Resource "Money" USD

Flow "Money" from "Buyer" to "Supplier" quantity 100

Policy transfer_limit
    per Constraint Obligation priority 5
    @rationale "Modeled money transfers must not exceed the direct-transfer limit"
    as:
        forall f in flows:
            (f.resource != "Money" or f.quantity <= 10000)

Metric "money_flow_count" as:
    count(f in flows where f.resource = "Money": f.quantity)
    @unit "flows"
```

You can read the important parts without knowing SEA.

There are buyers and suppliers. Money is measured in USD. The model contains a Money flow from Buyer to Supplier. A declared policy constrains modeled Money flows to a quantity of 10,000 or less. A metric counts those flows.

The same statements are available to DomainForge as typed semantic structure. They can be validated, queried, fingerprinted, and projected without asking another tool to reinterpret the prose.

Validate the model:

```bash
domainforge validate --format human payments.sea
```

Inspect what DomainForge parsed:

```bash
domainforge parse --ast --format json payments.sea
```

Project it:

```bash
domainforge project --format domain-python payments.sea ./out
```

The useful jump happens here. The model stops being a diagram that someone has to remember to consult. It becomes an input to the build.

---

## Start with one rule that keeps getting copied

Start with one domain where the same meaning already appears in several places.

Good first targets are boring:

* a resource limit that appears in code and tests
* an entity or event name copied across services
* a flow that shows up in architecture docs and implementation
* a policy agents keep needing in their prompt context
* a metric whose definition changes depending on who calculates it

Model that small slice in SEA. Validate it. Project one useful target. Put the validation or projection check in CI.

That gives you a real test of whether a semantic source helps before you model anything larger.

### Install the CLI

Install the CLI from the published Rust crate:

```bash
cargo install domainforge-core --features cli
```

Or install the published npm package:

```bash
npm install -g @godspeedai/domainforge
```

Verify the installation:

```bash
domainforge --version
```

### Start from a working model

```bash
cp fixtures/projection_cell/basic/model.sea my-domain.sea
domainforge validate --format human my-domain.sea
```

Change the names and flows to match something you actually own. Validate again after every small edit.

### Inspect the semantic model

```bash
domainforge parse --ast --format json my-domain.sea
domainforge parse --format json my-domain.sea
```

Parsing answers a useful debugging question: did DomainForge understand the model you thought you wrote?

### Generate one target

```bash
domainforge project --format domain-python my-domain.sea ./out
```

Then inspect the generated files. A projection should earn trust from its output and validators.

The shortest useful loop is:

```text
write a small model
  -> validate it
  -> inspect the parsed meaning
  -> project one target
  -> check the output
  -> change the source
  -> repeat
```

---

## One semantic source, many technical surfaces

A team usually starts duplicating domain meaning for reasonable reasons.

The backend needs types. Another service needs Protobuf. Security needs policy. Operations needs telemetry. Architecture wants CALM or ArchiMate. The agent needs context. Testing needs scenarios.

Each request creates another representation.

DomainForge treats those representations as projections from the same semantic source.

| The problem you need to solve        | Projection family or format                         | Typical output                                                            |
| ------------------------------------ | --------------------------------------------------- | ------------------------------------------------------------------------- |
| Put the domain into application code | `domain-python`, `domain-typescript`, `domain-rust` | Typed domain layers                                                       |
| Share contracts between services     | `protobuf`, `asyncapi`, `cloudevents`               | Schemas and event definitions                                             |
| Review architecture and process      | `calm`, `archimate`, `bpmn`, `cmmn`                 | Architecture and process representations                                  |
| Check behavior more formally         | `tla`, `alloy`, `lean`                              | Formal specifications and proof-oriented artifacts                        |
| Express access rules                 | `cedar`                                             | Cedar schema and policy output                                            |
| Give telemetry stable domain names   | `otel-semconv`                                      | OpenTelemetry semantic-convention registries                              |
| Query the domain as a graph          | `rdf`, `kg`                                         | RDF/OWL, Turtle, JSON-LD, and graph forms                                 |
| Build AI-facing artifacts            | `baml`, `dspy`, `zenml`, `ai-*`                     | Typed AI and dataset-oriented projections                                 |
| Derive behavioral scenarios          | `gauge`                                             | Given/When/Then scenarios                                                 |
| Define a bounded agent environment   | `cell`                                              | Environment, dependency, authority, network, evidence, and lock artifacts |

Projection maturity differs by target. See [projection target status](docs/projection-target-implementation-status.md) and [PROOFS.md](PROOFS.md) before treating a target as a production guarantee.

The architectural rule is simple:

> Keep generated targets downstream of the source model.

If a projection needs meaning that the source cannot express, that is useful information. Fix the source model, document the limitation, or decide that the target owns a concern outside the domain model.

---

## Why this matters for coding agents

A coding agent has access to files. That gives it evidence about the domain. Canonical interpretation still has to come from somewhere.

Consider a repository with these statements:

```text
docs/requirements.md       "Transfers above $10,000 need review"
services/payments.py       MAX_DIRECT_TRANSFER = 25000
tests/test_limits.py       assert transfer(15000).allowed
agents/payments.md         "Keep direct transfers at or below $10k"
```

A capable model can notice the contradiction if all four files reach its context and it knows the contradiction matters.

That is a large "if."

Now imagine three agents working in parallel. Each agent gets a different slice of the repository. Every one can produce a locally coherent change.

SEA gives the domain a canonical representation that an agent can inspect directly or consume through a projection. DomainForge can then check the source and regenerate the technical forms tied to it.

The agent can still be wrong. The failure surface changes.

Instead of asking every agent to reconstruct the domain from scattered clues, you can give it an explicit model and test whether its work remains compatible with that model.

That becomes more useful as agents get faster.

---

## A source file can still drift. DomainForge makes the drift inspectable.

A file named "source of truth" can still go stale.

DomainForge gives the semantic model stable fingerprints and typed diffs so changes can enter the same review machinery as code.

Build a pack:

```bash
domainforge pack build \
  --source payments.sea \
  --out pack.json
```

The pack contains a meaning fingerprint derived from the model.

Compare versions:

```bash
domainforge pack diff pack_a.json pack_b.json
```

A semantic change can now fail CI, trigger review, or become an explicit migration decision.

That is a different failure mode from discovering six weeks later that two teams implemented different meanings of the same rule.

---

## Telemetry gets more useful when events can point back to domain meaning

A normal log can tell you that code ran:

```text
INFO payment-svc: transfer ok user=blake amt=100 dest=acct-9
```

It may still leave basic questions unanswered later.

Which domain flow was this? What did "amt" mean? Which model version described the transaction? Did another service use the same names?

Project the SEA model into an OpenTelemetry semantic-convention registry:

```bash
domainforge project --format otel-semconv payments.sea ./out
```

DomainForge can generate stable attribute names tied to entities and flows, plus a model hash that identifies the semantic source.

The telemetry event can now carry a reference to the domain model that gave its fields meaning.

The log still needs independent evidence to prove a claim. The projection removes one common ambiguity: the event no longer has to invent its own private vocabulary.

---

## The compiler model

DomainForge becomes easier to reason about when each layer has one job.

### 1. SEA source

`.sea` is the human-authorable source language.

SEA means **Semantic Executable Abstraction**.

"Semantic" means the file describes domain meaning explicitly.

"Executable" means the meaning can participate in machine validation, policy evaluation, metrics, generation, and other computational work. SEA remains a declarative modeling language.

"Abstraction" matters because the model is a representation of the domain. It is deliberately smaller than reality. A good model captures the distinctions that downstream work needs and leaves unrelated detail out.

### 2. Semantic compilation

DomainForge parses declarations, resolves references and imports, builds the domain graph, and applies semantic validation.

A file can be syntactically valid and still fail here. A Flow that references an unknown Entity is one example.

### 3. Canonical semantic model

This is the compiler's semantic IR.

It holds the resolved meaning that projections consume. Downstream targets consume those resolved semantics directly.

### 4. Projection

A projection maps the canonical semantic model into a target representation.

Different targets can expose different parts of the model. A Protobuf schema, knowledge graph, CALM model, policy artifact, and Python domain layer have different jobs. They still begin from the same resolved semantics.

This boundary is what lets DomainForge act like compiler infrastructure instead of a bag of format converters.

---

## What SEA can represent today

SEA has source constructs for domain concepts such as:

* entities
* resources
* flows
* roles and relations
* policies
* metrics
* instances
* dimensions and units
* patterns
* mappings and projections
* imports and exports
* concept changes

A model only needs the concepts that matter to its domain.

For example, a small agent workflow may care about actors, resources, policy, evidence-facing metrics, and projection contracts. A financial model may need units, instances, constraints, and flows. An interaction model may use the same language to describe actors, resources, journeys, policies, outcomes, and interface projections where the current grammar can express them cleanly.

The parser and validator are the authority on what is legal SEA. See [the SEA language guide](docs/index.md) before relying on examples from old documents or generated prose.

---

## Keep the boundary clear

DomainForge removes a class of duplication. Domain work still requires judgment.

You still have to decide which concepts matter. A bad model can validate. A projection can be technically correct and still encode the wrong business meaning.

Your application runtime, database, workflow engine, agent harness, and architecture practice keep their own jobs. DomainForge gives them a shared semantic source they can consume.

Generated artifacts should be reviewed like generated code from any compiler. Mature targets have stronger validators than experimental ones. The repository records that difference explicitly.

That boundary is deliberate. The compiler makes meaning portable. Domain judgment still determines whether the model is right.

---

## Deterministic projections

A projection is much less useful if identical input can quietly produce different output.

DomainForge tests projection determinism by generating from the same model in isolated runs and comparing the results.

For deterministic targets:

```text
same .sea source
+ same DomainForge version
+ same projection flags
= byte-identical generated output
```

That property makes generated artifacts easier to review, cache, diff, and gate in CI.

See [PROOFS.md](PROOFS.md) for the exact claims and checks currently enforced by the repository.

---

## Proofs live next to the claims

The strongest evidence for DomainForge lives in the repository.

Run:

```bash
just prove
```

The proof suite exercises language checks, projection gates, determinism, round trips, drift detection, and other repository claims. It writes machine-readable evidence under:

```text
evidence/latest/
```

[PROOFS.md](PROOFS.md) classifies public claims as:

```text
proven
partial
planned
blocked
```

It also records what each validator actually proves.

A generated Python package passing `mypy --strict` is stronger evidence than checking that files exist. A TLA+ projection exercised by its own toolchain has a different evidence level from a target that only receives structural validation.

Those differences stay visible.

---

## Use DomainForge from Python, TypeScript, Rust, or the browser

The Rust core is available through language bindings when you want the semantic graph inside another program.

```bash
pip install domainforge
npm install domainforge
cargo add domainforge-core
```

The same core also supports WASM for browser use.

Today, projections are CLI-driven. The bindings are useful when another tool needs to parse, inspect, validate, or work with the domain model in-process.

This separation keeps the compiler core reusable without forcing every consumer to shell out for basic semantic access.

---

## Imports let a domain grow without becoming one giant file

A model can import named concepts from other SEA modules:

```sea
@namespace "commerce.orders"

import { Money } from "finance.common"
import { Customer } from "commerce.customers"

export Entity "Order"
```

Wildcard imports are also supported with an alias:

```sea
import * as core from "std:core"
```

A namespace registry can map module names to files.

Use semantic ownership as the split criterion. Keep concepts together while they share one coherent domain. Create module boundaries when the boundary makes ownership, reuse, or validation clearer.

---

## Model change as a first-class event

Domains change. SEA can record concept evolution instead of hiding it in Git history alone.

```sea
Entity "VendorV2" v2.0.0
    @replaces "Vendor" v1.0.0
    @changes ["added payment_terms"]
    in procurement

ConceptChange "Vendor_v2_migration"
    @from_version v1.0.0
    @to_version v2.0.0
    @migration_policy mandatory
    @breaking_change true
```

That gives downstream tooling a semantic statement about change alongside the textual diff.

For a small model, Git may be enough. The value appears when several generated targets depend on the same concept and a rename or contract change has consequences outside one file.

---

## A practical adoption path

The easiest way to make DomainForge useless is to model too much before anything depends on the model.

Start narrower.

### First: find duplicated meaning

Look for a concept or rule that currently has at least two technical representations and has changed before.

Examples include a payment state, workflow transition, event contract, policy threshold, or domain term agents repeatedly need explained.

### Next: encode the smallest useful SEA model

Represent enough context for the duplicated meaning to make sense.

Avoid modeling unrelated parts of the organization because they seem conceptually adjacent.

### Then: make one downstream artifact depend on it

Pick the output whose drift already costs you something.

For one team that may be Protobuf. For another it may be OpenTelemetry names, a Python domain layer, CALM, or an agent-facing projection.

### Finally: give the model a consequence

Add validation, generation, fingerprint comparison, or another appropriate check to CI.

At that point the model has a job. A change to its meaning can affect a build, a generated artifact, or a review.

Expand the model when the next duplicated distinction earns the cost.

---

## Where DomainForge fits

DomainForge is Apache 2.0 and can stand on its own.

You can use it under an application architecture, an AI coding workflow, a policy system, an architecture repository, or an internal platform. It works without a specific cloud or model provider.

Within GodSpeed AI, DomainForge supplies the semantic source used by other components. SEA-Forge can govern actions against declared domain meaning. Other tools can consume projections for software work, evidence, or agent operation.

Those integrations are optional.

The important boundary is smaller:

```text
your domain
    |
    v
   .sea
    |
    v
DomainForge
    |
    v
validated semantic model
    |
    v
the technical surfaces that need the same meaning
```

---

## When DomainForge is probably worth trying

A small experiment makes sense when you recognize one of these repository shapes:

* the same business rule is copied into code, tests, docs, schemas, or prompts
* coding agents keep rediscovering domain terms from scratch
* architecture artifacts fall behind implementation
* telemetry uses local names that are hard to relate back to business concepts
* policy changes require coordinated edits across several systems
* two technically valid artifacts can disagree about the same domain fact
* you want generated targets but need a human-readable source above them

If none of those are costing you anything, a new semantic layer may be extra machinery.

If they are already showing up, try one small model before adopting the larger idea.

---

## Documentation

* [Getting started and SEA language guide](docs/index.md)
* [Projection families](docs/projection-families.md)
* [Projection target status](docs/projection-target-implementation-status.md)
* [Project domain code](docs/how-tos/project-domain-code.md)
* [Project a cell environment](docs/how-tos/project-a-cell-environment.md)
* [PROOFS.md](PROOFS.md)

## Contributing

Issues and pull requests are welcome.

Run the repository proof suite before submitting:

```bash
just prove
```

If you add or strengthen a public capability, add the corresponding evidence to [PROOFS.md](PROOFS.md). If the implementation is partial, keep the claim partial.

## License

Apache 2.0. See [LICENSE](LICENSE).
