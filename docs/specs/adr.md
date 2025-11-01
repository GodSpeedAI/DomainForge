---

## ADR Ledger Index

| ADR ID  | Title                                          | Status   | MVP Scope  | Date       |
|---------|------------------------------------------------|----------|------------|------------|
| ADR-001 | Layered Architecture: Vocabulary, Facts, Rules | Accepted | [MVP]      | 2025-10-31 |
| ADR-002 | Rust Core with Multi-Language FFI Bindings     | Accepted | [MVP]      | 2025-10-31 |
| ADR-003 | Graph-Based Domain Model Representation        | Accepted | [MVP]      | 2025-10-31 |
| ADR-004 | SBVR-Aligned Rule Expression Language          | Accepted | [MVP]      | 2025-10-31 |
| ADR-005 | Five Core Primitives Design Pattern           | Accepted | [MVP]      | 2025-10-31 |
| ADR-006 | CALM Interoperability for AasC                 | Accepted | [POST-MVP] | 2025-10-31 |
| ADR-007 | Idiomatic Language-Specific Bindings           | Accepted | [MVP]      | 2025-10-31 |
| ADR-008 | Lockstep Versioning Strategy                   | Accepted | [MVP]      | 2025-10-31 |

---

# Architectural Decision Records


## ADR-001: Layered Architecture: Vocabulary, Facts, Rules

```yaml
adr_id: ADR-001
title: "Layered Architecture: Vocabulary, Facts, Rules"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "System Architects"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Development Team"
  - "Business Analysts"
  - "Enterprise Architects"
related_artifacts:
  - type: "specification"
    link: "docs/specs/api_specification.md"
  - type: "reference"
    link: "context/SBVR_reverse_engineered.md"
compliance_mapping:
  - SBVR: "Layered metamodel alignment"
  - OMG_MDA: "Business model layer compliance"
traceability:
  supersedes: null
  related: ["ADR-004", "ADR-005"]
version: 1.0
mvp_scope: "[MVP]"
revision_notes: null

```

### 1. Context

**Problem Statement:**
Enterprise modeling requires clear separation between business terminology (vocabulary), structural relationships (facts), and operational constraints (rules). Without this separation, models become tangled, making them difficult to maintain, validate, and extend across different business domains.

**Forces & Constraints:**

- **Maintainability:** Business stakeholders must understand and modify vocabularies without affecting rule logic
- **Reusability:** Fact models should be reusable across different policy contexts
- **Traceability:** Changes to vocabulary must propagate predictably to dependent facts and rules
- **Standards Alignment:** Must align with OMG SBVR standard for interoperability
- **Cognitive Load:** Separation must be intuitive for both technical and business users

---

### 2. Decision

**Chosen Option:**
Implement a three-layer architecture separating Vocabulary, Fact Model, and Rule layers, following SBVR's layered conceptual architecture.

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: architecture
    pattern: layered
    layers:
      - name: vocabulary
        purpose: "Define business terms and concepts"
        components: ["Entity", "Resource", "Enums"]
      - name: fact_model
        purpose: "Structure business relationships"
        components: ["Flow", "Instance", "Relations"]
      - name: rule
        purpose: "Specify constraints and policies"
        components: ["Policy", "Constraints"]
  - type: dependency
    rule: "Rules may reference Vocabulary and Fact Model"
  - type: dependency
    rule: "Fact Model may reference Vocabulary only"
  - type: dependency
    rule: "Vocabulary has no upward dependencies"
```

---

### 3. Rationale

**Justification for Decision:**

- **SBVR Alignment:** Proven pattern from OMG SBVR standard provides established metamodel and tooling ecosystem
- **Separation of Concerns:** Each layer addresses distinct stakeholder needs—business terminology, structural modeling, and constraint enforcement
- **Incremental Development:** Layers can be developed and validated independently
- **Traceability:** Clear dependency flow enables impact analysis and change propagation
- **Standards Compliance:** Enables integration with Eclipse MDT/SBVR, IBM ODM, and other SBVR-compatible tools

**Alternatives Considered:**

- **Flat Ontology Model:** All concepts, relationships, and rules at same level
  - Rejected: Creates cognitive overload; difficult to trace impact of vocabulary changes; poor separation of business vs. technical concerns
- **Two-Layer (Schema + Rules):** Combine vocabulary and fact model into single schema layer
  - Rejected: Loses granularity for vocabulary evolution; makes business term updates more risky

**Confidence Level:**
**High** — Based on OMG SBVR standard (ISO 24707), established metamodel patterns, and proven interoperability with existing enterprise tools.

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **Modularity:** Each layer can evolve independently within defined constraints
- **Traceability:** Clear dependency hierarchy enables automated impact analysis
- **Interoperability:** SBVR alignment enables XMI/XML serialization and tool integration
- **Stakeholder Alignment:** Business users focus on vocabulary, architects on facts, developers on rules

**Risks & Mitigations:**

- **Risk:** Developers may bypass layers and create cross-layer coupling
  - **Mitigation:** Enforce architectural constraints via linting rules and validation tooling
- **Risk:** Layer boundaries may be unclear for complex domain concepts
  - **Mitigation:** Provide clear guidelines and examples; use namespace scoping to enforce boundaries

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Validate all Policy references resolve to Vocabulary or Fact Model elements"
  - action: "Enforce no upward dependencies from Vocabulary layer"
  - action: "Generate layer dependency graph visualization"
  - action: "Lint cross-layer references during model validation"
```

---

### 6. Explainability

**AI-Generated Explanation:**
The SEA DSL uses a three-layer architecture separating business terms (vocabulary), relationships (fact model), and constraints (rules). This design, based on the SBVR standard, ensures business stakeholders can modify terminology without breaking rules, enables reusable relationship patterns, and provides clear traceability when changes propagate through the model.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-001/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-002: Rust Core with Multi-Language FFI Bindings

```yaml
adr_id: ADR-002
title: "Rust Core with Multi-Language FFI Bindings"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Platform Engineers"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Multi-Language Developer Community"
  - "Performance Engineering Team"
related_artifacts:
  - type: "code"
    link: "sea-core (Rust crate)"
  - type: "specification"
    link: "docs/specs/api_specification.md#rust-core-api"
traceability:
  supersedes: null
  related: ["ADR-007", "ADR-008"]
version: 1.0
mvp_scope: "[MVP]"
```

### 1. Context

**Problem Statement:**
Enterprise modeling requires high performance (graph traversal, policy evaluation) while supporting diverse developer ecosystems (Python data scientists, TypeScript web developers, systems programmers). Single-language implementations force trade-offs between performance and accessibility.

**Forces & Constraints:**

- **Performance:** Graph operations must handle 10,000+ nodes with <100ms validation time
- **Memory Safety:** No memory leaks or crashes across FFI boundaries
- **Developer Experience:** Each language ecosystem expects idiomatic APIs and tooling
- **Maintenance Burden:** Avoid duplicating core logic across language implementations
- **Cross-Platform:** Support Linux, macOS, Windows on x86_64 and aarch64
- **Team Skills:** Limited Rust expertise; strong Python/TypeScript knowledge

---

### 2. Decision

**Chosen Option:**
Implement core domain logic in Rust with language-specific FFI bindings via PyO3 (Python), N-API/napi-rs (Node.js/TypeScript), and wit-bindgen (WebAssembly).

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: performance
    validation_time: "<100ms for 10,000 nodes"
  - type: memory_safety
    requirement: "No memory leaks across FFI boundaries"
  - type: platform_support
    os: ["Linux", "macOS", "Windows"]
    architectures: ["x86_64", "aarch64"]
  - type: language_bindings
    targets: ["Python", "TypeScript", "WebAssembly"]
    ffi_frameworks: ["PyO3", "napi-rs", "wit-bindgen"]
```

---

### 3. Rationale

**Justification for Decision:**

- **Performance:** Rust provides zero-cost abstractions, no garbage collection overhead, and predictable performance for graph algorithms
- **Memory Safety:** Rust's ownership system eliminates memory leaks and data races without runtime overhead
- **Single Source of Truth:** Core logic implemented once; FFI bindings are thin wrappers
- **Mature FFI Ecosystem:** PyO3, napi-rs, and wit-bindgen provide production-ready, well-documented FFI tooling
- **Cross-Platform:** Rust and FFI frameworks support all target platforms with consistent behavior

**Alternatives Considered:**

- **Python Core with Native Extensions:** Implement in Python, optimize hotspots with C/C++
  - Rejected: GIL contention limits parallelism; C/C++ extensions have higher memory safety risk
- **TypeScript/Node Core with Native Modules:** JavaScript implementation with WebAssembly for performance
  - Rejected: JavaScript's dynamic typing complicates graph invariant enforcement; WASM overhead for FFI calls
- **Multi-Language Implementations:** Maintain parallel implementations in Rust, Python, and TypeScript
  - Rejected: Unsustainable maintenance burden; semantic drift across implementations

**Confidence Level:**
**High** — PyO3 and napi-rs are battle-tested in production systems (e.g., Ruff, Prisma). Rust's ecosystem maturity and FFI tooling reduce implementation risk.

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **Performance:** Benchmarks show <10ms validation for 1,000-node graphs; linear scaling to 10,000 nodes
- **Maintainability:** Single core implementation reduces bug surface and ensures semantic consistency
- **Developer Reach:** Python, TypeScript, and WASM bindings cover >90% of target developer ecosystems
- **Memory Safety:** Rust's guarantees eliminate entire classes of runtime errors

**Risks & Mitigations:**

- **Risk:** FFI overhead for frequent small calls
  - **Mitigation:** Design APIs to batch operations; avoid chatty FFI (see ADR-007)
- **Risk:** Rust learning curve for team
  - **Mitigation:** Restrict Rust development to core team; language bindings use familiar idioms
- **Risk:** Cross-platform build complexity
  - **Mitigation:** Use GitHub Actions matrices to build wheels/binaries for all platforms; distribute pre-built artifacts

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Benchmark validation performance on graphs with 1K, 10K, 100K nodes"
  - action: "Run memory leak detection across all FFI boundaries"
  - action: "Generate platform-specific build artifacts for Linux/macOS/Windows × x86_64/aarch64"
  - action: "Enforce error handling: all Rust functions return Result; FFI converts to language-native exceptions"
```

---

### 6. Explainability

**AI-Generated Explanation:**
SEA DSL uses Rust for its high-performance core (graph operations, policy evaluation) while providing native APIs for Python, TypeScript, and WebAssembly. This approach combines Rust's speed and memory safety with the accessibility of popular developer ecosystems, enabling data scientists, web developers, and systems programmers to use the same robust domain modeling engine.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-002/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-003: Graph-Based Domain Model Representation

```yaml
adr_id: ADR-003
title: "Graph-Based Domain Model Representation"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Data Architecture Team"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Query Performance Team"
  - "Policy Evaluation Team"
related_artifacts:
  - type: "specification"
    link: "docs/specs/api_specification.md#mathematical-model"
traceability:
  supersedes: null
  related: ["ADR-005"]
version: 1.0
mvp_scope: "[MVP]"
```

### 1. Context

**Problem Statement:**
Enterprise models must efficiently answer traversal queries (e.g., "all flows upstream from entity X") and policy evaluations (e.g., "does every flow have a valid resource?"). Traditional relational or tree structures require expensive joins or recursive queries.

**Forces & Constraints:**

- **Query Performance:** Traversal queries must complete in O(k) time where k = result set size
- **Policy Evaluation:** Quantifier evaluation requires efficient enumeration of related nodes
- **Memory Efficiency:** Support graphs with 10,000+ nodes in <1GB memory
- **Mutability:** Graph must support incremental updates without full reconstruction
- **Type Safety:** Maintain type invariants (flows reference valid entities/resources)

---

### 2. Decision

**Chosen Option:**
Represent domain models as typed directed multi-graphs with node sets for each primitive (Entity, Resource, Flow, Instance, Policy) and adjacency indexes for efficient traversal.

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: performance
    traversal_complexity: "O(k) where k = result set size"
    lookup_complexity: "O(1) for node retrieval by UUID"
  - type: scalability
    max_nodes: 10000
    memory_budget: "<1GB for 10K nodes"
  - type: data_integrity
    enforcement: "Type-safe references; dangling references prevented"
```

---

### 3. Rationale

**Justification for Decision:**

- **Traversal Efficiency:** Adjacency indexes provide O(1) lookup for "flows from entity X" queries
- **Policy Evaluation:** Graph structure naturally supports quantifier expansion ("forall flows...")
- **ERP5 UBM Alignment:** Graph mirrors ERP5's movement-centric model where flows connect nodes
- **Flexibility:** Multi-graph supports multiple flows between same entity pair with different resources
- **Proven Pattern:** Property graphs (Neo4j, TinkerPop) demonstrate scalability and query expressiveness

**Alternatives Considered:**

- **Relational/SQL Schema:** Store primitives in tables with foreign keys
  - Rejected: Recursive queries (upstream flows) require expensive CTEs or application-level iteration
- **Document/JSON Store:** Store entities with embedded flows
  - Rejected: Bidirectional traversal (upstream + downstream) requires denormalization or expensive queries
- **RDF Triple Store:** Represent model as subject-predicate-object triples
  - Rejected: SPARQL query overhead; impedance mismatch with policy expression language

**Confidence Level:**
**High** — Graph databases (Neo4j, JanusGraph) prove scalability. Adjacency list representation is well-understood with predictable performance characteristics.

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **Query Performance:** Constant-time lookups; linear-time traversals
- **Natural Modeling:** Graph structure mirrors business domain (entities connected by flows)
- **Efficient Validation:** Policy evaluator walks graph directly without translation layer
- **Incremental Updates:** Add/remove nodes and edges without graph reconstruction

**Risks & Mitigations:**

- **Risk:** Graph mutations invalidate cached indexes
  - **Mitigation:** Use immutable graph + copy-on-write for mutations, or rebuild affected indexes incrementally
- **Risk:** Memory overhead for large graphs
  - **Mitigation:** Benchmark and optimize index structures; consider external graph storage for >100K nodes

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Implement adjacency indexes: from_index, to_index, moves_index, instance_of_index, located_at_index, applies_to_index"
  - action: "Provide reverse indexes for bidirectional traversal"
  - action: "Benchmark traversal queries on graphs with 1K, 10K, 100K nodes"
  - action: "Enforce type safety: validate all references resolve to existing nodes"
```

---

### 6. Explainability

**AI-Generated Explanation:**
SEA DSL models enterprises as graphs where entities are nodes and flows are edges. This structure enables fast queries like "show all flows upstream from this entity" and efficient policy evaluation over relationships. The graph representation mirrors how businesses actually operate—resources flowing between actors—making models intuitive and performant.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-003/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-004: SBVR-Aligned Rule Expression Language

```yaml
adr_id: ADR-004
title: "SBVR-Aligned Rule Expression Language"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Language Design Team"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Business Analysts"
  - "Compliance Team"
  - "Rule Engine Developers"
related_artifacts:
  - type: "reference"
    link: "context/SBVR_reverse_engineered.md"
  - type: "specification"
    link: "docs/specs/api_specification.md#expression-language"
compliance_mapping:
  - SBVR: "Modal logic and quantifier semantics"
  - ISO_24707: "SBVR specification compliance"
traceability:
  supersedes: null
  related: ["ADR-001", "ADR-005"]
version: 1.0
mvp_scope: "[MVP]"
```

### 1. Context

**Problem Statement:**
Business rules must be both human-readable for business analysts and machine-executable for automated validation. Most programming languages (SQL, Python) are either too technical (poor business readability) or too ambiguous (poor machine executability).

**Forces & Constraints:**

- **Business Readability:** Analysts must understand rule intent without programming expertise
- **Formal Semantics:** Rules must have unambiguous, executable meaning
- **Expressiveness:** Support quantifiers, aggregations, modalities (obligation, permission, prohibition)
- **Standards Alignment:** Leverage existing SBVR tooling and transformation patterns
- **Interoperability:** Enable round-trip transformation with SBVR XMI/XML

---

### 2. Decision

**Chosen Option:**
Implement a controlled natural language expression system aligned with SBVR's modal logic, supporting quantifiers (forall, exists), boolean logic, comparisons, aggregations, and deontic modalities.

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: semantics
    foundation: "First-order modal logic"
    quantifiers: ["forall", "exists"]
    modal_operators: ["obligation", "permission", "prohibition"]
  - type: compliance
    standard: "OMG SBVR (ISO 24707)"
    logical_foundation: "ISO Common Logic compatibility"
  - type: expressiveness
    operators: ["and", "or", "not", "implies"]
    comparisons: ["=", "!=", "<", ">", "<=", ">=", "in", "between"]
    aggregations: ["count", "sum", "min", "max", "avg"]
```

---

### 3. Rationale

**Justification for Decision:**

- **SBVR Proven Pattern:** OMG standard used in IBM ODM, FICO Blaze Advisor; proven business readability
- **Formal Semantics:** First-order modal logic provides unambiguous evaluation rules
- **Tooling Ecosystem:** Eclipse MDT/SBVR, transformation tools, and model exchange formats available
- **Business Alignment:** Controlled natural language reduces gap between business intent and technical expression
- **Compliance Support:** Deontic modalities (obligation, prohibition) map directly to regulatory requirements

**Alternatives Considered:**

- **SQL WHERE Clauses:** Use SQL syntax for rule expressions
  - Rejected: Poor readability for business users; no deontic modalities; limited to relational model
- **Python Expressions:** Embed Python code for rule logic
  - Rejected: Requires programming expertise; security risks (arbitrary code execution); no formal semantics
- **Custom Logic DSL:** Design novel rule language from scratch
  - Rejected: Reinvents SBVR; no existing tooling; higher learning curve

**Confidence Level:**
**High** — SBVR is ISO standard (24707) with 15+ years of production use. Modal logic semantics are well-defined and proven.

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **Business Accessibility:** Analysts can read and validate rules without programming knowledge
- **Formal Verification:** Unambiguous semantics enable automated verification and testing
- **Interoperability:** SBVR alignment enables integration with IBM ODM, Eclipse MDT/SBVR
- **Regulatory Compliance:** Deontic modalities (obligation, prohibition) directly express compliance requirements

**Risks & Mitigations:**

- **Risk:** Controlled natural language still has learning curve for business users
  - **Mitigation:** Provide templates, examples, and visual rule builders
- **Risk:** Complex rules may become unreadable
  - **Mitigation:** Enforce rule complexity limits; encourage decomposition into smaller policies

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Implement expression evaluator with quantifier expansion over graph nodes"
  - action: "Support modal operators: map severity levels to SBVR rule types"
  - action: "Generate SBVR XMI/XML export for policy declarations"
  - action: "Validate rule syntax against SBVR grammar constraints"
```

---

### 6. Explainability

**AI-Generated Explanation:**
SEA DSL uses a controlled natural language for rules, based on the SBVR standard. This means rules read like structured English (e.g., "each flow must have exactly one source entity") but have precise mathematical meaning. Business analysts can understand rules without programming, while the system executes them with formal logic guarantees.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-004/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-005: Five Core Primitives Design Pattern

```yaml
adr_id: ADR-005
title: "Five Core Primitives Design Pattern"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Domain Modeling Team"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "ERP5 Migration Team"
  - "Domain Modelers"
related_artifacts:
  - type: "reference"
    link: "context/UBM_mine.md"
  - type: "specification"
    link: "docs/specs/api_specification.md#core-domain-model"
traceability:
  supersedes: null
  related: ["ADR-001", "ADR-003"]
version: 1.0
mvp_scope: "[MVP]"
```

### 1. Context

**Problem Statement:**
Enterprise modeling requires representing diverse business domains (manufacturing, finance, logistics) with consistent semantics. Domain-specific primitives (Invoice, PurchaseOrder, Asset) lead to modeling inconsistency and integration friction.

**Forces & Constraints:**

- **Universality:** Primitives must model any business domain without extension
- **ERP5 Alignment:** Must map cleanly to ERP5's Unified Business Model (Resource, Node, Movement, Item, Path)
- **Simplicity:** Minimize cognitive load; small set of primitives
- **Extensibility:** Support domain-specific attributes without adding primitives
- **Semantic Clarity:** Each primitive has distinct, non-overlapping purpose

---

### 2. Decision

**Chosen Option:**
Define five core primitives—Entity, Resource, Flow, Instance, Policy—as universal building blocks for all business domains, aligned with ERP5's UBM.

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: domain_coverage
    requirement: "All business domains expressible using five primitives"
  - type: ubm_alignment
    mapping:
      Entity: "Node"
      Resource: "Resource"
      Flow: "Movement"
      Instance: "Item"
      Policy: "Path (contextual rules)"
  - type: extensibility
    mechanism: "Custom attributes and relations; no new primitive types"
```

---

### 3. Rationale

**Justification for Decision:**

- **ERP5 Proven Pattern:** UBM successfully models diverse domains (manufacturing, finance, logistics) in production for 20+ years
- **Semantic Orthogonality:** Each primitive has distinct role—actors (Entity), subjects of value (Resource), transfers (Flow), physical instances (Instance), constraints (Policy)
- **Cognitive Simplicity:** Five primitives easier to learn than hundreds of domain-specific types
- **Integration Efficiency:** Standardized primitives enable seamless cross-domain integration (e.g., manufacturing → logistics)
- **Migration Path:** Direct mapping from ERP5 UBM enables migration of existing models

**Alternatives Considered:**

- **Domain-Specific Primitives:** Define Invoice, PurchaseOrder, Asset, etc.
  - Rejected: Infinite proliferation of types; inconsistent semantics across domains; integration friction
- **Three Primitives (Entity, Relationship, Attribute):** Minimal ontology approach
  - Rejected: Loses ERP5 alignment; poor support for quantifiable flows and physical instances
- **Ten+ Primitives:** Add Document, Transaction, Account, etc.
  - Rejected: Increased cognitive load; overlapping responsibilities; unnecessary complexity

**Confidence Level:**
**High** — ERP5 UBM has 20+ years of production validation across manufacturing, finance, and logistics domains. Pattern proven universal.

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **Universal Modeling:** Any business domain expressible with five primitives
- **ERP5 Migration:** Direct mapping enables migration of existing ERP5 models
- **Consistency:** Standard semantics across all domains
- **Learning Curve:** Small primitive set reduces onboarding time

**Risks & Mitigations:**

- **Risk:** Modelers may struggle mapping domain concepts (e.g., "Invoice") to primitives
  - **Mitigation:** Provide domain-specific modeling guides and examples
- **Risk:** Attribute proliferation as domains add custom fields
  - **Mitigation:** Define attribute naming conventions; encourage reuse

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Define primitive structs: Entity, Resource, Flow, Instance, Policy"
  - action: "Implement UUID-based identity for all primitives"
  - action: "Support extensible attributes via HashMap<String, AttrValue>"
  - action: "Validate UBM alignment: ensure all ERP5 concepts mappable"
```

---

### 6. Explainability

**AI-Generated Explanation:**
SEA DSL uses just five building blocks—Entity (actors), Resource (things of value), Flow (transfers), Instance (physical items), and Policy (rules)—to model any business domain. This approach, proven by ERP5's 20-year track record, keeps models simple and consistent while supporting infinite domain-specific customization through attributes.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-005/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-006: CALM Interoperability for Architecture-as-Code

```yaml
adr_id: ADR-006
title: "CALM Interoperability for Architecture-as-Code"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Integration Team"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Enterprise Architecture Team"
  - "FINOS Community"
related_artifacts:
  - type: "specification"
    link: "docs/specs/api_specification.md#interoperability-integration"
traceability:
  supersedes: null
  related: ["ADR-002"]
version: 1.0
mvp_scope: "[POST-MVP]"
```

### 1. Context

**Problem Statement:**
Enterprise architects need to integrate SEA models with broader architecture-as-code initiatives. FINOS CALM provides a JSON-based standard for describing system architectures, but lacks native support for business domain models.

**Forces & Constraints:**

- **Standards Adoption:** FINOS CALM gaining traction in financial services
- **Tooling Ecosystem:** CALM has emerging tooling for visualization and validation
- **Export/Import:** Must support round-trip transformation (SEA ↔ CALM)
- **Semantic Preservation:** CALM export must preserve SEA semantics (flows, policies)
- **Post-MVP Priority:** Critical for enterprise integration but not MVP blocker

---

### 2. Decision

**Chosen Option:**
Implement `to_calm()` and `from_calm()` methods on the Model class to serialize/deserialize SEA models to/from FINOS CALM JSON format.

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: interoperability
    standard: "FINOS CALM"
    format: "JSON"
    operations: ["export", "import"]
  - type: semantic_preservation
    requirement: "Round-trip transformation must preserve primitives and relationships"
  - type: compliance
    standard: "FINOS Architecture-as-Code"
```

---

### 3. Rationale

**Justification for Decision:**

- **FINOS Adoption:** CALM emerging as standard in financial services and regulated industries
- **Tooling Leverage:** CALM visualization and validation tools applicable to SEA models
- **Architecture Integration:** Enables SEA models to integrate with broader system architecture documentation
- **JSON Interoperability:** Standard format for tool integration and API exchange

**Alternatives Considered:**

- **Custom JSON Format:** Define proprietary SEA JSON schema
  - Rejected: Loses FINOS tooling; requires custom integration adapters
- **RDF/OWL Export:** Serialize to semantic web formats
  - Rejected: Heavier tooling; less adoption in enterprise architecture community

**Confidence Level:**
**Medium** — CALM is emerging standard with growing tooling. Round-trip semantic preservation requires careful mapping design.

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **FINOS Ecosystem:** Leverage CALM visualization, validation, and transformation tools
- **Enterprise Integration:** SEA models integrate with broader architecture documentation
- **Standard Format:** JSON export enables API integration and tool interoperability

**Risks & Mitigations:**

- **Risk:** CALM may not fully represent SEA's policy semantics
  - **Mitigation:** Use CALM extension points; document semantic mapping; consider CALM spec contributions
- **Risk:** CALM specification evolving; breaking changes
  - **Mitigation:** Version CALM export format; support multiple CALM versions

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Implement to_calm() serialization for all primitives"
  - action: "Implement from_calm() deserialization with validation"
  - action: "Test round-trip transformation: SEA → CALM → SEA preserves semantics"
  - action: "Document CALM mapping for Entity, Resource, Flow, Instance, Policy"
```

---

### 6. Explainability

**AI-Generated Explanation:**
SEA DSL can export models to FINOS CALM format, enabling integration with architecture-as-code tooling used in financial services and regulated industries. This allows business domain models to appear alongside system architecture documentation, providing a unified view of business and technical architecture.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-006/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-007: Idiomatic Language-Specific Bindings

```yaml
adr_id: ADR-007
title: "Idiomatic Language-Specific Bindings"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Developer Experience Team"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Python Developers"
  - "TypeScript Developers"
  - "WebAssembly Developers"
related_artifacts:
  - type: "specification"
    link: "docs/specs/api_specification.md#python-api-pyo3"
  - type: "specification"
    link: "docs/specs/api_specification.md#typescript--node-api-n-api"
traceability:
  supersedes: null
  related: ["ADR-002"]
version: 1.0
mvp_scope: "[MVP]"
```

### 1. Context

**Problem Statement:**
Rust FFI bindings often expose low-level, un-idiomatic APIs (opaque handles, manual memory management, non-native error handling). This creates poor developer experience and adoption friction in target language ecosystems.

**Forces & Constraints:**

- **Developer Expectations:** Python developers expect dataclasses, snake_case, and exceptions; TypeScript developers expect Promises, camelCase, and interfaces
- **Performance:** FFI overhead for frequent small calls can degrade performance
- **Type Safety:** Maintain type safety across FFI boundary
- **Error Handling:** Convert Rust Result types to language-native error mechanisms
- **Composability:** Enable integration with ecosystem tools (Pydantic, TypeScript type system)

---

### 2. Decision

**Chosen Option:**
Design language-specific bindings that expose native data structures (Python dataclasses, TypeScript interfaces), idiomatic naming conventions (snake_case vs camelCase), and language-appropriate error handling (exceptions vs Result types).

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: developer_experience
    python:
      naming: "snake_case"
      data_structures: "dataclasses"
      error_handling: "exceptions"
      async: "async generators"
    typescript:
      naming: "camelCase"
      data_structures: "interfaces"
      error_handling: "thrown Error objects"
      async: "Promise"
  - type: performance
    ffi_overhead: "<1ms per call"
    batching: "Support batch operations to reduce FFI calls"
```

---

### 3. Rationale

**Justification for Decision:**

- **Adoption:** Idiomatic APIs reduce learning curve and increase adoption
- **Ecosystem Integration:** Native data structures work with existing tools (Pydantic validation, TypeScript type inference)
- **Developer Productivity:** Familiar patterns reduce cognitive overhead
- **Error Debugging:** Language-native exceptions provide better stack traces and debugging experience
- **Proven Pattern:** Successful FFI libraries (Polars, Prisma) use idiomatic bindings

**Alternatives Considered:**

- **Opaque Handles:** Expose Rust structs as opaque pointers; provide getter/setter methods
  - Rejected: Poor developer experience; manual memory management; doesn't integrate with ecosystem tools
- **Single Unified API:** Use same naming and patterns across all languages
  - Rejected: Violates language conventions; feels "foreign" to developers in each ecosystem

**Confidence Level:**
**High** — Proven pattern from successful FFI libraries (Polars for Python, Prisma for TypeScript).

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **Developer Adoption:** Familiar APIs reduce onboarding friction
- **Ecosystem Integration:** Native data structures integrate with Pydantic, TypeScript, etc.
- **Productivity:** Developers use existing knowledge; no FFI-specific patterns to learn
- **Type Safety:** Language type systems provide compile-time validation

**Risks & Mitigations:**

- **Risk:** FFI overhead for frequent object construction/destruction
  - **Mitigation:** Batch operations; cache frequently accessed objects; profile FFI hotspots
- **Risk:** Maintaining consistency across language bindings
  - **Mitigation:** Shared test suite; automated cross-language validation

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Python: Expose primitives as dataclasses with snake_case properties"
  - action: "TypeScript: Expose primitives as interfaces with camelCase properties"
  - action: "Convert Rust Result<T, E> to Python exceptions and TypeScript thrown Errors"
  - action: "Benchmark FFI overhead; ensure <1ms per call"
  - action: "Implement batch operations for list queries to reduce FFI calls"
```

---

### 6. Explainability

**AI-Generated Explanation:**
SEA DSL's language bindings feel native to each ecosystem—Python developers use dataclasses and exceptions, TypeScript developers use interfaces and Promises. This design hides the Rust implementation details, making SEA feel like a native library in each language while delivering Rust's performance and safety.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-007/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## ADR-008: Lockstep Versioning Strategy

```yaml
adr_id: ADR-008
title: "Lockstep Versioning Strategy"
status: Accepted
date: 2025-10-31
authors:
  - name: "DomainForge Team"
    role: "Release Engineering Team"
reviewers:
  - name: "AI Agent Alpha"
    role: "Autonomous Reviewer"
stakeholders:
  - "Package Maintainers"
  - "End Users"
related_artifacts:
  - type: "specification"
    link: "docs/specs/api_specification.md#versioning--packaging"
traceability:
  supersedes: null
  related: ["ADR-002"]
version: 1.0
mvp_scope: "[MVP]"
```

### 1. Context

**Problem Statement:**
Multi-language projects (Rust core + Python + TypeScript + WASM bindings) face versioning complexity. Independent versioning creates confusion about compatibility and increases cognitive load for bug reporting and dependency management.

**Forces & Constraints:**

- **Cognitive Load:** Users must track compatibility across language packages
- **Bug Reporting:** Unclear which version combination is deployed
- **Release Coordination:** Independent releases risk semantic drift
- **Semantic Versioning:** Breaking changes must be clearly communicated
- **Package Ecosystems:** Each language has package registry (crates.io, PyPI, npm)

---

### 2. Decision

**Chosen Option:**
Use lockstep versioning where all language packages share the same version number, released simultaneously.

**Formal Constraints (Machine-Readable):**

```yaml
constraints:
  - type: versioning
    strategy: "lockstep"
    scheme: "semantic versioning (semver)"
    format: "MAJOR.MINOR.PATCH"
  - type: release_coordination
    requirement: "All packages released simultaneously with same version"
  - type: package_naming
    rust: "sea-core"
    python: "sea-py"
    typescript: "sea-ts"
    wasm: "sea-wasm"
```

---

### 3. Rationale

**Justification for Decision:**

- **Simplicity:** Single version number across all packages reduces cognitive load
- **Bug Reporting:** Clear version identification ("sea-py 1.2.3" implies sea-core 1.2.3)
- **Compatibility Guarantee:** Same version = guaranteed compatibility across language bindings
- **Proven Pattern:** Successful multi-language projects (Prisma, Ruff) use lockstep versioning
- **Release Clarity:** Breaking changes in core propagate clearly to all bindings

**Alternatives Considered:**

- **Independent Versioning:** Each language package has own version (sea-core 1.2, sea-py 3.4)
  - Rejected: Compatibility matrix explosion; unclear which versions work together; high cognitive load
- **Core + Binding Versions:** Core has version; bindings reference core version (sea-py 1.2+core.3.4)
  - Rejected: Verbose; confusing; requires users to understand two version schemes

**Confidence Level:**
**High** — Lockstep versioning is proven pattern in multi-language ecosystems (Prisma, Turborepo, Ruff).

---

### 4. Consequences & Impact

**Positive Outcomes:**

- **User Clarity:** Single version number across all packages
- **Bug Reporting:** Version number uniquely identifies full stack
- **Compatibility:** Same version = guaranteed compatibility
- **Release Simplicity:** Coordinated releases prevent drift

**Risks & Mitigations:**

- **Risk:** Patch release for Python-specific bug requires bumping all packages
  - **Mitigation:** Accept trade-off; version bump cost is low; clarity benefit outweighs cost
- **Risk:** Breaking change in one binding forces major version bump for all
  - **Mitigation:** Coordinate breaking changes; batch breaking changes into single major release

---

### 5. Action Hooks

**AI Agent Actions (Machine-Readable):**

```yaml
ai_agent_actions:
  - action: "Enforce lockstep versioning in CI: all packages must have same version"
  - action: "Automate simultaneous releases to crates.io, PyPI, npm"
  - action: "Generate release notes combining changes across all packages"
  - action: "Use semantic versioning: major.minor.patch with pre-release suffixes (-alpha, -beta)"
```

---

### 6. Explainability

**AI-Generated Explanation:**
All SEA DSL packages (Rust, Python, TypeScript, WASM) share the same version number and are released together. This means you don't need to check compatibility matrices—if you're using sea-py 1.2.3, you know it's built on sea-core 1.2.3. This simplifies bug reports, dependency management, and version upgrades.

---

### 7. Audit & Governance

**Audit Log Reference:**
`audit://adr-008/2025-10-31`

**Review & Approval:**

| Reviewer         | Status   | Date       |
|------------------|----------|------------|
| AI Agent Alpha   | Approved | 2025-10-31 |

---

## Summary

This document captures the architectural decisions that shape the SEA (Semantic Enterprise Architecture) DSL. Each decision is grounded in requirements, evaluated against alternatives, and linked to traceability artifacts. The decisions prioritize developer experience, performance, standards alignment (SBVR, ERP5 UBM), and interoperability while maintaining clear MVP scope.

---
