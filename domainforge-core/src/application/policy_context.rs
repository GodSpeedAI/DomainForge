//! Application-owned typed policy evaluation (reference §4 closed subset).
//!
//! This evaluator never touches Graph evaluation: it resolves facts only from
//! the typed [`ApplicationPolicyContext`]. Missing facts, `UNKNOWN`, and
//! evaluator errors fail closed — the caller maps `False`, `Unknown`, and
//! `Error` to the bound failure code.

use crate::application::contract::{ActorRef, ApplicationPolicyContext, TypedValue};
use crate::concept_id::ConceptId;
use crate::policy::{BinaryOp, Expression, UnaryOp};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Three-valued outcome of one precondition evaluation, plus the fail-closed
/// error channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationResult {
    True,
    False,
    Unknown,
    Error(String),
}

/// Evaluate one bound constraint-policy expression against the typed context.
pub fn evaluate_precondition(
    expression: &Expression,
    context: &ApplicationPolicyContext,
) -> EvaluationResult {
    match eval(expression, context) {
        Ok(Val::Truth(Some(true))) => EvaluationResult::True,
        Ok(Val::Truth(Some(false))) => EvaluationResult::False,
        Ok(Val::Truth(None) | Val::Unknown) => EvaluationResult::Unknown,
        Ok(_) => EvaluationResult::Error(
            "policy expression did not evaluate to a truth value".to_string(),
        ),
        Err(message) => EvaluationResult::Error(message),
    }
}

/// Validate an expression against the closed v0.1 executable subset.
/// Returns APP007 defect strings; an empty list means evaluable.
pub(crate) fn validate_policy_subset(
    expr: &Expression,
    input_fields: &[&str],
    state_fields: Option<&[&str]>,
    resolve_role: &mut dyn FnMut(&str) -> bool,
) -> Vec<String> {
    let mut defects = Vec::new();
    walk(expr, input_fields, state_fields, resolve_role, &mut defects);
    defects
}

fn is_actor_access(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(name) if name == "actor")
        || matches!(
            expr,
            Expression::MemberAccess { object, member } if object == "actor" && member == "role"
        )
}

fn walk(
    expr: &Expression,
    input_fields: &[&str],
    state_fields: Option<&[&str]>,
    resolve_role: &mut dyn FnMut(&str) -> bool,
    defects: &mut Vec<String>,
) {
    match expr {
        Expression::Literal(value) => {
            if value.is_null() || value.is_array() || value.is_object() {
                defects.push("literal is not a scalar".to_string());
            }
        }
        // ponytail: quantity literals need unit resolution the runtime context
        // cannot perform; a bare number adopts the field's declared unit.
        Expression::QuantityLiteral { unit, .. } => defects.push(format!(
            "quantity literal with unit '{unit}' is not evaluable from the typed context; \
             compare a bare number in the field's declared unit"
        )),
        Expression::Variable(name) => {
            if name != "actor" && !input_fields.contains(&name.as_str()) {
                defects.push(format!(
                    "'{name}' does not resolve to a declared input field"
                ));
            }
        }
        Expression::MemberAccess { object, member } => match object.as_str() {
            "input" => {
                if !input_fields.contains(&member.as_str()) {
                    defects.push(format!(
                        "'input.{member}' does not resolve to a declared input field"
                    ));
                }
            }
            "state" => match state_fields {
                Some(fields) if fields.contains(&member.as_str()) => {}
                _ => defects.push(format!(
                    "'state.{member}' does not resolve to a declared state field"
                )),
            },
            "actor" if member == "role" => {}
            _ => defects.push(format!(
                "member access '{object}.{member}' is outside the executable subset"
            )),
        },
        Expression::RoleReference { role } => defects.push(format!(
            "role<{role}> is legal only opposite actor.role in an equality comparison"
        )),
        Expression::Binary { op, left, right } => {
            use BinaryOp as B;
            match op {
                B::And
                | B::Or
                | B::Equal
                | B::NotEqual
                | B::GreaterThan
                | B::LessThan
                | B::GreaterThanOrEqual
                | B::LessThanOrEqual
                | B::Plus
                | B::Minus
                | B::Multiply
                | B::Divide => {}
                other => defects.push(format!("operator {other} is outside the executable subset")),
            }
            let role_equality = matches!(op, B::Equal | B::NotEqual);
            for (side, other_side) in [(left, right), (right, left)] {
                if let Expression::RoleReference { role } = side.as_ref() {
                    if role_equality && is_actor_access(other_side) {
                        if !resolve_role(role) {
                            defects
                                .push(format!("role<{role}> does not resolve to a declared role"));
                        }
                    } else {
                        defects.push(format!(
                            "role<{role}> is legal only opposite actor.role in an equality comparison"
                        ));
                    }
                } else {
                    walk(side, input_fields, state_fields, resolve_role, defects);
                }
            }
        }
        Expression::Unary { operand, .. } => {
            walk(operand, input_fields, state_fields, resolve_role, defects);
        }
        other => defects.push(format!(
            "expression node {other} is outside the executable subset"
        )),
    }
}

// ---- runtime evaluation ----

#[derive(Debug, Clone, PartialEq)]
enum Val {
    Truth(Option<bool>),
    Num(Decimal),
    Qty(Decimal, ConceptId),
    Str(String),
    Role(ConceptId),
    Uuid(uuid::Uuid),
    Unknown,
}

fn typed_to_val(value: &TypedValue) -> Result<Val, String> {
    Ok(match value {
        TypedValue::String(s) => Val::Str(s.clone()),
        TypedValue::Int(i) => Val::Num(Decimal::from(*i)),
        TypedValue::Decimal(d) => Val::Num(*d),
        TypedValue::Bool(b) => Val::Truth(Some(*b)),
        TypedValue::Uuid(u) => Val::Uuid(*u),
        TypedValue::Timestamp(t) => Val::Str(t.to_rfc3339()),
        TypedValue::Quantity { base_value, unit } => Val::Qty(*base_value, unit.clone()),
        TypedValue::Enum { wire, .. } => Val::Str(wire.clone()),
        TypedValue::EntityRef { .. } | TypedValue::List(_) => {
            return Err("entity references and lists are not evaluable operands".to_string())
        }
    })
}

fn actor_role(context: &ApplicationPolicyContext) -> Val {
    match &context.actor {
        ActorRef::Anonymous => Val::Unknown,
        ActorRef::Role { role } => Val::Role(role.clone()),
    }
}

fn eval(expr: &Expression, context: &ApplicationPolicyContext) -> Result<Val, String> {
    match expr {
        Expression::Literal(value) => match value {
            serde_json::Value::Bool(b) => Ok(Val::Truth(Some(*b))),
            serde_json::Value::Number(n) => Decimal::from_str(&n.to_string())
                .map(Val::Num)
                .map_err(|e| format!("invalid numeric literal: {e}")),
            serde_json::Value::String(s) => Ok(Val::Str(s.clone())),
            other => Err(format!("literal {other} is not a scalar")),
        },
        Expression::Variable(name) if name == "actor" => Ok(actor_role(context)),
        Expression::Variable(name) => Ok(context
            .input
            .get(name)
            .map(typed_to_val)
            .transpose()?
            .unwrap_or(Val::Unknown)),
        Expression::MemberAccess { object, member } => match object.as_str() {
            "input" => Ok(context
                .input
                .get(member)
                .map(typed_to_val)
                .transpose()?
                .unwrap_or(Val::Unknown)),
            "state" => Ok(context
                .pre_state
                .as_ref()
                .and_then(|state| state.get(member))
                .map(typed_to_val)
                .transpose()?
                .unwrap_or(Val::Unknown)),
            "actor" if member == "role" => Ok(actor_role(context)),
            _ => Err(format!(
                "member access '{object}.{member}' is outside the executable subset"
            )),
        },
        // Contract construction proves resolvability; at runtime the reference
        // carries the resolved role ConceptId serialization.
        Expression::RoleReference { role } => {
            serde_json::from_value::<ConceptId>(serde_json::Value::String(role.clone()))
                .map(Val::Role)
                .map_err(|_| format!("role<{role}> was not resolved at contract construction"))
        }
        Expression::Unary { op, operand } => {
            let value = eval(operand, context)?;
            match (op, value) {
                (_, Val::Unknown) => Ok(Val::Unknown),
                (UnaryOp::Not, Val::Truth(t)) => Ok(Val::Truth(t.map(|b| !b))),
                (UnaryOp::Negate, Val::Num(n)) => Ok(Val::Num(-n)),
                (UnaryOp::Negate, Val::Qty(n, u)) => Ok(Val::Qty(-n, u)),
                (op, value) => Err(format!("unary {op} does not apply to {value:?}")),
            }
        }
        Expression::Binary { op, left, right } => {
            use BinaryOp as B;
            match op {
                B::And | B::Or => {
                    let l = truth(eval(left, context)?)?;
                    let r = truth(eval(right, context)?)?;
                    Ok(Val::Truth(if matches!(op, B::And) {
                        kleene_and(l, r)
                    } else {
                        kleene_or(l, r)
                    }))
                }
                B::Equal | B::NotEqual => {
                    let l = eval(left, context)?;
                    let r = eval(right, context)?;
                    if matches!(l, Val::Unknown) || matches!(r, Val::Unknown) {
                        return Ok(Val::Truth(None));
                    }
                    let equal = values_equal(&l, &r)?;
                    Ok(Val::Truth(Some(if matches!(op, B::Equal) {
                        equal
                    } else {
                        !equal
                    })))
                }
                B::GreaterThan | B::LessThan | B::GreaterThanOrEqual | B::LessThanOrEqual => {
                    let l = eval(left, context)?;
                    let r = eval(right, context)?;
                    if matches!(l, Val::Unknown) || matches!(r, Val::Unknown) {
                        return Ok(Val::Truth(None));
                    }
                    let (a, b) = numeric_pair(l, r)?;
                    Ok(Val::Truth(Some(match op {
                        B::GreaterThan => a > b,
                        B::LessThan => a < b,
                        B::GreaterThanOrEqual => a >= b,
                        _ => a <= b,
                    })))
                }
                B::Plus | B::Minus | B::Multiply | B::Divide => {
                    let l = eval(left, context)?;
                    let r = eval(right, context)?;
                    if matches!(l, Val::Unknown) || matches!(r, Val::Unknown) {
                        return Ok(Val::Unknown);
                    }
                    arithmetic(op, l, r)
                }
                other => Err(format!("operator {other} is outside the executable subset")),
            }
        }
        other => Err(format!(
            "expression node {other} is outside the executable subset"
        )),
    }
}

fn truth(value: Val) -> Result<Option<bool>, String> {
    match value {
        Val::Truth(t) => Ok(t),
        Val::Unknown => Ok(None),
        other => Err(format!("{other:?} is not a truth value")),
    }
}

fn kleene_and(l: Option<bool>, r: Option<bool>) -> Option<bool> {
    match (l, r) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

fn kleene_or(l: Option<bool>, r: Option<bool>) -> Option<bool> {
    match (l, r) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

fn values_equal(l: &Val, r: &Val) -> Result<bool, String> {
    Ok(match (l, r) {
        (Val::Num(a), Val::Num(b)) => a == b,
        // A bare number compared with a quantity adopts the field's unit.
        (Val::Qty(a, _), Val::Num(b)) | (Val::Num(b), Val::Qty(a, _)) => a == b,
        (Val::Qty(a, ua), Val::Qty(b, ub)) => {
            if ua != ub {
                return Err("quantity units differ; comparison is not evaluable".to_string());
            }
            a == b
        }
        (Val::Str(a), Val::Str(b)) => a == b,
        (Val::Truth(Some(a)), Val::Truth(Some(b))) => a == b,
        (Val::Role(a), Val::Role(b)) => a == b,
        (Val::Uuid(a), Val::Uuid(b)) => a == b,
        _ => return Err(format!("{l:?} and {r:?} are not comparable")),
    })
}

/// Extract a comparable decimal pair; bare numbers adopt a quantity's unit.
fn numeric_pair(l: Val, r: Val) -> Result<(Decimal, Decimal), String> {
    Ok(match (l, r) {
        (Val::Num(a), Val::Num(b)) => (a, b),
        (Val::Qty(a, _), Val::Num(b)) | (Val::Num(a), Val::Qty(b, _)) => (a, b),
        (Val::Qty(a, ua), Val::Qty(b, ub)) => {
            if ua != ub {
                return Err("quantity units differ; comparison is not evaluable".to_string());
            }
            (a, b)
        }
        (l, r) => return Err(format!("{l:?} and {r:?} are not numerically comparable")),
    })
}

fn arithmetic(op: &BinaryOp, l: Val, r: Val) -> Result<Val, String> {
    use BinaryOp as B;
    let apply = |a: Decimal, b: Decimal| -> Result<Decimal, String> {
        match op {
            B::Plus => a.checked_add(b),
            B::Minus => a.checked_sub(b),
            B::Multiply => a.checked_mul(b),
            B::Divide => {
                if b.is_zero() {
                    return Err("division by zero".to_string());
                }
                a.checked_div(b)
            }
            _ => unreachable!(),
        }
        .ok_or_else(|| "arithmetic overflow".to_string())
    };
    Ok(match (l, r) {
        (Val::Num(a), Val::Num(b)) => Val::Num(apply(a, b)?),
        (Val::Qty(a, u), Val::Num(b)) | (Val::Num(a), Val::Qty(b, u)) => Val::Qty(apply(a, b)?, u),
        (Val::Qty(a, ua), Val::Qty(b, ub)) => {
            if ua != ub {
                return Err("quantity units differ; arithmetic is not evaluable".to_string());
            }
            Val::Qty(apply(a, b)?, ua)
        }
        (l, r) => return Err(format!("{l:?} and {r:?} do not support arithmetic")),
    })
}
