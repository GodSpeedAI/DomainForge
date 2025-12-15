# Protobuf Projection API Reference

This reference documents the public API for the Protobuf projection engine in `sea-core`.

## Module Location

```
sea-core/src/projection/protobuf.rs
```

## Core Types

### ProtoFile

Represents a complete `.proto` file.

```rust
pub struct ProtoFile {
    pub package: String,           // Package name (e.g., "sea.example")
    pub syntax: String,            // Always "proto3"
    pub imports: Vec<String>,      // Import statements
    pub options: ProtoOptions,     // File-level options
    pub enums: Vec<ProtoEnum>,     // Enum definitions
    pub messages: Vec<ProtoMessage>, // Message definitions
    pub services: Vec<ProtoService>, // gRPC service definitions
    pub metadata: ProtoMetadata,   // Projection metadata
}
```

**Methods:**

| Method                        | Description                                   |
| ----------------------------- | --------------------------------------------- |
| `new(package: &str) -> Self`  | Create a new ProtoFile with the given package |
| `to_proto_string() -> String` | Serialize to `.proto` text format             |
| `add_wkt_imports(&mut self)`  | Auto-add Well-Known Type imports              |

### ProtoMessage

Represents a Protobuf message definition.

```rust
pub struct ProtoMessage {
    pub name: String,                      // Message name (PascalCase)
    pub fields: Vec<ProtoField>,           // Field definitions
    pub nested_messages: Vec<ProtoMessage>, // Nested messages
    pub nested_enums: Vec<ProtoEnum>,      // Nested enums
    pub reserved_numbers: Vec<u32>,        // Reserved field numbers
    pub reserved_names: Vec<String>,       // Reserved field names
    pub comments: Vec<String>,             // Documentation comments
}
```

**Methods:**

| Method                        | Description                       |
| ----------------------------- | --------------------------------- |
| `new(name: &str) -> Self`     | Create a new empty message        |
| `to_proto_string() -> String` | Serialize to `.proto` text format |

### ProtoField

Represents a Protobuf field definition.

```rust
pub struct ProtoField {
    pub name: String,          // Field name (snake_case)
    pub number: u32,           // Field number (unique within message)
    pub proto_type: ProtoType, // Field type
    pub repeated: bool,        // Whether this is a repeated field
    pub optional: bool,        // Whether this is optional (proto3)
    pub comments: Vec<String>, // Documentation comments
}
```

### ProtoType

Represents a Protobuf type reference.

```rust
pub enum ProtoType {
    Scalar(ScalarType),           // Scalar types (int32, string, etc.)
    Message(String),              // Reference to another message
    Enum(String),                 // Reference to an enum
    Map { key: Box<ProtoType>, value: Box<ProtoType> }, // Map type
}
```

### ScalarType

Protobuf scalar types.

```rust
pub enum ScalarType {
    Double, Float,
    Int32, Int64, Uint32, Uint64,
    Sint32, Sint64,
    Fixed32, Fixed64, Sfixed32, Sfixed64,
    Bool, String, Bytes,
}
```

### ProtoEnum

Represents a Protobuf enum definition.

```rust
pub struct ProtoEnum {
    pub name: String,                  // Enum name (PascalCase)
    pub values: Vec<ProtoEnumValue>,   // Enum values
    pub comments: Vec<String>,         // Documentation comments
}
```

**Methods:**

| Method                        | Description                                |
| ----------------------------- | ------------------------------------------ |
| `new(name: &str) -> Self`     | Create enum with default UNSPECIFIED value |
| `add_value(name: &str)`       | Add a new enum value                       |
| `to_proto_string() -> String` | Serialize to `.proto` text format          |

### ProtoService

Represents a gRPC service definition.

```rust
pub struct ProtoService {
    pub name: String,                  // Service name (e.g., "PaymentService")
    pub methods: Vec<ProtoRpcMethod>,  // RPC methods
    pub comments: Vec<String>,         // Documentation comments
}
```

### ProtoRpcMethod

Represents an RPC method in a gRPC service.

```rust
pub struct ProtoRpcMethod {
    pub name: String,           // Method name
    pub request_type: String,   // Request message type
    pub response_type: String,  // Response message type
    pub streaming: StreamingMode, // Streaming mode
    pub comments: Vec<String>,  // Documentation comments
}
```

### StreamingMode

gRPC streaming mode for RPC methods.

```rust
pub enum StreamingMode {
    Unary,              // Single request, single response (default)
    ServerStreaming,    // Single request, stream of responses
    ClientStreaming,    // Stream of requests, single response
    Bidirectional,      // Stream of requests, stream of responses
}
```

**Methods:**

| Method                   | Description                                            |
| ------------------------ | ------------------------------------------------------ |
| `parse(s: &str) -> Self` | Parse from string (e.g., "streaming", "bidirectional") |

### ProtoOptions

File-level Protobuf options.

```rust
pub struct ProtoOptions {
    pub java_package: Option<String>,
    pub java_multiple_files: bool,
    pub go_package: Option<String>,
    pub csharp_namespace: Option<String>,
    pub php_namespace: Option<String>,
    pub ruby_package: Option<String>,
    pub swift_prefix: Option<String>,
    pub objc_class_prefix: Option<String>,
    pub optimize_for: Option<String>,  // SPEED, CODE_SIZE, LITE_RUNTIME
    pub deprecated: bool,
    pub custom_options: Vec<ProtoCustomOption>,
}
```

### WellKnownType

Google's Protobuf Well-Known Types.

```rust
pub enum WellKnownType {
    Timestamp, Duration, Any, Struct, Value, ListValue,
    Int32Value, Int64Value, UInt32Value, UInt64Value,
    FloatValue, DoubleValue, BoolValue, StringValue, BytesValue,
    FieldMask, Empty,
}
```

**Methods:**

| Method                                       | Description                       |
| -------------------------------------------- | --------------------------------- |
| `import_path() -> &'static str`              | Get the import path for this WKT  |
| `full_name() -> &'static str`                | Get the fully qualified type name |
| `from_type_name(name: &str) -> Option<Self>` | Parse from type name              |

---

## ProtobufEngine

The main projection engine for converting SEA graphs to Protobuf.

### Basic Projection

```rust
impl ProtobufEngine {
    /// Project a SEA graph to a single ProtoFile.
    pub fn project(
        graph: &Graph,
        namespace_filter: &str,
        package: &str,
    ) -> ProtoFile;
}
```

**Example:**

```rust
use sea_core::projection::protobuf::ProtobufEngine;

let proto = ProtobufEngine::project(&graph, "payments", "com.example.payments");
println!("{}", proto.to_proto_string());
```

### Multi-File Projection

```rust
impl ProtobufEngine {
    /// Project to multiple files, one per namespace.
    pub fn project_multi_file(
        graph: &Graph,
        base_package: &str,
    ) -> BTreeMap<String, ProtoFile>;
}
```

**Example:**

```rust
let files = ProtobufEngine::project_multi_file(&graph, "com.example");
for (namespace, proto) in files {
    let path = format!("{}/{}.proto", namespace, namespace);
    std::fs::write(&path, proto.to_proto_string())?;
}
```

### Service Generation

```rust
impl ProtobufEngine {
    /// Generate gRPC services from Flow patterns.
    pub fn flows_to_services(graph: &Graph) -> Vec<ProtoService>;
}
```

---

## Type Mapping

### map_sea_type_to_proto

Maps SEA types to Protobuf types.

```rust
pub fn map_sea_type_to_proto(sea_type: &str) -> ProtoType;
```

| SEA Type                        | Protobuf Type               |
| ------------------------------- | --------------------------- |
| `String`                        | `string`                    |
| `Int`                           | `int64`                     |
| `Float`                         | `double`                    |
| `Bool`                          | `bool`                      |
| `Date`, `DateTime`, `Timestamp` | `google.protobuf.Timestamp` |
| `Duration`                      | `google.protobuf.Duration`  |
| `UUID`                          | `string`                    |
| `Bytes`, `Binary`               | `bytes`                     |
| `Any`, `Dynamic`                | `google.protobuf.Any`       |
| Entity name                     | Message reference           |

---

## Compatibility Checking

### CompatibilityChecker

Validates schema changes against compatibility rules.

```rust
pub struct CompatibilityChecker;

impl CompatibilityChecker {
    /// Check compatibility between old and new schemas.
    pub fn check(
        old: &ProtoFile,
        new: &ProtoFile,
        mode: CompatibilityMode,
    ) -> CompatibilityResult;
}
```

### CompatibilityMode

```rust
pub enum CompatibilityMode {
    Additive,  // Only additions allowed
    Backward,  // Additions + safe removals (with reserved)
    Breaking,  // Any changes allowed
}
```

### CompatibilityResult

```rust
pub struct CompatibilityResult {
    pub is_compatible: bool,
    pub violations: Vec<CompatibilityViolation>,
}
```

### Compatibility Rules

| Change Type         | Additive | Backward      | Breaking |
| ------------------- | -------- | ------------- | -------- |
| Add field           | ✅       | ✅            | ✅       |
| Remove field        | ❌       | ✅ (reserved) | ✅       |
| Rename field        | ❌       | ❌            | ✅       |
| Change field type   | ❌       | ❌            | ✅       |
| Change field number | ❌       | ❌            | ✅       |
| Add enum value      | ✅       | ✅            | ✅       |
| Remove enum value   | ❌       | ✅ (reserved) | ✅       |
| Add message         | ✅       | ✅            | ✅       |
| Remove message      | ❌       | ❌            | ✅       |

---

## Buf.build Integration

### BufIntegration

Optional integration with the buf.build CLI.

```rust
pub struct BufIntegration;

impl BufIntegration {
    /// Check if buf CLI is available.
    pub fn is_available(&self) -> bool;

    /// Get buf CLI version.
    pub fn get_version(&self) -> BufResult<String>;

    /// Lint proto files.
    pub fn lint(&self, proto_dir: &Path) -> BufResult<BufLintResult>;

    /// Check for breaking changes.
    pub fn breaking_check(
        &self,
        new_dir: &Path,
        old_dir: &Path,
    ) -> BufResult<BufBreakingResult>;

    /// Generate buf.yaml configuration.
    pub fn generate_buf_yaml() -> String;

    /// Generate buf.gen.yaml for code generation.
    pub fn generate_buf_gen_yaml(languages: &[BufLanguage]) -> String;
}
```

### BufLanguage

Supported languages for buf code generation.

```rust
pub enum BufLanguage {
    Go, Rust, Python, TypeScript, Java, CSharp, Cpp, Swift,
}
```

---

## Error Types

### ProjectionError

```rust
pub enum ProjectionError {
    InvalidProjection(String),
    NamespaceNotFound(String),
    CompatibilityViolation(CompatibilityViolation),
    IoError(std::io::Error),
}
```

### BufError

```rust
pub enum BufError {
    NotInstalled,
    CommandFailed { exit_code: i32, stderr: String },
    IoError(String),
    ConfigError(String),
}
```

---

## See Also

- [How-To: Export to Protobuf](../how-tos/export-to-protobuf.md)
- [SDS-001: Protobuf Projection Engine](../specs/SDS-001-protobuf-projection-engine.md)
- [ADR-003: Protobuf as Projection Target](../specs/ADR-003-protobuf-projection-target.md)
- [Protobuf Advanced Features Plan](../plans/protobuf_advanced_features_plan.md)
