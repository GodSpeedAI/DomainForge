//! AST Schema types for JSON serialization and schema generation.
//!
//! This module provides serializable versions of the AST types with JsonSchema
//! derives for programmatic JSON Schema generation. These mirror the types in
//! `ast.rs` but are specifically designed for schema export and JSON serialization.
//!
//! ## Design Rationale
//!
//! The schema types are separate from the internal AST types to:
//! - Provide a stable JSON API contract that doesn't change with internal refactoring
//! - Allow the internal AST to evolve without breaking downstream tools
//! - Enable precise control over JSON serialization format (e.g., tagged enums)
//! - Support JSON Schema generation via `schemars`
//!
//! ## Usage
//!
//! Convert from internal AST to schema types:
//! ```rust,ignore
//! use domainforge_core::parser::{parse, ast_schema};
//!
//! let internal_ast = parse(source)?;
//! let schema_ast: ast_schema::Ast = internal_ast.into();
//! let json = serde_json::to_string_pretty(&schema_ast)?;
//! ```
//!
//! ## Schema Generation
//!
//! To regenerate the schema, run:
//! ```bash
//! cargo test generate_ast_schema -- --ignored --nocapture
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// =========================================================================
// File Metadata
// =========================================================================

/// Import declaration for a module file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ImportDecl {
    pub specifier: ImportSpecifier,
    pub from_module: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum ImportSpecifier {
    Named { items: Vec<ImportItem> },
    Wildcard { alias: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ImportItem {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

/// File-level metadata from header annotations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct FileMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ImportDecl>,
}

// =========================================================================
// Policy Metadata
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum PolicyKind {
    Constraint,
    Derivation,
    Obligation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum PolicyModality {
    Obligation,
    Prohibition,
    Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PolicyMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<PolicyKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<PolicyModality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

// =========================================================================
// Metric Metadata
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MetricMetadata {
    /// Refresh interval in ISO 8601 duration format (e.g., "PT1H")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Decimal value as string for precision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}

// =========================================================================
// Mapping and Projection
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum TargetFormat {
    Calm,
    Kg,
    Sbvr,
    Protobuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MappingRule {
    pub primitive_type: String,
    pub primitive_name: String,
    pub target_type: String,
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ProjectionOverride {
    pub primitive_type: String,
    pub primitive_name: String,
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

// =========================================================================
// Expression Types (for Policy and Metric)
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct WindowSpec {
    pub duration: u64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum BinaryOp {
    And,
    Or,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
    HasRole,
    Before,
    After,
    During,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum Quantifier {
    ForAll,
    Exists,
    ExistsUnique,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum Expression {
    Literal {
        value: serde_json::Value,
    },
    QuantityLiteral {
        /// Decimal value as string for precision
        value: String,
        unit: String,
    },
    TimeLiteral {
        timestamp: String,
    },
    IntervalLiteral {
        start: String,
        end: String,
    },
    Variable {
        name: String,
    },
    GroupBy {
        variable: String,
        collection: Box<Expression>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<Box<Expression>>,
        key: Box<Expression>,
        condition: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    Cast {
        operand: Box<Expression>,
        target_type: String,
    },
    Quantifier {
        quantifier: Quantifier,
        variable: String,
        collection: Box<Expression>,
        condition: Box<Expression>,
    },
    MemberAccess {
        object: String,
        member: String,
    },
    Aggregation {
        function: AggregateFunction,
        collection: Box<Expression>,
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<Box<Expression>>,
    },
    AggregationComprehension {
        function: AggregateFunction,
        variable: String,
        collection: Box<Expression>,
        #[serde(skip_serializing_if = "Option::is_none")]
        window: Option<WindowSpec>,
        predicate: Box<Expression>,
        projection: Box<Expression>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_unit: Option<String>,
    },
    /// Contextual `role<SymbolRef>` leaf (ADR-013 §2). Opaque until
    /// application policy evaluation resolves it.
    RoleReference {
        role: String,
    },
}

// =========================================================================
// AST Node Types
// =========================================================================

/// Spanned AST node with source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SpannedAstNode {
    pub node: AstNode,
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number
    pub column: usize,
}

/// AST Node types representing all SEA DSL declarations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum AstNode {
    /// Export wrapper for public declarations
    Export { declaration: Box<SpannedAstNode> },

    /// Entity declaration
    Entity {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
        /// Optional typed body `entity "X" { ... }`. Absent on legacy
        /// bodyless entities so existing AST JSON stays byte-identical.
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<EntityBody>,
    },

    /// Resource declaration
    Resource {
        name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unit_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
    },

    /// Flow declaration - resource transfer between entities
    Flow {
        resource_name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
        from_entity: String,
        to_entity: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        quantity: Option<String>,
    },

    /// Pattern declaration - named regex for string validation
    Pattern { name: String, regex: String },

    /// Role declaration - participant category
    Role {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
    },

    /// Relation declaration - predicate connecting roles
    Relation {
        name: String,
        subject_role: String,
        predicate: String,
        object_role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        via_flow: Option<String>,
    },

    /// Dimension declaration for units
    Dimension { name: String },

    /// Unit declaration - custom unit definition
    UnitDeclaration {
        symbol: String,
        dimension: String,
        /// Decimal conversion factor as string
        factor: String,
        base_unit: String,
    },

    /// Policy declaration - validation rule or constraint
    Policy {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        metadata: PolicyMetadata,
        expression: Expression,
    },

    /// Instance declaration - entity instance with field values
    Instance {
        name: String,
        entity_type: String,
        #[serde(default)]
        fields: HashMap<String, Expression>,
    },

    /// ConceptChange declaration - version migration
    ConceptChange {
        name: String,
        from_version: String,
        to_version: String,
        migration_policy: String,
        breaking_change: bool,
    },

    /// Metric declaration - observable metric
    Metric {
        name: String,
        expression: Expression,
        metadata: MetricMetadata,
    },

    /// Mapping declaration - format mapping rules
    MappingDecl {
        name: String,
        target: TargetFormat,
        rules: Vec<MappingRule>,
    },

    /// Projection declaration - output configuration
    ProjectionDecl {
        name: String,
        target: TargetFormat,
        overrides: Vec<ProjectionOverride>,
    },

    /// Cell declaration - isolated agent execution unit (cell-environment projection)
    Cell {
        name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// SystemDependency declaration - OS package or native library
    SystemDependency {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// Runtime declaration - language runtime or compiler toolchain
    Runtime {
        name: String,
        version: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// Tool declaration - package manager, task runner, or developer CLI
    Tool {
        name: String,
        version: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// DependencySet declaration - application dependency closure
    DependencySet {
        name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// Service declaration - background or attached service
    Service {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// Mount declaration - filesystem exposure
    Mount {
        name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// Endpoint declaration - declared network destination
    Endpoint {
        name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// NetworkFlow declaration - allowed communication relation
    NetworkFlow {
        name: String,
        from_ref: String,
        to_ref: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    /// Credential declaration - scoped secret requirement
    Credential {
        name: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        annotations: HashMap<String, serde_json::Value>,
    },

    // ---- SEA application contract declarations (ADR-013) ----
    /// Record declaration - structured DTO input/output
    Record { declaration: RecordDecl },

    /// Enum declaration - closed set of named wires
    Enum { declaration: EnumDecl },

    /// Operation declaration - inbound command/query contract
    Operation { declaration: OperationDecl },
}

/// Typed body of an `entity` declaration (ADR-013).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EntityBody {
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct FieldDecl {
    pub is_key: bool,
    pub name: String,
    pub field_type: FieldType,
    pub is_optional: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<FieldConstraintDecl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum FieldType {
    Scalar { symbol: FieldTypeRef },
    Quantity { symbol: FieldTypeRef },
    Ref { symbol: FieldTypeRef },
    List { element: Box<FieldType> },
    Named { symbol: FieldTypeRef },
}

/// Alias-qualified symbol reference inside a field type. Stored verbatim
/// until application resolution (Task 7).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct FieldTypeRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum FieldConstraintDecl {
    MinLength { value: u64 },
    MaxLength { value: u64 },
    MinItems { value: u64 },
    MaxItems { value: u64 },
    ExclusiveMin { value: String },
    ExclusiveMax { value: String },
    Min { value: String },
    Max { value: String },
    Pattern { symbol: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct RecordDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EnumDecl {
    pub name: String,
    pub members: Vec<EnumMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EnumMember {
    pub name: String,
    pub wire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct OperationDecl {
    pub name: String,
    pub clauses: Vec<OperationClause>,
}

/// Authored operation clause. Stored verbatim; the application module
/// resolves and lowers each clause.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum OperationClause {
    Intent {
        text: String,
    },
    Direction {
        kind: String,
    },
    Actor {
        actor: String,
    },
    AccessPublic,
    AccessPolicyGoverned {
        bindings: Vec<PolicyBinding>,
    },
    Input {
        reference: String,
    },
    Output {
        reference: String,
    },
    State {
        reference: String,
    },
    Effect {
        kind: String,
        reference: String,
    },
    Transaction {
        kind: String,
    },
    Failure {
        code: String,
        kinds: Vec<String>,
        message: String,
    },
    IdempotencyKeyed {
        field: String,
    },
    IdempotencyInherent,
    IdempotencyNotApplicable {
        reason: String,
        explanation: String,
    },
    ConcurrencyUnique {
        field: String,
    },
    ConcurrencyOptimistic {
        field: String,
    },
    ConcurrencyReadSnapshot,
    EvidenceOperationTrace,
    LifecycleSynchronousRequestResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PolicyBinding {
    pub policy: String,
    pub enforcement_point: String,
    pub failure_code: String,
}

// =========================================================================
// Root AST Structure
// =========================================================================

/// Abstract Syntax Tree for SEA DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct Ast {
    pub metadata: FileMetadata,
    pub declarations: Vec<SpannedAstNode>,
}

// =========================================================================
// Schema Generation (test-only)
// =========================================================================

#[cfg(all(test, feature = "json-schema"))]
mod tests {
    use super::*;
    use schemars::schema_for;

    #[test]
    #[ignore] // Run manually: cargo test generate_ast_schema -- --ignored --nocapture
    fn generate_ast_schema() {
        let schema = schema_for!(Ast);
        let json = serde_json::to_string_pretty(&schema).expect("Failed to serialize schema");

        // Write to schemas directory
        let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("schemas")
            .join("ast-v3.schema.json");

        std::fs::write(&schema_path, &json).expect("Failed to write schema file");

        println!("Schema written to: {}", schema_path.display());
        println!("\n{}", json);
    }
}
