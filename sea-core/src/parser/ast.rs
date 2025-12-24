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

/// Abstract Syntax Tree for SEA DSL
/// Abstract Syntax Tree for SEA DSL
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

/// AST Node types
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Export(Box<Spanned<AstNode>>),
    Entity {
        name: String,
        version: Option<String>,
        annotations: HashMap<String, JsonValue>,
        domain: Option<String>,
    },
    Resource {
        name: String,
        annotations: HashMap<String, JsonValue>,
        unit_name: Option<String>,
        domain: Option<String>,
    },
    Flow {
        resource_name: String,
        annotations: HashMap<String, JsonValue>,
        from_entity: String,
        to_entity: String,
        quantity: Option<i32>,
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
        Rule::import_specifier | Rule::import_named => {
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

    for part in inner {
        match part.as_rule() {
            Rule::version => {
                version = Some(part.as_str().to_string());
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
                }
            }
            Rule::string_literal => {
                // These are from_entity and to_entity in order
                let parsed = parse_string_literal(part)?;
                if from_entity.is_none() {
                    from_entity = Some(parsed);
                } else if to_entity.is_none() {
                    to_entity = Some(parsed);
                }
            }
            Rule::number => {
                quantity = Some(parse_number(part)?);
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

    let domain = if let Some(domain_pair) = inner.next() {
        Some(parse_identifier(domain_pair)?)
    } else {
        None
    };

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
        let value = match value_pair.as_rule() {
            Rule::string_literal => JsonValue::String(parse_string_literal(value_pair)?),
            Rule::boolean => JsonValue::Bool(value_pair.as_str().eq_ignore_ascii_case("true")),
            Rule::number => {
                let d = parse_decimal(value_pair)?;
                // Parse the decimal string as f64 to create a JSON Number
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
                    ParseError::InvalidQuantity(format!(
                        "Cannot create JSON Number from decimal {}",
                        d
                    ))
                })?;
                JsonValue::Number(num)
            }
            _ => {
                return Err(ParseError::GrammarError(
                    "Unexpected object field value".to_string(),
                ))
            }
        };
        map.insert(key, value);
    }
    Ok(JsonValue::Object(map))
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
    let mut entity_map = HashMap::new();
    let mut role_map = HashMap::new();
    let mut resource_map = HashMap::new();
    let mut relation_map = HashMap::new();

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
                            if existing != &unit {
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
                if role_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration_no_loc(format!(
                        "Role '{}' already declared",
                        name
                    )));
                }

                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
                let role = Role::new_with_namespace(name.clone(), namespace);
                let role_id = role.id().clone();
                graph
                    .add_role(role)
                    .map_err(|e| ParseError::GrammarError(format!("Failed to add role: {}", e)))?;
                role_map.insert(name.clone(), role_id);
            }
            AstNode::Entity {
                name,
                domain,
                version,
                annotations,
            } => {
                if entity_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration_no_loc(format!(
                        "Entity '{}' already declared",
                        name
                    )));
                }

                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
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
                entity_map.insert(name.clone(), entity_id);
            }
            AstNode::Resource {
                name,
                unit_name,
                domain,
                ..
            } => {
                if resource_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration_no_loc(format!(
                        "Resource '{}' already declared",
                        name
                    )));
                }

                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
                let unit = unit_from_string(unit_name.as_deref().unwrap_or("units"));
                let resource = Resource::new_with_namespace(name.clone(), unit, namespace);
                let resource_id = resource.id().clone();
                graph.add_resource(resource).map_err(|e| {
                    ParseError::GrammarError(format!("Failed to add resource: {}", e))
                })?;
                resource_map.insert(name.clone(), resource_id);
            }
            _ => {}
        }
    }

    // Third pass: Add flows
    for node in &ast.declarations {
        let node = unwrap_export(node);
        if let AstNode::Flow {
            resource_name,
            from_entity,
            to_entity,
            quantity,
            ..
        } = node
        {
            let from_id = entity_map
                .get(from_entity)
                .ok_or_else(|| ParseError::undefined_entity_no_loc(from_entity))?;

            let to_id = entity_map
                .get(to_entity)
                .ok_or_else(|| ParseError::undefined_entity_no_loc(to_entity))?;

            let resource_id = resource_map
                .get(resource_name)
                .ok_or_else(|| ParseError::undefined_resource_no_loc(resource_name))?;

            let qty = quantity.map(Decimal::from).unwrap_or(Decimal::ZERO);
            let flow = Flow::new(resource_id.clone(), from_id.clone(), to_id.clone(), qty);

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

            let subject_id = role_map.get(subject_role).ok_or_else(|| {
                ParseError::GrammarError(format!("Undefined subject role '{}'", subject_role))
            })?;

            let object_id = role_map.get(object_role).ok_or_else(|| {
                ParseError::GrammarError(format!("Undefined object role '{}'", object_role))
            })?;

            let via_flow_id = if let Some(flow_name) = via_flow {
                Some(
                    resource_map
                        .get(flow_name)
                        .cloned()
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
