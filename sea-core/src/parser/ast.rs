use crate::graph::Graph;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::{ParseOptions, Rule, SeaParser};
use crate::policy::{
    AggregateFunction, BinaryOp, Expression, Policy, PolicyKind as CorePolicyKind,
    PolicyModality as CorePolicyModality, Quantifier as PolicyQuantifier, UnaryOp,
};
use crate::primitives::{Entity, Flow, Resource};
use crate::units::unit_from_string;
use crate::SemanticVersion;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
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
        Rule::policy_decl => parse_policy(pair),
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

    let domain = if let Some(domain_pair) = inner.next() {
        Some(parse_identifier(domain_pair)?)
    } else {
        None
    };

    Ok(AstNode::Entity { name, domain })
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
        _ if op_str.eq_ignore_ascii_case("before") => Ok(BinaryOp::Before),
        _ if op_str.eq_ignore_ascii_case("after") => Ok(BinaryOp::After),
        _ if op_str.eq_ignore_ascii_case("during") => Ok(BinaryOp::During),
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
    let collection = parse_collection(collection_pair)?;

    let mut field: Option<String> = None;
    let mut filter: Option<Expression> = None;

    // Parse optional field and filter
    for item in inner {
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

/// Convert AST to Graph
pub fn ast_to_graph(ast: Ast) -> ParseResult<Graph> {
    ast_to_graph_with_options(ast, &ParseOptions::default())
}

pub fn ast_to_graph_with_options(ast: Ast, options: &ParseOptions) -> ParseResult<Graph> {
    let mut graph = Graph::new();
    let mut entity_map = HashMap::new();
    let mut resource_map = HashMap::new();

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
                let dim = Dimension::Custom(name.clone());
                UnitRegistry::global().register_dimension(dim);
            }
            AstNode::UnitDeclaration {
                symbol,
                dimension,
                factor,
                base_unit,
            } => {
                use crate::units::{Dimension, Unit, UnitRegistry};
                let dim = Dimension::Custom(dimension.clone());
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

    // Second pass: Add entities and resources
    for node in &ast.declarations {
        match node {
            AstNode::Entity { name, domain } => {
                if entity_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration(format!(
                        "Entity '{}' already declared",
                        name
                    )));
                }

                let namespace = domain.as_ref().unwrap_or(&default_namespace).clone();
                let entity = Entity::new_with_namespace(name.clone(), namespace);
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

    // Second pass: Add flows
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

    // Third pass: Add policies
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
