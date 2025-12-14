# SDS-003: Parser and Semantic Graph

**System:** DomainForge  
**Component:** SEA Parser & Graph Builder  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Overview

This document describes the parsing pipeline that transforms SEA-DSL source text into a semantic graph. The pipeline consists of:

1. **Lexing/Parsing**: PEG-based parsing via `pest`
2. **AST Construction**: Structured representation of syntax
3. **Graph Building**: Semantic graph with validated references
4. **Linting**: Style and quality checks

---

## 2. Grammar (sea.pest)

SEA uses a Parsing Expression Grammar (PEG) defined in `sea.pest`.

### 2.1 Top-Level Declarations

```pest
program = { SOI ~ (declaration | COMMENT)* ~ EOI }

declaration = {
    entity_decl
  | resource_decl
  | flow_decl
  | instance_decl
  | role_decl
  | policy_decl
  | metric_decl
  | pattern_decl
  | concept_change_decl
  | mapping_decl
  | projection_decl
  | import_decl
  | profile_decl
}
```

### 2.2 Entity Declaration

```pest
entity_decl = {
    "Entity" ~ string ~ ("in" ~ namespace)?
    ~ ("{" ~ entity_body ~ "}")?
}

entity_body = {
    (attribute_decl | relation_decl)*
}

attribute_decl = {
    identifier ~ ":" ~ type_expr
}
```

### 2.3 Example Syntax

```sea
Entity "Customer" in crm {
    id: UUID
    name: String
    email: Email
    tier: CustomerTier
}

Resource "Order" units in commerce

Flow "Order" from "Customer" to "Fulfillment" quantity 1

Policy customer_has_name as: Customer.name != ""
```

---

## 3. AST Types

### 3.1 AST Root

```rust
pub struct Ast {
    pub nodes: Vec<AstNode>,
    pub imports: Vec<ImportSpec>,
    pub profile: Option<String>,
}
```

### 3.2 AST Node Variants

```rust
pub enum AstNode {
    Entity(EntityNode),
    Resource(ResourceNode),
    Flow(FlowNode),
    Instance(InstanceNode),
    Role(RoleNode),
    Policy(PolicyNode),
    Metric(MetricNode),
    Pattern(PatternNode),
    ConceptChange(ConceptChangeNode),
    Mapping(MappingNode),
    Projection(ProjectionNode),
    Relation(RelationNode),
}
```

### 3.3 Entity Node

```rust
pub struct EntityNode {
    pub name: String,
    pub namespace: Option<String>,
    pub attributes: Vec<AttributeNode>,
    pub relations: Vec<RelationNode>,
    pub tags: Vec<String>,
    pub span: SourceRange,
}
```

---

## 4. Parsing API

### 4.1 Basic Parsing

```rust
use sea_core::parser::{parse, parse_to_graph};

// Parse to AST only
let ast = parse(source)?;

// Parse to validated Graph
let graph = parse_to_graph(source)?;
```

### 4.2 Parsing with Options

```rust
use sea_core::parser::{parse_to_graph_with_options, ParseOptions};

let options = ParseOptions {
    default_namespace: Some("my.namespace".to_string()),
    namespace_registry: Some(registry),
    entry_path: Some(PathBuf::from("model.sea")),
    active_profile: Some("strict".to_string()),
    tolerate_profile_warnings: false,
};

let graph = parse_to_graph_with_options(source, &options)?;
```

### 4.3 ParseOptions

| Field                       | Type                        | Description                                     |
| --------------------------- | --------------------------- | ----------------------------------------------- |
| `default_namespace`         | `Option<String>`            | Fallback namespace for unqualified declarations |
| `namespace_registry`        | `Option<NamespaceRegistry>` | Registry for module resolution                  |
| `entry_path`                | `Option<PathBuf>`           | Path to entry file for relative imports         |
| `active_profile`            | `Option<String>`            | Profile to enforce                              |
| `tolerate_profile_warnings` | `bool`                      | Downgrade profile violations to warnings        |

---

## 5. Graph Construction

### 5.1 AST to Graph Pipeline

```rust
pub fn ast_to_graph_with_options(
    ast: Ast,
    options: &ParseOptions,
) -> ParseResult<Graph> {
    let mut graph = Graph::new();

    // Phase 1: Add all declarations (forward references OK)
    for node in &ast.nodes {
        match node {
            AstNode::Entity(e) => add_entity(&mut graph, e, options)?,
            AstNode::Resource(r) => add_resource(&mut graph, r, options)?,
            // ... other node types
        }
    }

    // Phase 2: Resolve references
    resolve_flow_references(&graph)?;
    resolve_relation_references(&graph)?;

    // Phase 3: Validate semantic constraints
    validate_graph(&graph)?;

    Ok(graph)
}
```

### 5.2 Reference Resolution

Flows and relations reference entities by name. Resolution is namespace-aware:

```rust
fn resolve_entity_reference(
    graph: &Graph,
    name: &str,
    current_namespace: &str,
) -> Result<ConceptId, ParseError> {
    // 1. Try same namespace first
    if let Some(id) = graph.find_entity_by_name_and_namespace(name, current_namespace) {
        return Ok(id);
    }

    // 2. Try global lookup
    if let Some(id) = graph.find_entity_by_name(name) {
        return Ok(id);
    }

    // 3. Generate helpful error with suggestions
    Err(ParseError::UndefinedEntity {
        name: name.to_string(),
        suggestions: fuzzy::did_you_mean(name, graph.all_entity_names()),
    })
}
```

---

## 6. Linting

### 6.1 Built-in Rules

| Rule                | Severity | Description                           |
| ------------------- | -------- | ------------------------------------- |
| `naming-convention` | Warning  | Entity names should be PascalCase     |
| `unused-entity`     | Warning  | Entity declared but never referenced  |
| `missing-namespace` | Info     | Declaration has no explicit namespace |
| `empty-entity`      | Warning  | Entity has no attributes              |

### 6.2 Lint API

```rust
use sea_core::parser::lint::{lint_ast, LintConfig};

let config = LintConfig::default();
let warnings = lint_ast(&ast, &config);

for warning in warnings {
    eprintln!("{}: {}", warning.range, warning.message);
}
```

---

## 7. Profiles

Profiles restrict allowed DSL constructs for specific use cases.

### 7.1 Built-in Profiles

| Profile    | Allowed Constructs     |
| ---------- | ---------------------- |
| `basic`    | Entity, Resource, Flow |
| `standard` | basic + Policy, Metric |
| `full`     | All constructs         |

### 7.2 Profile Declaration

```sea
profile strict

Entity "Account" in finance { /* ... */ }
```

### 7.3 Profile Enforcement

```rust
// Fails if source uses constructs outside profile
let options = ParseOptions {
    active_profile: Some("basic".to_string()),
    tolerate_profile_warnings: false,
    ..Default::default()
};
let result = parse_to_graph_with_options(source, &options);
```

---

## 8. Module Resolution

### 8.1 Import Syntax

```sea
import "common/types.sea"
import namespace core from "std/core.sea"
```

### 8.2 Namespace Registry

```rust
pub struct NamespaceRegistry {
    bindings: HashMap<String, PathBuf>,
}

impl NamespaceRegistry {
    pub fn from_file(path: &Path) -> Result<Self, RegistryError> {
        // Parse .sea-registry.toml
    }

    pub fn resolve(&self, namespace: &str) -> Option<&PathBuf> {
        self.bindings.get(namespace)
    }
}
```

### 8.3 Registry Configuration

```toml
# .sea-registry.toml
[namespaces]
core = "sea-core/std/core.sea"
http = "sea-core/std/http.sea"
aws = "sea-core/std/aws.sea"
```

---

## 9. Pretty Printer

Round-trip formatting from Graph back to SEA source:

```rust
use sea_core::parser::PrettyPrinter;

let printer = PrettyPrinter::new();
let formatted = printer.print(&graph);
```

---

## 10. Error Examples

### 10.1 Syntax Error

```
error[E1001]: unexpected token
  --> model.sea:5:12
   |
 5 |   Entity Customer {
   |          ^^^^^^^^ expected quoted string
   |
   = help: Entity names must be quoted: Entity "Customer"
```

### 10.2 Semantic Error

```
error[E2000]: undefined entity
  --> model.sea:10:25
   |
10 |   Flow "Order" from "Custmer" to "Warehouse"
   |                     ^^^^^^^^^ entity not found
   |
   = help: Did you mean "Customer"?
```

---

## Related Documents

- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [ADR-006: Error Handling Strategy](./ADR-006-error-handling-strategy.md)
