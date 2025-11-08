use pest::Parser;
use pest::iterators::{Pair, Pairs};
use crate::parser::{SeaParser, Rule};
use crate::parser::error::{ParseError, ParseResult};
use crate::policy::{Expression, BinaryOp, UnaryOp, Quantifier as PolicyQuantifier, AggregateFunction};
use crate::graph::Graph;
use crate::primitives::{Entity, Resource, Flow};
use crate::units::unit_from_string;
use rust_decimal::Decimal;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// Abstract Syntax Tree for SEA DSL
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
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
    Policy {
        name: String,
        version: Option<String>,
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
    let mut declarations = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for inner in pair.into_inner() {
                    if let Rule::declaration = inner.as_rule() {
                        for decl in inner.into_inner() {
                            let node = parse_declaration(decl)?;
                            declarations.push(node);
                        }
                    }
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(Ast { declarations })
}

/// Parse a single declaration
fn parse_declaration(pair: Pair<Rule>) -> ParseResult<AstNode> {
    match pair.as_rule() {
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

/// Parse entity declaration
fn parse_entity(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let name = parse_name(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected entity name".to_string())
    })?)?;

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

    let name = parse_name(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected resource name".to_string())
    })?)?;

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
                    "Unexpected token in resource declaration".to_string()
                ));
            }
        }
    }

    Ok(AstNode::Resource { name, unit_name, domain })
}

/// Parse flow declaration
fn parse_flow(pair: Pair<Rule>) -> ParseResult<AstNode> {
    let mut inner = pair.into_inner();

    let resource_name = parse_string_literal(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected resource name".to_string())
    })?)?;

    let from_entity = parse_string_literal(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected from entity".to_string())
    })?)?;

    let to_entity = parse_string_literal(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected to entity".to_string())
    })?)?;

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

    let name = parse_identifier(inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected policy name".to_string())
    })?)?;

    let mut version: Option<String> = None;
    let mut next_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("Expected policy version or expression".to_string())
    })?;

    if next_pair.as_rule() == Rule::version {
        version = Some(next_pair.as_str().to_string());
        next_pair = inner.next().ok_or_else(|| {
            ParseError::GrammarError("Expected policy expression".to_string())
        })?;
    }

    let expression = parse_expression(next_pair)?;

    Ok(AstNode::Policy { name, version, expression })
}

/// Parse expression
fn parse_expression(pair: Pair<Rule>) -> ParseResult<Expression> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("Empty expression".to_string())
            })?;
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

/// Parse OR expression
fn parse_or_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected left expression in OR".to_string()))?)?;

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
    let mut left = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected left expression in AND".to_string()))?)?;

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
    let first = inner.next().ok_or_else(|| ParseError::GrammarError("Expected expression in NOT".to_string()))?;

    match first.as_rule() {
        Rule::not_expr => {
            // NOT operation
            let expr = parse_expression(first)?;
            Ok(Expression::Unary {
                op: UnaryOp::Not,
                operand: Box::new(expr),
            })
        }
        _ => parse_expression(first),
    }
}

/// Parse comparison expression
fn parse_comparison_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected left expression in comparison".to_string()))?)?;

    if let Some(op_pair) = inner.next() {
        let op = parse_comparison_op(op_pair)?;
        let right = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected right expression in comparison".to_string()))?)?;
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
        _ => Err(ParseError::InvalidExpression(format!(
            "Unknown comparison operator: {}",
            op_str
        ))),
    }
}

/// Parse additive expression
fn parse_additive_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected left expression in additive".to_string()))?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => BinaryOp::Plus,
            "-" => BinaryOp::Minus,
            _ => return Err(ParseError::InvalidExpression("Invalid additive operator".to_string())),
        };
        let right = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected right expression in additive".to_string()))?)?;
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
    let mut left = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected left expression in multiplicative".to_string()))?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "*" => BinaryOp::Multiply,
            "/" => BinaryOp::Divide,
            _ => return Err(ParseError::InvalidExpression("Invalid multiplicative operator".to_string())),
        };
        let right = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected right expression in multiplicative".to_string()))?)?;
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
    let first = inner.next().ok_or_else(|| ParseError::GrammarError("Expected expression in unary".to_string()))?;

    if first.as_str() == "-" {
        // Unary minus
        let expr = parse_expression(inner.next().ok_or_else(|| ParseError::GrammarError("Expected expression after unary minus".to_string()))?)?;
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
    let inner = pair.into_inner().next().ok_or_else(|| ParseError::GrammarError("Expected inner expression in primary".to_string()))?;

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

    let function_pair = inner.next()
        .ok_or_else(|| ParseError::GrammarError("Expected aggregate function in aggregation expression".to_string()))?;
    let function = parse_aggregate_fn(function_pair)?;

    let collection_pair = inner.next()
        .ok_or_else(|| ParseError::GrammarError("Expected collection in aggregation expression".to_string()))?;
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

    let quantifier = parse_quantifier(inner.next().unwrap())?;
    let variable = parse_identifier(inner.next().unwrap())?;
    let collection = parse_collection(inner.next().unwrap())?;
    let condition = parse_expression(inner.next().unwrap())?;

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
    let object = parse_identifier(inner.next().unwrap())?;
    let member = parse_identifier(inner.next().unwrap())?;

    Ok(Expression::MemberAccess {
        object,
        member,
    })
}

/// Parse literal expression
fn parse_literal_expr(pair: Pair<Rule>) -> ParseResult<Expression> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::string_literal => {
            let s = parse_string_literal(inner)?;
            Ok(Expression::Literal(JsonValue::String(s)))
        }
        Rule::multiline_string => {
            let s = parse_multiline_string(inner)?;
            Ok(Expression::Literal(JsonValue::String(s)))
        }
        Rule::number => {
            let n = parse_decimal(inner)?;
            Ok(Expression::Literal(JsonValue::String(n.to_string())))
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
    let inner = pair.into_inner().next()
        .ok_or_else(|| ParseError::GrammarError("Expected inner token for name but got empty pair".to_string()))?;
    match inner.as_rule() {
        Rule::string_literal => parse_string_literal(inner),
        Rule::multiline_string => parse_multiline_string(inner),
        _ => Err(ParseError::GrammarError(format!("Expected string or multiline string for name, got {:?}", inner.as_rule())))
    }
}

/// Parse string literal (removes quotes)
fn parse_string_literal(pair: Pair<Rule>) -> ParseResult<String> {
    let s = pair.as_str();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Err(ParseError::GrammarError(format!("Invalid string literal: {}", s)))
    }
}

/// Parse multiline string (removes triple quotes)
fn parse_multiline_string(pair: Pair<Rule>) -> ParseResult<String> {
    let s = pair.as_str();
    if s.len() >= 6 && s.starts_with("\"\"\"") && s.ends_with("\"\"\"") {
        Ok(s[3..s.len() - 3].to_string())
    } else {
        Err(ParseError::GrammarError(format!("Invalid multiline string: {}", s)))
    }
}

/// Parse identifier
fn parse_identifier(pair: Pair<Rule>) -> ParseResult<String> {
    Ok(pair.as_str().to_string())
}

/// Parse number as i32
fn parse_number(pair: Pair<Rule>) -> ParseResult<i32> {
    pair.as_str().parse().map_err(|_| {
        ParseError::InvalidQuantity(format!("Invalid number: {}", pair.as_str()))
    })
}

/// Parse number as Decimal
fn parse_decimal(pair: Pair<Rule>) -> ParseResult<Decimal> {
    pair.as_str().parse().map_err(|_| {
        ParseError::InvalidQuantity(format!("Invalid decimal: {}", pair.as_str()))
    })
}

/// Convert AST to Graph
pub fn ast_to_graph(ast: Ast) -> ParseResult<Graph> {
    let mut graph = Graph::new();
    let mut entity_map = HashMap::new();
    let mut resource_map = HashMap::new();

    // First pass: Add entities and resources
    for node in &ast.declarations {
        match node {
            AstNode::Entity { name, domain } => {
                if entity_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration(format!(
                        "Entity '{}' already declared",
                        name
                    )));
                }

                let entity = if let Some(d) = domain {
                    Entity::new_with_namespace(name.clone(), d.clone())
                } else {
                    Entity::new(name.clone())
                };
                let entity_id = entity.id().clone();
                graph.add_entity(entity).map_err(|e| {
                    ParseError::GrammarError(format!("Failed to add entity: {}", e))
                })?;
                entity_map.insert(name.clone(), entity_id);
            }
            AstNode::Resource { name, unit_name, domain } => {
                if resource_map.contains_key(name) {
                    return Err(ParseError::duplicate_declaration(format!(
                        "Resource '{}' already declared",
                        name
                    )));
                }

                let resource = match (unit_name, domain) {
                    (Some(unit_str), Some(d)) => {
                        let unit = unit_from_string(unit_str.clone());
                        Resource::new_with_namespace(name.clone(), unit, d.clone())
                    },
                    (Some(unit_str), None) => {
                        let unit = unit_from_string(unit_str.clone());
                        Resource::new(name.clone(), unit)
                    },
                    (None, Some(d)) => {
                        let unit = unit_from_string("units");
                        Resource::new_with_namespace(name.clone(), unit, d.clone())
                    },
                    (None, None) => {
                        let unit = unit_from_string("units");
                        Resource::new(name.clone(), unit)
                    },
                };
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

            graph.add_flow(flow).map_err(|e| {
                ParseError::GrammarError(format!("Failed to add flow: {}", e))
            })?;
        }
    }

    // Third pass: Add policies (not yet implemented - skip for now)
    for node in &ast.declarations {
        if let AstNode::Policy { .. } = node {
            // TODO: Add policy support to Graph
            // For now, just skip policies to avoid breaking existing tests
            continue;
        }
    }

    Ok(graph)
}
