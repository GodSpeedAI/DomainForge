//! AST Schema types for JSON Schema generation.
//!
//! This module provides serializable versions of the AST types with JsonSchema
//! derives for programmatic JSON Schema generation. These mirror the types in
//! `ast.rs` but are specifically designed for schema export.
//!
//! To regenerate the schema, run:
//! ```bash
//! cargo test generate_ast_schema -- --ignored --nocapture
//! ```

#[cfg(test)]
mod schema_gen {
    use schemars::{schema_for, JsonSchema};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    // =========================================================================
    // File Metadata
    // =========================================================================

    /// Import declaration for a module file
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ImportDecl {
        pub specifier: ImportSpecifier,
        pub from_module: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(tag = "type")]
    pub enum ImportSpecifier {
        Named { items: Vec<ImportItem> },
        Wildcard { alias: String },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ImportItem {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alias: Option<String>,
    }

    /// File-level metadata from header annotations
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
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

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum PolicyKind {
        Constraint,
        Derivation,
        Obligation,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum PolicyModality {
        Obligation,
        Prohibition,
        Permission,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum Severity {
        Info,
        Warning,
        Error,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum TargetFormat {
        Calm,
        Kg,
        Sbvr,
        Protobuf,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct MappingRule {
        pub primitive_type: String,
        pub primitive_name: String,
        pub target_type: String,
        #[serde(default)]
        pub fields: HashMap<String, serde_json::Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ProjectionOverride {
        pub primitive_type: String,
        pub primitive_name: String,
        #[serde(default)]
        pub fields: HashMap<String, serde_json::Value>,
    }

    // =========================================================================
    // Expression Types (for Policy and Metric)
    // =========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct WindowSpec {
        pub duration: u64,
        pub unit: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum UnaryOp {
        Not,
        Negate,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum Quantifier {
        ForAll,
        Exists,
        ExistsUnique,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum AggregateFunction {
        Count,
        Sum,
        Min,
        Max,
        Avg,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    }

    // =========================================================================
    // AST Node Types
    // =========================================================================

    /// Spanned AST node with source location information
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct SpannedAstNode {
        pub node: AstNode,
        /// 1-indexed line number
        pub line: usize,
        /// 1-indexed column number
        pub column: usize,
    }

    /// AST Node types representing all SEA DSL declarations
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
            quantity: Option<i32>,
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
    }

    // =========================================================================
    // Root AST Structure
    // =========================================================================

    /// Abstract Syntax Tree for SEA DSL
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Ast {
        pub metadata: FileMetadata,
        pub declarations: Vec<SpannedAstNode>,
    }

    // =========================================================================
    // Schema Generation Test
    // =========================================================================

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
