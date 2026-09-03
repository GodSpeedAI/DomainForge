# Reference: Primitives API Reference

This document provides a technical API reference for the core domain primitives implemented in `domainforge-core/src/primitives/`.

---

## 1. `ConceptId`

Represents a deterministic, 128-bit content-addressed concept identifier.

- **File**: `domainforge-core/src/concept_id.rs`
- **Underlying Type**: `uuid::Uuid` (UUID v5)

### Constructors & Methods
```rust
impl ConceptId {
    /// Creates a deterministic ConceptId from namespace and concept name using DNS UUID v5.
    pub fn from_concept(namespace: &str, name: &str) -> Self;

    /// Wraps an existing UUID v4 or v5.
    pub fn from_uuid(uuid: Uuid) -> Self;

    /// Returns reference to internal Uuid.
    pub fn as_uuid(&self) -> &Uuid;
}
```

---

## 2. `Entity`

Represents an enterprise participant, service boundary, or aggregate root.

- **File**: `domainforge-core/src/primitives/entity.rs`

### Constructors & Methods
```rust
impl Entity {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn new_with_namespace(name: impl Into<String>, namespace: impl Into<String>) -> Self;

    pub fn id(&self) -> &ConceptId;
    pub fn name(&self) -> &str;
    pub fn namespace(&self) -> &str;
    pub fn contract(&self) -> Option<&EntityContract>;
    pub fn set_contract(&mut self, contract: EntityContract);
}
```

---

## 3. `Resource`

Represents an asset, token, material, currency, or payload that moves between entities.

- **File**: `domainforge-core/src/primitives/resource.rs`

### Constructors & Methods
```rust
impl Resource {
    pub fn new(name: impl Into<String>, unit: Unit) -> Self;
    pub fn new_with_namespace(name: impl Into<String>, unit: Unit, namespace: impl Into<String>) -> Self;

    pub fn id(&self) -> &ConceptId;
    pub fn name(&self) -> &str;
    pub fn unit(&self) -> &Unit;
    pub fn namespace(&self) -> &str;
}
```

---

## 4. `Flow`

Captures the directed transfer of a quantified resource from a source entity to a destination entity.

- **File**: `domainforge-core/src/primitives/flow.rs`

### Constructors & Methods
```rust
impl Flow {
    /// Note: Flow takes ConceptIds (IDs, not object references).
    pub fn new(
        resource_id: ConceptId,
        from_id: ConceptId,
        to_id: ConceptId,
        quantity: Decimal,
    ) -> Self;

    pub fn id(&self) -> &ConceptId;
    pub fn resource_id(&self) -> &ConceptId;
    pub fn from_id(&self) -> &ConceptId;
    pub fn to_id(&self) -> &ConceptId;
    pub fn quantity(&self) -> Decimal;
    pub fn namespace(&self) -> &str;
}
```

---

## 5. `Instance`

Represents a concrete instance of an entity containing field-value mappings.

- **File**: `domainforge-core/src/primitives/instance.rs`

### Constructors & Methods
```rust
impl Instance {
    pub fn new(id: ConceptId, entity_id: ConceptId, fields: HashMap<String, Value>) -> Self;

    pub fn id(&self) -> &ConceptId;
    pub fn entity_id(&self) -> &ConceptId;
    pub fn fields(&self) -> &HashMap<String, Value>;
    pub fn get_field(&self, key: &str) -> Option<&Value>;
}
```

---

## 6. `Policy`

Represents an executable constraint, derivation, or obligation rule.

- **File**: `domainforge-core/src/policy/core.rs`

### Constructors & Methods
```rust
impl Policy {
    pub fn new(name: impl Into<String>, expression: Expression) -> Self;
    pub fn evaluate(&self, graph: &Graph) -> Result<EvaluationResult, String>;
    pub fn evaluate_with_mode(&self, graph: &Graph, use_three_valued: bool) -> Result<EvaluationResult, String>;

    pub fn name(&self) -> &str;
    pub fn modality(&self) -> PolicyModality;
    pub fn kind(&self) -> PolicyKind;
    pub fn priority(&self) -> i32;
    pub fn rationale(&self) -> Option<&str>;
}
```

---

## Source Trail
- `domainforge-core/src/primitives/` — Implementations of domain primitives
- `domainforge-core/src/concept_id.rs` — `ConceptId` implementation
- `domainforge-core/src/policy/core.rs` — `Policy` struct implementation
