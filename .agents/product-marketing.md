# DomainForge product marketing context

**Status:** Working canonical product-marketing context
**Product:** DomainForge
**Company:** GodSpeed AI
**Primary use:** README copy, website copy, launch materials, developer documentation, product strategy, investor explanation, enterprise communication, and AI-generated marketing content
**Last updated:** July 29, 2026

---

## 1. Source-of-truth hierarchy

When sources disagree, use this order:

1. The current DomainForge repository, tests, schemas, specifications, CLI behavior, and license.
2. The current DomainForge README and architecture decisions.
3. This product-marketing context.
4. The GodSpeed AI Brand Voice Style Guide.
5. Strategic architecture documents and the DomainForge Strategic Archive.
6. Older language describing DomainForge primarily as SEA-DSL, a projection engine, or an ontology tool.

Never let positioning override current implementation, test evidence, technical boundaries, or license terms.

Separate claims into four levels:

* **Evidence-backed:** supported by current code, tests, schemas, commands, fixtures, or license text.
* **Partially proven:** implemented in meaningful pieces but missing complete end-to-end or production proof.
* **Strategic interpretation:** follows from the product architecture and should be presented as positioning rather than shipped capability.
* **Roadmap:** specified, planned, or intended but not yet proven.

---

## 2. The framing hierarchy

Use these terms at different levels. Do not treat them as interchangeable.

### `.sea`

`.sea` is the source language.

It is the human-readable language used to declare organizational distinctions such as:

* entities;
* roles;
* resources;
* flows;
* policies;
* relations;
* metrics;
* units;
* instances;
* mappings;
* projections;
* concept changes.

Calling `.sea` a domain-specific language is technically correct.

It is not a sufficient description of DomainForge’s product value.

### DomainForge

DomainForge is the semantic build system.

It:

* parses and validates `.sea`;
* constructs a canonical semantic graph;
* preserves concept identity;
* evaluates supported semantic rules;
* imports and exports target formats;
* produces deterministic projections;
* supports bindings and generated application structures;
* links operational evidence back to declared meaning;
* compares declared and observed reality;
* proposes revisions when the two diverge.

### Organizational source code

Organizational source code is the product category and buyer-facing value.

It is the versioned, implementation-independent representation of what an organization means:

* what exists;
* which distinctions matter;
* how concepts relate;
* which resources flow;
* which roles participate;
* what must, may, or must not happen;
* what is measured;
* what evidence means;
* which changes break prior assumptions;
* which forms other tools and runtimes require.

### Canonical formulation

> `.sea` is the source language. DomainForge is the semantic build system. Organizational source code is what the system produces and maintains.

### Necessary correction

Organizational source code is not reality itself.

It is the organization’s declared representation of reality.

Operational evidence can confirm it, contradict it, expose missing distinctions, or reveal that the organization behaves differently from what it claims.

Use this distinction:

```text
Declared source
+ observed evidence
+ reconciliation history
= living organizational model
```

Never imply that writing a `.sea` file makes the declared model automatically true.

---

## 3. Product summary

### Plain-language definition

DomainForge turns the meaning scattered across an organization into versioned source that people and machines can share.

Instead of separately rewriting the same entities, roles, rules, flows, metrics, and relationships in documents, code, schemas, architecture diagrams, policies, prompts, and dashboards, teams declare the meaning once and project it into the forms each environment needs.

### Product definition

DomainForge is the semantic build system for organizational source code.

It turns `.sea` models into a validated canonical semantic graph, then deterministically projects that meaning into software contracts, knowledge graphs, architecture models, policy surfaces, documentation, generated applications, agent interfaces, and other operational forms.

### Smallest useful explanation

> DomainForge gives organizational meaning a source file and a build system.

### One-sentence position

> DomainForge turns scattered business meaning into versioned organizational source code that can be validated, projected, executed, observed, and revised.

### Sharp public version

> Your organization already has source code. It is scattered across documents, software, diagrams, tickets, policies, and people’s heads.

### Controlling product truth

> Software drifts when teams edit generated artifacts instead of the source. Organizations drift for the same reason.

### Extended formulation

> Every organization runs on entities, roles, resources, flows, policies, metrics, and decisions. Most organizations encode those distinctions repeatedly in incompatible tools. DomainForge gives that meaning a canonical source, projects it into the environments where work happens, and compares the declaration with operational evidence when reality disagrees.

---

## 4. The market wedge

DomainForge’s initial wedge is not “model the entire organization.”

The first problem is smaller:

> Stop translating the same domain meaning by hand across code, schemas, architecture, governance, AI context, and documentation.

The first credible use case should contain:

* one bounded domain;
* one `.sea` source model;
* a small set of entities, resources, flows, policies, and metrics;
* at least two deterministic projections;
* target-native validation;
* one semantic event or observed trace linked back to the model;
* one declared-versus-observed comparison.

The buyer does not need to accept the complete organizational-source-code vision before using DomainForge.

They need to recognize this immediate problem:

```text
The same business concept exists in several systems.
Each system represents it differently.
Each representation changes independently.
No one can prove which meaning is canonical.
AI makes the disagreement move faster.
```

### Recommended wedge line

> Declare domain meaning once. Project it into every system that needs a native form.

### Recommended wedge category

> Semantic build system for domain contracts.

### Larger product category

> Organizational source code.

### GodSpeed category

> The semantic substrate of syntelligent infrastructure.

Use these categories in sequence. Do not force a reader to accept “organizational source code” before showing the repeated-translation problem.

---

## 5. The core analogy

The organizational-source-code analogy should carry the product argument without being pushed beyond its limits.

### Software model

```text
source code
→ compiler
→ build artifacts
→ runtime behavior
→ tests and telemetry
→ source revision
```

### DomainForge model

```text
.sea organizational source
→ canonical semantic graph
→ deterministic projections
→ activated systems and workflows
→ semantic evidence
→ declared-versus-observed diff
→ human-reviewed source revision
```

### Mapping

| Software concept          | DomainForge concept                                                                    |
| ------------------------- | -------------------------------------------------------------------------------------- |
| Source code               | `.sea` declared organizational meaning                                                 |
| Compiler and build system | DomainForge parser, validator, graph, and projectors                                   |
| Build artifact            | Schema, graph, architecture model, code contract, documentation, generated application |
| Runtime configuration     | Provider and binding configuration                                                     |
| Executable environment    | Host tool, application runtime, workflow engine, agent system                          |
| Test and telemetry output | Semantic evidence linked to declared concepts                                          |
| Failing test              | Observed behavior contradicting a policy, relation, flow, or expected outcome          |
| Source change             | Reviewed update to the `.sea` model                                                    |
| Rebuild                   | Regenerate affected projections from the revised source                                |
| Drift                     | Generated or observed reality no longer agrees with declared meaning                   |

### Boundary of the analogy

An organization is not reducible to code.

People interpret, negotiate, resist, improvise, learn, and change. Power, culture, judgment, tacit knowledge, and local context cannot be fully compiled into a file.

The point of organizational source code is not total representation.

The point is to make important distinctions explicit enough to:

* share;
* validate;
* version;
* project;
* govern;
* observe;
* compare;
* revise.

Use the analogy as an operational model, not a claim that an organization is a deterministic machine.

---

## 6. Why now

Organizations have always suffered from semantic fragmentation.

AI makes the cost more visible and more urgent.

### Before widespread agents

Humans absorbed translation costs through:

* meetings;
* documentation;
* code review;
* architecture review;
* informal clarification;
* institutional memory;
* manual reconciliation.

The translation was slow, expensive, and unreliable, but people often noticed contradictions before acting.

### With AI systems

Agents consume whatever representation happens to enter context:

* a stale policy;
* an incomplete schema;
* a code implementation;
* an architecture diagram;
* a ticket;
* a prompt;
* a document fragment;
* a vector-search result.

The agent may then act quickly and confidently on one local version of the organization’s meaning.

The underlying problem is not only missing context.

It is the absence of canonical, executable organizational meaning.

### Why the old source-of-truth model fails

Most organizations have many systems of record but no semantic source.

They may know:

* where customer rows live;
* where process diagrams live;
* where policies live;
* where code lives;
* where metrics live;
* where architecture diagrams live.

They often cannot answer:

* whether “Customer” means the same thing in all of them;
* which policy definition is canonical;
* which metric formula should govern;
* which flow an operational event belongs to;
* which version an agent should use;
* how a change should propagate;
* what runtime evidence contradicts the declared model.

### Why DomainForge matters now

AI lowers the cost of generating implementations.

As generation becomes cheaper, semantic disagreement becomes the bottleneck.

> When code becomes easy to produce, deciding what the code is supposed to mean becomes more valuable.

---

## 7. The primary problem: semantic translation by hand

Every serious organization expresses the same meaning repeatedly in incompatible places.

* Business teams describe it in documents and meetings.
* Developers encode it in services and types.
* Data teams encode it in schemas and ontologies.
* Architects encode it in models and diagrams.
* Governance teams encode it in policies and controls.
* Operations teams encode it in workflows.
* Observability teams encode it in telemetry names.
* AI teams encode fragments in prompts, tools, evals, and retrieval systems.
* Auditors later ask which interpretation was authoritative.

The same entities, roles, resources, flows, policies, and metrics are maintained by different people at different speeds.

### Current pattern

```text
Organizational meaning
  ├─→ documents
  ├─→ source code
  ├─→ APIs and schemas
  ├─→ workflow tools
  ├─→ architecture diagrams
  ├─→ policies and controls
  ├─→ telemetry
  ├─→ knowledge graphs
  └─→ prompts and agent context
```

Each branch can drift.

### DomainForge pattern

```text
.sea organizational source
  → canonical semantic graph
      ├─→ software and API contracts
      ├─→ architecture projections
      ├─→ knowledge graphs
      ├─→ policy surfaces
      ├─→ documentation
      ├─→ agent contracts
      ├─→ generated applications
      └─→ semantic telemetry contracts
```

The value is not one language replacing every tool.

The value is one semantic source producing the native forms those tools require.

---

## 8. Primary audiences

### 8.1 Primary technical adopter

**Who they are**

* platform engineers;
* domain architects;
* enterprise architects;
* staff and principal engineers;
* AI infrastructure engineers;
* knowledge-graph engineers;
* developer-platform teams;
* model-driven engineering teams;
* technical founders building AI-native systems.

**Current situation**

They maintain equivalent domain meaning across several forms:

* domain classes;
* database schemas;
* API contracts;
* event schemas;
* architecture diagrams;
* ontologies;
* policy engines;
* documentation;
* agent tools and prompts.

They know those forms disagree.

They lack an acceptable canonical layer upstream of all of them.

**What they want**

* one implementation-independent semantic source;
* deterministic projections;
* stable concept identity;
* target-native validation;
* explicit versioning and change semantics;
* cross-language consistency;
* fewer manual translations;
* clearer ownership of meaning;
* the ability to connect runtime evidence back to the model.

**What they distrust**

* universal modeling languages that claim to replace every specialist tool;
* low-code systems that trap meaning inside one vendor;
* ontology projects that never reach execution;
* code generators that own the model inside proprietary metadata;
* architecture repositories disconnected from runtime evidence;
* AI-generated schemas without deterministic validation.

### 8.2 Primary organizational buyer

**Who they are**

* CTO;
* CIO;
* chief architect;
* head of platform engineering;
* head of enterprise architecture;
* chief data or knowledge officer;
* AI transformation leader;
* regulated-technology leader;
* operations or transformation executive.

**What they need**

* a coherent representation of how the organization works;
* traceability from business meaning to implementation;
* fewer inconsistencies among policy, process, data, software, and AI;
* faster system generation without semantic drift;
* clearer change impact;
* evidence of where declared and observed reality diverge;
* a foundation for governed agents and organizational twins.

**Emotional state**

They know the organization has accumulated years of disconnected representations.

They do not want another enterprise repository that becomes a beautifully governed graveyard.

They want architecture and domain modeling to become consequential.

### 8.3 Domain authors

**Who they are**

* business analysts;
* domain experts;
* policy owners;
* process owners;
* quality and compliance professionals;
* product managers;
* Domain Engineers;
* Forward-Deployed Engineers.

**What they need**

* a human-readable way to express domain distinctions;
* agent-assisted authoring rather than hand-coding everything;
* validation before technical teams build on the model;
* reviewable changes;
* visible relationships among rules, flows, metrics, and systems;
* confidence that their meaning survives projection.

Domain authors should not need to become compiler engineers.

The intended authoring experience is:

```text
domain expert describes the situation
→ agent proposes .sea
→ human reviews and corrects meaning
→ DomainForge validates the model
→ projections and bindings are generated
→ evidence returns for further review
```

### 8.4 Secondary audiences

* software generators and agent-framework builders;
* governance-platform builders;
* digital-twin teams;
* process-mining teams;
* AI evaluation teams;
* data-contract teams;
* standards bodies;
* systems integrators;
* researchers in model-driven engineering and organizational cognition.

Do not write one paragraph aimed equally at every audience.

For the README:

* lead for the technical adopter;
* show organizational significance without requiring executive jargon;
* preserve a path for domain authors;
* keep the broader twin and syntelligent-infrastructure story below the immediate wedge.

---

## 9. Core jobs to be done

### Functional jobs

When my organization expresses the same domain meaning in several systems, help me:

1. identify the distinctions that must remain stable;
2. represent those distinctions in a human-readable source;
3. validate references, types, units, policies, and relationships;
4. assign stable identity to domain concepts;
5. version changes to those concepts;
6. construct one canonical semantic graph;
7. project the graph into target-native forms;
8. preserve traceability from every projection back to the source;
9. identify which projections need only validation and which need runtime binding;
10. generate software and application structures without making generated code canonical;
11. give agents explicit domain contracts rather than loose context fragments;
12. attach runtime events to declared semantic concepts;
13. compare declared behavior with observed behavior;
14. detect semantic, implementation, and operational drift;
15. propose source changes without automatically rewriting organizational truth;
16. rebuild affected projections after a reviewed change.

### Emotional jobs

Help me feel:

* confident that teams are implementing the same meaning;
* less dependent on tribal memory;
* less afraid that automation will amplify hidden disagreement;
* able to explain which source is canonical;
* comfortable changing one model without manually chasing every representation;
* able to see disagreement rather than pretending alignment.

### Social jobs

Help me:

* give business and technical teams a shared contract;
* show executives that architecture can drive implementation;
* give auditors traceability from declared policy to operational evidence;
* let AI teams move faster without inventing domain meaning locally;
* make governance executable without handing control to one proprietary platform;
* demonstrate that the organization’s operating model is versioned rather than folkloric.

---

## 10. Core customer problems

### 10.1 The organization has systems of record but no semantic source

A CRM may own customer records.

A service may own payment logic.

A policy repository may own a control statement.

An architecture tool may own a system diagram.

None necessarily owns the meaning of “Customer,” “Payment,” “Authorization,” or “Accepted.”

Underlying problem:

> Data location is not semantic authority.

### 10.2 The same distinction is implemented many times

A concept may exist as:

* a paragraph;
* a database table;
* a TypeScript type;
* a Rust struct;
* a Protobuf message;
* an RDF class;
* a diagram node;
* a policy rule;
* a metric;
* an agent tool description.

Underlying problem:

> The organization repeatedly pays to translate meaning, then pays again when the translations disagree.

### 10.3 Code has become the accidental source of business meaning

Teams often say, “The code is the source of truth.”

Code is the source of truth for what one implementation currently does.

It may not reveal:

* why the concept exists;
* which business policy it implements;
* which alternative implementations are valid;
* which organizational meaning should survive a rewrite;
* whether current behavior is intentional or drift.

Underlying problem:

> Implementation has been mistaken for intent.

### 10.4 Documents describe rules but cannot make them operational

Documents are readable and flexible.

They are poor at:

* deterministic validation;
* stable identity;
* automated projection;
* cross-language conformance;
* runtime linking;
* drift detection.

Underlying problem:

> Important meaning is visible to humans but unavailable as infrastructure.

### 10.5 AI systems amplify semantic fragmentation

An agent may receive one version of a concept from retrieval, another from code, and another from a ticket.

It can generate a coherent answer from contradictory premises.

Underlying problem:

> Better generation cannot repair an organization that has not stabilized what its words mean.

### 10.6 Architecture is separated from operation

Architecture models often describe intended systems.

Runtime telemetry describes actual systems.

The two are rarely linked at concept level.

Underlying problem:

> Architecture cannot learn from operation because operational evidence has no semantic address.

### 10.7 Generated code becomes a new source of drift

Code generation often starts from a model but ends with developers modifying generated artifacts.

The generated output gradually becomes canonical.

Underlying problem:

> The build artifact silently replaces the source.

DomainForge must keep generated zones derived and regenerate them from reviewed source changes.

### 10.8 Organizational change has no semantic diff

When a business concept changes, teams struggle to identify:

* which systems are affected;
* which policies depend on it;
* which schemas change;
* which agents require new context;
* which evidence becomes incompatible;
* whether the change is breaking.

Underlying problem:

> Organizations manage semantic change through meetings and memory rather than versioned impact analysis.

### 10.9 Declared reality and observed reality drift apart

Organizations claim one process and perform another.

That difference may be:

* legitimate adaptation;
* policy violation;
* an incomplete model;
* a new capability;
* hidden work;
* technical debt;
* an exception that became normal.

Underlying problem:

> Most organizations cannot represent the difference between what they say they do and what they repeatedly prove they do.

---

## 11. Status quo and alternatives

DomainForge competes first with semantic fragmentation.

It also overlaps with several established categories. Position it as an upstream semantic layer, not a universal replacement.

### 11.1 Documents, wikis, and policy repositories

**Why teams use them**

* easy to author;
* readable;
* flexible;
* familiar;
* support review and discussion.

**Where they fail**

* weak formal identity;
* limited machine validation;
* no deterministic projection;
* difficult impact analysis;
* disconnected from runtime evidence;
* meaning often depends on interpretation.

**DomainForge position**

> Documents explain meaning. DomainForge gives important meaning a validated operational form.

Do not claim that DomainForge eliminates prose, policy documents, or narrative explanation.

### 11.2 Source code

**Why teams treat it as truth**

* executable;
* tested;
* versioned;
* close to production behavior.

**Where it falls short**

* coupled to one implementation;
* domain meaning is distributed across modules;
* business rationale may be absent;
* hard for non-developers to review;
* cross-language implementations can diverge;
* actual behavior can differ from intended behavior.

**DomainForge position**

> Code is one projection of organizational meaning, not the only place that meaning should live.

### 11.3 JSON, YAML, and general configuration formats

**Why teams use them**

* ubiquitous;
* easy to parse;
* flexible;
* supported by every language.

**Where they fall short**

* syntax does not supply domain semantics;
* identity and reference rules must be reinvented;
* validation remains schema-specific;
* projection logic becomes custom glue;
* meaning is implicit in field names and conventions.

**DomainForge position**

> JSON and YAML are containers. DomainForge defines the semantic contract they would otherwise have to imply.

### 11.4 Protobuf, OpenAPI, JSON Schema, and data contracts

**Why teams use them**

* mature ecosystems;
* precise interfaces;
* code generation;
* compatibility tooling;
* strong validation.

**Where they fall short**

* optimized for specific contract surfaces;
* do not capture the whole organizational semantic model;
* policies, metrics, roles, flows, and concept changes may remain elsewhere;
* each schema family becomes another source to maintain.

**DomainForge position**

> DomainForge does not replace contract ecosystems. It projects shared meaning into them.

### 11.5 DDD, UML, BPMN, CMMN, ArchiMate, and enterprise architecture tools

**Why teams use them**

* rich specialized notation;
* established methods;
* useful visualizations;
* mature communities.

**Where they fall short**

* each captures a specific view;
* models may remain disconnected;
* runtime traceability varies;
* implementation handoff often becomes manual;
* no single view should own the whole domain.

**DomainForge position**

> Specialist models remain valuable. DomainForge supplies the shared semantic source from which appropriate views can be projected or into which they can be imported.

Do not claim that one `.sea` model preserves every nuance of every specialist notation.

Projection contracts must state what meaning is preserved and what is lost.

### 11.6 Knowledge graphs and ontology platforms

**Why teams use them**

* relational expressiveness;
* inference;
* semantic interoperability;
* graph queries;
* standards support.

**Where they fall short**

* authoring can be inaccessible;
* execution and application generation are not always central;
* governance and software-contract projections may require custom work;
* ontology programs can become disconnected from daily development.

**DomainForge position**

> DomainForge can project into knowledge-graph ecosystems while keeping a concise, versioned source language and deterministic build path.

Do not diminish RDF, SHACL, OWL, or graph-native systems. They are target ecosystems and potential import surfaces.

### 11.7 Low-code and application-generation platforms

**Why teams use them**

* rapid application delivery;
* visual authoring;
* integrated runtime;
* reduced coding burden.

**Where they fall short**

* meaning may be trapped in proprietary metadata;
* generated applications may be tightly coupled to one runtime;
* the platform often owns the source and the execution environment;
* semantic portability is weak.

**DomainForge position**

> DomainForge separates organizational source from any one application generator or runtime.

Application generation is a projection of the source, not ownership of the source.

### 11.8 Enterprise architecture repositories

**Why teams use them**

* centralized models;
* portfolio visibility;
* governance workflows;
* impact analysis.

**Where they fall short**

* models can become detached from code and operations;
* updates often rely on manual stewardship;
* runtime evidence is not automatically semantically linked;
* architecture becomes descriptive rather than generative.

**DomainForge position**

> DomainForge makes architecture a build input and an evidence target, not only a documentation surface.

### 11.9 Process mining and digital twins

**Why teams use them**

* discover actual behavior;
* identify bottlenecks;
* visualize process variation;
* simulate or monitor systems.

**Where they fall short**

* observed patterns may lack declared semantic intent;
* frequency does not establish legitimacy;
* actual behavior can encode workarounds or violations;
* process data alone does not own the organizational source.

**DomainForge position**

> Observed behavior should challenge the declared model, not silently replace it.

Process-mined patterns can become proposed `.sea` changes subject to review.

### 11.10 Prompts, RAG, and agent memory

**Why teams use them**

* fast to deploy;
* natural-language friendly;
* flexible;
* useful for local context.

**Where they fall short**

* context is selected per task;
* retrieved fragments can conflict;
* identity and versioning are weak;
* prose does not guarantee semantic consistency;
* agents can invent connective meaning.

**DomainForge position**

> Retrieval gives an agent information. DomainForge gives the agent a shared world model.

---

## 12. Positioning

### Positioning statement

For organizations and technical teams that repeatedly encode the same domain meaning across software, architecture, policy, data, and AI systems, DomainForge is the semantic build system for organizational source code.

Unlike documents, schemas, code, diagrams, ontologies, or application platforms used independently, DomainForge keeps implementation-independent meaning in a versioned `.sea` source, validates it as a canonical semantic graph, projects it into target-native forms, and links operational evidence back to the model so declared reality can be compared with observed reality.

### Category ladder

Use categories in this order:

1. **Immediate problem:** repeated semantic translation across tools;
2. **Technical category:** semantic compiler and projection engine;
3. **Product category:** semantic build system;
4. **Market category:** organizational source code;
5. **GodSpeed category:** semantic substrate for syntelligent infrastructure.

### Core contrast

```text
Typical organization

documents + code + schemas + diagrams + policies + prompts
→ several interpretations of the same domain
→ manual translation
→ drift
```

```text
With DomainForge

.sea source
→ canonical semantic graph
→ deterministic native projections
→ runtime evidence
→ semantic diff
→ reviewed source revision
```

### Product promise

> Change the meaning once, rebuild the forms that depend on it, and see where reality disagrees.

### Strategic value

DomainForge helps organizations move from:

* scattered meaning to versioned semantic source;
* manual translation to deterministic projection;
* implementation-owned meaning to implementation-independent contracts;
* architecture as documentation to architecture as build input;
* logs without context to evidence with semantic identity;
* accidental drift to visible semantic difference;
* AI context assembled from fragments to shared machine-readable domain structure;
* one-off code generation to regenerable application structure;
* declared truth without feedback to a model accountable to observation.

---

## 13. The core mechanism

DomainForge’s differentiation is the complete semantic loop, not merely `.sea` syntax.

### 13.1 Declare

Capture the domain’s meaningful distinctions:

* entities;
* roles;
* resources;
* flows;
* relations;
* policies;
* metrics;
* units;
* instances;
* mappings;
* projections;
* concept changes.

### 13.2 Validate

Check that the source:

* parses;
* uses valid syntax;
* resolves references;
* preserves type and unit constraints;
* contains coherent policies;
* satisfies graph-level rules;
* uses stable concept identities;
* makes versioned changes explicit.

### 13.3 Normalize

Construct a canonical semantic graph or intermediate representation.

The graph is a derived representation of the `.sea` source, not a second independently edited truth.

### 13.4 Project

Generate target-native forms such as:

* AST and graph JSON;
* RDF and Turtle;
* SBVR;
* CALM;
* Protobuf and gRPC contracts;
* schemas;
* architecture models;
* documentation;
* knowledge-graph events;
* policy inputs;
* application manifests;
* ports and adapter contracts;
* generated code and application structure;
* agent participation contracts;
* proof and evaluation artifacts.

Only describe projections as shipped when current repository evidence supports them.

### 13.5 Activate

Some projections work when loaded by a host tool.

Others need:

* configuration;
* provider bindings;
* adapters;
* credentials;
* task runners;
* event streams;
* governance;
* deployment infrastructure.

DomainForge must distinguish semantic source from runtime bindings.

Use this rule:

> `.sea` owns meaning. Binding configuration owns environment-specific realization.

Provider YAML, manifests, runtime configuration, and generated adapters must not quietly become the canonical domain model.

### 13.6 Observe

Operational systems emit events, traces, logs, and evidence linked to canonical semantic identities.

The purpose is not merely richer telemetry.

The purpose is to state what the event is evidence of.

### 13.7 Compare

Construct an observed semantic model and compare it with declared reality.

Identify:

* conformance;
* missing distinctions;
* policy violations;
* unexpected flows;
* implementation drift;
* stable local adaptations;
* obsolete concepts;
* new candidate capabilities.

### 13.8 Revise

Produce a proposed semantic diff or `.sea` patch for human review.

Observed frequency does not automatically confer legitimacy.

The organization must decide whether the difference should be:

* accepted into the model;
* rejected as violation;
* represented as an exception;
* governed more tightly;
* investigated;
* redesigned;
* retired.

### Canonical loop

```text
Declare once.
Validate the model.
Project many.
Activate where needed.
Observe what happens.
Compare reality with the declaration.
Revise the source.
Rebuild.
```

---

## 14. Declared, observed, and reconciled reality

This distinction is central to DomainForge’s product identity.

### Declared reality

The `.sea` source records what the organization currently claims or intends:

* what exists;
* what relationships hold;
* which flows are expected;
* which policies apply;
* what metrics mean;
* which changes are valid.

Declared reality is intentional and reviewable.

It can still be incomplete, outdated, biased, politically convenient, or wrong.

### Observed reality

Semantic events and traces record what the organization actually does.

Observed reality may expose:

* undocumented work;
* repeated exceptions;
* policy violations;
* emergent capability;
* local adaptation;
* technical constraints;
* hidden dependencies;
* inaccurate declared models.

Observed reality is not automatically desirable.

Repeated behavior can represent debt, workaround, coercion, or pathology.

### Reconciled organizational model

The organizational twin emerges from the relationship between declared and observed reality.

```text
Declared model
+ observed evidence
+ accepted and rejected semantic changes
+ reconciliation history
= organizational twin
```

### Canonical line

> Your `.sea` source is what the organization says it is. Its semantic evidence is what the organization proves it does.

### Governance line

> The difference between declared and observed reality is not noise to erase. It is the primary object of organizational learning and governance.

---

## 15. Organizational source code as a control surface

Organizational source code is valuable because representation changes what can be built, checked, and governed.

The chain is:

```text
distinction
→ representation
→ validation
→ projection
→ execution
→ evidence
→ correction
```

Before an organization can reliably automate, govern, measure, or teach a concept to an agent, it must represent that concept well enough to preserve its identity across environments.

### Drucker extension

> You cannot control what you cannot measure.
> You cannot measure what you cannot represent.
> You cannot govern what you cannot make consequential.

DomainForge operates upstream of measurement and governance.

It stabilizes the distinctions those systems depend on.

### Important limit

Representation does not create reality.

A perfect `.sea` model with no projection, activation, observation, or revision remains an elegant declaration.

DomainForge’s value comes from closing the loop between meaning and operation.

---

## 16. Feature-to-outcome translation

Do not market features without their consequence.

| Capability                           | Reader consequence                                                                                   |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------- |
| Human-readable `.sea` source         | Domain meaning can be reviewed without reading each implementation.                                  |
| Parser and formatter                 | The source has a stable, repeatable form.                                                            |
| Semantic validation                  | Broken references and invalid relationships fail before projections inherit them.                    |
| Canonical semantic graph             | Each target begins from the same normalized meaning.                                                 |
| Stable concept identity              | Events, projections, and changes can refer to the same concept across tools and time.                |
| Deterministic projections            | Teams stop hand-translating the same meaning into each target format.                                |
| Target-native validators             | A projection must work in its destination, not merely look plausible.                                |
| Import surfaces                      | Existing models can participate without requiring an immediate rewrite from scratch.                 |
| Versioned concept changes            | Semantic changes become reviewable and can be classified as breaking or compatible.                  |
| Unit and metric semantics            | Measurements keep their meaning across systems.                                                      |
| Policy expressions                   | Rules become machine-checkable rather than remaining prose alone.                                    |
| Mapping and projection contracts     | The system states how meaning should appear in a target environment.                                 |
| Cross-language surfaces              | Rust, Python, TypeScript, and browser tools can share the same semantic model.                       |
| Generated manifests and applications | Implementations can be rebuilt from the semantic source rather than becoming one-off artifacts.      |
| Semantic event contracts             | Logs can identify what declared concept they are evidence of.                                        |
| Declared-versus-observed diff        | The organization can see where behavior and intention diverge.                                       |
| Proposed `.sea` revisions            | Operational learning can return to the source without automatic model mutation.                      |
| Apache 2.0 licensing                 | The semantic standard can spread across tools and vendors without becoming a proprietary toll booth. |

---

## 17. Product and architecture boundaries

### DomainForge owns

* `.sea` language and grammar;
* parsing and formatting;
* semantic validation;
* canonical AST and semantic graph;
* domain concept identity;
* policy and metric representation;
* imports and projections;
* semantic change representation;
* application and capability projection contracts;
* semantic event contracts;
* declared-versus-observed comparison;
* proposed semantic revisions.

### DomainForge does not automatically own

* runtime authority;
* side-effect execution;
* secrets;
* approvals;
* deployment state;
* production orchestration;
* evidence-ledger integrity;
* final settlement;
* all generated runtime behavior;
* organizational truth without observation.

### SEA Forge owns

* authority before side effects;
* bounded execution;
* approvals and escalation;
* evidence collection;
* settlement;
* integrity-protected execution records;
* governed materialization of privileged projections.

Canonical distinction:

> DomainForge defines what the organization means. SEA Forge governs what may become consequential.

### Application generators own

* target-specific implementation templates;
* generated code;
* framework conventions;
* last-mile application structure.

They consume DomainForge projections.

They do not own canonical organizational meaning.

### Provider and binding configuration owns

* dependency selection;
* concrete adapters;
* service endpoints;
* deployment-specific configuration;
* credentials references;
* environment realization.

### Host tools own

* their native files;
* rendering;
* specialized validation;
* execution behavior;
* target-specific semantics not represented by DomainForge.

### Agents own

* proposed source changes;
* translation assistance;
* generation work;
* analysis;
* candidate mappings.

Agents do not own final semantic authority.

Humans or governed organizational processes review consequential model changes.

---

## 18. Application generation

DomainForge’s application-generation story should be framed carefully.

### Intended flow

```text
person describes a need, situation, goal, or domain
→ agent proposes a .sea model
→ human reviews organizational meaning
→ DomainForge validates the source
→ agent or system selects provider bindings
→ provider configuration is reviewed
→ DomainForge generates application structure and contracts
→ generated application is validated
→ governed runtime activates consequential behavior
```

### Core principle

> The application is a build artifact. The organizational meaning remains the source.

### What DomainForge should generate

Depending on the selected profile and available projectors:

* domain types;
* bounded contexts;
* commands and events;
* ports and adapter contracts;
* API and message schemas;
* validation rules;
* documentation;
* tests;
* policy contracts;
* workflows;
* agentic hooks;
* observability contracts;
* manifests;
* application scaffolds;
* generated zones.

### Profiles

Use profiles to price complexity rather than forcing every domain into one enterprise architecture.

| Profile  | Intended output                                                                          |
| -------- | ---------------------------------------------------------------------------------------- |
| Thin     | Basic entity, action, view, validation, and evidence structure                           |
| Standard | Bounded contexts, ports, adapters, documentation, tests, and workflows                   |
| Full     | Aggregates, events, contracts, observability, and integration tests                      |
| Governed | Authority, evidence, proof hooks, agent participation, and fail-closed runtime contracts |

### Claims discipline

Do not claim complete general application generation unless the current repository proves the complete path.

Separate:

* existing deterministic projections;
* current application-kernel contracts;
* reference generators;
* active proof surfaces;
* roadmap target families.

---

## 19. Agent-native value

DomainForge should not be positioned as a prompt-management product.

Its value to agents is more structural.

### Without DomainForge

An agent must infer the organization from:

* prose;
* code;
* schemas;
* tickets;
* examples;
* tool descriptions;
* retrieved documents.

The agent reconstructs a local world for each task.

### With DomainForge

An agent can receive:

* canonical concept identities;
* typed relationships;
* valid resources and flows;
* active policies;
* metrics and units;
* projection contracts;
* explicit change history;
* allowed participation surfaces;
* evidence obligations.

### Agentic hooks

Generated systems should be agent-ready without becoming agent-owned.

An agentic hook should declare:

* trigger;
* participation mode;
* required capability;
* input and output contract;
* allowed operations;
* policy;
* evidence obligation;
* failure behavior;
* optional human review.

### Core line

> Agents should work inside the organization’s declared world, not reconstruct the organization from whichever fragments fit in the context window.

### Important boundary

A semantic contract does not guarantee safe action.

SEA Forge or another runtime authority layer must govern consequential execution.

---

## 20. Differentiators

### 20.1 Organizational meaning is upstream of implementation

DomainForge keeps the source independent of any one language, framework, database, graph engine, architecture tool, or agent runtime.

### 20.2 Projection rather than replacement

DomainForge does not require every ecosystem to operate directly on `.sea`.

It translates shared meaning into native target forms.

### 20.3 Closed semantic loop

The product is not a bag of exporters.

It connects:

```text
declared meaning
→ projection
→ operation
→ evidence
→ semantic comparison
→ source revision
```

### 20.4 Declared and observed reality remain distinct

DomainForge does not automatically treat documented intent as truth or frequent behavior as legitimacy.

The difference is preserved for review.

### 20.5 Generated artifacts remain derived

Generated code, schemas, manifests, and diagrams should not silently become canonical.

Change the source or projector and rebuild.

### 20.6 Meaning is portable across scales

The same semantic primitives can represent:

* an atomic capability;
* an application;
* a workflow;
* a team;
* an organizational function;
* an organization;
* a population-level pattern.

Do not imply that every scale uses the same profile or payment burden.

### 20.7 Meaning is readable by humans and usable by machines

DomainForge’s advantage is not maximal formalism.

It is enough structure for validation and projection while remaining reviewable by domain authors.

### 20.8 The semantic layer is permissively licensed

Apache 2.0 allows the language, graph, validators, and core projections to become shared infrastructure rather than a proprietary semantic silo.

### 20.9 Meaning can become executable without becoming runtime-owned

DomainForge can produce executable contracts and applications while preserving the boundary between semantic source and governed execution.

---

## 21. Messaging hierarchy

Explain DomainForge in this order.

### Message 1: The familiar problem

> Every team rewrites the same business meaning in a different tool.

### Message 2: The consequence

> Those translations drift, and AI makes the disagreement move faster.

### Message 3: The correction

> Organizations need a canonical semantic source upstream of code, schemas, diagrams, policies, and prompts.

### Message 4: The mechanism

> DomainForge captures that meaning in `.sea`, validates one semantic graph, and projects it into each environment’s native form.

### Message 5: The feedback loop

> Runtime evidence returns to the model so declared reality can be compared with observed reality.

### Message 6: The category

> That makes DomainForge organizational source code, not merely another DSL.

Do not begin with Message 6 unless the next sentence immediately explains Messages 1 through 4.

---

## 22. Approved message formulations

### Primary headline

> Give your organization source code.

### Stronger problem-led headline

> Your organization already has source code. It is scattered everywhere.

### Alternative headline

> Stop rewriting the same business meaning in every system.

### Primary subhead

> DomainForge turns entities, roles, resources, flows, policies, metrics, and relationships into a versioned semantic source, then projects that meaning into the software, schemas, architecture, policies, and agent contracts that need it.

### Feedback-loop subhead

> Declare what the organization means. Project it into operation. Compare the declaration with what the organization actually does.

### Plain one-liner

> DomainForge gives organizational meaning a source file and a build system.

### Technical one-liner

> An Apache 2.0 semantic compiler and projection system for implementation-independent organizational meaning.

### Enterprise one-liner

> DomainForge creates a versioned semantic source across business, technology, governance, data, and AI systems.

### Developer one-liner

> Define the domain once, validate it, and generate the contracts each stack needs.

### AI-native one-liner

> Give agents a shared domain model instead of asking each one to reconstruct the organization from context fragments.

### Organizational-twin one-liner

> The source says what the organization believes. The traces show what it does. DomainForge preserves the difference.

### Application-generation one-liner

> Describe the organization, review the meaning, then generate the application as a projection of that source.

### Sharp lines

* Code should be a projection of meaning, not the only place meaning survives.
* A system of record tells you where data lives. It does not tell you what the data means.
* AI cannot preserve distinctions the organization never made explicit.
* Every manual translation is another place for the business to disagree with itself.
* Architecture that cannot affect a build is usually documentation.
* Your logs know what happened. DomainForge makes them know what it meant.
* The generated app is not the source. It is one compiled expression of the source.
* A knowledge graph without an operational loop can become an expensive diagram.
* Declared reality is a claim. Observed reality is evidence. The difference is where learning begins.
* The organization should not have to explain itself from scratch to every agent.
* DomainForge turns meaning into infrastructure.
* Declare once. Project many. Observe. Revise.

Use one or two sharp lines per surface. Do not make every paragraph an aphorism.

---

## 23. Voice and tone

DomainForge copy should sound:

* precise;
* calm;
* technically literate;
* conceptually ambitious;
* anti-theater;
* respectful of existing tools;
* honest about projection loss and maturity;
* clear about the distinction between declared and observed reality.

The voice should sound like:

> A compiler engineer, enterprise architect, and domain modeler finally agreeing that the organization should stop translating itself by hand.

### Use Speed Mode for

* the organizational-source-code hook;
* the repeated-translation problem;
* category creation;
* AI-context contrasts;
* launch materials.

### Use Journey Mode for

* projection boundaries;
* architecture;
* declared-versus-observed reality;
* governance;
* licensing;
* model-authority concerns;
* implementation maturity.

### Humor boundary

Mock semantic absurdity, not the people maintaining it.

Good:

> If “Customer” has six definitions, the problem is not that the agent needs a larger context window.

Bad:

> Your architects and analysts cannot agree on anything.

---

## 24. Language rules

### Prefer

* organizational source code;
* semantic source;
* canonical meaning;
* declared reality;
* observed reality;
* semantic graph;
* projection;
* target-native;
* stable identity;
* domain contract;
* semantic diff;
* concept change;
* validation;
* binding;
* activation;
* evidence;
* drift;
* implementation-independent;
* generated artifact;
* build;
* rebuild;
* human-reviewed revision.

### Explain before relying on

* `.sea`;
* semantic IR;
* organizational twin;
* reverse semantic mining;
* activation mode;
* agentic hook;
* canonical graph;
* syntelligent infrastructure;
* affordance;
* projection contract.

### Avoid

* one language to rule them all;
* universal ontology;
* single source of truth without qualification;
* digital twin when only a declared model exists;
* self-updating organization;
* fully autonomous organization;
* seamless;
* revolutionary;
* unlock;
* empower;
* AI-powered solution;
* no-code;
* replace every tool;
* compile the whole company;
* perfect alignment;
* automatically learns the organization;
* production-ready without proof.

### Canonical distinctions

Do not cycle among these terms as if they mean the same thing:

* **Source:** authored `.sea` meaning.
* **Graph or IR:** normalized derived representation.
* **Projection:** deterministic target-specific expression.
* **Manifest:** handoff description for generation or activation.
* **Binding:** environment-specific connection to tools, services, or providers.
* **Generated artifact:** derived output such as code, schema, or diagram.
* **Observed evidence:** runtime record linked to semantic identity.
* **Semantic diff:** comparison between declared and observed meaning.
* **Organizational twin:** reconciled declared and observed model with history.

---

## 25. Common objections

### “Isn’t this just another DSL?”

Response:

`.sea` is a DSL. DomainForge is the system around it.

The product value comes from validation, canonical identity, deterministic projection, target-native proof, runtime semantic evidence, declared-versus-observed comparison, and source revision.

Calling DomainForge only a DSL is like calling a compiler “a text-file format.”

### “Why not use JSON or YAML?”

Response:

JSON and YAML can encode a model, but they do not supply the model’s semantics.

Teams would still need to define:

* concept identity;
* reference rules;
* type and unit behavior;
* policies;
* projection contracts;
* change semantics;
* imports;
* validation;
* target mappings;
* evidence linkage.

DomainForge makes those contracts explicit.

### “Why not keep business meaning in code?”

Response:

Code accurately describes one implementation.

DomainForge preserves the domain meaning that should survive changes in language, framework, service boundary, architecture, or runtime.

Code remains important. It becomes a generated or manually implemented projection of an upstream semantic contract.

### “Why not use an ontology or knowledge graph?”

Response:

Use them where they fit.

DomainForge can import from or project into graph ecosystems. Its distinction is a concise source language, deterministic build behavior, software and architecture projections, and a closed operational feedback loop.

It is not a claim that `.sea` should replace all graph-native authoring.

### “Why not use DDD, UML, BPMN, or ArchiMate?”

Response:

Those methods provide valuable specialized views.

DomainForge provides a shared semantic source beneath those views and should preserve only the meaning each target can accurately express.

A projection that loses meaning must state the loss rather than pretending equivalence.

### “One model cannot describe the whole organization.”

Response:

Correct.

DomainForge should not require one giant model.

Use namespaces, imports, bounded domains, profiles, and versioned concepts. The architecture should support federated semantic ownership with explicit boundaries and shared contracts.

Organizational source code means coordinated source, not one monolithic file.

### “Who decides what the organization means?”

Response:

DomainForge does not solve organizational authority by syntax.

Domain owners, policy owners, technical owners, and governed review processes must decide which changes are accepted.

The product makes those decisions explicit, versioned, testable, and traceable.

It does not remove politics or judgment.

### “What happens when the model is wrong?”

Response:

That is why DomainForge separates declared and observed reality.

Operational evidence can expose contradictions and generate proposed semantic changes.

The system should never automatically turn repeated behavior into legitimate organizational truth.

### “Does DomainForge generate complete applications?”

Response:

DomainForge owns the semantic source, canonical IR, projection contracts, and deterministic generation path.

The exact completeness of generated applications depends on current projectors, profiles, provider bindings, and target validation.

Describe the implemented proof rather than claiming universal application generation.

### “Will generated code be editable?”

Response:

Handwritten extension zones may be editable.

Generated zones should be changed by updating the source, projector, or provider configuration and regenerating.

Otherwise the generated artifact will become a competing source.

### “Is this a digital twin?”

Response:

A `.sea` model alone is not an organizational twin.

A twin requires:

* declared meaning;
* observed semantic evidence;
* reconciliation;
* history;
* continued revision.

DomainForge provides the semantic substrate and loop for that twin.

### “Will this replace our existing tools?”

Response:

No.

DomainForge should project into or import from existing ecosystems. The product’s role is to preserve shared meaning across them.

The specialist tools remain responsible for their native functions.

### “Does this make AI safe?”

Response:

No.

DomainForge gives AI systems stable domain meaning and constrained participation contracts.

Runtime authority, isolation, evidence, and settlement belong to SEA Forge or another governed execution layer.

### “Is organizational source code too rigid?”

Response:

Unversioned ambiguity is not flexibility.

DomainForge should make change explicit through namespaces, concept versions, mappings, migrations, and reviewed semantic diffs.

The model can evolve. The point is to make the evolution visible.

### “Is the model a surveillance system?”

Response:

It can become one if operational evidence is captured without clear governance.

Semantic capture must have:

* declared purpose;
* scoped evidence;
* access control;
* review;
* retention policy;
* consent and legal consideration where required;
* visibility into how observed behavior affects the model.

The framework should expose organizational behavior without turning every human action into indiscriminate telemetry.

---

## 26. Proof inventory

Marketing may point to these proof surfaces when they exist on the current branch.

### Source and parsing proof

* `.sea` grammar;
* parser;
* formatter;
* AST;
* semantic graph;
* validation diagnostics;
* namespaces;
* imports;
* exports;
* concept versioning.

### Semantic primitive proof

* entities;
* resources;
* flows;
* roles;
* relations;
* patterns;
* instances;
* dimensions;
* units;
* policies;
* metrics;
* mappings;
* projections;
* concept changes.

### Projection proof

* AST JSON;
* graph JSON;
* CALM;
* RDF and Turtle;
* SBVR;
* Protobuf and gRPC;
* DSL regeneration;
* other targets only when present and tested.

### Import proof

* supported import families;
* namespace resolution;
* export checks;
* dependency-cycle detection;
* normalized graph generation.

### Policy and measurement proof

* policy parsing and evaluation;
* traceable decisions;
* aggregation;
* units;
* conversions;
* metrics;
* three-valued logic where implemented.

### Cross-language proof

* Rust;
* Python;
* TypeScript;
* WASM or browser surfaces;
* conformance tests across bindings.

### Evidence-loop proof

When implemented:

* semantic event schema;
* canonical identity references;
* model version and hash references;
* observed graph construction;
* declared-versus-observed diff;
* proposed `.sea` patch;
* human acceptance or rejection of the patch.

### Application-generation proof

When implemented:

* profile selection;
* deterministic manifest;
* generated domain types;
* ports and adapters;
* contracts;
* tests;
* application scaffold;
* target-native validation;
* regeneration after a source change.

### Repository discipline proof

* golden fixtures;
* deterministic output comparisons;
* target-native validators in CI;
* migration tests;
* semantic diff tests;
* generated-zone checks;
* documentation linked to executable commands.

Only use proof that remains accurate in the current branch.

---

## 27. Claims and maturity boundaries

### Safe evidence-backed positioning

Use when confirmed by the current repository:

* DomainForge parses and validates `.sea`.
* DomainForge builds a normalized semantic graph.
* DomainForge assigns or preserves domain concept identities.
* DomainForge supports deterministic projection into implemented target formats.
* DomainForge supports implemented imports and bindings.
* DomainForge represents policies, metrics, units, mappings, and concept changes.
* DomainForge exposes implemented Rust, Python, TypeScript, and browser surfaces.
* DomainForge is Apache 2.0.
* `.sea` remains the semantic source while projections are derived forms.

### Claims requiring qualification

* organizational source code;
* organization compiler;
* complete application generation;
* digital twin;
* reverse semantic mining;
* runtime learning;
* self-updating model;
* universal interoperability;
* one source for the whole organization;
* production-ready agent hooks;
* governance enforcement;
* full architecture generation;
* proof generation;
* organizational simulation.

These may be valid strategic interpretations or active proof surfaces, but marketing must state the implemented boundary.

### Roadmap or strategic claims

Do not present these as complete without repository proof:

* complete general-purpose application generation;
* continuous ingestion from enterprise telemetry;
* automatic observed semantic graph construction;
* production organizational-twin dashboards;
* live semantic drift remediation;
* universal projection families;
* automatic model promotion from observed patterns;
* complete bidirectional synchronization with every target;
* fully governed agentic application generation;
* population-scale capability learning.

---

## 28. Licensing and commercial context

DomainForge is open source under Apache 2.0.

### Approved framing

> Organizational meaning becomes more valuable when more tools can speak it. DomainForge is Apache 2.0 so the semantic layer can spread.

### Strategic role

DomainForge is the broad adoption layer in the GodSpeed stack.

It should make it easy for:

* developers;
* agents;
* architecture tools;
* data systems;
* governance systems;
* application generators;
* enterprise platforms;
* researchers;
* third-party vendors

to read, write, validate, and project organizational meaning.

### Economic model

The semantic source should not be the toll booth.

Commercial value can exist above the permissive substrate through:

* SEA Forge governed execution;
* enterprise activation;
* managed deployments;
* evidence and integrity services;
* organizational-twin workbenches;
* advanced connectors;
* certified domain models;
* vertical packs;
* support;
* implementation services;
* training and certification;
* hosted collaboration;
* commercial application generators.

### Canonical line

> DomainForge spreads the semantic standard. SEA Forge monetizes governed operation.

Do not imply that all future DomainForge-adjacent products must be free merely because the core is Apache 2.0.

---

## 29. Adoption path

### Stage 1: Stabilize one bounded domain

Choose:

* one domain;
* a small concept set;
* one owner;
* one current semantic disagreement;
* one useful target projection.

Author and validate a `.sea` model.

### Stage 2: Prove projection value

Generate at least two useful forms, such as:

* Protobuf plus RDF;
* CALM plus documentation;
* graph JSON plus code contracts.

Validate each output in its native ecosystem.

### Stage 3: Introduce concept identity

Link code, schemas, policies, or documentation back to canonical semantic identities.

### Stage 4: Attach operational evidence

Emit one semantic event containing:

* model identity;
* model version or hash;
* actor;
* operation;
* resource;
* outcome;
* relevant policy or flow;
* evidence meaning.

### Stage 5: Detect one real difference

Compare one declared expectation with observed behavior.

Classify the difference as:

* conforming;
* violation;
* missing model concept;
* implementation drift;
* legitimate adaptation;
* unknown.

### Stage 6: Review one source revision

Generate a proposed `.sea` change.

Have the domain owner accept, reject, or modify it.

Rebuild affected projections.

### Stage 7: Generate an application or capability

Use `.sea` plus reviewed provider bindings to generate a bounded application or capability.

Keep generated artifacts derived.

### Stage 8: Expand into organizational source

Federate additional domains through namespaces, imports, shared contracts, and governed concept ownership.

Do not begin with the entire enterprise.

---

## 30. Primary use cases

### Cross-system domain contracts

Keep business concepts aligned across services, schemas, languages, and teams.

### Architecture as build input

Project organizational meaning into CALM, ArchiMate, knowledge graphs, contracts, and documentation.

### AI-native application generation

Let a domain expert describe a need, review the `.sea` source, bind providers, and generate an application from the approved meaning.

### Agent context and tool contracts

Give agents canonical concepts, valid relationships, policies, and participation surfaces.

### Semantic telemetry

Link logs, events, and traces to the declared concepts they instantiate.

### Organizational drift detection

Compare declared flows, policies, and structures with observed operation.

### Policy and metric portability

Represent policies, units, and metrics once and project them into compatible execution and reporting surfaces.

### Digital and organizational twins

Maintain declared meaning, observed semantic evidence, and reconciliation history.

### Standards interoperability

Translate one semantic source into target-native standards without making any target the universal source.

### Domain-led modernization

Preserve business meaning while replacing languages, platforms, databases, or architectures.

### Governed agentic systems

Supply SEA Forge with stable domain identities, policies, resources, flows, and projection contracts.

---

## 31. README-specific guidance

The README must not begin by teaching `.sea` grammar.

Its upper section should answer:

1. Why does organizational meaning need source code?
2. Where is that meaning currently scattered?
3. What does drift cost?
4. What does DomainForge make canonical?
5. What can the reader run immediately?
6. What proves that projections preserve meaning?
7. How can runtime evidence return to the source?

### Recommended upper-page sequence

1. problem-led organizational-source-code headline;
2. plain-language explanation;
3. repeated-translation failure pattern;
4. before-and-after architecture;
5. 60-second `.sea` example;
6. two or more target projections;
7. proof that the source remains canonical;
8. declared-versus-observed loop;
9. current capability and roadmap table;
10. deeper technical reference.

### Ten-second outcome

The reader should understand:

> DomainForge lets teams define domain meaning once and generate the forms each system needs.

### Sixty-second outcome

The reader should understand:

* the organization currently rewrites meaning across tools;
* `.sea` is the authored semantic source;
* DomainForge validates a canonical graph;
* target outputs are projections;
* code is not automatically canonical;
* runtime evidence can link back to the model;
* declared and observed reality remain distinct;
* the model can be revised and rebuilt.

### Primary developer action

> Write a small `.sea` model, validate it, and project the same meaning into two target ecosystems.

### Secondary actions

* inspect AST and graph outputs;
* run target-native validation;
* review available projections;
* add a semantic event;
* examine a declared-versus-observed diff;
* contribute a golden fixture or projector.

---

## 32. Content acceptance tests

Before publishing DomainForge copy, verify that the reader can answer:

1. What problem exists before DomainForge?
2. Why are documents, code, schemas, and diagrams insufficient as separate sources?
3. What does “organizational source code” mean?
4. Why is DomainForge more than a DSL?
5. What remains canonical?
6. What is a projection?
7. Why is generated code not the source?
8. What needs binding or activation?
9. What is declared reality?
10. What is observed reality?
11. What makes an organizational twin?
12. How does DomainForge help agents?
13. What does DomainForge not govern?
14. How does SEA Forge relate?
15. Which projections are implemented?
16. Which capabilities remain proof surfaces or roadmap?
17. How is DomainForge licensed?
18. What should the reader do next?

Apply five final tests.

### So what?

Does every technical capability connect to reduced translation, reduced drift, faster generation, clearer governance, or stronger traceability?

### Prove it

Can every implementation claim be supported by current repository evidence?

### Source integrity

Does the copy keep `.sea` canonical and projections derived?

### Reality boundary

Does the copy distinguish declared meaning from observed evidence?

### Compression

Can a technically competent reader explain DomainForge in one sentence after reading the opening?

---

## 33. Final strategic summary

DomainForge should no longer be framed primarily as a DSL.

That language describes how people author the source, but not why the product matters.

The immediate problem is semantic translation by hand:

```text
one business meaning
→ many documents, schemas, services, diagrams, policies, and prompts
→ independent changes
→ organizational drift
```

DomainForge changes the relationship:

```text
.sea organizational source
→ canonical semantic graph
→ deterministic projections
→ activated systems
→ observed semantic evidence
→ declared-versus-observed diff
→ reviewed source revision
```

Its smallest useful promise is:

> Define domain meaning once and project it into every system that needs a native form.

Its category claim is:

> DomainForge is the semantic build system for organizational source code.

Its deeper strategic role is:

> DomainForge gives an organization a versioned representation of what it means, then makes that representation accountable to what the organization actually does.

Lead with scattered meaning.

Show the source.

Prove the projections.

Close the loop with reality.
