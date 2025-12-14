# SDS-001: Protobuf Projection Engine

**System:** SEA Projection Framework  
**Component:** Protobuf Projection Engine  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Draft

---

## 1. High-Level Architecture

```
SEA DSL
  ↓
Parser
  ↓
Semantic Graph
  ↓
Projection Resolver
  ↓
Projection Engine (Protobuf)
  ↓
.proto Files
```

### Data Flow

1. **SEA DSL Source** → Parsed into AST
2. **AST** → Built into Semantic Graph
3. **Projection Declarations** → Resolved into `ProjectionSpec`
4. **Semantic Graph + ProjectionSpec** → Filtered and transformed
5. **Protobuf IR** → Written to `.proto` files

---

## 2. Core Modules

### 2.1 Projection Resolver

**Location:** `sea-core/src/projection/resolver.rs`

**Responsibilities:**

- Parse `Projection` declarations from AST
- Resolve scopes and filters against semantic graph
- Produce validated `ProjectionSpec`

```rust
pub struct ProjectionSpec {
    pub name: String,
    pub format: ProjectionFormat,
    pub filters: ProjectionFilters,
    pub metadata: ProjectionMetadata,
}

pub struct ProjectionFilters {
    pub namespaces: Vec<String>,
    pub kinds: Vec<SeaKind>,
    pub tags: Vec<String>,
    pub profiles: Vec<String>,
}

pub struct ProjectionMetadata {
    pub stability: Stability,
    pub compatibility: Compatibility,
    pub audience: Vec<String>,
    pub version: SemanticVersion,
}

pub enum Compatibility {
    Additive,
    Backward,
    Breaking,
}

pub enum Stability {
    Experimental,
    Stable,
    Deprecated,
}
```

### 2.2 Protobuf Projection Engine

**Location:** `sea-core/src/projection/protobuf/engine.rs`

**Responsibilities:**

- Walk semantic graph using `ProjectionSpec`
- Build intermediate Protobuf IR
- Enforce compatibility rules
- Emit `.proto` files

```rust
pub trait ProjectionEngine {
    fn project(&self, graph: &SemanticGraph, spec: &ProjectionSpec) -> Result<ProjectionOutput, ProjectionError>;
}

pub struct ProtobufEngine {
    type_mapper: TypeMapper,
    compatibility_checker: CompatibilityChecker,
}

impl ProjectionEngine for ProtobufEngine {
    fn project(&self, graph: &SemanticGraph, spec: &ProjectionSpec) -> Result<ProjectionOutput, ProjectionError> {
        // 1. Filter graph nodes
        // 2. Build Protobuf IR
        // 3. Check compatibility
        // 4. Generate output
    }
}
```

---

## 3. Protobuf Intermediate Representation (IR)

**Location:** `sea-core/src/projection/protobuf/ir.rs`

```rust
pub struct ProtoFile {
    pub package: String,
    pub syntax: String,  // "proto3"
    pub imports: Vec<String>,
    pub options: ProtoOptions,
    pub enums: Vec<ProtoEnum>,
    pub messages: Vec<ProtoMessage>,
    pub metadata: ProtoMetadata,
}

pub struct ProtoMessage {
    pub name: String,
    pub fields: Vec<ProtoField>,
    pub nested_messages: Vec<ProtoMessage>,
    pub nested_enums: Vec<ProtoEnum>,
    pub reserved_fields: Vec<u32>,
    pub reserved_names: Vec<String>,
    pub comments: Vec<String>,
}

pub struct ProtoField {
    pub name: String,
    pub number: u32,
    pub proto_type: ProtoType,
    pub label: FieldLabel,  // optional, repeated, required
    pub comments: Vec<String>,
}

pub struct ProtoEnum {
    pub name: String,
    pub values: Vec<ProtoEnumValue>,
    pub comments: Vec<String>,
}

pub struct ProtoEnumValue {
    pub name: String,
    pub number: i32,
}

pub struct ProtoOptions {
    pub java_package: Option<String>,
    pub java_multiple_files: bool,
    pub go_package: Option<String>,
}

pub struct ProtoMetadata {
    pub projection_name: String,
    pub semantic_version: String,
    pub source_namespace: String,
    pub generated_at: String,
}
```

---

## 4. Mapping Rules

### 4.1 SEA Entities / Resources → Protobuf Messages

| SEA Construct | Protobuf Construct |
| ------------- | ------------------ |
| `Entity`      | `message`          |
| `Resource`    | `message`          |
| `ValueObject` | `message`          |
| Attributes    | Fields             |
| Relations     | Field references   |

**Field Ordering:**

- Fields are ordered deterministically by name (alphabetical)
- Field numbers are assigned sequentially starting at 1
- Once assigned, field numbers are immutable

### 4.2 SEA Types → Protobuf Types

| SEA Type      | Protobuf Type               |
| ------------- | --------------------------- |
| `String`      | `string`                    |
| `Int`         | `int64`                     |
| `Float`       | `double`                    |
| `Bool`        | `bool`                      |
| `Date`        | `google.protobuf.Timestamp` |
| `DateTime`    | `google.protobuf.Timestamp` |
| `Money`       | Custom `Money` message      |
| `UUID`        | `string`                    |
| `List<T>`     | `repeated T`                |
| `Optional<T>` | `optional T`                |
| Custom Entity | Reference message           |

### 4.3 SEA Enums → Protobuf Enums

- First value is always `UNSPECIFIED = 0`
- Subsequent values numbered sequentially
- Names converted to SCREAMING_SNAKE_CASE

**Example:**

```sea
enum PaymentStatus {
  Pending
  Completed
  Failed
}
```

Becomes:

```proto
enum PaymentStatus {
  PAYMENT_STATUS_UNSPECIFIED = 0;
  PAYMENT_STATUS_PENDING = 1;
  PAYMENT_STATUS_COMPLETED = 2;
  PAYMENT_STATUS_FAILED = 3;
}
```

### 4.4 Flows / Policies / Metrics

Emit standardized governance and telemetry messages:

```proto
message GovernanceEvent {
  string event_id = 1;
  string flow_name = 2;
  string policy_name = 3;
  google.protobuf.Timestamp timestamp = 4;
  map<string, string> context = 5;
}

message MetricEvent {
  string metric_name = 1;
  double value = 2;
  google.protobuf.Timestamp timestamp = 3;
  map<string, string> dimensions = 4;
}
```

---

## 5. Compatibility Enforcement Algorithm

**Location:** `sea-core/src/projection/protobuf/compatibility.rs`

```rust
pub struct CompatibilityChecker {
    history_store: Box<dyn SchemaHistoryStore>,
}

impl CompatibilityChecker {
    pub fn check(
        &self,
        new_schema: &ProtoFile,
        spec: &ProjectionSpec,
    ) -> Result<(), CompatibilityViolation> {
        // 1. Load previous schema (if available)
        let previous = self.history_store.load(&spec.name)?;

        if let Some(prev) = previous {
            // 2. Diff old vs new
            let diff = self.diff(&prev, new_schema);

            // 3. Apply compatibility rules
            match spec.metadata.compatibility {
                Compatibility::Additive => self.check_additive(&diff)?,
                Compatibility::Backward => self.check_backward(&diff)?,
                Compatibility::Breaking => self.check_breaking(&diff)?,
            }
        }

        // 4. Store new schema for future checks
        self.history_store.save(&spec.name, new_schema)?;

        Ok(())
    }
}
```

### Compatibility Rules Matrix

| Change Type         | Additive | Backward      | Breaking |
| ------------------- | -------- | ------------- | -------- |
| Add field           | ✅       | ✅            | ✅       |
| Remove field        | ❌       | ✅ (reserved) | ✅       |
| Rename field        | ❌       | ❌            | ✅       |
| Change field type   | ❌       | ❌            | ✅       |
| Change field number | ❌       | ❌            | ✅       |
| Add enum value      | ✅       | ✅            | ✅       |
| Remove enum value   | ❌       | ✅ (reserved) | ✅       |

---

## 6. Output Structure

```
/generated
  /protobuf
    /com
      /example
        /finance
          payments.proto
          risk.proto
        /hr
          employees.proto
```

### File Header Template

```proto
// =============================================================================
// Generated by SEA Projection Framework
//
// Projection: PaymentsProto
// Semantic Version: 1.3.0
// Source Namespace: com.example.finance.payments
// Generated At: 2025-12-14T15:47:05Z
//
// DO NOT EDIT - This file is generated from SEA-DSL source
// =============================================================================

syntax = "proto3";

package com.example.finance.payments;

option java_package = "com.example.finance.payments";
option java_multiple_files = true;
option go_package = "github.com/example/finance/payments";
```

---

## 7. Extension Points

### 7.1 New Projection Formats

Implement the `ProjectionEngine` trait:

```rust
pub struct GraphQLEngine;
pub struct AvroEngine;
pub struct OpenAPIEngine;

impl ProjectionEngine for GraphQLEngine { /* ... */ }
```

### 7.2 Service/RPC Projections

Future support for gRPC service definitions:

```rust
pub struct ProtoService {
    pub name: String,
    pub methods: Vec<ProtoMethod>,
}

pub struct ProtoMethod {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    pub streaming: StreamingMode,
}
```

### 7.3 Semantic Diff Tooling

```rust
pub trait SchemaDiffTool {
    fn diff(&self, old: &ProtoFile, new: &ProtoFile) -> SchemaDiff;
    fn report(&self, diff: &SchemaDiff) -> String;
}
```

---

## 8. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Invalid projection: {0}")]
    InvalidProjection(String),

    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),

    #[error("Compatibility violation: {0}")]
    CompatibilityViolation(CompatibilityViolation),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct CompatibilityViolation {
    pub rule: Compatibility,
    pub message_name: String,
    pub field_name: String,
    pub violation_type: ViolationType,
}
```

---

## 9. Testing Strategy

### Unit Tests

- Type mapping correctness
- Field numbering determinism
- Compatibility rule enforcement

### Integration Tests

- End-to-end projection from SEA → Protobuf
- Round-trip parsing of generated `.proto` files
- Multi-namespace projections

### Golden Tests

- Snapshot testing of generated output
- Ensure deterministic generation

---

## Related Documents

- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [ADR-002: Projection as First-Class Construct](./ADR-002-projection-first-class-construct.md)
- [ADR-003: Protobuf as Projection Target](./ADR-003-protobuf-projection-target.md)
- [ADR-004: Projection Compatibility Semantics](./ADR-004-projection-compatibility-semantics.md)
- [PRD-001: SEA Projection Framework](./PRD-001-sea-projection-framework.md)
