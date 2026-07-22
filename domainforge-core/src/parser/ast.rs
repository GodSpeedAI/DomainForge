use crate::graph::Graph;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::{ParseOptions, Rule, SeaParser};
use crate::patterns::Pattern;
use crate::policy::{
    AggregateFunction, BinaryOp, Expression, Policy, PolicyKind as CorePolicyKind,
    PolicyModality as CorePolicyModality, Quantifier as PolicyQuantifier, UnaryOp, WindowSpec,
};
use crate::primitives::{ConceptChange, Entity, Flow, RelationType, Resource, Role, Severity};
use crate::units::unit_from_string;
use crate::SemanticVersion;
use chrono::Duration;
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Span};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// File-level metadata from header annotations
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileMetadata {
    pub namespace: Option<String>,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub profile: Option<String>,
    pub imports: Vec<ImportDecl>,
}

/// Import declaration for a module file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDecl {
    pub specifier: ImportSpecifier,
    pub from_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportSpecifier {
    Named(Vec<ImportItem>),
    Wildcard(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

/// Policy metadata
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetadata {
    pub kind: Option<PolicyKind>,
    pub modality: Option<PolicyModality>,
    pub priority: Option<i32>,
    pub rationale: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyKind {
    Constraint,
    Derivation,
    Obligation,
}

impl std::fmt::Display for PolicyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyKind::Constraint => write!(f, "Constraint"),
            PolicyKind::Derivation => write!(f, "Derivation"),
            PolicyKind::Obligation => write!(f, "Obligation"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyModality {
    Obligation,
    Prohibition,
    Permission,
}

impl std::fmt::Display for PolicyModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyModality::Obligation => write!(f, "Obligation"),
            PolicyModality::Prohibition => write!(f, "Prohibition"),
            PolicyModality::Permission => write!(f, "Permission"),
        }
    }
}

/// Metric declaration AST node
#[derive(Debug, Clone, PartialEq)]
pub struct MetricMetadata {
    pub refresh_interval: Option<Duration>,
    pub unit: Option<String>,
    pub threshold: Option<Decimal>,
    pub severity: Option<Severity>,
    pub target: Option<Decimal>,
    pub window: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetFormat {
    Calm,
    Kg,
    Sbvr,
    Protobuf,
}

impl std::fmt::Display for TargetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetFormat::Calm => write!(f, "CALM"),
            TargetFormat::Kg => write!(f, "KG"),
            TargetFormat::Sbvr => write!(f, "SBVR"),
            TargetFormat::Protobuf => write!(f, "Protobuf"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappingRule {
    pub primitive_type: String,
    pub primitive_name: String,
    pub target_type: String,
    pub fields: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionOverride {
    pub primitive_type: String,
    pub primitive_name: String,
    pub fields: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub metadata: FileMetadata,
    pub declarations: Vec<Spanned<AstNode>>,
}

// ---- SEA application contract AST extensions (ADR-013) ----
// These types carry authored application declarations verbatim; semantic
// resolution and validation happen in the application module. Field clauses
// are stored in authored order so partial operations preserve evidence for
// APP001 diagnostic construction.

/// Typed body of an `entity` declaration: a sequence of fields.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBody {
    pub fields: Vec<FieldDecl>,
}

/// One field of a `record` or typed `entity` body.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub is_key: bool,
    pub name: String,
    pub field_type: FieldType,
    pub is_optional: bool,
    pub constraints: Vec<FieldConstraintDecl>,
    /// Authored `default <literal>` value, type-checked during application
    /// validation. Stored as an `Expression` for round-trip parity, though
    /// only the `literal` grammar branch is accepted.
    pub default: Option<Expression>,
}

/// Type of a record/entity field. Closes the v0.1 type model: scalars,
/// `quantity<Unit>`, `ref<Target>`, `list<...>`, and named symbol references.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Scalar(FieldTypeRef),
    Quantity(FieldTypeRef),
    Ref(FieldTypeRef),
    List(Box<FieldType>),
    Named(FieldTypeRef),
}

/// A possibly alias-qualified symbol reference: `Order` or `alias.Order`.
/// Used by `quantity<>`, `ref<>`, and named field types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldTypeRef {
    pub alias: Option<String>,
    pub symbol: String,
}

/// Authored field constraint; validation maps these to the closed enum and
/// type-checks the operand against the field type.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldConstraintDecl {
    MinLength(u64),
    MaxLength(u64),
    MinItems(u64),
    MaxItems(u64),
    ExclusiveMin(Decimal),
    ExclusiveMax(Decimal),
    Min(Decimal),
    Max(Decimal),
    /// `pattern <symbol_ref>` - the symbol reference text is stored verbatim
    /// and resolved against the closure's pattern declarations.
    Pattern(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub members: Vec<EnumMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumMember {
    pub name: String,
    pub wire: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationDecl {
    pub name: String,
    /// Authored clauses in source order. Validation (APP001) reports missing,
    /// duplicate, or out-of-order clauses against this list; it never
    /// re-orders it.
    pub clauses: Vec<OperationClause>,
}

/// One authored operation clause. Strings carry the raw symbol_ref / kind
/// text; the application module resolves and lowers them.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationClause {
    Intent(String),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyBinding {
    /// Raw symbol_ref text (`order_total_within_limit` or `alias.foo`).
    pub policy: String,
    /// `precondition` | `invariant` | `postcondition` (validated in APP007).
    pub enforcement_point: String,
    pub failure_code: String,
}

/// AST Node types
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Export(Box<Spanned<AstNode>>),
    Entity {
        name: String,
        version: Option<String>,
        annotations: HashMap<String, JsonValue>,
        domain: Option<String>,
        /// Optional typed body `entity "X" { ... }`. Absent for legacy
        /// bodyless entities.
        body: Option<EntityBody>,
    },
    Resource {
        name: String,
        annotations: HashMap<String, JsonValue>,
        unit_name: Option<String>,
        domain: Option<String>,
    },
    Record(RecordDecl),
    Enum(EnumDecl),
    Operation(OperationDecl),
    Flow {
        resource_name: String,
        annotations: HashMap<String, JsonValue>,
        from_entity: String,
        to_entity: String,
        quantity: Option<Decimal>,
    },
    Pattern {
        name: String,
        regex: String,
    },
    Role {
        name: String,
        domain: Option<String>,
    },
    Relation {
        name: String,
        subject_role: String,
        predicate: String,
        object_role: String,
        via_flow: Option<String>,
    },
    Dimension {
        name: String,
    },
    UnitDeclaration {
        symbol: String,
        dimension: String,
        factor: Decimal,
        base_unit: String,
    },
    Policy {
        name: String,
        version: Option<String>,
        metadata: PolicyMetadata,
        expression: Expression,
    },
    Instance {
        name: String,
        entity_type: String,
        fields: HashMap<String, Expression>,
    },
    ConceptChange {
        name: String,
        from_version: String,
        to_version: String,
        migration_policy: String,
        breaking_change: bool,
    },
    Metric {
        name: String,
        expression: Expression,
        metadata: MetricMetadata,
    },
    MappingDecl {
        name: String,
        target: TargetFormat,
        rules: Vec<MappingRule>,
    },
    ProjectionDecl {
        name: String,
        target: TargetFormat,
        overrides: Vec<ProjectionOverride>,
    },
    Cell {
        name: String,
        annotations: HashMap<String, JsonValue>,
    },
    SystemDependency {
        name: String,
        version: Option<String>,
        annotations: HashMap<String, JsonValue>,
    },
    Runtime {
        name: String,
        version: String,
        annotations: HashMap<String, JsonValue>,
    },
    Tool {
        name: String,
        version: String,
        annotations: HashMap<String, JsonValue>,
    },
    DependencySet {
        name: String,
        annotations: HashMap<String, JsonValue>,
    },
    Service {
        name: String,
        version: Option<String>,
        annotations: HashMap<String, JsonValue>,
    },
    Mount {
        name: String,
        annotations: HashMap<String, JsonValue>,
    },
    Endpoint {
        name: String,
        annotations: HashMap<String, JsonValue>,
    },
    NetworkFlow {
        name: String,
        from_ref: String,
        to_ref: String,
        annotations: HashMap<String, JsonValue>,
    },
    Credential {
        name: String,
        annotations: HashMap<String, JsonValue>,
    },
}

/// Parse source code into an AST
pub fn parse_source(source: &str) -> ParseResult<Ast> {
    let pairs = SeaParser::parse(Rule::program, source)?;
    build_ast(pairs)
}

/// Build AST from pest pairs
fn build_ast(pairs: Pairs<Rule>) -> ParseResult<Ast> {
    let mut metadata = FileMetadata::default();
    let mut declarations = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::file_header => {
                            metadata = parse_file_header(inner)?;
                        }
                        Rule::declaration => {
                            for decl in inner.into_inner() {
                                let node = parse_declaration(decl)?;
                                declarations.push(node);
                            }
                        }
                        Rule::EOI => {}
                        _ => {}
                    }
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(Ast {
        metadata,
        declarations,
    })
}

/// Parse file header annotations
fn parse_file_header(pair: Pair<Rule>) -> ParseResult<FileMetadata> {
    let mut metadata = FileMetadata::default();

    for annotation in pair.into_inner() {
        match annotation.as_rule() {
            Rule::annotation => {
                let mut inner = annotation.into_inner();
                let name = inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected annotation name".to_string())
                })?;
                let value = inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected annotation value".to_string())
                })?;

                let name_str = name.as_str().to_lowercase();
                let value_str = parse_string_literal(value)?;

                match name_str.as_str() {
                    "namespace" => metadata.namespace = Some(value_str),
                    "version" => metadata.version = Some(value_str),
                    "owner" => metadata.owner = Some(value_str),
                    "profile" => metadata.profile = Some(value_str),
                    _ => {
                        return Err(ParseError::GrammarError(format!(
                            "Unknown annotation: {}",
                            name_str
                        )))
                    }
                }
            }
            Rule::import_decl => {
                metadata.imports.push(parse_import_decl(annotation)?);
            }
            _ => {}
        }
    }

    Ok(metadata)
}

/// Parse a single declaration
fn parse_declaration(pair: Pair<Rule>) -> ParseResult<Spanned<AstNode>> {
    let (line, column) = pair.line_col();
    let node = match pair.as_rule() {
        Rule::export_decl => {
            let mut inner = pair.into_inner();
            let wrapped = inner.next().ok_or_else(|| {
                ParseError::GrammarError("Expected declaration after export".to_string())
            })?;
            let node = parse_declaration(wrapped)?;
            Ok(AstNode::Export(Box::new(node)))
        }
        Rule::declaration_inner => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Empty declaration".to_string()))?;
            // Recursively call parse_declaration, but we need to unwrap the result if we want to avoid double spanning?
            // Actually declaration_inner is just a wrapper. The inner parse_declaration returns Spanned<AstNode>.
            // We should just return that directly.
            return parse_declaration(inner);
        }
        Rule::dimension_decl => parse_dimension(pair),
        Rule::unit_decl => parse_unit_declaration(pair),
        Rule::record_decl => parse_record(pair),
        Rule::enum_decl => parse_enum(pair),
        Rule::operation_decl => parse_operation(pair),
        Rule::entity_decl => parse_entity(pair),
        Rule::resource_decl => parse_resource(pair),
        Rule::flow_decl => parse_flow(pair),
        Rule::pattern_decl => parse_pattern(pair),
        Rule::role_decl => parse_role(pair),
        Rule::relation_decl => parse_relation(pair),
        Rule::instance_decl => parse_instance(pair),
        Rule::policy_decl => parse_policy(pair),
        Rule::concept_change_decl => parse_concept_change(pair),
        Rule::metric_decl => parse_metric(pair),
        Rule::mapping_decl => parse_mapping(pair),
        Rule::projection_decl => parse_projection(pair),
        Rule::cell_decl => parse_cell(pair),
        Rule::system_dependency_decl => parse_system_dependency(pair),
        Rule::runtime_decl => parse_runtime(pair),
        Rule::tool_decl => parse_tool(pair),
        Rule::dependency_set_decl => parse_dependency_set(pair),
        Rule::service_decl => parse_service(pair),
        Rule::mount_decl => parse_mount(pair),
        Rule::endpoint_decl => parse_endpoint(pair),
        Rule::network_flow_decl => parse_network_flow(pair),
        Rule::credential_decl => parse_credential(pair),
        _ => Err(ParseError::GrammarError(format!(
            "Unexpected rule: {:?}",
            pair.as_rule()
        ))),
    }?;

    Ok(Spanned { node, line, column })
}

fn parse_import_decl(pair: Pair<Rule>) -> ParseResult<ImportDecl> {
    let mut inner = pair.into_inner();
    let specifier_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Missing import specifier".to_string()))?;
    let from_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Missing import source".to_string()))?;

    let specifier = match specifier_pair.as_rule() {
        Rule::import_specifier | Rule::import_named | Rule::import_wildcard => {
            parse_import_specifier(specifier_pair)?
        }
        _ => {
            return Err(ParseError::GrammarError(format!(
                "Unexpected import specifier: {:?}",
                specifier_pair.as_rule()
            )))
        }
    };

    Ok(ImportDecl {
        specifier,
        from_module: parse_string_literal(from_pair)?,
    })
}

fn parse_import_specifier(pair: Pair<Rule>) -> ParseResult<ImportSpecifier> {
    match pair.as_rule() {
        Rule::import_wildcard => {
            let mut inner = pair.into_inner();
            let alias = parse_identifier(inner.next().ok_or_else(|| {
                ParseError::GrammarError("Expected alias for wildcard import".to_string())
            })?)?;
            Ok(ImportSpecifier::Wildcard(alias))
        }
        // `import_specifier` wraps exactly one of `import_named` /
        // `import_wildcard`; descend one level so we iterate the actual
        // `import_item` pairs (or the wildcard alias) below.
        Rule::import_specifier => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Empty import specifier".to_string()))?;
            parse_import_specifier(inner)
        }
        Rule::import_named => {
            let mut items = Vec::new();
            for item in pair.into_inner() {
                if item.as_rule() == Rule::import_item {
                    items.push(parse_import_item(item)?);
                }
            }
            Ok(ImportSpecifier::Named(items))
        }
        _ => Err(ParseError::GrammarError(
            "Invalid import specifier".to_string(),
        )),
    }
}

fn parse_import_item(pair: Pair<Rule>) -> ParseResult<ImportItem> {
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected import item name".to_string()))?;
    let alias = inner.next().map(parse_identifier).transpose()?;

    Ok(ImportItem {
        name: parse_identifier(name_pair)?,
        alias,
    })
}

/// Parse dimension declaration
fn parse_dimension(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected dimension name".to_string()))?,
    )?;

    Ok(AstNode::Dimension { name })
}

/// Parse unit declaration
fn parse_unit_declaration(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let symbol = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected unit symbol".to_string()))?,
    )?;

    let dimension = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected dimension name".to_string()))?,
    )?;

    let factor_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected factor".to_string()))?;
    let factor = parse_decimal(factor_pair)?;

    let base_unit = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected base unit".to_string()))?,
    )?;

    Ok(AstNode::UnitDeclaration {
        symbol,
        dimension,
        factor,
        base_unit,
    })
}

// ---- SEA application contract parsers (ADR-013 §2) ----

fn parse_entity_body(pair: Pair<Rule>) -> ParseResult<EntityBody> {
    let mut fields = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::entity_field {
            let mut field_inner = inner.into_inner();
            let mut is_key = false;
            let mut next = field_inner.next().ok_or_else(|| {
                ParseError::GrammarError("Expected entity_field content".to_string())
            })?;
            if next.as_rule() == Rule::key_marker {
                is_key = true;
                next = field_inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected field_decl after key".to_string())
                })?;
            }
            if next.as_rule() != Rule::field_decl {
                return Err(ParseError::GrammarError(format!(
                    "Expected field_decl in entity body, got {:?}",
                    next.as_rule()
                )));
            }
            fields.push(parse_field_decl(next, is_key)?);
        }
    }
    Ok(EntityBody { fields })
}

fn parse_record(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected record name".to_string()))?,
    )?;
    let body_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected record body".to_string()))?;
    let mut fields = Vec::new();
    for field_pair in body_pair.into_inner() {
        if field_pair.as_rule() == Rule::field_decl {
            fields.push(parse_field_decl(field_pair, false)?);
        }
    }
    Ok(AstNode::Record(RecordDecl { name, fields }))
}

fn parse_enum(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected enum name".to_string()))?,
    )?;
    let mut members = Vec::new();
    for member_pair in inner {
        if member_pair.as_rule() != Rule::enum_member {
            continue;
        }
        let mut member_inner = member_pair.into_inner();
        let member_name =
            parse_identifier(member_inner.next().ok_or_else(|| {
                ParseError::GrammarError("Expected enum member name".to_string())
            })?)?;
        let wire = parse_string_literal(member_inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected enum member wire value".to_string())
        })?)?;
        members.push(EnumMember {
            name: member_name,
            wire,
        });
    }
    Ok(AstNode::Enum(EnumDecl { name, members }))
}

fn parse_operation(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected operation name".to_string()))?,
    )?;
    let body_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected operation body".to_string()))?;
    let mut clauses = Vec::new();
    for clause_pair in body_pair.into_inner() {
        if let Some(clause) = parse_operation_clause(clause_pair)? {
            clauses.push(clause);
        }
    }
    Ok(AstNode::Operation(OperationDecl { name, clauses }))
}

fn parse_operation_clause(pair: Pair<Rule>) -> ParseResult<Option<OperationClause>> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::GrammarError("Empty operation clause".to_string()))?;
    let clause = match inner.as_rule() {
        Rule::intent_clause => {
            let text =
                parse_string_literal(inner.into_inner().next().ok_or_else(|| {
                    ParseError::GrammarError("Expected intent text".to_string())
                })?)?;
            OperationClause::Intent(text)
        }
        Rule::direction_clause => {
            let kind = inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected direction kind".to_string()))?
                .as_str()
                .trim()
                .to_string();
            OperationClause::Direction { kind }
        }
        Rule::actor_clause => {
            // `^"anonymous"` is a literal and produces no sub-pair; only
            // the `symbol_ref` branch yields an inner pair.
            let actor = match inner.into_inner().next() {
                Some(p) => parse_symbol_ref_text(p)?.trim().to_string(),
                None => "anonymous".to_string(),
            };
            OperationClause::Actor { actor }
        }
        Rule::access_clause => {
            // `^"public"` is a literal and produces no sub-pair; only the
            // `policy_governed_clause` branch yields an inner pair.
            let access_inner = match inner.into_inner().next() {
                Some(p) => p,
                None => return Ok(Some(OperationClause::AccessPublic)),
            };
            match access_inner.as_rule() {
                Rule::policy_governed_clause => {
                    let mut bindings = Vec::new();
                    for binding_pair in access_inner.into_inner() {
                        if binding_pair.as_rule() != Rule::policy_binding {
                            continue;
                        }
                        let mut b = binding_pair.into_inner();
                        let policy = parse_symbol_ref_text(b.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected policy in binding".to_string())
                        })?)?
                        .trim()
                        .to_string();
                        let enforcement_point = b
                            .next()
                            .ok_or_else(|| {
                                ParseError::GrammarError("Expected enforcement point".to_string())
                            })?
                            .as_str()
                            .trim()
                            .to_string();
                        // skip `fails with`
                        let failure_code = parse_identifier(b.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected failure code".to_string())
                        })?)?;
                        bindings.push(PolicyBinding {
                            policy,
                            enforcement_point,
                            failure_code,
                        });
                    }
                    OperationClause::AccessPolicyGoverned { bindings }
                }
                _ => OperationClause::AccessPublic,
            }
        }
        Rule::input_clause => OperationClause::Input {
            reference: parse_symbol_ref_text(inner.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Expected input reference".to_string())
            })?)?
            .trim()
            .to_string(),
        },
        Rule::output_clause => OperationClause::Output {
            reference: parse_symbol_ref_text(inner.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Expected output reference".to_string())
            })?)?
            .trim()
            .to_string(),
        },
        Rule::state_clause => OperationClause::State {
            reference: parse_symbol_ref_text(inner.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Expected state reference".to_string())
            })?)?
            .trim()
            .to_string(),
        },
        Rule::effect_clause => {
            let mut e = inner.into_inner();
            let kind = e
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected effect kind".to_string()))?
                .as_str()
                .trim()
                .to_string();
            let reference =
                parse_symbol_ref_text(e.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected effect target".to_string())
                })?)?
                .trim()
                .to_string();
            OperationClause::Effect { kind, reference }
        }
        Rule::transaction_clause => OperationClause::Transaction {
            kind: inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected transaction kind".to_string()))?
                .as_str()
                .trim()
                .to_string(),
        },
        Rule::failure_clause => {
            let mut f = inner.into_inner();
            let code =
                parse_identifier(f.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected failure code".to_string())
                })?)?;
            // skip `for`
            let mut kinds = Vec::new();
            let mut message = String::new();
            for part in f {
                match part.as_rule() {
                    Rule::failure_kind => kinds.push(part.as_str().trim().to_string()),
                    Rule::string_literal => {
                        message = parse_string_literal(part)?;
                    }
                    _ => {}
                }
            }
            OperationClause::Failure {
                code,
                kinds,
                message,
            }
        }
        Rule::idempotency_clause => {
            // `^"inherent"` is a literal and produces no sub-pair.
            let body = match inner.into_inner().next() {
                Some(p) => p,
                None => return Ok(Some(OperationClause::IdempotencyInherent)),
            };
            match body.as_rule() {
                Rule::idempotency_keyed => OperationClause::IdempotencyKeyed {
                    field: parse_identifier(body.into_inner().next().ok_or_else(|| {
                        ParseError::GrammarError("Expected keyed_by field".to_string())
                    })?)?,
                },
                Rule::not_applicable_value => {
                    let mut n = body.into_inner();
                    let reason = n
                        .next()
                        .ok_or_else(|| ParseError::GrammarError("Expected na_reason".to_string()))?
                        .as_str()
                        .trim()
                        .to_string();
                    let explanation = parse_string_literal(n.next().ok_or_else(|| {
                        ParseError::GrammarError("Expected na explanation".to_string())
                    })?)?;
                    OperationClause::IdempotencyNotApplicable {
                        reason,
                        explanation,
                    }
                }
                _ => OperationClause::IdempotencyInherent,
            }
        }
        Rule::concurrency_clause => {
            // `^"read_snapshot"` is a literal and produces no sub-pair.
            let body = match inner.into_inner().next() {
                Some(p) => p,
                None => return Ok(Some(OperationClause::ConcurrencyReadSnapshot)),
            };
            match body.as_rule() {
                Rule::concurrency_unique => OperationClause::ConcurrencyUnique {
                    field: parse_identifier(body.into_inner().next().ok_or_else(|| {
                        ParseError::GrammarError("Expected unique_key field".to_string())
                    })?)?,
                },
                Rule::concurrency_optimistic => OperationClause::ConcurrencyOptimistic {
                    field: parse_identifier(body.into_inner().next().ok_or_else(|| {
                        ParseError::GrammarError("Expected optimistic_version field".to_string())
                    })?)?,
                },
                _ => OperationClause::ConcurrencyReadSnapshot,
            }
        }
        Rule::evidence_clause => OperationClause::EvidenceOperationTrace,
        Rule::lifecycle_clause => OperationClause::LifecycleSynchronousRequestResponse,
        other => {
            return Err(ParseError::GrammarError(format!(
                "Unexpected operation clause rule: {:?}",
                other
            )))
        }
    };
    Ok(Some(clause))
}

fn parse_field_decl(pair: Pair<Rule>, is_key: bool) -> ParseResult<FieldDecl> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected field name".to_string()))?,
    )?;
    let field_type_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected field type".to_string()))?;
    let field_type = parse_field_type(field_type_pair)?;

    let mut is_optional = false;
    let mut constraints: Vec<FieldConstraintDecl> = Vec::new();
    let mut default: Option<Expression> = None;

    for part in inner {
        match part.as_rule() {
            Rule::optional_marker => is_optional = true,
            Rule::field_constraints => {
                for c in part.into_inner() {
                    if c.as_rule() == Rule::field_constraint {
                        constraints.push(parse_field_constraint(c)?);
                    }
                }
            }
            Rule::field_default => {
                let lit = c_to_lit(part.into_inner().next().ok_or_else(|| {
                    ParseError::GrammarError("Expected default literal".to_string())
                })?)?;
                default = Some(lit);
            }
            _ => {}
        }
    }

    Ok(FieldDecl {
        is_key,
        name,
        field_type,
        is_optional,
        constraints,
        default,
    })
}

fn parse_field_type(pair: Pair<Rule>) -> ParseResult<FieldType> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::GrammarError("Empty field type".to_string()))?;
    Ok(match inner.as_rule() {
        Rule::scalar_type => FieldType::Scalar(FieldTypeRef {
            alias: None,
            symbol: inner.as_str().to_string(),
        }),
        Rule::quantity_type => FieldType::Quantity(parse_symbol_ref_pair(
            inner.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Expected unit in quantity<>".to_string())
            })?,
        )?),
        Rule::ref_type => FieldType::Ref(parse_symbol_ref_pair(
            inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected target in ref<>".to_string()))?,
        )?),
        Rule::list_type => {
            let elem = inner.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Expected list element type".to_string())
            })?;
            FieldType::List(Box::new(parse_element_type(elem)?))
        }
        Rule::named_type => FieldType::Named(parse_symbol_ref_pair(
            inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected named type".to_string()))?,
        )?),
        other => {
            return Err(ParseError::GrammarError(format!(
                "Unexpected field_type rule: {:?}",
                other
            )))
        }
    })
}

fn parse_element_type(pair: Pair<Rule>) -> ParseResult<FieldType> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::GrammarError("Empty element type".to_string()))?;
    Ok(match inner.as_rule() {
        Rule::scalar_type => FieldType::Scalar(FieldTypeRef {
            alias: None,
            symbol: inner.as_str().to_string(),
        }),
        Rule::quantity_type => FieldType::Quantity(parse_symbol_ref_pair(
            inner.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Expected unit in quantity<>".to_string())
            })?,
        )?),
        Rule::ref_type => FieldType::Ref(parse_symbol_ref_pair(
            inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected target in ref<>".to_string()))?,
        )?),
        Rule::named_type => FieldType::Named(parse_symbol_ref_pair(
            inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected named type".to_string()))?,
        )?),
        other => {
            return Err(ParseError::GrammarError(format!(
                "Unexpected element_type rule: {:?}",
                other
            )))
        }
    })
}

fn parse_field_constraint(pair: Pair<Rule>) -> ParseResult<FieldConstraintDecl> {
    let mut inner = pair.into_inner();
    let head = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Empty field constraint".to_string()))?;
    let value_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected constraint operand".to_string()))?;
    Ok(match head.as_rule() {
        Rule::fc_min_length => FieldConstraintDecl::MinLength(parse_unsigned_int(value_pair)?),
        Rule::fc_max_length => FieldConstraintDecl::MaxLength(parse_unsigned_int(value_pair)?),
        Rule::fc_min_items => FieldConstraintDecl::MinItems(parse_unsigned_int(value_pair)?),
        Rule::fc_max_items => FieldConstraintDecl::MaxItems(parse_unsigned_int(value_pair)?),
        Rule::fc_exclusive_min => FieldConstraintDecl::ExclusiveMin(parse_decimal(value_pair)?),
        Rule::fc_exclusive_max => FieldConstraintDecl::ExclusiveMax(parse_decimal(value_pair)?),
        Rule::fc_min => FieldConstraintDecl::Min(parse_decimal(value_pair)?),
        Rule::fc_max => FieldConstraintDecl::Max(parse_decimal(value_pair)?),
        Rule::fc_pattern => FieldConstraintDecl::Pattern(parse_symbol_ref_text(value_pair)?),
        other => {
            return Err(ParseError::GrammarError(format!(
                "Unexpected field constraint head rule: {:?}",
                other
            )))
        }
    })
}

fn parse_unsigned_int(pair: Pair<Rule>) -> ParseResult<u64> {
    pair.as_str().parse::<u64>().map_err(|_| {
        ParseError::GrammarError(format!("Invalid unsigned integer: {}", pair.as_str()))
    })
}

/// Reconstruct the raw text of a `symbol_ref` (`Order` or `alias.Order`).
/// Stored verbatim until resolution; the application module resolves it.
/// We reconstruct from inner atomic `identifier` pairs because `symbol_ref`'s
/// `as_str()` can include trailing implicit whitespace in some contexts.
fn parse_symbol_ref_text(pair: Pair<Rule>) -> ParseResult<String> {
    match pair.as_rule() {
        Rule::symbol_ref => {
            let mut parts: Vec<String> = Vec::new();
            for p in pair.into_inner() {
                if p.as_rule() == Rule::identifier {
                    parts.push(p.as_str().to_string());
                }
            }
            Ok(parts.join("."))
        }
        Rule::identifier => Ok(pair.as_str().to_string()),
        other => Err(ParseError::GrammarError(format!(
            "Unexpected symbol_ref rule: {:?}",
            other
        ))),
    }
}

/// Parse a `symbol_ref` pair into an `alias.symbol` (or bare `symbol`).
fn parse_symbol_ref_pair(pair: Pair<Rule>) -> ParseResult<FieldTypeRef> {
    let mut inner = pair.into_inner();
    let first = parse_identifier(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected identifier in symbol_ref".to_string())
    })?)?;
    if let Some(second) = inner.next() {
        Ok(FieldTypeRef {
            alias: Some(first),
            symbol: parse_identifier(second)?,
        })
    } else {
        Ok(FieldTypeRef {
            alias: None,
            symbol: first,
        })
    }
}

/// Map a `Rule::literal` pair to a literal-shaped `Expression`. Only the
/// grammar's `literal` branch is accepted for field defaults.
fn c_to_lit(pair: Pair<Rule>) -> ParseResult<Expression> {
    parse_literal_expr(pair)
}

/// Parse entity declaration
fn parse_entity(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected entity name".to_string()))?,
    )?;

    let mut version = None;
    let mut annotations = HashMap::new();
    let mut domain = None;
    let mut body: Option<EntityBody> = None;

    for part in inner {
        match part.as_rule() {
            Rule::version => {
                version = Some(part.as_str().to_string());
            }
            Rule::entity_body => {
                body = Some(parse_entity_body(part)?);
            }
            Rule::entity_annotation => {
                let mut annotation_inner = part.into_inner();
                let key_pair = annotation_inner
                    .next()
                    .ok_or_else(|| ParseError::GrammarError("Empty annotation".to_string()))?;

                match key_pair.as_rule() {
                    Rule::ea_replaces => {
                        let target_name =
                            parse_name(annotation_inner.next().ok_or_else(|| {
                                ParseError::GrammarError("Expected name in replaces".to_string())
                            })?)?;
                        // Check for optional version
                        let mut target_version = None;
                        if let Some(next) = annotation_inner.next() {
                            if next.as_rule() == Rule::version {
                                target_version = Some(next.as_str().to_string());
                            }
                        }

                        let value = if let Some(v) = target_version {
                            format!("{} v{}", target_name, v)
                        } else {
                            target_name
                        };
                        annotations.insert("replaces".to_string(), JsonValue::String(value));
                    }
                    Rule::ea_changes => {
                        let array_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected string array in changes".to_string())
                        })?;
                        let mut changes = Vec::new();
                        for item in array_pair.into_inner() {
                            changes.push(parse_string_literal(item)?);
                        }
                        annotations.insert(
                            "changes".to_string(),
                            JsonValue::Array(changes.into_iter().map(JsonValue::String).collect()),
                        );
                    }
                    Rule::identifier => {
                        let key = parse_identifier(key_pair)?.to_lowercase();
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected annotation value".to_string())
                        })?;
                        let value = parse_annotation_value(value_pair)?;
                        println!("debug insert annotation {} = {:?}", key, value);
                        annotations.insert(key, value);
                    }
                    _ => {
                        return Err(ParseError::GrammarError(format!(
                            "Unexpected flow annotation: {}",
                            key_pair.as_str()
                        )));
                    }
                }
            }
            Rule::in_keyword => {
                // Skip "in"
            }
            Rule::identifier => {
                // This must be the domain
                domain = Some(parse_identifier(part)?);
            }
            _ => {}
        }
    }

    Ok(AstNode::Entity {
        name,
        version,
        annotations,
        domain,
        body,
    })
}

/// Parse concept change declaration
fn parse_concept_change(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name =
        parse_name(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected concept change name".to_string())
        })?)?;

    let mut from_version = String::new();
    let mut to_version = String::new();
    let mut migration_policy = String::new();
    let mut breaking_change = false;

    for part in inner {
        if part.as_rule() == Rule::concept_change_annotation {
            let mut annotation_inner = part.into_inner();

            let key_pair = annotation_inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected annotation key".to_string()))?;
            let value_pair = annotation_inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected annotation value".to_string()))?;

            match key_pair.as_rule() {
                Rule::cc_from_version => from_version = parse_version(value_pair)?,
                Rule::cc_to_version => to_version = parse_version(value_pair)?,
                Rule::cc_migration_policy => migration_policy = parse_identifier(value_pair)?,
                Rule::cc_breaking_change => {
                    breaking_change = value_pair.as_str() == "true";
                }
                _ => {}
            }
        }
    }

    if from_version.is_empty() {
        return Err(ParseError::GrammarError(
            "Missing cc_from_version annotation".to_string(),
        ));
    }

    if to_version.is_empty() {
        return Err(ParseError::GrammarError(
            "Missing cc_to_version annotation".to_string(),
        ));
    }

    if migration_policy.is_empty() {
        return Err(ParseError::GrammarError(
            "Missing cc_migration_policy annotation".to_string(),
        ));
    }

    Ok(AstNode::ConceptChange {
        name,
        from_version,
        to_version,
        migration_policy,
        breaking_change,
    })
}

/// Parse resource declaration
fn parse_resource(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected resource name".to_string()))?,
    )?;

    let mut annotations = HashMap::new();
    let mut unit_name = None;
    let mut domain = None;
    let mut saw_in_keyword = false;

    for part in inner {
        match part.as_rule() {
            Rule::resource_annotation => {
                let mut annotation_inner = part.into_inner();
                let key_pair = annotation_inner
                    .next()
                    .ok_or_else(|| ParseError::GrammarError("Empty annotation".to_string()))?;

                match key_pair.as_rule() {
                    Rule::ea_replaces => {
                        let target_name =
                            parse_name(annotation_inner.next().ok_or_else(|| {
                                ParseError::GrammarError("Expected name in replaces".to_string())
                            })?)?;
                        // Check for optional version
                        let mut target_version = None;
                        if let Some(next) = annotation_inner.next() {
                            if next.as_rule() == Rule::version {
                                target_version = Some(next.as_str().to_string());
                            }
                        }

                        let value = if let Some(v) = target_version {
                            format!("{} v{}", target_name, v)
                        } else {
                            target_name
                        };
                        annotations.insert("replaces".to_string(), JsonValue::String(value));
                    }
                    Rule::ea_changes => {
                        let array_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected string array in changes".to_string())
                        })?;
                        let mut changes = Vec::new();
                        for item in array_pair.into_inner() {
                            changes.push(parse_string_literal(item)?);
                        }
                        annotations.insert(
                            "changes".to_string(),
                            JsonValue::Array(changes.into_iter().map(JsonValue::String).collect()),
                        );
                    }
                    Rule::identifier => {
                        let key = parse_identifier(key_pair)?.to_lowercase();
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected annotation value".to_string())
                        })?;
                        let value = parse_annotation_value(value_pair)?;
                        annotations.insert(key, value);
                    }
                    _ => {
                        return Err(ParseError::GrammarError(format!(
                            "Unexpected flow annotation: {}",
                            key_pair.as_str()
                        )));
                    }
                }
            }
            Rule::in_keyword => {
                // Mark that we've seen 'in', next identifier is domain
                saw_in_keyword = true;
            }
            Rule::identifier => {
                // If we've seen 'in' keyword, this is domain; otherwise it's unit
                if saw_in_keyword {
                    domain = Some(parse_identifier(part)?);
                    saw_in_keyword = false; // Reset for safety
                } else {
                    unit_name = Some(parse_identifier(part)?);
                }
            }
            _ => {}
        }
    }

    Ok(AstNode::Resource {
        name,
        annotations,
        unit_name,
        domain,
    })
}

/// Parse flow declaration
fn parse_flow(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let resource_name = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected resource name".to_string()))?,
    )?;

    let mut annotations = HashMap::new();
    let mut from_entity = None;
    let mut to_entity = None;
    let mut quantity = None;
    let mut pending_annotation_key: Option<String> = None;

    for part in inner {
        match part.as_rule() {
            Rule::flow_annotation => {
                let mut annotation_inner = part.into_inner();
                let key_pair = annotation_inner
                    .next()
                    .ok_or_else(|| ParseError::GrammarError("Empty annotation".to_string()))?;

                match key_pair.as_rule() {
                    Rule::ea_replaces => {
                        let target_name =
                            parse_name(annotation_inner.next().ok_or_else(|| {
                                ParseError::GrammarError("Expected name in replaces".to_string())
                            })?)?;
                        // Check for optional version
                        let mut target_version = None;
                        if let Some(next) = annotation_inner.next() {
                            if next.as_rule() == Rule::version {
                                target_version = Some(next.as_str().to_string());
                            }
                        }

                        let value = if let Some(v) = target_version {
                            format!("{} v{}", target_name, v)
                        } else {
                            target_name
                        };
                        annotations.insert("replaces".to_string(), JsonValue::String(value));
                    }
                    Rule::ea_changes => {
                        let array_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected string array in changes".to_string())
                        })?;
                        let mut changes = Vec::new();
                        for item in array_pair.into_inner() {
                            changes.push(parse_string_literal(item)?);
                        }
                        annotations.insert(
                            "changes".to_string(),
                            JsonValue::Array(changes.into_iter().map(JsonValue::String).collect()),
                        );
                    }
                    Rule::identifier => {
                        let key = parse_identifier(key_pair)?.to_lowercase();
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected annotation value".to_string())
                        })?;
                        let value = parse_annotation_value(value_pair)?;
                        annotations.insert(key, value);
                    }
                    _ => {
                        return Err(ParseError::GrammarError(format!(
                            "Unexpected flow annotation: {}",
                            key_pair.as_str()
                        )));
                    }
                }
            }
            Rule::identifier => {
                if pending_annotation_key.is_some() {
                    return Err(ParseError::GrammarError(
                        "Unexpected annotation key without value".to_string(),
                    ));
                }
                pending_annotation_key = Some(parse_identifier(part)?.to_lowercase());
            }
            Rule::annotation_value
            | Rule::object_literal
            | Rule::string_array
            | Rule::boolean
            | Rule::number
            | Rule::string_literal => {
                if let Some(key) = pending_annotation_key.take() {
                    let value = parse_annotation_value(part)?;
                    annotations.insert(key, value);
                    continue;
                }

                if part.as_rule() == Rule::string_literal {
                    // These are from_entity and to_entity in order
                    let parsed = parse_string_literal(part)?;
                    if from_entity.is_none() {
                        from_entity = Some(parsed);
                    } else if to_entity.is_none() {
                        to_entity = Some(parsed);
                    }
                } else if part.as_rule() == Rule::number {
                    quantity = Some(parse_decimal(part)?);
                }
            }
            _ => {}
        }
    }

    Ok(AstNode::Flow {
        resource_name,
        annotations,
        from_entity: from_entity
            .ok_or_else(|| ParseError::GrammarError("Expected from entity".to_string()))?,
        to_entity: to_entity
            .ok_or_else(|| ParseError::GrammarError("Expected to entity".to_string()))?,
        quantity,
    })
}

/// Parse pattern declaration
fn parse_pattern(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected pattern name".to_string()))?,
    )?;

    let regex_literal = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected regex for pattern".to_string()))?;
    let regex = parse_string_literal(regex_literal)?;

    Ok(AstNode::Pattern { name, regex })
}

/// Parse role declaration
fn parse_role(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected role name".to_string()))?,
    )?;

    let mut domain = None;
    for part in inner {
        match part.as_rule() {
            Rule::in_keyword => {
                // Skip "in"
            }
            Rule::identifier => {
                domain = Some(parse_identifier(part)?);
            }
            _ => {}
        }
    }

    Ok(AstNode::Role { name, domain })
}

/// Parse relation declaration
fn parse_relation(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected relation name".to_string()))?,
    )?;

    let subject_literal = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected subject role in relation".to_string()))?;
    let subject_role = parse_string_literal(subject_literal)?;

    let predicate_literal = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected predicate in relation".to_string()))?;
    let predicate = parse_string_literal(predicate_literal)?;

    let object_literal = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected object role in relation".to_string()))?;
    let object_role = parse_string_literal(object_literal)?;

    let via_flow = if let Some(via_pair) = inner.next() {
        match via_pair.as_rule() {
            Rule::string_literal => Some(parse_string_literal(via_pair)?),
            _ => {
                let mut via_inner = via_pair.into_inner();
                let flow_literal = via_inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected flow name after via".to_string())
                })?;
                Some(parse_string_literal(flow_literal)?)
            }
        }
    } else {
        None
    };

    Ok(AstNode::Relation {
        name,
        subject_role,
        predicate,
        object_role,
        via_flow,
    })
}

/// Parse instance declaration
fn parse_instance(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_identifier(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected instance name".to_string()))?,
    )?;

    let entity_type = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected entity type".to_string()))?,
    )?;

    let mut fields = HashMap::new();

    // Parse optional instance body
    if let Some(body_pair) = inner.next() {
        if body_pair.as_rule() == Rule::instance_body {
            for field_pair in body_pair.into_inner() {
                if field_pair.as_rule() == Rule::instance_field {
                    let span = field_pair.as_span();
                    let mut field_inner = field_pair.into_inner();

                    let field_name = parse_identifier(field_inner.next().ok_or_else(|| {
                        ParseError::GrammarError("Expected field name".to_string())
                    })?)?;

                    if fields.contains_key(&field_name) {
                        let (line, column) = span.start_pos().line_col();
                        return Err(ParseError::GrammarError(format!(
                            "Duplicate field name '{}' at line {}, column {}",
                            field_name, line, column
                        )));
                    }

                    let field_value = parse_expression(field_inner.next().ok_or_else(|| {
                        ParseError::GrammarError("Expected field value".to_string())
                    })?)?;

                    fields.insert(field_name, field_value);
                }
            }
        }
    }

    Ok(AstNode::Instance {
        name,
        entity_type,
        fields,
    })
}

/// Parse policy declaration
fn parse_policy(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_identifier(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected policy name".to_string()))?,
    )?;

    let mut metadata = PolicyMetadata {
        kind: None,
        modality: None,
        priority: None,
        rationale: None,
        tags: Vec::new(),
    };
    let mut version: Option<String> = None;

    for next_pair in inner {
        match next_pair.as_rule() {
            Rule::policy_kind => {
                metadata.kind = Some(match next_pair.as_str().to_lowercase().as_str() {
                    "constraint" => PolicyKind::Constraint,
                    "derivation" => PolicyKind::Derivation,
                    "obligation" => PolicyKind::Obligation,
                    _ => {
                        return Err(ParseError::GrammarError(format!(
                            "Unknown policy kind: {}",
                            next_pair.as_str()
                        )))
                    }
                });
            }
            Rule::policy_modality => {
                metadata.modality = Some(match next_pair.as_str().to_lowercase().as_str() {
                    "obligation" => PolicyModality::Obligation,
                    "prohibition" => PolicyModality::Prohibition,
                    "permission" => PolicyModality::Permission,
                    _ => {
                        return Err(ParseError::GrammarError(format!(
                            "Unknown policy modality: {}",
                            next_pair.as_str()
                        )))
                    }
                });
            }
            Rule::number => {
                metadata.priority = Some(parse_number(next_pair)?);
            }
            Rule::policy_annotation => {
                parse_policy_annotation(next_pair, &mut metadata)?;
            }
            Rule::version => {
                version = Some(next_pair.as_str().to_string());
            }
            Rule::expression => {
                let expression = parse_expression(next_pair)?;
                return Ok(AstNode::Policy {
                    name,
                    version,
                    metadata,
                    expression,
                });
            }
            _ => {}
        }
    }

    Err(ParseError::GrammarError(
        "Policy missing expression".to_string(),
    ))
}

fn parse_policy_annotation(pair: Pair<Rule>, metadata: &mut PolicyMetadata) -> ParseResult<()> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected annotation name".to_string()))?;
    let value = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected annotation value".to_string()))?;

    let name_str = name.as_str().to_lowercase();

    match name_str.as_str() {
        "rationale" => {
            metadata.rationale = Some(parse_string_literal(value)?);
        }
        "tags" => {
            if value.as_rule() == Rule::string_array {
                for tag_pair in value.into_inner() {
                    if tag_pair.as_rule() == Rule::string_literal {
                        metadata.tags.push(parse_string_literal(tag_pair)?);
                    }
                }
            } else {
                metadata.tags.push(parse_string_literal(value)?);
            }
        }
        _ => {
            return Err(ParseError::GrammarError(format!(
                "Unknown policy annotation: {}",
                name_str
            )))
        }
    }

    Ok(())
}

/// Parse expression
fn parse_expression(pair: Pair<Rule>) -> ParseResult<Expression> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::GrammarError("Empty expression".to_string()))?;
            parse_expression(inner)
        }
        Rule::or_expr => parse_or_expr(pair),
        Rule::and_expr => parse_and_expr(pair),
        Rule::not_expr => parse_not_expr(pair),
        Rule::comparison_expr => parse_comparison_expr(pair),
        Rule::additive_expr => parse_additive_expr(pair),
        Rule::multiplicative_expr => parse_multiplicative_expr(pair),
        Rule::unary_expr => parse_unary_expr(pair),
        Rule::cast_expr => parse_cast_expr(pair),
        Rule::primary_expr => parse_primary_expr(pair),
        _ => Err(ParseError::InvalidExpression(format!(
            "Unexpected expression rule: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Parse a single expression string into an Expression AST node.
pub fn parse_expression_from_str(source: &str) -> ParseResult<Expression> {
    let pairs = SeaParser::parse(Rule::expression, source)
        .map_err(|e| ParseError::GrammarError(format!("Parse error: {}", e)))?;
    let parsed_pairs: Vec<_> = pairs.collect();

    if parsed_pairs.is_empty() {
        return Err(ParseError::GrammarError("Empty expression".to_string()));
    }
    if parsed_pairs.len() > 1 {
        return Err(ParseError::GrammarError(
            "Trailing input detected after expression".to_string(),
        ));
    }

    let pair = parsed_pairs.into_iter().next().unwrap();

    // Check that the entire input was consumed (no trailing characters)
    let consumed = pair.as_span().end();
    let trimmed_source = source.trim_end();
    if consumed < trimmed_source.len() {
        return Err(ParseError::GrammarError(format!(
            "Trailing input after expression: '{}'",
            &source[consumed..]
        )));
    }

    parse_expression(pair)
}

/// Parse OR expression
fn parse_or_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left =
        parse_expression(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected left expression in OR".to_string())
        })?)?;

    for right_pair in inner {
        let right = parse_expression(right_pair)?;
        left = Expression::Binary {
            left: Box::new(left),
            op: BinaryOp::Or,
            right: Box::new(right),
        };
    }

    Ok(left)
}

/// Parse AND expression
fn parse_and_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left =
        parse_expression(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected left expression in AND".to_string())
        })?)?;

    for right_pair in inner {
        let right = parse_expression(right_pair)?;
        left = Expression::Binary {
            left: Box::new(left),
            op: BinaryOp::And,
            right: Box::new(right),
        };
    }

    Ok(left)
}

/// Parse NOT expression
fn parse_not_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected expression in NOT".to_string()))?;

    // Check if this is actually a NOT expression or just a comparison_expr
    if first.as_rule() == Rule::not_expr {
        // This is a recursive NOT, parse it
        let expr = parse_expression(first)?;
        Ok(Expression::Unary {
            op: UnaryOp::Not,
            operand: Box::new(expr),
        })
    } else {
        // This is just a comparison_expr, parse it directly
        parse_expression(first)
    }
}

/// Parse comparison expression
fn parse_comparison_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected left expression in comparison".to_string())
    })?)?;

    if let Some(op_pair) = inner.next() {
        let op = parse_comparison_op(op_pair)?;
        let right = parse_expression(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected right expression in comparison".to_string())
        })?)?;
        left = Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

/// Parse comparison operator
fn parse_comparison_op(pair: Pair<Rule>) -> ParseResult<BinaryOp> {
    let op_str = pair.as_str();
    match op_str {
        "=" => Ok(BinaryOp::Equal),
        "!=" => Ok(BinaryOp::NotEqual),
        ">" => Ok(BinaryOp::GreaterThan),
        "<" => Ok(BinaryOp::LessThan),
        ">=" => Ok(BinaryOp::GreaterThanOrEqual),
        "<=" => Ok(BinaryOp::LessThanOrEqual),
        _ if op_str.eq_ignore_ascii_case("contains") => Ok(BinaryOp::Contains),
        _ if op_str.eq_ignore_ascii_case("startswith") => Ok(BinaryOp::StartsWith),
        _ if op_str.eq_ignore_ascii_case("endswith") => Ok(BinaryOp::EndsWith),
        _ if op_str.eq_ignore_ascii_case("matches") => Ok(BinaryOp::Matches),
        _ if op_str.eq_ignore_ascii_case("before") => Ok(BinaryOp::Before),
        _ if op_str.eq_ignore_ascii_case("after") => Ok(BinaryOp::After),
        _ if op_str.eq_ignore_ascii_case("during") => Ok(BinaryOp::During),
        _ if op_str.eq_ignore_ascii_case("has_role") => Ok(BinaryOp::HasRole),
        _ => Err(ParseError::InvalidExpression(format!(
            "Unknown comparison operator: {}",
            op_str
        ))),
    }
}

/// Parse additive expression
fn parse_additive_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected left expression in additive".to_string())
    })?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => BinaryOp::Plus,
            "-" => BinaryOp::Minus,
            _ => {
                return Err(ParseError::InvalidExpression(
                    "Invalid additive operator".to_string(),
                ))
            }
        };
        let right = parse_expression(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected right expression in additive".to_string())
        })?)?;
        left = Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

/// Parse multiplicative expression
fn parse_multiplicative_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected left expression in multiplicative".to_string())
    })?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "*" => BinaryOp::Multiply,
            "/" => BinaryOp::Divide,
            _ => {
                return Err(ParseError::InvalidExpression(
                    "Invalid multiplicative operator".to_string(),
                ))
            }
        };
        let right = parse_expression(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected right expression in multiplicative".to_string())
        })?)?;
        left = Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

/// Parse cast expression
fn parse_cast_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let primary = parse_expression(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected primary expression in cast".to_string())
    })?)?;

    if let Some(as_pair) = inner.next() {
        let target_type = parse_string_literal(as_pair)?;
        Ok(Expression::cast(primary, target_type))
    } else {
        Ok(primary)
    }
}

/// Parse unary expression
fn parse_unary_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected expression in unary".to_string()))?;

    if first.as_str() == "-" {
        // Unary minus
        let expr = parse_expression(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected expression after unary minus".to_string())
        })?)?;
        Ok(Expression::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(expr),
        })
    } else {
        parse_expression(first)
    }
}

/// Parse primary expression
fn parse_primary_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let inner = pair.into_inner().next().ok_or_else(|| {
        ParseError::GrammarError("Expected inner expression in primary".to_string())
    })?;

    match inner.as_rule() {
        Rule::expression => parse_expression(inner),
        Rule::group_by_expr => parse_group_by_expr(inner),
        Rule::aggregation_expr => parse_aggregation_expr(inner),
        Rule::quantified_expr => parse_quantified_expr(inner),
        Rule::member_access => parse_member_access(inner),
        Rule::application_role_ref => parse_application_role_ref(inner),
        Rule::literal => parse_literal_expr(inner),
        Rule::identifier => {
            let name = parse_identifier(inner)?;
            Ok(Expression::Variable(name))
        }
        _ => Err(ParseError::InvalidExpression(format!(
            "Unexpected primary expression: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_application_role_ref(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let sym_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected symbol in role<...>".to_string()))?;
    let mut sym_inner = sym_pair.into_inner();
    let first = parse_identifier(sym_inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected identifier in role symbol_ref".to_string())
    })?)?;
    let role = match sym_inner.next() {
        Some(second) => format!("{}.{}", first, parse_identifier(second)?),
        None => first,
    };
    Ok(Expression::RoleReference { role })
}

/// Parse aggregation expression
fn parse_aggregation_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();

    let function_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError(
            "Expected aggregate function in aggregation expression".to_string(),
        )
    })?;
    let function = parse_aggregate_fn(function_pair)?;

    let collection_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected collection in aggregation expression".to_string())
    })?;

    // Aggregation can be a simple form (e.g., sum(flows), sum(flows.quantity))
    // or the comprehension form (e.g., sum(f in flows where f.resource = "Money": f.quantity as "USD")).
    if collection_pair.as_rule() == Rule::aggregation_comprehension {
        // Parse aggregation comprehension
        let mut comp_inner = collection_pair.into_inner();
        let variable_pair = comp_inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected variable in aggregation comprehension".to_string())
        })?;
        let variable = parse_identifier(variable_pair)?;
        // Next token should be collection
        let collection_token = comp_inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected collection in aggregation comprehension".to_string())
        })?;
        let collection_name = parse_collection(collection_token)?;
        let collection = Box::new(Expression::Variable(collection_name));

        let mut next_pair = comp_inner.next().ok_or_else(|| {
            ParseError::GrammarError(
                "Expected predicate expression, projection, or window in aggregation comprehension"
                    .to_string(),
            )
        })?;

        let mut window = None;
        if next_pair.as_rule() == Rule::window_clause {
            window = Some(parse_window_clause(next_pair)?);
            next_pair = comp_inner.next().ok_or_else(|| {
                ParseError::GrammarError(
                    "Expected predicate or projection expression in aggregation comprehension"
                        .to_string(),
                )
            })?;
        }

        let remaining_pairs: Vec<Pair<Rule>> =
            std::iter::once(next_pair).chain(comp_inner).collect();

        let mut expr_pairs: Vec<Pair<Rule>> = Vec::new();
        let mut target_unit: Option<String> = None;
        for pair in remaining_pairs {
            match pair.as_rule() {
                Rule::expression => expr_pairs.push(pair),
                Rule::string_literal => {
                    target_unit = Some(parse_string_literal(pair)?);
                }
                Rule::identifier if pair.as_str().eq_ignore_ascii_case("as") => {
                    // skip explicit AS token
                }
                other => {
                    return Err(ParseError::GrammarError(format!(
                        "Unexpected token {:?} in aggregation comprehension",
                        other
                    )))
                }
            }
        }

        let (predicate, projection) = match expr_pairs.len() {
            2 => {
                let mut expr_iter = expr_pairs.into_iter();
                let predicate_expr = parse_expression(expr_iter.next().ok_or_else(|| {
                    ParseError::GrammarError(
                        "Expected predicate expression in aggregation comprehension".to_string(),
                    )
                })?)?;
                let projection_expr = parse_expression(expr_iter.next().ok_or_else(|| {
                    ParseError::GrammarError(
                        "Expected projection expression in aggregation comprehension".to_string(),
                    )
                })?)?;
                (predicate_expr, projection_expr)
            }
            1 => {
                let projection_expr =
                    parse_expression(expr_pairs.into_iter().next().ok_or_else(|| {
                        ParseError::GrammarError(
                            "Expected projection expression in aggregation comprehension"
                                .to_string(),
                        )
                    })?)?;
                (Expression::Literal(JsonValue::Bool(true)), projection_expr)
            }
            other => {
                return Err(ParseError::GrammarError(format!(
                    "Unexpected number of expressions in aggregation comprehension: {}",
                    other
                )))
            }
        };

        return Ok(Expression::AggregationComprehension {
            function,
            variable,
            collection,
            window,
            predicate: Box::new(predicate),
            projection: Box::new(projection),
            target_unit,
        });
    }

    // Parse aggregation_simple
    let mut simple_inner = collection_pair.into_inner();

    // First item is either collection or identifier
    let first_pair = simple_inner.next().ok_or_else(|| {
        ParseError::GrammarError(
            "Expected collection or identifier in aggregation_simple".to_string(),
        )
    })?;

    let collection = match first_pair.as_rule() {
        Rule::collection => parse_collection(first_pair)?,
        Rule::identifier => parse_identifier(first_pair)?,
        _ => {
            return Err(ParseError::GrammarError(format!(
                "Expected collection or identifier, got {:?}",
                first_pair.as_rule()
            )))
        }
    };

    let mut field: Option<String> = None;
    let mut filter: Option<Expression> = None;

    // Parse optional field and filter
    for item in simple_inner {
        match item.as_rule() {
            Rule::identifier => {
                field = Some(parse_identifier(item)?);
            }
            Rule::expression => {
                filter = Some(parse_expression(item)?);
            }
            _ => {}
        }
    }

    Ok(Expression::aggregation(
        function,
        Expression::Variable(collection),
        field,
        filter,
    ))
}

/// Parse aggregate function
fn parse_aggregate_fn(pair: Pair<Rule>) -> ParseResult<AggregateFunction> {
    let fn_str = pair.as_str();
    match fn_str.to_lowercase().as_str() {
        "count" => Ok(AggregateFunction::Count),
        "sum" => Ok(AggregateFunction::Sum),
        "min" => Ok(AggregateFunction::Min),
        "max" => Ok(AggregateFunction::Max),
        "avg" => Ok(AggregateFunction::Avg),
        _ => Err(ParseError::InvalidExpression(format!(
            "Unknown aggregate function: {}",
            fn_str
        ))),
    }
}

/// Parse quantified expression
fn parse_quantified_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();

    let quantifier = parse_quantifier(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected quantifier in quantified expression".to_string())
    })?)?;
    let variable = parse_identifier(inner.next().ok_or_else(|| {
        ParseError::GrammarError(
            "Expected variable identifier in quantified expression".to_string(),
        )
    })?)?;
    let collection = parse_collection(inner.next().ok_or_else(|| {
        ParseError::GrammarError(
            "Expected collection identifier in quantified expression".to_string(),
        )
    })?)?;
    let condition = parse_expression(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected quantified condition expression".to_string())
    })?)?;

    Ok(Expression::Quantifier {
        quantifier,
        variable,
        collection: Box::new(Expression::Variable(collection)),
        condition: Box::new(condition),
    })
}

/// Parse quantifier
fn parse_quantifier(pair: Pair<Rule>) -> ParseResult<PolicyQuantifier> {
    let q_str = pair.as_str();
    match q_str.to_lowercase().as_str() {
        "forall" => Ok(PolicyQuantifier::ForAll),
        "exists" => Ok(PolicyQuantifier::Exists),
        "exists_unique" => Ok(PolicyQuantifier::ExistsUnique),
        _ => Err(ParseError::InvalidExpression(format!(
            "Unknown quantifier: {}",
            q_str
        ))),
    }
}

/// Parse collection type
fn parse_collection(pair: Pair<Rule>) -> ParseResult<String> {
    Ok(pair.as_str().to_lowercase())
}

/// Parse member access
fn parse_member_access(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let object = parse_identifier(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected object identifier in member access".to_string())
    })?)?;
    let member = parse_identifier(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected member identifier in member access".to_string())
    })?)?;

    Ok(Expression::MemberAccess { object, member })
}

/// Parse literal expression
fn parse_literal_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected literal content".to_string()))?;

    match inner.as_rule() {
        Rule::string_literal => {
            let s = parse_string_literal(inner)?;
            Ok(Expression::Literal(JsonValue::String(s)))
        }
        Rule::multiline_string => {
            let s = parse_multiline_string(inner)?;
            Ok(Expression::Literal(JsonValue::String(s)))
        }
        Rule::quantity_literal => {
            let mut parts = inner.into_inner();
            let number_part = parts.next().ok_or_else(|| {
                ParseError::GrammarError("Expected number in quantity literal".to_string())
            })?;
            let unit_part = parts.next().ok_or_else(|| {
                ParseError::GrammarError("Expected unit string in quantity literal".to_string())
            })?;
            let value = parse_decimal(number_part)?;
            let unit = parse_string_literal(unit_part)?;
            Ok(Expression::QuantityLiteral { value, unit })
        }
        Rule::time_literal => {
            // Parse ISO 8601 timestamp (already includes quotes in grammar)
            let timestamp = inner.as_str();
            // Remove surrounding quotes
            let timestamp = timestamp.trim_start_matches('"').trim_end_matches('"');
            Ok(Expression::TimeLiteral(timestamp.to_string()))
        }
        Rule::interval_literal => {
            // Parse interval("start", "end")
            let mut parts = inner.into_inner();
            let start_part = parts.next().ok_or_else(|| {
                ParseError::GrammarError("Expected start time in interval literal".to_string())
            })?;
            let end_part = parts.next().ok_or_else(|| {
                ParseError::GrammarError("Expected end time in interval literal".to_string())
            })?;
            let start = parse_string_literal(start_part)?;
            let end = parse_string_literal(end_part)?;
            Ok(Expression::IntervalLiteral { start, end })
        }
        Rule::number => {
            let n = parse_decimal(inner)?;
            // Convert Decimal to f64 for JSON Number representation
            let f = n.to_f64().ok_or_else(|| {
                ParseError::InvalidQuantity(format!(
                    "Decimal value {} cannot be represented as f64",
                    n
                ))
            })?;
            // Ensure the value is finite
            if !f.is_finite() {
                return Err(ParseError::InvalidQuantity(format!(
                    "Decimal value {} converts to non-finite f64: {}",
                    n, f
                )));
            }
            let num = serde_json::Number::from_f64(f).ok_or_else(|| {
                ParseError::InvalidQuantity(format!(
                    "Cannot create JSON Number from f64 value: {}",
                    f
                ))
            })?;
            Ok(Expression::Literal(JsonValue::Number(num)))
        }
        Rule::boolean => {
            let b = inner.as_str().eq_ignore_ascii_case("true");
            Ok(Expression::Literal(JsonValue::Bool(b)))
        }
        _ => Err(ParseError::InvalidExpression(format!(
            "Unknown literal type: {:?}",
            inner.as_rule()
        ))),
    }
}

/// Parse name (handles both string_literal and multiline_string)
fn parse_name(pair: Pair<Rule>) -> ParseResult<String> {
    let inner = pair.into_inner().next().ok_or_else(|| {
        ParseError::GrammarError("Expected inner token for name but got empty pair".to_string())
    })?;
    match inner.as_rule() {
        Rule::string_literal => parse_string_literal(inner),
        Rule::multiline_string => parse_multiline_string(inner),
        _ => Err(ParseError::GrammarError(format!(
            "Expected string or multiline string for name, got {:?}",
            inner.as_rule()
        ))),
    }
}

/// Parse string literal (handles escape sequences)
fn parse_string_literal(pair: Pair<Rule>) -> ParseResult<String> {
    let s = pair.as_str();
    if s.len() < 2 || !s.starts_with('"') || !s.ends_with('"') {
        return Err(ParseError::GrammarError(format!(
            "Invalid string literal: {}",
            s
        )));
    }

    // Use serde_json to properly parse and unescape the string
    // The string already has quotes, so pass it directly
    match serde_json::from_str(s) {
        Ok(unescaped) => Ok(unescaped),
        Err(e) => Err(ParseError::GrammarError(format!(
            "Invalid string literal escape sequences: {} - {}",
            s, e
        ))),
    }
}

/// Parse multiline string (removes triple quotes and handles escape sequences)
fn parse_multiline_string(pair: Pair<Rule>) -> ParseResult<String> {
    let s = pair.as_str();
    if s.len() < 6 || !s.starts_with("\"\"\"") || !s.ends_with("\"\"\"") {
        return Err(ParseError::GrammarError(format!(
            "Invalid multiline string: {}",
            s
        )));
    }

    let content = &s[3..s.len() - 3];

    // Escape special characters for JSON compatibility
    let escaped = content
        .replace('\\', "\\\\") // Backslash must be first
        .replace('"', "\\\"") // Double quotes
        .replace('\n', "\\n") // Newlines
        .replace('\r', "\\r") // Carriage returns
        .replace('\t', "\\t"); // Tabs

    // Create a JSON string and parse it to handle escape sequences
    let json_string = format!("\"{}\"", escaped);
    match serde_json::from_str(&json_string) {
        Ok(unescaped) => Ok(unescaped),
        Err(e) => Err(ParseError::GrammarError(format!(
            "Invalid multiline string escape sequences in '{}': {}",
            s, e
        ))),
    }
}

/// Parse identifier
fn parse_identifier(pair: Pair<Rule>) -> ParseResult<String> {
    Ok(pair.as_str().to_string())
}

/// Parse semantic version (validated by grammar)
fn parse_version(pair: Pair<Rule>) -> ParseResult<String> {
    Ok(pair.as_str().to_string())
}

/// Parse number as i32
fn parse_number(pair: Pair<Rule>) -> ParseResult<i32> {
    pair.as_str()
        .parse()
        .map_err(|_| ParseError::InvalidQuantity(format!("Invalid number: {}", pair.as_str())))
}

/// Parse number as Decimal
fn parse_decimal(pair: Pair<Rule>) -> ParseResult<Decimal> {
    pair.as_str()
        .parse()
        .map_err(|_| ParseError::InvalidQuantity(format!("Invalid decimal: {}", pair.as_str())))
}

/// Parse group_by expression
fn parse_group_by_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();

    let variable_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected variable in group_by".to_string()))?;
    let variable = parse_identifier(variable_pair)?;

    let collection_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected collection in group_by".to_string()))?;
    let collection_name = parse_collection(collection_pair)?;
    let collection = Box::new(Expression::Variable(collection_name));

    let next_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected key or where clause in group_by".to_string())
    })?;

    // Collect remaining pairs to determine structure without cloning
    let remaining: Vec<Pair<Rule>> = std::iter::once(next_pair).chain(inner).collect();

    let (filter_expr, key_expr, condition_expr) = match remaining.len() {
        3 => {
            let mut iter = remaining.into_iter();
            let filter = parse_expression(iter.next().ok_or_else(|| {
                ParseError::GrammarError("Expected filter expression in group_by".to_string())
            })?)?;
            let key = parse_expression(iter.next().ok_or_else(|| {
                ParseError::GrammarError("Expected key expression in group_by".to_string())
            })?)?;
            let condition = parse_expression(iter.next().ok_or_else(|| {
                ParseError::GrammarError("Expected condition expression in group_by".to_string())
            })?)?;
            (Some(filter), key, condition)
        }
        2 => {
            let mut iter = remaining.into_iter();
            let key = parse_expression(iter.next().ok_or_else(|| {
                ParseError::GrammarError("Expected key expression in group_by".to_string())
            })?)?;
            let condition = parse_expression(iter.next().ok_or_else(|| {
                ParseError::GrammarError("Expected condition expression in group_by".to_string())
            })?)?;
            (None, key, condition)
        }
        other => {
            return Err(ParseError::GrammarError(format!(
                "Unexpected number of expressions in group_by: {}",
                other
            )))
        }
    };

    Ok(Expression::GroupBy {
        variable,
        collection,
        filter: filter_expr.map(Box::new),
        key: Box::new(key_expr),
        condition: Box::new(condition_expr),
    })
}

/// Parse window clause
fn parse_window_clause(pair: Pair<Rule>) -> ParseResult<WindowSpec> {
    let mut inner = pair.into_inner();

    let duration_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected duration in window clause".to_string())
    })?;
    let duration_i32 = parse_number(duration_pair)?;
    if duration_i32 < 0 {
        return Err(ParseError::InvalidQuantity(
            "Window duration must be non-negative".to_string(),
        ));
    }
    let duration = duration_i32 as u64;

    let unit_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected unit in window clause".to_string()))?;
    let unit = parse_string_literal(unit_pair)?;

    Ok(WindowSpec { duration, unit })
}

fn expression_kind(expr: &Expression) -> &'static str {
    match expr {
        Expression::Literal(_) => "literal",
        Expression::QuantityLiteral { .. } => "quantity_literal",
        Expression::TimeLiteral(_) => "time_literal",
        Expression::IntervalLiteral { .. } => "interval_literal",
        Expression::Variable(_) => "variable",
        Expression::GroupBy { .. } => "group_by",
        Expression::Binary { .. } => "binary",
        Expression::Unary { .. } => "unary",
        Expression::Cast { .. } => "cast",
        Expression::Quantifier { .. } => "quantifier",
        Expression::MemberAccess { .. } => "member_access",
        Expression::Aggregation { .. } => "aggregation",
        Expression::AggregationComprehension { .. } => "aggregation_comprehension",
        Expression::RoleReference { .. } => "role_reference",
    }
}

/// Convert an Expression to a JSON Value for instance fields
fn expression_to_json(expr: &Expression) -> ParseResult<JsonValue> {
    match expr {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::Variable(name) => Ok(JsonValue::String(name.clone())),
        Expression::QuantityLiteral { value, unit } => Ok(json!({
            "value": value.to_string(),
            "unit": unit
        })),
        Expression::TimeLiteral(timestamp) => Ok(JsonValue::String(timestamp.clone())),
        _ => Err(ParseError::UnsupportedExpression {
            kind: expression_kind(expr).to_string(),
            span: None,
        }),
    }
}

/// Parse mapping declaration
fn parse_mapping(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected mapping name".to_string()))?,
    )?;

    let target_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected target format".to_string()))?;
    let target = parse_target_format(target_pair)?;

    let mut rules = Vec::new();

    for rule_pair in inner {
        if rule_pair.as_rule() == Rule::mapping_rule {
            rules.push(parse_mapping_rule(rule_pair)?);
        }
    }

    Ok(AstNode::MappingDecl {
        name,
        target,
        rules,
    })
}

fn parse_target_format(pair: Pair<Rule>) -> ParseResult<TargetFormat> {
    match pair.as_str().to_lowercase().as_str() {
        "calm" => Ok(TargetFormat::Calm),
        "kg" => Ok(TargetFormat::Kg),
        "sbvr" => Ok(TargetFormat::Sbvr),
        "protobuf" | "proto" => Ok(TargetFormat::Protobuf),
        _ => Err(ParseError::GrammarError(format!(
            "Unknown target format: {}",
            pair.as_str()
        ))),
    }
}

fn parse_mapping_rule(pair: Pair<Rule>) -> ParseResult<MappingRule> {
    let mut inner = pair.into_inner();

    let primitive_type = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected primitive type".to_string()))?
        .as_str()
        .to_string();

    let primitive_name = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected primitive name".to_string()))?,
    )?;

    let target_structure = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected target structure".to_string()))?;

    let mut target_inner = target_structure.into_inner();
    let target_type = parse_identifier(
        target_inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected target type".to_string()))?,
    )?;

    let mut fields = HashMap::new();
    for field_pair in target_inner {
        if field_pair.as_rule() == Rule::mapping_field {
            let mut field_inner = field_pair.into_inner();
            let key = parse_identifier(
                field_inner
                    .next()
                    .ok_or_else(|| ParseError::GrammarError("Expected field key".to_string()))?,
            )?;
            let value_pair = field_inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected field value".to_string()))?;

            let value = match value_pair.as_rule() {
                Rule::string_literal => JsonValue::String(parse_string_literal(value_pair)?),
                Rule::boolean => JsonValue::Bool(value_pair.as_str().eq_ignore_ascii_case("true")),
                Rule::object_literal => parse_object_literal(value_pair)?,
                _ => {
                    return Err(ParseError::GrammarError(
                        "Unexpected mapping field value".to_string(),
                    ))
                }
            };
            fields.insert(key, value);
        }
    }

    Ok(MappingRule {
        primitive_type,
        primitive_name,
        target_type,
        fields,
    })
}

fn parse_object_literal(pair: Pair<Rule>) -> ParseResult<JsonValue> {
    let mut map = serde_json::Map::new();
    let mut inner = pair.into_inner();
    while let Some(key_pair) = inner.next() {
        let key = parse_string_literal(key_pair)?;
        let value_pair = inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected value in object literal".to_string())
        })?;
        let value = parse_annotation_value(value_pair)?;
        map.insert(key, value);
    }
    Ok(JsonValue::Object(map))
}

fn parse_annotation_value(pair: Pair<Rule>) -> ParseResult<JsonValue> {
    match pair.as_rule() {
        Rule::annotation_value => {
            let mut inner = pair.into_inner();
            let value_pair = inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected annotation value".to_string()))?;
            parse_annotation_value(value_pair)
        }
        Rule::string_literal => Ok(JsonValue::String(parse_string_literal(pair)?)),
        Rule::string_array => {
            let mut values = Vec::new();
            for item in pair.into_inner() {
                values.push(JsonValue::String(parse_string_literal(item)?));
            }
            Ok(JsonValue::Array(values))
        }
        Rule::boolean => Ok(JsonValue::Bool(pair.as_str().eq_ignore_ascii_case("true"))),
        Rule::number => {
            let d = parse_decimal(pair)?;
            let f = d.to_f64().ok_or_else(|| {
                ParseError::InvalidQuantity(format!(
                    "Decimal value {} cannot be represented as f64",
                    d
                ))
            })?;
            if !f.is_finite() {
                return Err(ParseError::InvalidQuantity(format!(
                    "Decimal value {} converts to non-finite f64",
                    d
                )));
            }
            let num = serde_json::Number::from_f64(f).ok_or_else(|| {
                ParseError::InvalidQuantity(format!("Cannot create JSON Number from decimal {}", d))
            })?;
            Ok(JsonValue::Number(num))
        }
        Rule::object_literal => parse_object_literal(pair),
        _ => Err(ParseError::GrammarError(format!(
            "Unexpected annotation value: {}",
            pair.as_str()
        ))),
    }
}

/// Parse projection declaration
fn parse_projection(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected projection name".to_string()))?,
    )?;

    let target_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected target format".to_string()))?;
    let target = parse_target_format(target_pair)?;

    let mut overrides = Vec::new();

    for rule_pair in inner {
        if rule_pair.as_rule() == Rule::projection_rule {
            overrides.push(parse_projection_rule(rule_pair)?);
        }
    }

    Ok(AstNode::ProjectionDecl {
        name,
        target,
        overrides,
    })
}

fn parse_projection_rule(pair: Pair<Rule>) -> ParseResult<ProjectionOverride> {
    let mut inner = pair.into_inner();

    let primitive_type = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected primitive type".to_string()))?
        .as_str()
        .to_string();

    let primitive_name = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected primitive name".to_string()))?,
    )?;

    let mut fields = HashMap::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::projection_field {
            let mut field_inner = field_pair.into_inner();
            let key = parse_identifier(
                field_inner
                    .next()
                    .ok_or_else(|| ParseError::GrammarError("Expected field key".to_string()))?,
            )?;
            let value_pair = field_inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected field value".to_string()))?;

            let value = match value_pair.as_rule() {
                Rule::string_literal => JsonValue::String(parse_string_literal(value_pair)?),
                Rule::property_mapping => parse_property_mapping(value_pair)?,
                _ => {
                    return Err(ParseError::GrammarError(
                        "Unexpected projection field value".to_string(),
                    ))
                }
            };
            fields.insert(key, value);
        }
    }

    Ok(ProjectionOverride {
        primitive_type,
        primitive_name,
        fields,
    })
}

/// Parse a `generic_annotation*` sequence (`"@" key value`) shared by all
/// cell-environment declarations into a flat annotation map. Duplicate keys
/// (after lowercasing) on the same declaration are an authoring error and are
/// rejected, including case-insensitive collisions (e.g. `@Host` after
/// `@host`).
fn parse_generic_annotations<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
) -> ParseResult<HashMap<String, JsonValue>> {
    let mut annotations = HashMap::new();
    for part in pairs {
        if part.as_rule() == Rule::generic_annotation {
            let mut inner = part.into_inner();
            let key =
                parse_identifier(inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected annotation key".to_string())
                })?)?
                .to_lowercase();
            let value_pair = inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected annotation value".to_string()))?;
            if annotations.contains_key(&key) {
                return Err(ParseError::GrammarError(format!(
                    "Duplicate annotation '@{key}' on the same declaration"
                )));
            }
            annotations.insert(key, parse_annotation_value(value_pair)?);
        }
    }
    Ok(annotations)
}

fn parse_cell(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected cell name".to_string()))?,
    )?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Cell { name, annotations })
}

fn parse_system_dependency(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner().peekable();
    let name =
        parse_name(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected system dependency name".to_string())
        })?)?;
    let mut version = None;
    if let Some(peeked) = inner.peek() {
        if peeked.as_rule() == Rule::string_literal {
            version = Some(parse_string_literal(inner.next().unwrap())?);
        }
    }
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::SystemDependency {
        name,
        version,
        annotations,
    })
}

fn parse_runtime(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected runtime name".to_string()))?,
    )?;
    let version_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected runtime version".to_string()))?;
    let version = parse_string_literal(version_pair)?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Runtime {
        name,
        version,
        annotations,
    })
}

fn parse_tool(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected tool name".to_string()))?,
    )?;
    let version_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Expected tool version".to_string()))?;
    let version = parse_string_literal(version_pair)?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Tool {
        name,
        version,
        annotations,
    })
}

fn parse_dependency_set(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name =
        parse_name(inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected dependency set name".to_string())
        })?)?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::DependencySet { name, annotations })
}

fn parse_service(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner().peekable();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected service name".to_string()))?,
    )?;
    let mut version = None;
    if let Some(peeked) = inner.peek() {
        if peeked.as_rule() == Rule::string_literal {
            version = Some(parse_string_literal(inner.next().unwrap())?);
        }
    }
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Service {
        name,
        version,
        annotations,
    })
}

fn parse_mount(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected mount name".to_string()))?,
    )?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Mount { name, annotations })
}

fn parse_endpoint(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected endpoint name".to_string()))?,
    )?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Endpoint { name, annotations })
}

fn parse_network_flow(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected network flow name".to_string()))?,
    )?;
    let from_ref = parse_string_literal(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected network flow 'from' reference".to_string())
    })?)?;
    let to_ref = parse_string_literal(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected network flow 'to' reference".to_string())
    })?)?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::NetworkFlow {
        name,
        from_ref,
        to_ref,
        annotations,
    })
}

fn parse_credential(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();
    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected credential name".to_string()))?,
    )?;
    let annotations = parse_generic_annotations(inner)?;
    Ok(AstNode::Credential { name, annotations })
}

fn parse_property_mapping(pair: Pair<Rule>) -> ParseResult<JsonValue> {
    let mut map = serde_json::Map::new();
    let mut inner = pair.into_inner();
    while let Some(key_pair) = inner.next() {
        let key = parse_string_literal(key_pair)?;
        let value_pair = inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected value in property mapping".to_string())
        })?;
        let value = parse_string_literal(value_pair)?;
        map.insert(key, JsonValue::String(value));
    }
    Ok(JsonValue::Object(map))
}

fn unwrap_export(spanned: &Spanned<AstNode>) -> &AstNode {
    match &spanned.node {
        AstNode::Export(inner) => &inner.node,
        other => other,
    }
}

/// Resolve a name in a namespace-keyed map.
/// Tries (default_namespace, name) first, then scans all namespaces for an unqualified match.
/// Returns None if not found, or the concept ID if exactly one match exists.
fn resolve_by_name(
    map: &HashMap<(String, String), crate::ConceptId>,
    name: &str,
    default_namespace: &str,
) -> Result<Option<crate::ConceptId>, ParseError> {
    // Try exact match in default namespace first
    if let Some(id) = map.get(&(default_namespace.to_string(), name.to_string())) {
        return Ok(Some(id.clone()));
    }
    // Fall back to any namespace with this name
    let matches: Vec<(&str, &crate::ConceptId)> = map
        .iter()
        .filter(|((_, n), _)| n == name)
        .map(|((namespace, _), id)| (namespace.as_str(), id))
        .collect();
    if matches.len() == 1 {
        return Ok(Some(matches[0].1.clone()));
    }
    if matches.len() > 1 {
        let mut namespaces: Vec<&str> = matches
            .into_iter()
            .map(|(namespace, _)| namespace)
            .collect();
        namespaces.sort_unstable();
        namespaces.dedup();
        return Err(ParseError::Validation(format!(
            "Ambiguous reference '{}' found in namespaces: {}",
            name,
            namespaces.join(", ")
        )));
    }
    Ok(None)
}

/// Convert AST to Graph
pub fn ast_to_graph(ast: Ast) -> ParseResult<Graph> {
    ast_to_graph_with_options(ast, &ParseOptions::default())
}

pub fn ast_to_graph_with_options(mut ast: Ast, options: &ParseOptions) -> ParseResult<Graph> {
    use crate::parser::profiles::ProfileRegistry;
    let registry = ProfileRegistry::global();
    let active_profile = ast
        .metadata
        .profile
        .clone()
        .or_else(|| options.active_profile.clone())
        .unwrap_or_else(|| "default".to_string());
    ast.metadata.profile.get_or_insert(active_profile.clone());

    if registry.get(&active_profile).is_none() {
        let available = registry.list_names().join(", ");
        let message = format!(
            "Unknown profile: '{}'. Available profiles: {}",
            active_profile, available
        );
        if options.tolerate_profile_warnings {
            log::warn!("{}", message);
        } else {
            return Err(ParseError::Validation(message));
        }
    }

    let mut graph = Graph::new();
    let mut entity_map: HashMap<(String, String), crate::ConceptId> = HashMap::new();
    let mut role_map: HashMap<(String, String), crate::ConceptId> = HashMap::new();
    let mut resource_map: HashMap<(String, String), crate::ConceptId> = HashMap::new();
    let mut relation_map: HashMap<String, crate::ConceptId> = HashMap::new();

    let default_namespace = ast
        .metadata
        .namespace
        .clone()
        .or_else(|| options.default_namespace.clone())
        .unwrap_or_else(|| "default".to_string());

    // First pass: Register dimensions and units
    {
        use crate::units::{Dimension, Unit, UnitError, UnitRegistry};
        let registry = UnitRegistry::global();
        let mut registry = registry.write().map_err(|e| {
            ParseError::GrammarError(format!("Failed to lock unit registry: {}", e))
        })?;

        for node in &ast.declarations {
            let node = unwrap_export(node);
            match node {
                AstNode::Dimension { name } => {
                    let dim = Dimension::parse(name);
                    registry.register_dimension(dim);
                }
                AstNode::UnitDeclaration {
                    symbol,
                    dimension,
                    factor,
                    base_unit,
                } => {
                    let dim = Dimension::parse(dimension);
                    let unit = Unit::new(
                        symbol.clone(),
                        symbol.clone(),
                        dim,
                        *factor,
                        base_unit.clone(),
                    );
                    match registry.get_unit(symbol) {
                        Ok(existing) => {
                            // Conflict on semantic fields only; a redeclaration
                            // matching a preloaded unit's dimension/factor/base
                            // is compatible even if display names differ.
                            if existing.dimension() != unit.dimension()
                                || existing.base_factor() != unit.base_factor()
                                || existing.base_unit() != unit.base_unit()
                            {
                                return Err(ParseError::GrammarError(format!(
                                    "Conflicting unit '{}' already registered (existing: dimension={}, base_factor={}, base_unit={}; new: dimension={}, base_factor={}, base_unit={})",
                                    symbol,
                                    existing.dimension(),
                                    existing.base_factor(),
                                    existing.base_unit(),
                                    unit.dimension(),
                                    unit.base_factor(),
                                    unit.base_unit(),
                                )));
                            }
                        }
                        Err(UnitError::UnitNotFound(_)) => {
                            registry.register(unit).map_err(|e| {
                                ParseError::GrammarError(format!("Failed to register unit: {}", e))
                            })?;
                        }
                        Err(err) => {
                            return Err(ParseError::GrammarError(format!(
                                "Failed to inspect unit '{}': {}",
                                symbol, err
                            )));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Register patterns with eager regex validation
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Pattern { name, regex } = node {
            let namespace = default_namespace.clone();
            let pattern = Pattern::new(name.clone(), namespace, regex.clone())
                .map_err(ParseError::GrammarError)?;

            graph
                .add_pattern(pattern)
                .map_err(ParseError::GrammarError)?;
        }
    }

    // Register concept changes
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::ConceptChange {
            name,
            from_version,
            to_version,
            migration_policy,
            breaking_change,
        } = node
        {
            let change = ConceptChange::new(
                name.clone(),
                from_version.clone(),
                to_version.clone(),
                migration_policy.clone(),
                *breaking_change,
            );
            graph.add_concept_change(change).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add concept change: {}", e))
            })?;
        }
    }

    // Second pass: Add roles, entities, and resources
    for node in &ast.declarations {
        let node = unwrap_export(node);
        match node {
            AstNode::Role { name, domain } => {
                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
                let key = (namespace.clone(), name.clone());
                if role_map.contains_key(&key) {
                    return Err(ParseError::duplicate_declaration_no_loc(format!(
                        "Role '{}' already declared in namespace '{}'",
                        name, namespace
                    )));
                }

                let role = Role::new_with_namespace(name.clone(), namespace);
                let role_id = role.id().clone();
                graph
                    .add_role(role)
                    .map_err(|e| ParseError::GrammarError(format!("Failed to add role: {}", e)))?;
                role_map.insert(key, role_id);
            }
            AstNode::Entity {
                name,
                domain,
                version,
                annotations,
                body: _,
            } => {
                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
                let key = (namespace.clone(), name.clone());
                if entity_map.contains_key(&key) {
                    return Err(ParseError::duplicate_declaration_no_loc(format!(
                        "Entity '{}' already declared in namespace '{}'",
                        name, namespace
                    )));
                }

                let mut entity = Entity::new_with_namespace(name.clone(), namespace);

                if let Some(v_str) = version {
                    let sem_ver = SemanticVersion::parse(v_str).map_err(|e| {
                        ParseError::GrammarError(format!(
                            "Invalid entity version '{}': {}",
                            v_str, e
                        ))
                    })?;
                    entity = entity.with_version(sem_ver);
                }

                if let Some(replaces_val) = annotations.get("replaces") {
                    if let Some(replaces_str) = replaces_val.as_str() {
                        entity = entity.with_replaces(replaces_str.to_string());
                    }
                }

                if let Some(changes_val) = annotations.get("changes") {
                    if let Some(changes_arr) = changes_val.as_array() {
                        let changes: Vec<String> = changes_arr
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        entity = entity.with_changes(changes);
                    }
                }

                let entity_id = entity.id().clone();
                graph.add_entity(entity).map_err(|e| {
                    ParseError::GrammarError(format!("Failed to add entity: {}", e))
                })?;
                entity_map.insert(key, entity_id);
            }
            AstNode::Resource {
                name,
                unit_name,
                domain,
                ..
            } => {
                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
                let key = (namespace.clone(), name.clone());
                if resource_map.contains_key(&key) {
                    return Err(ParseError::duplicate_declaration_no_loc(format!(
                        "Resource '{}' already declared in namespace '{}'",
                        name, namespace
                    )));
                }

                let unit = unit_from_string(unit_name.as_deref().unwrap_or("units"));
                let resource = Resource::new_with_namespace(name.clone(), unit, namespace);
                let resource_id = resource.id().clone();
                graph.add_resource(resource).map_err(|e| {
                    ParseError::GrammarError(format!("Failed to add resource: {}", e))
                })?;
                resource_map.insert(key, resource_id);
            }
            _ => {}
        }
    }

    // Third pass: Add flows
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Flow {
            resource_name,
            annotations,
            from_entity,
            to_entity,
            quantity,
        } = node
        {
            let from_id = resolve_by_name(&entity_map, from_entity, &default_namespace)?
                .ok_or_else(|| ParseError::undefined_entity_no_loc(from_entity))?;

            let to_id = resolve_by_name(&entity_map, to_entity, &default_namespace)?
                .ok_or_else(|| ParseError::undefined_entity_no_loc(to_entity))?;

            let resource_id = resolve_by_name(&resource_map, resource_name, &default_namespace)?
                .ok_or_else(|| ParseError::undefined_resource_no_loc(resource_name))?;

            let qty = quantity.unwrap_or(Decimal::ZERO);
            let mut flow = Flow::new_with_namespace(
                resource_id.clone(),
                from_id.clone(),
                to_id.clone(),
                qty,
                default_namespace.clone(),
            );

            // Preserve flow annotations (e.g. `@cqrs { "kind": "command" }`)
            // onto the flow's attribute map so downstream projection targets
            // can read them via `ResolvedFlow::annotations`.
            for (key, value) in annotations {
                flow.set_attribute(key.clone(), value.clone());
            }

            graph
                .add_flow(flow)
                .map_err(|e| ParseError::GrammarError(format!("Failed to add flow: {}", e)))?;
        }
    }

    // Fourth pass: Add relations
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Relation {
            name,
            subject_role,
            predicate,
            object_role,
            via_flow,
        } = node
        {
            if relation_map.contains_key(name) {
                return Err(ParseError::duplicate_declaration_no_loc(format!(
                    "Relation '{}' already declared",
                    name
                )));
            }

            let subject_id = resolve_by_name(&role_map, subject_role, &default_namespace)?
                .ok_or_else(|| {
                    ParseError::GrammarError(format!("Undefined subject role '{}'", subject_role))
                })?;

            let object_id = resolve_by_name(&role_map, object_role, &default_namespace)?
                .ok_or_else(|| {
                    ParseError::GrammarError(format!("Undefined object role '{}'", object_role))
                })?;

            let via_flow_id = if let Some(flow_name) = via_flow {
                Some(
                    resolve_by_name(&resource_map, flow_name, &default_namespace)?
                        .ok_or_else(|| ParseError::undefined_resource_no_loc(flow_name))?,
                )
            } else {
                None
            };

            let relation = RelationType::new(
                name.clone(),
                default_namespace.clone(),
                subject_id.clone(),
                predicate.clone(),
                object_id.clone(),
                via_flow_id,
            );

            let relation_id = relation.id().clone();
            graph.add_relation_type(relation).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add relation '{}': {}", name, e))
            })?;
            relation_map.insert(name.clone(), relation_id);
        }
    }

    // Instance pass: Add instances (after entities are created)
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Instance {
            name,
            entity_type,
            fields,
        } = node
        {
            let namespace = default_namespace.clone();
            let mut instance = crate::primitives::Instance::new_with_namespace(
                name.clone(),
                entity_type.clone(),
                namespace,
            );

            // Evaluate and set fields
            for (field_name, field_expr) in fields {
                let value = expression_to_json(field_expr)?;
                instance.set_field(field_name.clone(), value);
            }

            graph.add_entity_instance(instance).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add entity instance '{}': {}", name, e))
            })?;
        }
    }

    // Fifth pass: Add policies
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Policy {
            name,
            version,
            metadata,
            expression,
        } = node
        {
            let namespace = ast
                .metadata
                .namespace
                .as_ref()
                .cloned()
                .or_else(|| options.default_namespace.clone())
                .unwrap_or_else(|| "default".to_string());

            let kind = metadata.kind.as_ref().map(|kind| match kind {
                PolicyKind::Constraint => CorePolicyKind::Constraint,
                PolicyKind::Derivation => CorePolicyKind::Derivation,
                PolicyKind::Obligation => CorePolicyKind::Obligation,
            });

            let modality = metadata.modality.as_ref().map(|modality| match modality {
                PolicyModality::Obligation => CorePolicyModality::Obligation,
                PolicyModality::Prohibition => CorePolicyModality::Prohibition,
                PolicyModality::Permission => CorePolicyModality::Permission,
            });

            let mut policy =
                Policy::new_with_namespace(name.clone(), namespace, expression.clone())
                    .with_metadata(
                        kind,
                        modality,
                        metadata.priority,
                        metadata.rationale.clone(),
                        metadata.tags.clone(),
                    );

            let version_to_apply = version
                .as_ref()
                .cloned()
                .or_else(|| ast.metadata.version.clone());

            if let Some(version_str) = version_to_apply {
                let semantic_version = SemanticVersion::parse(&version_str).map_err(|err| {
                    ParseError::GrammarError(format!(
                        "Invalid policy version '{}': {}",
                        version_str, err
                    ))
                })?;
                policy = policy.with_version(semantic_version);
            }

            graph.add_policy(policy).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add policy '{}': {}", name, e))
            })?;
        }
    }

    // Sixth pass: Add metrics
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Metric {
            name,
            expression,
            metadata,
        } = node
        {
            let namespace = ast
                .metadata
                .namespace
                .as_ref()
                .cloned()
                .or_else(|| options.default_namespace.clone())
                .unwrap_or_else(|| "default".to_string());

            let mut metric =
                crate::primitives::Metric::new(name.clone(), namespace, expression.clone());

            if let Some(duration) = metadata.refresh_interval {
                metric = metric.with_refresh_interval(duration);
            }

            if let Some(unit) = &metadata.unit {
                metric = metric.with_unit(unit.clone());
            }

            if let Some(threshold) = metadata.threshold {
                metric = metric.with_threshold(threshold);
            }

            if let Some(severity) = metadata.severity.clone() {
                metric = metric.with_severity(severity);
            }

            if let Some(target) = metadata.target {
                metric = metric.with_target(target);
            }

            if let Some(duration) = metadata.window {
                metric = metric.with_window(duration);
            }

            graph.add_metric(metric).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add metric '{}': {}", name, e))
            })?;
        }
    }

    // Seventh pass: Add mappings
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::MappingDecl {
            name,
            target,
            rules,
        } = node
        {
            let namespace = ast
                .metadata
                .namespace
                .clone()
                .or_else(|| options.default_namespace.clone())
                .unwrap_or_else(|| "default".to_string());
            let mapping = crate::primitives::MappingContract::new(
                crate::ConceptId::from_concept(&namespace, name),
                name.clone(),
                namespace,
                target.clone(),
                rules.clone(),
            );
            graph
                .add_mapping(mapping)
                .map_err(|e| ParseError::GrammarError(format!("Failed to add mapping: {}", e)))?;
        }
    }

    // Eighth pass: Add projections
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::ProjectionDecl {
            name,
            target,
            overrides,
        } = node
        {
            let namespace = ast
                .metadata
                .namespace
                .clone()
                .or_else(|| options.default_namespace.clone())
                .unwrap_or_else(|| "default".to_string());
            let projection = crate::primitives::ProjectionContract::new(
                crate::ConceptId::from_concept(&namespace, name),
                name.clone(),
                namespace,
                target.clone(),
                overrides.clone(),
            );
            graph.add_projection(projection).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add projection: {}", e))
            })?;
        }
    }

    Ok(graph)
}

/// Parse metric declaration
fn parse_metric(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected metric name".to_string()))?,
    )?;

    let mut metadata = MetricMetadata {
        refresh_interval: None,
        unit: None,
        threshold: None,
        severity: None,
        target: None,
        window: None,
    };

    let mut expression_pair = None;

    for part in inner {
        match part.as_rule() {
            Rule::metric_annotation => {
                let mut annotation_inner = part.into_inner();
                let key_pair = annotation_inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected annotation key".to_string())
                })?;

                match key_pair.as_rule() {
                    Rule::ma_refresh_interval => {
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected refresh interval value".to_string())
                        })?;
                        let value = parse_number_i64(value_pair)?;
                        let unit_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected refresh interval unit".to_string())
                        })?;
                        let unit = parse_string_literal(unit_pair.clone())?;
                        let duration = parse_duration_with_unit(value, &unit, unit_pair.as_span())?;
                        metadata.refresh_interval = Some(duration);
                    }
                    Rule::ma_unit => {
                        let unit_pair = annotation_inner
                            .next()
                            .ok_or_else(|| ParseError::GrammarError("Expected unit".to_string()))?;
                        metadata.unit = Some(parse_string_literal(unit_pair)?);
                    }
                    Rule::ma_threshold => {
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected threshold value".to_string())
                        })?;
                        metadata.threshold = Some(parse_decimal(value_pair)?);
                    }
                    Rule::ma_severity => {
                        let severity_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected severity".to_string())
                        })?;
                        let severity_str = parse_string_literal(severity_pair.clone())?;
                        let severity =
                            parse_severity_value(&severity_str, severity_pair.as_span())?;
                        metadata.severity = Some(severity);
                    }
                    Rule::ma_target => {
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected target value".to_string())
                        })?;
                        metadata.target = Some(parse_decimal(value_pair)?);
                    }
                    Rule::ma_window => {
                        let value_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected window value".to_string())
                        })?;
                        let value = parse_number_i64(value_pair)?;
                        let unit_pair = annotation_inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected window unit".to_string())
                        })?;
                        let unit = parse_string_literal(unit_pair.clone())?;
                        let duration = parse_duration_with_unit(value, &unit, unit_pair.as_span())?;
                        metadata.window = Some(duration);
                    }
                    _ => {
                        let (line, column) = key_pair.as_span().start_pos().line_col();
                        return Err(ParseError::GrammarError(format!(
                            "Unknown metric annotation '{}' at {}:{}",
                            key_pair.as_str(),
                            line,
                            column
                        )));
                    }
                }
            }
            Rule::expression => {
                expression_pair = Some(part);
            }
            _ => {}
        }
    }

    let expression = parse_expression(
        expression_pair
            .ok_or_else(|| ParseError::GrammarError("Expected metric expression".to_string()))?,
    )?;

    Ok(AstNode::Metric {
        name,
        expression,
        metadata,
    })
}

fn parse_duration_with_unit(value: i64, unit: &str, span: Span<'_>) -> ParseResult<Duration> {
    let normalized_unit = unit.to_ascii_lowercase();
    let multiplier = match normalized_unit.as_str() {
        "second" => Some(1),
        "seconds" | "s" => Some(1),
        "minute" => Some(60),
        "minutes" | "m" => Some(60),
        "hour" => Some(60 * 60),
        "hours" | "h" => Some(60 * 60),
        "day" => Some(60 * 60 * 24),
        "days" | "d" => Some(60 * 60 * 24),
        _ => None,
    }
    .ok_or_else(|| {
        let (line, column) = span.start_pos().line_col();
        ParseError::GrammarError(format!(
            "Invalid duration unit '{}' at {}:{} (allowed: second(s)/s, minute(s)/m, hour(s)/h, day(s)/d)",
            unit, line, column
        ))
    })?;

    let total_seconds = value.checked_mul(multiplier).ok_or_else(|| {
        let (line, column) = span.start_pos().line_col();
        ParseError::GrammarError(format!(
            "Duration overflow for value {} {} at {}:{}",
            value, unit, line, column
        ))
    })?;

    Ok(Duration::seconds(total_seconds))
}

fn parse_severity_value(value: &str, span: Span<'_>) -> ParseResult<Severity> {
    let normalized = value.to_ascii_lowercase();
    match normalized.as_str() {
        "info" => Ok(Severity::Info),
        "warning" => Ok(Severity::Warning),
        "error" => Ok(Severity::Error),
        "critical" => Ok(Severity::Critical),
        _ => {
            let (line, column) = span.start_pos().line_col();
            Err(ParseError::GrammarError(format!(
                "Unknown severity '{}' at {}:{} (expected one of: info, warning, error, critical)",
                value, line, column
            )))
        }
    }
}

fn parse_number_i64(pair: Pair<Rule>) -> ParseResult<i64> {
    let s = pair.as_str();
    s.parse::<i64>()
        .map_err(|_| ParseError::GrammarError(format!("Invalid integer: {}", s)))
}
