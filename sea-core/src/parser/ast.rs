use crate::graph::Graph;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::{ParseOptions, Rule, SeaParser};
use crate::patterns::Pattern;
use crate::policy::{
    AggregateFunction, BinaryOp, Expression, Policy, PolicyKind as CorePolicyKind,
    PolicyModality as CorePolicyModality, Quantifier as PolicyQuantifier, UnaryOp, WindowSpec,
};
use crate::primitives::{ConceptChange, Entity, Flow, RelationType, Resource, Role};
use crate::units::unit_from_string;
use crate::SemanticVersion;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use serde_json::json;
use std::collections::HashMap;

/// File-level metadata from header annotations
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileMetadata {
    pub namespace: Option<String>,
    pub version: Option<String>,
    pub owner: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyModality {
    Obligation,
    Prohibition,
    Permission,
}

/// Abstract Syntax Tree for SEA DSL
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub metadata: FileMetadata,
    pub declarations: Vec<AstNode>,
}

/// AST Node types
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Entity {
        name: String,
        version: Option<String>,
        annotations: HashMap<String, JsonValue>,
        domain: Option<String>,
    },
    Resource {
        name: String,
        unit_name: Option<String>,
        domain: Option<String>,
    },
    Flow {
        resource_name: String,
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
        if annotation.as_rule() == Rule::annotation {
            let mut inner = annotation.into_inner();
            let name = inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected annotation name".to_string()))?;
            let value = inner
                .next()
                .ok_or_else(|| ParseError::GrammarError("Expected annotation value".to_string()))?;

            let name_str = name.as_str().to_lowercase();
            let value_str = parse_string_literal(value)?;

            match name_str.as_str() {
                "namespace" => metadata.namespace = Some(value_str),
                "version" => metadata.version = Some(value_str),
                "owner" => metadata.owner = Some(value_str),
                _ => {
                    return Err(ParseError::GrammarError(format!(
                        "Unknown annotation: {}",
                        name_str
                    )))
                }
            }
        }
    }

    Ok(metadata)
}

/// Parse a single declaration
fn parse_declaration(pair: Pair<Rule>) -> ParseResult<AstNode> {
    match pair.as_rule() {
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
        _ => Err(ParseError::GrammarError(format!(
            "Unexpected rule: {:?}",
            pair.as_rule()
        ))),
    }
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

    let mut unit_name = None;
    let mut domain = None;

    // The grammar produces these patterns (with in_keyword as a separate token):
    // 1. string_literal + identifier + in_keyword + identifier = unit + domain
    // 2. string_literal + in_keyword + identifier = domain only
    // 3. string_literal + identifier = unit only
    // 4. string_literal only = nothing

    if let Some(first) = inner.next() {
        match first.as_rule() {
            Rule::in_keyword => {
                // Case 2: in_keyword + identifier (domain only)
                domain = Some(parse_identifier(inner.next().ok_or_else(|| {
                    ParseError::GrammarError("Expected domain after 'in'".to_string())
                })?)?);
            }
            Rule::identifier => {
                // Case 1 or 3: starts with identifier
                unit_name = Some(parse_identifier(first)?);

                // Check if followed by in_keyword + identifier
                if let Some(second) = inner.next() {
                    if second.as_rule() == Rule::in_keyword {
                        // Case 1: identifier + in_keyword + identifier
                        domain = Some(parse_identifier(inner.next().ok_or_else(|| {
                            ParseError::GrammarError("Expected domain after 'in'".to_string())
                        })?)?);
                    }
                    // If second is not in_keyword, it's unexpected (grammar shouldn't allow this)
                }
                // If no second token, it's case 3 (unit only)
            }
            _ => {
                return Err(ParseError::GrammarError(
                    "Unexpected token in resource declaration".to_string(),
                ));
            }
        }
    }

    Ok(AstNode::Resource {
        name,
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

    let from_entity = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected from entity".to_string()))?,
    )?;

    let to_entity = parse_string_literal(
        inner
            .next()
            .ok_or_else(|| ParseError::GrammarError("Expected to entity".to_string()))?,
    )?;

    let quantity = if let Some(qty_pair) = inner.next() {
        Some(parse_number(qty_pair)?)
    } else {
        None
    };

    Ok(AstNode::Flow {
        resource_name,
        from_entity,
        to_entity,
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
                    let mut field_inner = field_pair.into_inner();
                    
                    let field_name = parse_identifier(
                        field_inner
                            .next()
                            .ok_or_else(|| ParseError::GrammarError("Expected field name".to_string()))?,
                    )?;
                    
                    let field_value = parse_expression(
                        field_inner
                            .next()
                            .ok_or_else(|| ParseError::GrammarError("Expected field value".to_string()))?,
                    )?;
                    
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

/// Convert an Expression to a JSON Value for instance fields
fn expression_to_json(expr: &Expression) -> ParseResult<JsonValue> {
    match expr {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::Variable(name) => Ok(JsonValue::String(name.clone())),
        Expression::QuantityLiteral { value, unit } => {
            Ok(json!({
                "value": value.to_string(),
                "unit": unit
            }))
        }
        Expression::TimeLiteral(timestamp) => Ok(JsonValue::String(timestamp.clone())),
        _ => {
            // For complex expressions, convert to string representation
            Ok(JsonValue::String(format!("{:?}", expr)))
        }
    }
}

/// Convert AST to Graph
pub fn ast_to_graph(ast: Ast) -> ParseResult<Graph> {
    ast_to_graph_with_options(ast, &ParseOptions::default())
}

pub fn ast_to_graph_with_options(ast: Ast, options: &ParseOptions) -> ParseResult<Graph> {
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
    for node in &ast.declarations {
        match node {
            AstNode::Dimension { name } => {
                use crate::units::{Dimension, UnitRegistry};
                let dim = Dimension::parse(name);
                UnitRegistry::global().register_dimension(dim);
            }
            AstNode::UnitDeclaration {
                symbol,
                dimension,
                factor,
                base_unit,
            } => {
                use crate::units::{Dimension, Unit, UnitRegistry};
                let dim = Dimension::parse(dimension);
                let unit = Unit::new(
                    symbol.clone(),
                    symbol.clone(),
                    dim,
                    *factor,
                    base_unit.clone(),
                );
                UnitRegistry::global().register(unit).map_err(|e| {
                    ParseError::GrammarError(format!("Failed to register unit: {}", e))
                })?;
            }
            _ => {}
        }
    }

    // Register patterns with eager regex validation
    for node in &ast.declarations {
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
        match node {
            AstNode::Role { name, domain } => {
                if role_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration(format!(
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
                    return Err(ParseError::duplicate_declaration(format!(
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
            } => {
                if resource_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration(format!(
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
        if let AstNode::Flow {
            resource_name,
            from_entity,
            to_entity,
            quantity,
        } = node
        {
            let from_id = entity_map
                .get(from_entity)
                .ok_or_else(|| ParseError::undefined_entity(from_entity))?;

            let to_id = entity_map
                .get(to_entity)
                .ok_or_else(|| ParseError::undefined_entity(to_entity))?;

            let resource_id = resource_map
                .get(resource_name)
                .ok_or_else(|| ParseError::undefined_resource(resource_name))?;

            let qty = quantity.map(Decimal::from).unwrap_or(Decimal::ZERO);
            let flow = Flow::new(resource_id.clone(), from_id.clone(), to_id.clone(), qty);

            graph
                .add_flow(flow)
                .map_err(|e| ParseError::GrammarError(format!("Failed to add flow: {}", e)))?;
        }
    }

    // Fourth pass: Add relations
    for node in &ast.declarations {
        if let AstNode::Relation {
            name,
            subject_role,
            predicate,
            object_role,
            via_flow,
        } = node
        {
            if relation_map.contains_key(name) {
                return Err(ParseError::duplicate_declaration(format!(
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
                        .ok_or_else(|| ParseError::undefined_resource(flow_name))?,
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

    Ok(graph)
}
