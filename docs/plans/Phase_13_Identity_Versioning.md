# 🧭 Phase 13: Stable Identity & Versioning

**Status:** Planned
**Priority:** P1 — High (Cross-Repo Stability)
**Created:** 2025-11-08
**Estimated Duration:** 10 days
**Complexity:** Medium

---

## 1. Objectives and Context

**Goal:** Implement stable, deterministic identifiers using UUIDv5 based on {namespace, name} and add semantic versioning to policies.

**Problem Statement:**
Current UUID generation uses random v4, creating different IDs for the same concept across environments. No version tracking on policies makes change management impossible.

**Scope:**

- ✅ ConceptId type (UUIDv5 from namespace + name)
- ✅ Policy versioning (semver: major.minor.patch)
- ✅ Namespace enforcement (required for stable IDs)
- ✅ ID migration tooling for existing models
- ❌ NO cross-version migration logic in MVP

**Dependencies:**

- **Prerequisite:** None (can run parallel with Phase 11/12)
- **Blocks:** None

---

## 2. Architecture & Design

### ConceptId System

```rust
use uuid::Uuid;
use sha1::{Sha1, Digest};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(Uuid);

impl ConceptId {
    /// Generate deterministic UUIDv5 from namespace + name
    pub fn from_concept(namespace: &str, name: &str) -> Self {
        let namespace_uuid = Uuid::NAMESPACE_DNS; // or custom namespace
        let data = format!("{}::{}", namespace, name);
        let uuid = Uuid::new_v5(&namespace_uuid, data.as_bytes());
        ConceptId(uuid)
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ConceptId(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}
```

### Policy Versioning

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: ConceptId,          // NEW: Stable identifier
    pub name: String,
    pub version: SemanticVersion, // NEW: Version tracking
    pub expression: Expression,
    pub modality: DeonticModality,
    pub namespace: String,      // NEW: Required for ConceptId
}
```

### Namespace Enforcement

```rust
impl Entity {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let id = ConceptId::from_concept(&namespace, &name);

        Self {
            id,
            name,
            namespace: Some(namespace),
            attributes: HashMap::new(),
        }
    }
}
```

---

## 3. TDD Cycles

### Cycle 13A — ConceptId Implementation

#### 13A — RED Phase

```rust
#[test]
fn test_concept_id_deterministic() {
    let id1 = ConceptId::from_concept("logistics", "Warehouse");
    let id2 = ConceptId::from_concept("logistics", "Warehouse");
    assert_eq!(id1, id2);
}

#[test]
fn test_concept_id_different_namespace() {
    let id1 = ConceptId::from_concept("logistics", "Camera");
    let id2 = ConceptId::from_concept("finance", "Camera");
    assert_ne!(id1, id2);
}

#[test]
fn test_concept_id_serialization() {
    let id = ConceptId::from_concept("test", "Entity");
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: ConceptId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}
```

#### 13A — GREEN Phase

Implement `sea-core/src/concept_id.rs` as shown above.

#### 13A — REFACTOR Phase

Add custom namespace UUID instead of NAMESPACE_DNS.

#### 13A — REGRESSION Phase

```bash
cargo test --test concept_id_tests
```

**Label:** → **13A-COMPLETE**

---

### Cycle 13B — Semantic Versioning

#### 13B — RED Phase

```rust
#[test]
fn test_semantic_version_parsing() {
    let v = SemanticVersion::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
}

#[test]
fn test_version_comparison() {
    let v1 = SemanticVersion::new(1, 0, 0);
    let v2 = SemanticVersion::new(2, 0, 0);
    assert!(v1 < v2);
}

#[test]
fn test_policy_with_version() {
    let policy = Policy::new(
        "Test Policy",
        Expression::literal(true),
    ).with_version(SemanticVersion::new(1, 0, 0));

    assert_eq!(policy.version.major, 1);
}
```

#### 13B — GREEN Phase

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err("Version must be in format major.minor.patch".to_string());
        }

        Ok(Self {
            major: parts[0].parse().map_err(|_| "Invalid major version")?,
            minor: parts[1].parse().map_err(|_| "Invalid minor version")?,
            patch: parts[2].parse().map_err(|_| "Invalid patch version")?,
        })
    }

    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    pub fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
```

**Label:** → **13B-COMPLETE**

---

### Cycle 13C — Primitive Integration

#### 13C — RED Phase

Update all primitives to use ConceptId and require namespace.

#### 13C — GREEN Phase

Migrate Entity, Resource, Flow, Instance to use ConceptId.

#### 13C — REFACTOR Phase

Provide migration helper: `ConceptId::from_legacy_uuid(old_uuid)` for backward compatibility.

#### 13C — REGRESSION Phase

Full test suite with ConceptId.

**Label:** → **13C-COMPLETE**

---

### Cycle 13D — Parser & DSL Updates

#### 13D — RED Phase

```rust
#[test]
fn test_parse_policy_with_version() {
    let source = r#"
        Policy min_quantity v1.0.0 as: forall f in flows: f.quantity > 10
    "#;

    let graph = parse_to_graph(source).unwrap();
    let policy = graph.policies().next().unwrap();
    assert_eq!(policy.version.to_string(), "1.0.0");
}
```

#### 13D — GREEN Phase

Update grammar:

```pest
policy_decl = {
    ^"policy" ~ identifier ~ (^"v" ~ version)? ~ ^"as" ~ ":" ~ expression
}

version = @{ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+ }
```

**Label:** → **13D-COMPLETE**

---

### Cycle 13E — FFI Bindings

Expose ConceptId and SemanticVersion in Python, TypeScript, WASM.

**Label:** → **13E-COMPLETE**

---

## 4. Success Criteria

- [ ] ConceptId generates deterministic UUIDs
- [ ] Same {namespace, name} pair always produces same ID
- [ ] Policy versioning supports semver operations
- [ ] Namespace required for all primitives
- [ ] Migration tooling for existing models
- [ ] CALM export preserves ConceptIds

---

## 5. Traceability

**Addresses:**

- Critique 3 (Namespaces/Versioning)
- SDS-001 (Namespace system)

**Updates Required:**

- All primitive constructors
- CALM export to include ConceptId metadata
- Documentation on stable ID usage

---

## 6. Sign-Off Checklist

- [ ] All cycles complete
- [ ] Documentation updated
- [ ] Migration guide provided
- [ ] Merged to main

**Completion Date:** _____________________
