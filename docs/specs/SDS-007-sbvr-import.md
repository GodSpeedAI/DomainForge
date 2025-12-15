# SDS-007: SBVR Import

**System:** DomainForge  
**Component:** SBVR XMI Import  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Overview

The SBVR (Semantics of Business Vocabulary and Rules) import module enables DomainForge to:

- Import OMG SBVR XMI files into SEA graphs
- Convert SBVR fact types to SEA relations
- Transform SBVR rules to SEA policies
- Preserve business vocabulary semantics

---

## 2. SBVR Background

SBVR is an OMG standard for expressing business vocabularies and rules in natural language with formal semantics. Key concepts:

- **Noun Concepts**: Business terms (e.g., "Customer", "Order")
- **Verb Concepts**: Relationships between terms (e.g., "places", "contains")
- **Fact Types**: Structured propositions (e.g., "Customer places Order")
- **Business Rules**: Constraints and derivation rules

---

## 3. Architecture

```
┌────────────────────────────────────────────────────┐
│                 sbvr.rs Module                      │
├────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐               │
│  │  SbvrModel   │  │ NounConcept  │               │
│  └──────────────┘  └──────────────┘               │
│  ┌──────────────┐  ┌──────────────┐               │
│  │  VerbConcept │  │  FactType    │               │
│  └──────────────┘  └──────────────┘               │
│  ┌──────────────┐  ┌──────────────┐               │
│  │BusinessRule  │  │ Definition   │               │
│  └──────────────┘  └──────────────┘               │
├────────────────────────────────────────────────────┤
│  Parsing Methods                                   │
│  - from_xmi()                                      │
│  - to_graph()                                      │
└────────────────────────────────────────────────────┘
```

---

## 4. Core Types

### 4.1 SbvrModel

```rust
pub struct SbvrModel {
    pub name: String,
    pub noun_concepts: Vec<NounConcept>,
    pub verb_concepts: Vec<VerbConcept>,
    pub fact_types: Vec<FactType>,
    pub business_rules: Vec<BusinessRule>,
    pub definitions: Vec<Definition>,
}
```

### 4.2 NounConcept

```rust
pub struct NounConcept {
    pub id: String,
    pub name: String,
    pub definition: Option<String>,
    pub synonyms: Vec<String>,
    pub general_concept: Option<String>,  // Parent concept
}
```

### 4.3 VerbConcept

```rust
pub struct VerbConcept {
    pub id: String,
    pub name: String,      // e.g., "places"
    pub roles: Vec<Role>,  // Subject and object roles
}

pub struct Role {
    pub name: String,
    pub noun_concept_id: String,
    pub cardinality: Cardinality,
}

pub enum Cardinality {
    One,
    ZeroOrOne,
    OneOrMore,
    ZeroOrMore,
}
```

### 4.4 FactType

```rust
pub struct FactType {
    pub id: String,
    pub reading: String,           // e.g., "Customer places Order"
    pub verb_concept_id: String,
    pub role_bindings: Vec<(String, String)>,  // (role_id, concept_id)
}
```

### 4.5 BusinessRule

```rust
pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub statement: String,         // Natural language
    pub rule_type: RuleType,
    pub fact_type_refs: Vec<String>,
}

pub enum RuleType {
    StructuralRule,     // Cardinality, uniqueness
    DerivationRule,     // Computed facts
    OperativeRule,      // Business constraints
}
```

---

## 5. XMI Parsing

### 5.1 Parser Implementation

```rust
impl SbvrModel {
    pub fn from_xmi(xmi: &str) -> Result<Self, SbvrError> {
        let doc = roxmltree::Document::parse(xmi)
            .map_err(|e| SbvrError::Parse(e.to_string()))?;

        let mut model = SbvrModel::default();

        // Parse noun concepts
        for node in doc.descendants().filter(|n| n.has_tag_name("nounConcept")) {
            model.noun_concepts.push(NounConcept {
                id: node.attribute("xmi:id").unwrap_or_default().to_string(),
                name: node.attribute("name").unwrap_or_default().to_string(),
                definition: node.children()
                    .find(|n| n.has_tag_name("definition"))
                    .and_then(|n| n.text().map(String::from)),
                synonyms: Vec::new(),
                general_concept: node.attribute("generalConcept").map(String::from),
            });
        }

        // Parse verb concepts
        for node in doc.descendants().filter(|n| n.has_tag_name("verbConcept")) {
            // ...
        }

        // Parse fact types
        for node in doc.descendants().filter(|n| n.has_tag_name("factType")) {
            // ...
        }

        // Parse business rules
        for node in doc.descendants().filter(|n| n.has_tag_name("businessRule")) {
            // ...
        }

        Ok(model)
    }
}
```

---

## 6. Conversion to SEA Graph

### 6.1 Mapping Strategy

| SBVR Concept               | SEA Mapping                  |
| -------------------------- | ---------------------------- |
| Noun Concept               | Entity                       |
| Verb Concept               | Relation (predicate)         |
| Fact Type                  | Role-based Relation          |
| Business Rule (structural) | Policy                       |
| Business Rule (operative)  | Policy                       |
| Definition                 | Entity description attribute |

### 6.2 Conversion Implementation

```rust
impl SbvrModel {
    pub fn to_graph(&self) -> Result<Graph, SbvrError> {
        let mut graph = Graph::new();

        // Convert noun concepts to entities
        for noun in &self.noun_concepts {
            let mut entity = Entity::new_with_namespace(
                noun.name.clone(),
                "sbvr".to_string(),
            );

            if let Some(def) = &noun.definition {
                entity.set_attribute("description", serde_json::json!(def));
            }

            graph.add_entity(entity)?;
        }

        // Convert verb concepts to roles
        for verb in &self.verb_concepts {
            for role in &verb.roles {
                let sea_role = Role::new_with_namespace(
                    role.name.clone(),
                    "sbvr".to_string(),
                );
                graph.add_role(sea_role)?;
            }
        }

        // Convert fact types to relations
        for fact in &self.fact_types {
            if let Some(verb) = self.verb_concepts.iter().find(|v| v.id == fact.verb_concept_id) {
                if verb.roles.len() >= 2 {
                    let subject_role_id = graph.find_role_by_name(&verb.roles[0].name)
                        .ok_or(SbvrError::Conversion("Role not found".to_string()))?;
                    let object_role_id = graph.find_role_by_name(&verb.roles[1].name)
                        .ok_or(SbvrError::Conversion("Role not found".to_string()))?;

                    let relation = RelationType::new(
                        verb.name.clone(),
                        subject_role_id,
                        object_role_id,
                        verb.name.clone(),  // predicate
                    );
                    graph.add_relation_type(relation)?;
                }
            }
        }

        // Convert business rules to policies
        for rule in &self.business_rules {
            let policy = self.convert_rule_to_policy(rule)?;
            graph.add_policy(policy)?;
        }

        Ok(graph)
    }

    fn convert_rule_to_policy(&self, rule: &BusinessRule) -> Result<Policy, SbvrError> {
        // Convert SBVR statement to SEA expression
        // This is a simplified mapping; full SBVR requires more complex parsing
        let expression = self.parse_sbvr_statement(&rule.statement)?;

        Ok(Policy {
            id: ConceptId::new("sbvr", &rule.name),
            name: rule.name.clone(),
            expression,
            severity: match rule.rule_type {
                RuleType::StructuralRule => Severity::Error,
                RuleType::OperativeRule => Severity::Warning,
                RuleType::DerivationRule => Severity::Info,
            },
            description: Some(rule.statement.clone()),
        })
    }
}
```

---

## 7. SBVR Statement Parsing

### 7.1 Supported Patterns

| SBVR Pattern               | SEA Expression             |
| -------------------------- | -------------------------- |
| "Each X must Y"            | `forall x in X: Y(x)`      |
| "At most one X per Y"      | Cardinality check          |
| "It is necessary that..."  | Policy with error severity |
| "It is permitted that..."  | No policy (allowed)        |
| "It is obligatory that..." | Policy with error severity |

### 7.2 Expression Conversion

```rust
fn parse_sbvr_statement(&self, statement: &str) -> Result<Expression, SbvrError> {
    let normalized = statement.to_lowercase();

    // Pattern: "Each X must have at least one Y"
    if normalized.contains("each") && normalized.contains("must have") {
        // Extract subject and object
        // Build forall expression
        // ...
    }

    // Pattern: "It is necessary that..."
    if normalized.starts_with("it is necessary that") {
        // Extract constraint
        // Build expression
        // ...
    }

    // Default: store as raw statement for manual review
    Ok(Expression::Literal(Value::String(statement.to_string())))
}
```

---

## 8. CLI Integration

```bash
# Import SBVR XMI file
sea import --from sbvr vocabulary.xmi > model.sea

# Import with namespace override
sea import --from sbvr --namespace my-domain vocabulary.xmi
```

---

## 9. Limitations

| Limitation           | Workaround                        |
| -------------------- | --------------------------------- |
| Complex quantifiers  | Manual policy authoring           |
| Computed derivations | Not supported (logged as warning) |
| Temporal rules       | Limited support via ConceptChange |
| Modal logic          | Mapped to severity levels         |

---

## Related Documents

- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [SDS-004: Policy Engine Design](./SDS-004-policy-engine-design.md)
- [PRD-003: DSL Core Capabilities](./PRD-003-dsl-core-capabilities.md)
