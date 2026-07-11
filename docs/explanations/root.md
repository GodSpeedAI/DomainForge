



# DomainForge, Semantic Exhaust, Projections, and the GitHub Strategy

## The whole idea in one sentence

**DomainForge is a semantic intermediate representation for organizations: it captures declared organizational meaning in `.sea`, projects that meaning into the ecosystems where work happens, then uses semantically anchored operational exhaust to compare declared reality with observed reality and improve the model.**

It is not merely a DSL, exporter, app generator, ontology tool, or logging product. It is the semantic contract between organizational cognition and organizational execution. fileciteturn2file0

The full loop is:

```text
Organizational distinctions
        ↓
.sea canonical semantic model
        ↓
Canonical graph / semantic IR
        ↓
Deterministic projections
        ↓
Activation and governed execution
        ↓
Logs, events, traces, artifacts, decisions
        ↓
Semantic exhaust
        ↓
Observed semantic graph
        ↓
Declared-vs-observed semantic diff
        ↓
Human-reviewed model revision
        ↓
Re-project
```

That is the connective tissue across the recent discussions.

---

# 1. What DomainForge actually owns

DomainForge owns **meaning**, not every runtime that consumes meaning.

A `.sea` model expresses the distinctions the organization considers consequential:

```text
entities
roles
resources
flows
relations
states
actions
policies
permissions
obligations
prohibitions
metrics
evidence requirements
capabilities
concept changes
```

The durable hierarchy is:

```text
.sea / canonical graph owns meaning.
Projection manifests own deterministic handoff shape.
Projectors own target-specific artifacts.
Runners and adapters own activation.
SEA-Forge owns runtime authority and evidence.
Operational systems produce observations.
DomainForge reconciles observations against meaning.
```

This matters because the generated BPMN file, Protobuf schema, knowledge graph, or application scaffold must never quietly become a second source of truth. Generated artifacts are consequences of the model. They do not get to redefine it. fileciteturn2file0

The upstream extension of the usual measurement doctrine is:

```text
You cannot control what you cannot measure.
You cannot measure what you cannot represent.
You cannot govern what you cannot make consequential.
```

So the operating chain becomes:

```text
Distinction
→ Representation
→ Measurement
→ Governance
→ Execution
→ Evidence
→ Learning
→ Capability
```

---

# 2. DomainForge as the semantic IR

The strongest strategic analogy is not “DomainForge is a low-code platform.”

It is:

```text
LLVM is an IR for programming languages.
ONNX is an IR for machine-learning models.
Protobuf is a canonical wire representation.
OpenAPI is an interface representation.

DomainForge wants to become the IR for organizational meaning.
```

Organizations currently maintain the same meaning independently in several places:

```text
Process team       → BPMN
Case-management    → CMMN
Architects         → ArchiMate
Data/ontology team → RDF/OWL
Developers         → schemas and code
Observability team → telemetry conventions
Governance team    → policies and controls
AI team            → prompts, evals, datasets
```

That creates an N×M semantic-translation problem. Every discipline translates reality into its own format, and every copy begins drifting.

DomainForge changes the shape to:

```text
Reality
  ↓
.sea
  ↓
N projection targets
```

The disruption is not principally file generation. It is the elimination of repeated semantic translation. Generated files are the receipt. fileciteturn2file1

---

# 3. What “projection” means

A projection is a deterministic rendering of the same underlying meaning for a particular ecosystem.

The claim is not:

> One language runs everywhere.

The defensible claim is:

> One semantic source can produce many ecosystem-native representations because those ecosystems need different expressions of the same meaning.

## Projection families

| Family | Representative targets | What becomes reachable |
|---|---|---|
| Canonical/programmatic | AST, JSON, canonical graph | Linters, analyzers, custom projectors, agents |
| Semantic graph | RDF, Turtle, JSON-LD, OWL, SHACL | Knowledge graphs, reasoning, semantic queries |
| Architecture | ArchiMate, FINOS CALM | Enterprise architecture and architectural governance |
| Ordered work | BPMN | Processes, lanes, tasks, gateways |
| Adaptive work | CMMN | Cases, discretionary work, milestones, policy sentries |
| Service contracts | Protobuf/gRPC, OpenAPI, JSON Schema | APIs, validation, message contracts |
| Event systems | CloudEvents, AsyncAPI | Event envelopes, channels, producers, consumers, replay contracts |
| Observability | OpenTelemetry semantic conventions | Business-native logs, traces, and metrics |
| Formal assurance | Lean and related proof obligations | Machine-checkable organizational constraints |
| AI representation | BAML | Typed AI functions derived from domain capabilities |
| AI optimization | DSPy | Optimizers and evaluations grounded in domain policy |
| Learning pipelines | ZenML, datasets, eval sets | Source-traceable capability learning |
| Application structure | DDD/hexagonal manifests, ports and adapters | Generated applications without surrendering semantic ownership |
| Agent participation | Governed agentic hooks | Observe, suggest, transform, execute, or escalate through bounded interfaces |
| Runtime/infrastructure | Dagger, CubeSandbox, Pulumi | Deterministic execution and infrastructure activation |
| Human interface | React Flow, Leo/ODE-style views, native modeling tools | Visual editing and inspection without making the UI canonical |

The current repository evidence already supports some core surfaces such as AST/JSON, Protobuf, RDF/Turtle/KG, CALM-oriented outputs, and language/browser bindings. The broader family—including CMMN, BPMN, OpenTelemetry, AI-learning targets, CloudEvents, AsyncAPI, and richer UI projections—is the strategic projection program and should be labeled by actual maturity rather than presented as uniformly shipped. fileciteturn4file5

---

# 4. CloudEvents, AsyncAPI, and OpenTelemetry

These three targets solve different parts of the semantic-exhaust path.

## CloudEvents: the event instance

CloudEvents should be treated as a transport projection or profile for individual events.

It answers:

```text
What event occurred?
Who produced it?
When did it occur?
What type is it?
What subject does it concern?
What semantic payload does it carry?
```

DomainForge supplies the meaning and identifiers. CloudEvents supplies a standardized event envelope.

A projected event might carry:

```json
{
  "specversion": "1.0",
  "type": "godspeed.payment.authorization.denied",
  "source": "urn:godspeed:checkout",
  "subject": "sea:flow.PaymentAuthorization",
  "id": "evt_789",
  "time": "2026-07-07T14:32:10Z",
  "datacontenttype": "application/json",
  "domainmodelhash": "sha256:...",
  "data": {
    "policy_ref": "sea:policy.PaymentMustBeAuthorizedBeforeCapture",
    "outcome": "denied",
    "reason": "card_declined"
  }
}
```

CloudEvents does not own the domain model. It carries events whose semantics originate in `.sea`.

## AsyncAPI: the event system contract

AsyncAPI describes the communication topology around those events:

```text
channels
subjects/topics
publishers
subscribers
message schemas
correlation rules
security requirements
bindings
replay expectations
failure channels
```

DomainForge can project the declared events, flows, roles, and policies into an AsyncAPI document. This makes the event architecture inspectable and implementable without hand-authoring another semantic copy.

The clean relationship is:

```text
.sea defines what the events mean.
CloudEvents standardizes event instances.
AsyncAPI describes how those events move.
```

## OpenTelemetry: observed execution

OpenTelemetry records what occurred during execution:

```text
traces  → causal flow
logs    → event detail
metrics → aggregate health
```

The DomainForge projection provides semantic conventions so telemetry can carry:

```text
domain model hash
model element ID
domain
concept
policy
capability
actor
operation
resource
correlation ID
causation ID
settlement/evidence references
```

Then an observability query can move beyond:

> Which service was slow?

toward:

> Which version of the PaymentAuthorization capability was slow, under which policy, and what settlement resulted?

This business-native observability is one of the key bridges from ordinary logs to semantic exhaust. fileciteturn3file4

---

# 5. What semantic exhaust is

Raw exhaust is everything emitted while work happens:

```text
logs
events
traces
plans
prompts
tool calls
terminal commands
file changes
commits
pull requests
reviews
authority decisions
operator interventions
artifacts
proof results
failures
retries
settlement decisions
```

That exhaust becomes **semantic exhaust** when it is anchored to declared meaning.

At minimum, a useful semantic record needs some combination of:

```text
domain_model_hash
model_element_id
actor
operation
resource
context
intent
policy reference
capability reference
correlation and causation
observed outcome
evidence references
provenance
settlement status
```

Without those anchors, the system knows that something happened.

With those anchors, the system knows what the occurrence is evidence **about**.

## Tool sources discussed recently

The tools you have considered are valuable because they expose different slices of work:

| Source | Valuable exhaust |
|---|---|
| Traycer | Intent, plans, phases, task artifacts, verification state, execution history |
| AgentPet | Operator interactions, interventions, notifications, session activity |
| Shepherd | Typed execution traces, permissions, retained outputs, reversible state |
| CubeSandbox | Sandbox lifecycle, commands, process results, artifacts, egress and environment state |
| SWE_SEED | Route, context, required artifact, proof command, trace |
| GitHub | Issues, commits, diffs, PRs, reviews, CI checks, releases |
| SEA-Forge | Authority requests, allow/deny/escalate decisions, evidence obligations |
| OpenTelemetry | Technical logs, spans, metrics, causal relationships |
| Context Kernel | Context requests, citations, retrieved evidence |
| GodSpeed-Agent | Settlement, capability change, developmental-memory updates |

The strategic move is **not** to make Traycer, AgentPet, GitHub, or any other product the canonical data model.

The move is:

```text
Native source event
      ↓
Source-specific adapter
      ↓
Canonical semantic envelope
      ↓
Durable ledger + knowledge graph
      ↓
DomainForge-compatible observed model
```

The newer SEA-Forge architecture work expresses this as one governed capability lifecycle:

```text
Intent
→ Domain interpretation
→ Authority decision
→ Plan
→ Execution
→ Observation
→ Evidence
→ Settlement
→ Semantic envelope
```

Every run emits structured residue that can later feed DomainForge and `.sea` projections. fileciteturn3file17

---

# 6. Before and after DomainForge

This is the sharpest public demonstration because it makes the mechanism obvious without asking the reader to understand the entire architecture.

## Before

```json
{
  "timestamp": "2026-07-07T14:32:10Z",
  "service": "checkout-api",
  "event": "payment_failed",
  "user_id": "u_123",
  "order_id": "ord_456",
  "error": "card_declined"
}
```

This is useful operational data. It tells us a payment failed.

But it does not tell us:

- which declared flow this event belongs to;
- which policy applied;
- whether the failure represents correct or incorrect behavior;
- which capability was exercised;
- what the event proves;
- whether the model should change;
- which version of the model authorized the behavior.

## After

```json
{
  "event_id": "evt_789",
  "observed_at": "2026-07-07T14:32:10Z",
  "domain_model_hash": "sha256:...",
  "semantic_ref": "sea:flow.PaymentAuthorization",
  "policy_ref": "sea:policy.PaymentMustBeAuthorizedBeforeCapture",
  "capability_ref": "sea:capability.AuthorizePayment",
  "actor": "Customer",
  "resource": "Payment",
  "from": "Checkout",
  "to": "PaymentProcessor",
  "outcome": "denied",
  "reason": "card_declined",
  "evidence_for": [
    "payment_authorization_attempted",
    "capture_blocked_correctly"
  ],
  "semantic_diff": {
    "status": "no_model_change_required",
    "explanation": "Observed behavior matches declared payment policy."
  }
}
```

Now the failure can be interpreted correctly: the payment failed, but the system may have behaved exactly as intended.

The public hook is:

> **Before DomainForge, your logs tell you what happened. After DomainForge, your logs tell you what happened relative to declared reality.**

Or even more directly:

> **DomainForge does not replace your logs. It makes your logs know what they are evidence of.**

That before/after comparison is the central GitHub narrative. fileciteturn2file0

---

# 7. Declared reality, observed reality, and the organizational twin

The architecture separates three models.

| Model | Source | Meaning |
|---|---|---|
| Declared model | Human-authored `.sea` | What the organization says should exist |
| Observed model | Runtime events, logs, traces, artifacts | What the organization demonstrably does |
| Organizational twin | Reconciled declared and observed models | The living model of how the organization currently operates |

This is not just logging and not merely a digital twin.

It is a **semantic twin**: a model of organizational meaning, authority, roles, policies, flows, capabilities, and observed outcomes.

The reverse-semantic-mining loop is:

```text
1. Declare an initial model in .sea.
2. Run projected systems and workflows.
3. Emit model-linked semantic exhaust.
4. Materialize observations into an observed graph.
5. Detect repeated patterns, exceptions, and violations.
6. Compare observed behavior with declared meaning.
7. Produce a typed semantic diff.
8. Propose additions, corrections, retirements, or concept changes.
9. Require human review.
10. Update .sea and regenerate projections.
```

Useful diff classifications include:

```text
matches_declared_model
policy_violation
undeclared_behavior
missing_concept
stale_concept
emergent_pattern
projection_drift
implementation_drift
insufficient_evidence
model_change_proposed
```

The critical guardrail is that exhaust must not silently rewrite the constitutional source. Observed regularity is evidence, not automatic legitimacy. A repeated bad practice is still a bad practice. Human and governance review remain between observation and recomposition. fileciteturn2file0

---

# 8. Projection activation modes

One important correction from the discussions was that not every projection needs a custom runtime.

Some projections are already useful when loaded into their native ecosystem. Others require increasingly expensive activation.

| Mode | Meaning | Examples |
|---|---|---|
| `host_native` | Load it into existing tooling | ArchiMate, RDF, documentation |
| `validation_only` | Check syntax, schema, proof, or consistency | JSON Schema, Protobuf schema, Lean |
| `binding_required` | Select adapters and runtime wiring | OpenAPI service, DDD/hex app |
| `runner_required` | Execute deterministic tasks | Dagger, generated pipelines |
| `streaming_required` | Bind producers, consumers, broker, replay | AsyncAPI/event schemas |
| `governed_binding_required` | Require authority, scope, and evidence | Agentic hooks |
| `privileged_runner_required` | Can mutate infrastructure or sensitive state | Pulumi, deployment operations |

This avoids building an unnecessary GodSpeed runtime around every artifact.

The decision rule is:

> A projection needs activation only when it exchanges with an environment, performs side effects, binds credentials, runs continuously, or coordinates live behavior.

The golden fixture work was designed to make this classification executable and testable rather than leaving it as architecture prose. fileciteturn2file2

---

# 9. The GitHub strategy

The DomainForge GitHub repository should function as an **executable argument**, not as a conventional project page with a feature list.

A visitor should understand the category claim through one complete worked case.

## The first-screen message

```text
DomainForge turns organizational meaning into infrastructure.

Author your domain once in .sea.
Project it into the ecosystems where work happens.
Use runtime evidence to keep the model honest.
```

Immediately below that:

```text
Before DomainForge:
Logs tell you something failed.

After DomainForge:
Logs tell you which capability ran, which policy applied,
what the event proves, and whether declared reality needs revision.
```

## Recommended README sequence

### 1. The before/after log

Make the difference visible in thirty seconds.

### 2. The `.sea` source

Show the small payment model that defines:

```text
Customer
Checkout
PaymentProcessor
Payment
PaymentAuthorization
Authorization policy
Expected evidence
```

### 3. The projection gallery

Show the same source compiled into materially different forms:

```text
RDF/Turtle
Protobuf
CALM or ArchiMate
OpenTelemetry conventions
CloudEvents schema/profile
AsyncAPI channels
BPMN/CMMN when available
React Flow visual graph
```

Each output should display:

```text
source model hash
source element IDs
projector version
validation result
```

### 4. The running-system evidence

Show an actual emitted event or trace carrying the model identity.

### 5. The semantic diff

Show one observation that matches the model and one that exposes drift.

### 6. The proposed model change

Show the system proposing a `.sea` patch, while making human approval explicit.

### 7. The proof commands

A visitor should be able to run:

```bash
domainforge validate examples/payment-authorization/model.sea
domainforge project --target rdf examples/payment-authorization/model.sea
domainforge project --target protobuf examples/payment-authorization/model.sea
domainforge observe examples/payment-authorization/events/
domainforge diff examples/payment-authorization/model.sea \
  examples/payment-authorization/observed/
```

The exact CLI may differ, but the repository experience should settle those jobs.

### 8. The projection status matrix

Every target should be marked:

```text
implemented
experimental
partial
planned
research
```

This preserves claims discipline and prevents an ambitious projection roadmap from looking like unsupported vaporware.

---

# 10. Recommended repository shape

```text
/
├── README.md
├── docs/
│   ├── why-domainforge.md
│   ├── semantic-ir.md
│   ├── semantic-projection.md
│   ├── semantic-exhaust.md
│   ├── reverse-semantic-mining.md
│   ├── organizational-twin.md
│   ├── projection-activation.md
│   └── projection-status.md
│
├── examples/
│   └── payment-authorization/
│       ├── model.sea
│       ├── raw-logs/
│       │   └── payment-failed.json
│       ├── semantic-events/
│       │   └── payment-authorization-denied.json
│       ├── projections/
│       │   ├── ast/
│       │   ├── rdf/
│       │   ├── protobuf/
│       │   ├── calm/
│       │   ├── cloudevents/
│       │   ├── asyncapi/
│       │   ├── otel/
│       │   └── visual/
│       ├── observed/
│       │   └── observed-graph.ttl
│       ├── semantic-diff.json
│       └── proposed-model-change.patch
│
├── fixtures/
│   ├── semantic-primitives/
│   ├── application-kernel/
│   ├── projection-activation/
│   ├── governance-evidence/
│   ├── organizational-twin/
│   ├── reverse-semantic-mining/
│   ├── learning-projections/
│   └── formal-assurance/
│
├── schemas/
│   ├── semantic-envelope/
│   ├── cloudevents/
│   └── observed-events/
│
└── .github/
    └── workflows/
        ├── validate-sea.yml
        ├── deterministic-projections.yml
        ├── target-conformance.yml
        └── stale-projection-check.yml
```

The golden fixture suite already follows this proof-oriented philosophy: it is meant to demonstrate semantic primitives, fractal scale, application generation, activation modes, governance, twins, reverse mining, learning projections, formal assurance, and DomainForge modeling itself. fileciteturn1file11

---

# 11. The associated content strategy

The repository supplies the proof. The surrounding content explains one mechanism at a time.

## Core content pillars

### “Your logs do not know what they mean”

Use the before/after example to introduce semantic exhaust.

### “Semantic translation is an invisible enterprise tax”

Show how the same policy is repeatedly translated into diagrams, code, telemetry, AI prompts, and compliance documents.

### “One model, many native ecosystems”

Show `.sea` becoming RDF, Protobuf, CALM, OpenTelemetry, CloudEvents, AsyncAPI, and visual forms.

### “Declared reality versus observed reality”

Explain why process documents and architecture diagrams become fiction when no evidence returns to them.

### “A semantic twin, not another dashboard”

The twin is not a prettier monitoring screen. It is the reconciled model of what the organization claims and proves.

### “Agents should be participants, not owners”

Show governed agentic hooks: agents can observe, suggest, transform, execute, or escalate only through typed surfaces.

### “Projection is not activation”

Explain why a diagram can be useful immediately, while a deployment projection requires runners, authority, secrets, state, and evidence.

### “DomainForge is an IR, not a universal runtime”

This prevents the project from sounding like it wants to replace BPMN engines, architecture tools, observability platforms, graph databases, or AI toolchains.

### “Reverse semantic mining turns exhaust into organizational memory”

Show the path from logs and traces to stable patterns, model proposals, and capability memory.

---

# 12. The visual-interface strategy

React Flow, LeoEditor-style outlines, CMMN tools, BPMN tools, and ArchiMate tools should be treated as **views over the semantic source**.

They solve different attention jobs:

```text
React Flow
→ flexible graph editing and custom GodSpeed visual language

Leo/ODE-style interface
→ literate, hierarchical, outline-oriented semantic development

CMMN
→ adaptive case-work view

BPMN
→ ordered process view

ArchiMate
→ enterprise architecture view

RDF graph tooling
→ ontology and relationship view
```

None should independently own the model.

The safe pattern is:

```text
.sea
↔ canonical graph
↔ visual projection / controlled editing operations
```

A UI edit should become a typed semantic operation—add entity, modify policy, connect flow—not an arbitrary mutation of generated files.

---

# 13. Open and commercial boundaries

The GitHub strategy and licensing strategy reinforce each other.

DomainForge should remain the broad semantic-adoption layer:

```text
.sea language
parser
validator
canonical graph
core projections
fixtures
basic CLI
basic reverse semantic mining
local semantic diff
proposed .sea patches
```

The strategic formulation is:

> **Give away the roads’ grammar. Charge for governed travel, activation, evidence, orchestration, and compounding.**

Commercial GodSpeed surfaces then sit above it:

```text
SEA-Forge authority runtime
production event ingestion
policy and evidence ledger
managed activation
enterprise connectors
organizational-twin dashboards
advanced semantic-diff recommendations
compliance evidence packs
managed sandbox execution
support, certification, audits
```

Free DomainForge gets declared and observed reality into a common semantic form. Paid GodSpeed infrastructure governs and operationalizes the loop at enterprise scale. fileciteturn1file11

---

# 14. What this affords strategically

## It collapses semantic duplication

One change to the domain model can deterministically update architecture, process, observability, event contracts, data representations, and AI surfaces.

## It inherits existing ecosystems

DomainForge does not need to rebuild graph stores, BPMN engines, OpenTelemetry backends, architecture tools, workflow tools, or AI optimizers. It projects into their native forms.

## It gives every artifact provenance

A trace, diagram, message, training record, or schema can answer:

```text
Which model version created me?
Which semantic element do I represent?
Which projector produced me?
Has the source changed since I was produced?
```

## It creates a living semantic system

The model is no longer a document that only flows outward. Runtime evidence returns and challenges the model.

## It turns agent activity into durable organizational residue

Agent logs stop dying in session history. Plans, actions, evidence, failures, corrections, and settlements can become structured developmental memory.

## It creates a keystone position

Each new projection makes `.sea` more useful. Each new `.sea` model makes the projections more useful. That is the plausible ecosystem flywheel.

That flywheel remains a strategic hypothesis until adoption and repeated integrations prove it, but the mechanism is coherent and falsifiable. fileciteturn2file1

---

# 15. The minimum falsifiable GitHub proof

The next affordable settlement is not fifty projection targets.

It is one complete loop.

## Recommended example

Use **Payment Authorization** because it naturally contains actors, resources, flows, policy, failure, evidence, and settlement.

Build:

```text
1 canonical .sea model
3 materially different projections
1 raw log
1 semantically enriched event
1 observed graph
1 semantic diff
1 human-reviewed model proposal
1 CI workflow proving deterministic regeneration
```

The three projections should cover different concerns:

```text
RDF/Turtle       → semantic graph
Protobuf         → program/service contract
OpenTelemetry or CloudEvents → runtime evidence
```

AsyncAPI can then document the channels through which those events travel.

## Success criteria

The proof settles only when:

1. The `.sea` model validates.
2. All generated elements carry deterministic source identity.
3. The destination ecosystem’s own validator accepts each projection.
4. A runtime event carries the exact model hash and element reference.
5. DomainForge correctly classifies one matching observation and one drift condition.
6. A model change regenerates all projections consistently.
7. CI fails when a generated artifact becomes stale.
8. Observations can propose but cannot silently apply a constitutional model change.

That is the smallest demonstration that DomainForge is more than an exporter.

It proves:

```text
Declare meaning.
Project meaning.
Run work.
Observe consequence.
Compare reality.
Recompose meaning.
```

And that is the coherent center of the DomainForge, semantic-exhaust, projection, GitHub, UI, governance, and organizational-twin strategy.
