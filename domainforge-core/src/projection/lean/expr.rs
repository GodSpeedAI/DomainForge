//! Lowers `policy::Expression` into Lean 4 propositions.
//!
//! Every policy is classified as either *groundable* — fully evaluable over
//! the declared model, emitted with a `by decide` proof — or *deferred*,
//! emitted as a documented proof obligation stub. The Rust evaluator here
//! evaluates exactly the semantics that get emitted to Lean (same scaled
//! integers, same operators), so a `decide` proof emitted as "holds" is
//! guaranteed to check.

use crate::policy::{BinaryOp, Expression, Quantifier, UnaryOp};
use rust_decimal::Decimal;

/// Result of lowering one policy expression.
pub enum Lowered {
    /// A well-formed Lean `Prop`, plus whether the declared model satisfies it.
    Groundable { lean: String, holds: bool },
    /// Not auto-groundable in v1; becomes a proof obligation stub.
    Deferred { reason: String },
}

/// Ground context shared by all policies of one model.
pub struct GroundCtx {
    /// Shared decimal scale `k`: every quantity is an integer `value × 10⁻ᵏ`.
    pub scale: u32,
    /// Scaled flow quantities, in the same order as the emitted `flows` list.
    /// `None` when the model emits no `flows` definition (flow-referencing
    /// policies must then defer).
    pub flow_quantities: Option<Vec<i128>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Num,
    Str,
    Prop,
}

#[derive(Clone, PartialEq, Debug)]
enum V {
    N(i128),
    S(String),
    P(bool),
}

pub fn lower_policy(expr: &Expression, ctx: &GroundCtx) -> Lowered {
    let (body, kind) = match render(expr, ctx) {
        Ok(r) => r,
        Err(reason) => return Lowered::Deferred { reason },
    };
    if kind != Kind::Prop {
        return Lowered::Deferred {
            reason: "expression is not a proposition".to_string(),
        };
    }
    if uses_flow(expr) {
        let Some(quantities) = &ctx.flow_quantities else {
            return Lowered::Deferred {
                reason: "references Flow.quantity but the model declares no flows".to_string(),
            };
        };
        let mut holds = true;
        for q in quantities {
            match eval(expr, ctx, Some(*q)) {
                Ok(V::P(b)) => holds &= b,
                Ok(_) => unreachable!("render checked Prop kind"),
                Err(reason) => return Lowered::Deferred { reason },
            }
        }
        Lowered::Groundable {
            lean: format!("∀ f ∈ flows, {body}"),
            holds,
        }
    } else {
        match eval(expr, ctx, None) {
            Ok(V::P(holds)) => Lowered::Groundable { lean: body, holds },
            Ok(_) => unreachable!("render checked Prop kind"),
            Err(reason) => Lowered::Deferred { reason },
        }
    }
}

/// Collect every decimal literal in an expression (for shared-scale computation).
pub fn collect_decimals(expr: &Expression, out: &mut Vec<Decimal>) {
    match expr {
        Expression::Literal(serde_json::Value::Number(n)) => {
            if let Ok(d) = n.to_string().parse::<Decimal>() {
                out.push(d);
            }
        }
        Expression::QuantityLiteral { value, .. } => out.push(*value),
        Expression::Binary { left, right, .. } => {
            collect_decimals(left, out);
            collect_decimals(right, out);
        }
        Expression::Unary { operand, .. } | Expression::Cast { operand, .. } => {
            collect_decimals(operand, out);
        }
        Expression::Quantifier {
            collection,
            condition,
            ..
        } => {
            collect_decimals(collection, out);
            collect_decimals(condition, out);
        }
        _ => {}
    }
}

/// Scale a decimal to the shared integer scale `k`.
pub fn scaled(d: Decimal, k: u32) -> Result<i128, String> {
    let d = d.normalize();
    if d.scale() > k {
        return Err(format!("decimal {d} exceeds shared scale {k}"));
    }
    let factor = 10i128
        .checked_pow(k - d.scale())
        .ok_or_else(|| format!("scale {k} too large"))?;
    d.mantissa()
        .checked_mul(factor)
        .ok_or_else(|| format!("scaled value of {d} overflows"))
}

fn uses_flow(expr: &Expression) -> bool {
    match expr {
        Expression::MemberAccess { object, member } => object == "Flow" && member == "quantity",
        Expression::Binary { left, right, .. } => uses_flow(left) || uses_flow(right),
        Expression::Unary { operand, .. } => uses_flow(operand),
        _ => false,
    }
}

fn escape_lean_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn numeral(v: i128) -> String {
    if v < 0 {
        format!("({v})")
    } else {
        v.to_string()
    }
}

fn defer_op(op: &BinaryOp) -> String {
    format!("uses operator {op:?} (not auto-groundable in v1)")
}

/// Render to a Lean term. The string is flow-independent (`Flow.quantity`
/// renders as the bound variable `f.quantity`).
fn render(expr: &Expression, ctx: &GroundCtx) -> Result<(String, Kind), String> {
    match expr {
        Expression::Literal(v) => match v {
            serde_json::Value::Bool(true) => Ok(("True".to_string(), Kind::Prop)),
            serde_json::Value::Bool(false) => Ok(("False".to_string(), Kind::Prop)),
            serde_json::Value::Number(n) => {
                let d = n
                    .to_string()
                    .parse::<Decimal>()
                    .map_err(|e| format!("unsupported numeric literal {n}: {e}"))?;
                Ok((numeral(scaled(d, ctx.scale)?), Kind::Num))
            }
            serde_json::Value::String(s) => {
                Ok((format!("\"{}\"", escape_lean_string(s)), Kind::Str))
            }
            other => Err(format!("unsupported literal {other} (not auto-groundable)")),
        },
        Expression::QuantityLiteral { value, .. } => {
            Ok((numeral(scaled(*value, ctx.scale)?), Kind::Num))
        }
        Expression::MemberAccess { object, member } => {
            if object == "Flow" && member == "quantity" {
                Ok(("f.quantity".to_string(), Kind::Num))
            } else {
                Err(format!(
                    "references {object}.{member} (only Flow.quantity is auto-groundable in v1)"
                ))
            }
        }
        Expression::Binary { op, left, right } => {
            let (l, lk) = render(left, ctx)?;
            let (r, rk) = render(right, ctx)?;
            let prop = |sym: &str| -> Result<(String, Kind), String> {
                Ok((format!("({l} {sym} {r})"), Kind::Prop))
            };
            match op {
                BinaryOp::And | BinaryOp::Or => {
                    if lk != Kind::Prop || rk != Kind::Prop {
                        return Err("boolean operator applied to non-propositions".to_string());
                    }
                    prop(if *op == BinaryOp::And { "∧" } else { "∨" })
                }
                BinaryOp::Equal | BinaryOp::NotEqual => {
                    if lk != rk || lk == Kind::Prop {
                        return Err("equality on mismatched or non-comparable operands".to_string());
                    }
                    prop(if *op == BinaryOp::Equal { "=" } else { "≠" })
                }
                BinaryOp::GreaterThan
                | BinaryOp::LessThan
                | BinaryOp::GreaterThanOrEqual
                | BinaryOp::LessThanOrEqual => {
                    if lk != Kind::Num || rk != Kind::Num {
                        return Err("ordering comparison on non-numeric operands".to_string());
                    }
                    prop(match op {
                        BinaryOp::GreaterThan => ">",
                        BinaryOp::LessThan => "<",
                        BinaryOp::GreaterThanOrEqual => "≥",
                        _ => "≤",
                    })
                }
                BinaryOp::Plus | BinaryOp::Minus => {
                    if lk != Kind::Num || rk != Kind::Num {
                        return Err("arithmetic on non-numeric operands".to_string());
                    }
                    let sym = if *op == BinaryOp::Plus { "+" } else { "-" };
                    Ok((format!("({l} {sym} {r})"), Kind::Num))
                }
                // ponytail: Multiply/Divide change the shared decimal scale;
                // support them by tracking per-term scale if models need it.
                other => Err(defer_op(other)),
            }
        }
        Expression::Unary { op, operand } => {
            let (o, k) = render(operand, ctx)?;
            match op {
                UnaryOp::Not => {
                    if k != Kind::Prop {
                        return Err("negation of a non-proposition".to_string());
                    }
                    Ok((format!("(¬ {o})"), Kind::Prop))
                }
                UnaryOp::Negate => {
                    if k != Kind::Num {
                        return Err("arithmetic negation of a non-number".to_string());
                    }
                    Ok((format!("(-{o})"), Kind::Num))
                }
            }
        }
        Expression::TimeLiteral(_) => Err("uses a time literal (not auto-groundable)".to_string()),
        Expression::IntervalLiteral { .. } => {
            Err("uses an interval literal (not auto-groundable)".to_string())
        }
        Expression::Variable(v) => Err(format!("uses free variable `{v}` (not auto-groundable)")),
        Expression::GroupBy { .. } => Err("uses group-by (not auto-groundable)".to_string()),
        Expression::Cast { .. } => Err("uses a cast (not auto-groundable)".to_string()),
        Expression::Quantifier { quantifier, .. } => Err(format!(
            "uses quantifier {} (not auto-groundable in v1)",
            match quantifier {
                Quantifier::ForAll => "forall",
                Quantifier::Exists => "exists",
                Quantifier::ExistsUnique => "exists-unique",
            }
        )),
        Expression::Aggregation { .. } | Expression::AggregationComprehension { .. } => {
            Err("uses aggregation (not auto-groundable)".to_string())
        }
    }
}

/// Evaluate with the same semantics `render` emits. `flow_q` binds `f.quantity`.
fn eval(expr: &Expression, ctx: &GroundCtx, flow_q: Option<i128>) -> Result<V, String> {
    match expr {
        Expression::Literal(v) => match v {
            serde_json::Value::Bool(b) => Ok(V::P(*b)),
            serde_json::Value::Number(n) => {
                let d = n
                    .to_string()
                    .parse::<Decimal>()
                    .map_err(|e| format!("unsupported numeric literal {n}: {e}"))?;
                Ok(V::N(scaled(d, ctx.scale)?))
            }
            serde_json::Value::String(s) => Ok(V::S(s.clone())),
            other => Err(format!("unsupported literal {other}")),
        },
        Expression::QuantityLiteral { value, .. } => Ok(V::N(scaled(*value, ctx.scale)?)),
        Expression::MemberAccess { .. } => {
            flow_q.map(V::N).ok_or_else(|| "no flow bound".to_string())
        }
        Expression::Binary { op, left, right } => {
            let l = eval(left, ctx, flow_q)?;
            let r = eval(right, ctx, flow_q)?;
            match (op, l, r) {
                (BinaryOp::And, V::P(a), V::P(b)) => Ok(V::P(a && b)),
                (BinaryOp::Or, V::P(a), V::P(b)) => Ok(V::P(a || b)),
                (BinaryOp::Equal, a, b) => Ok(V::P(a == b)),
                (BinaryOp::NotEqual, a, b) => Ok(V::P(a != b)),
                (BinaryOp::GreaterThan, V::N(a), V::N(b)) => Ok(V::P(a > b)),
                (BinaryOp::LessThan, V::N(a), V::N(b)) => Ok(V::P(a < b)),
                (BinaryOp::GreaterThanOrEqual, V::N(a), V::N(b)) => Ok(V::P(a >= b)),
                (BinaryOp::LessThanOrEqual, V::N(a), V::N(b)) => Ok(V::P(a <= b)),
                (BinaryOp::Plus, V::N(a), V::N(b)) => a
                    .checked_add(b)
                    .map(V::N)
                    .ok_or_else(|| "arithmetic overflow".to_string()),
                (BinaryOp::Minus, V::N(a), V::N(b)) => a
                    .checked_sub(b)
                    .map(V::N)
                    .ok_or_else(|| "arithmetic overflow".to_string()),
                (other, _, _) => Err(defer_op(other)),
            }
        }
        Expression::Unary { op, operand } => {
            let o = eval(operand, ctx, flow_q)?;
            match (op, o) {
                (UnaryOp::Not, V::P(b)) => Ok(V::P(!b)),
                (UnaryOp::Negate, V::N(n)) => n
                    .checked_neg()
                    .map(V::N)
                    .ok_or_else(|| "arithmetic overflow".to_string()),
                _ => Err("type error in unary operator".to_string()),
            }
        }
        other => Err(format!("unsupported expression {other:?}")),
    }
}

const LEAN_KEYWORDS: &[&str] = &[
    "abbrev",
    "axiom",
    "by",
    "class",
    "def",
    "deriving",
    "do",
    "else",
    "end",
    "example",
    "fun",
    "if",
    "import",
    "in",
    "inductive",
    "instance",
    "let",
    "match",
    "namespace",
    "open",
    "sorry",
    "structure",
    "then",
    "theorem",
    "where",
];

/// Sanitize a `.sea` name into a valid Lean identifier (deterministic).
pub fn lean_ident(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, 'X');
    }
    if LEAN_KEYWORDS.contains(&out.as_str()) {
        out.push('_');
    }
    out
}

/// Deterministically deduplicate sanitized identifiers (`_2`, `_3`, … suffixes).
pub fn unique_idents<'a>(names: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    let mut out = Vec::new();
    for name in names {
        let base = lean_ident(name);
        let n = seen.entry(base.clone()).or_insert(0);
        *n += 1;
        if *n == 1 {
            out.push(base);
        } else {
            out.push(format!("{base}_{n}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(quantities: &[i128]) -> GroundCtx {
        GroundCtx {
            scale: 0,
            flow_quantities: Some(quantities.to_vec()),
        }
    }

    fn num(n: i64) -> Expression {
        Expression::Literal(serde_json::json!(n))
    }

    fn flow_quantity() -> Expression {
        Expression::MemberAccess {
            object: "Flow".to_string(),
            member: "quantity".to_string(),
        }
    }

    fn gt(l: Expression, r: Expression) -> Expression {
        Expression::Binary {
            op: BinaryOp::GreaterThan,
            left: Box::new(l),
            right: Box::new(r),
        }
    }

    #[test]
    fn flow_policy_renders_and_holds() {
        let expr = gt(flow_quantity(), num(0));
        match lower_policy(&expr, &ctx(&[100, 5])) {
            Lowered::Groundable { lean, holds } => {
                assert_eq!(lean, "∀ f ∈ flows, (f.quantity > 0)");
                assert!(holds);
            }
            Lowered::Deferred { reason } => panic!("deferred: {reason}"),
        }
    }

    #[test]
    fn violated_flow_policy_grounds_false() {
        let expr = gt(flow_quantity(), num(1000));
        match lower_policy(&expr, &ctx(&[100])) {
            Lowered::Groundable { holds, .. } => assert!(!holds),
            Lowered::Deferred { reason } => panic!("deferred: {reason}"),
        }
    }

    #[test]
    fn empty_flows_are_vacuously_true() {
        let expr = gt(flow_quantity(), num(0));
        match lower_policy(&expr, &ctx(&[])) {
            Lowered::Groundable { holds, .. } => assert!(holds),
            Lowered::Deferred { reason } => panic!("deferred: {reason}"),
        }
    }

    #[test]
    fn flow_policy_defers_without_flow_context() {
        let expr = gt(flow_quantity(), num(0));
        let no_flows = GroundCtx {
            scale: 0,
            flow_quantities: None,
        };
        assert!(matches!(
            lower_policy(&expr, &no_flows),
            Lowered::Deferred { .. }
        ));
    }

    #[test]
    fn closed_expressions_ground_without_quantifier() {
        let expr = Expression::Binary {
            op: BinaryOp::And,
            left: Box::new(gt(num(3), num(2))),
            right: Box::new(Expression::Unary {
                op: UnaryOp::Not,
                operand: Box::new(gt(num(1), num(2))),
            }),
        };
        match lower_policy(&expr, &ctx(&[])) {
            Lowered::Groundable { lean, holds } => {
                assert_eq!(lean, "((3 > 2) ∧ (¬ (1 > 2)))");
                assert!(holds);
            }
            Lowered::Deferred { reason } => panic!("deferred: {reason}"),
        }
    }

    #[test]
    fn string_equality_grounds() {
        let expr = Expression::Binary {
            op: BinaryOp::Equal,
            left: Box::new(Expression::Literal(serde_json::json!("a\"b"))),
            right: Box::new(Expression::Literal(serde_json::json!("a\"b"))),
        };
        match lower_policy(&expr, &ctx(&[])) {
            Lowered::Groundable { lean, holds } => {
                assert_eq!(lean, "(\"a\\\"b\" = \"a\\\"b\")");
                assert!(holds);
            }
            Lowered::Deferred { reason } => panic!("deferred: {reason}"),
        }
    }

    #[test]
    fn decimals_share_the_model_scale() {
        let expr = gt(
            flow_quantity(),
            Expression::QuantityLiteral {
                value: "1.5".parse().unwrap(),
                unit: "units".to_string(),
            },
        );
        let c = GroundCtx {
            scale: 2,
            flow_quantities: Some(vec![160]), // 1.60 scaled
        };
        match lower_policy(&expr, &c) {
            Lowered::Groundable { lean, holds } => {
                assert_eq!(lean, "∀ f ∈ flows, (f.quantity > 150)");
                assert!(holds);
            }
            Lowered::Deferred { reason } => panic!("deferred: {reason}"),
        }
    }

    #[test]
    fn unsupported_constructs_defer_with_reason() {
        let cases: Vec<(Expression, &str)> = vec![
            (
                Expression::MemberAccess {
                    object: "Entity".to_string(),
                    member: "name".to_string(),
                },
                "Entity.name",
            ),
            (
                Expression::Cast {
                    operand: Box::new(num(1)),
                    target_type: "int".to_string(),
                },
                "cast",
            ),
            (Expression::Variable("x".to_string()), "free variable"),
            (
                Expression::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(num(2)),
                    right: Box::new(num(3)),
                },
                "Multiply",
            ),
        ];
        for (expr, needle) in cases {
            match lower_policy(&expr, &ctx(&[])) {
                Lowered::Deferred { reason } => {
                    assert!(reason.contains(needle), "reason `{reason}` ∌ `{needle}`")
                }
                Lowered::Groundable { lean, .. } => panic!("unexpectedly grounded: {lean}"),
            }
        }
    }

    #[test]
    fn idents_are_sanitized_and_deduplicated() {
        assert_eq!(lean_ident("Camera Units"), "Camera_Units");
        assert_eq!(lean_ident("9lives"), "X9lives");
        assert_eq!(lean_ident("theorem"), "theorem_");
        let ids = unique_idents(["A B", "A_B", "ok"].into_iter());
        assert_eq!(ids, vec!["A_B", "A_B_2", "ok"]);
    }
}
