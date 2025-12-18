//! Canonical Expression Normalizer
//!
//! Transforms policy expressions into a deterministic, minimal canonical form.
//! This enables:
//! - Semantic equivalence checking via normalized comparison
//! - Stable caching with deterministic hash keys
//! - Consistent hover signatures in LSP
//! - Deterministic golden test output
//!
//! # Normalization Rules Applied
//!
//! 1. **Commutativity**: Sort operands of AND/OR/==/!=/+/* lexicographically
//! 2. **Identity**: Remove identity elements (`true AND x` → `x`, `false OR x` → `x`)
//! 3. **Idempotence**: Deduplicate identical operands (`a AND a` → `a`)
//! 4. **Absorption**: Remove absorbed terms (`a OR (a AND b)` → `a`)
//! 5. **Domination**: Short-circuit dominating elements (`false AND x` → `false`)
//! 6. **Double Negation**: Eliminate `NOT NOT x` → `x`
//! 7. **Chain Flattening**: Flatten nested same-operator chains
//! 8. Comparison negation: NOT (a == b) → a != b

use super::{BinaryOp, Expression, UnaryOp};
use std::fmt;
use std::hash::{Hash, Hasher};
use xxhash_rust::xxh64::Xxh64;

/// A normalized expression with precomputed stable hash.
///
/// `NormalizedExpression` wraps an `Expression` that has been transformed
/// into canonical form, along with a stable hash value that remains consistent
/// across runs and platforms.
#[derive(Clone)]
pub struct NormalizedExpression {
    inner: Expression,
    hash: u64,
}

impl NormalizedExpression {
    /// Create a normalized form of the given expression.
    pub fn new(expr: &Expression) -> Self {
        let normalized = normalize_expr(expr);
        let hash = compute_stable_hash(&normalized);
        Self {
            inner: normalized,
            hash,
        }
    }

    /// Access the normalized expression.
    pub fn inner(&self) -> &Expression {
        &self.inner
    }

    /// Consume and return the inner normalized expression.
    pub fn into_inner(self) -> Expression {
        self.inner
    }

    /// Get the stable hash value for this normalized expression.
    ///
    /// This hash is deterministic across runs and platforms, making it
    /// suitable for cache keys.
    pub fn stable_hash(&self) -> u64 {
        self.hash
    }
}

impl PartialEq for NormalizedExpression {
    fn eq(&self, other: &Self) -> bool {
        // Fast path: compare hashes first
        self.hash == other.hash && self.inner == other.inner
    }
}

impl Eq for NormalizedExpression {}

impl Hash for NormalizedExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl fmt::Display for NormalizedExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::Debug for NormalizedExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NormalizedExpression")
            .field("inner", &self.inner)
            .field("hash", &format!("{:016x}", self.hash))
            .finish()
    }
}

// ============================================================================
// Stable Hashing
// ============================================================================

/// Compute a stable hash using Xxh64 which is portable and deterministic.
fn compute_stable_hash(expr: &Expression) -> u64 {
    // Use a fixed seed (0) for stability across runs and updates
    let mut hasher = Xxh64::new(0);
    hash_expression(expr, &mut hasher);
    hasher.finish()
}

/// Recursively hash an expression in a stable, deterministic manner.
fn hash_expression<H: Hasher>(expr: &Expression, hasher: &mut H) {
    // Hash discriminant first for type differentiation
    std::mem::discriminant(expr).hash(hasher);

    match expr {
        Expression::Literal(v) => {
            // Use string representation for consistent JSON value hashing
            v.to_string().hash(hasher);
        }
        Expression::Variable(s) => s.hash(hasher),
        Expression::QuantityLiteral { value, unit } => {
            value.to_string().hash(hasher);
            unit.hash(hasher);
        }
        Expression::TimeLiteral(s) => s.hash(hasher),
        Expression::IntervalLiteral { start, end } => {
            start.hash(hasher);
            end.hash(hasher);
        }
        Expression::MemberAccess { object, member } => {
            object.hash(hasher);
            member.hash(hasher);
        }
        Expression::Binary { op, left, right } => {
            std::mem::discriminant(op).hash(hasher);
            hash_expression(left, hasher);
            hash_expression(right, hasher);
        }
        Expression::Unary { op, operand } => {
            std::mem::discriminant(op).hash(hasher);
            hash_expression(operand, hasher);
        }
        Expression::Cast {
            operand,
            target_type,
        } => {
            hash_expression(operand, hasher);
            target_type.hash(hasher);
        }
        Expression::Quantifier {
            quantifier,
            variable,
            collection,
            condition,
        } => {
            std::mem::discriminant(quantifier).hash(hasher);
            variable.hash(hasher);
            hash_expression(collection, hasher);
            hash_expression(condition, hasher);
        }
        Expression::GroupBy {
            variable,
            collection,
            filter,
            key,
            condition,
        } => {
            variable.hash(hasher);
            hash_expression(collection, hasher);
            if let Some(f) = filter {
                true.hash(hasher);
                hash_expression(f, hasher);
            } else {
                false.hash(hasher);
            }
            hash_expression(key, hasher);
            hash_expression(condition, hasher);
        }
        Expression::Aggregation {
            function,
            collection,
            field,
            filter,
        } => {
            std::mem::discriminant(function).hash(hasher);
            hash_expression(collection, hasher);
            field.hash(hasher);
            if let Some(f) = filter {
                true.hash(hasher);
                hash_expression(f, hasher);
            } else {
                false.hash(hasher);
            }
        }
        Expression::AggregationComprehension {
            function,
            variable,
            collection,
            window,
            predicate,
            projection,
            target_unit,
        } => {
            std::mem::discriminant(function).hash(hasher);
            variable.hash(hasher);
            hash_expression(collection, hasher);
            if let Some(w) = window {
                true.hash(hasher);
                w.duration.hash(hasher);
                w.unit.hash(hasher);
            } else {
                false.hash(hasher);
            }
            hash_expression(predicate, hasher);
            hash_expression(projection, hasher);
            target_unit.hash(hasher);
        }
    }
}

// ============================================================================
// Main Normalization Logic
// ============================================================================

/// Normalize an expression into canonical form.
fn normalize_expr(expr: &Expression) -> Expression {
    match expr {
        // Leaf nodes: pass through as-is
        Expression::Literal(_)
        | Expression::Variable(_)
        | Expression::QuantityLiteral { .. }
        | Expression::TimeLiteral(_)
        | Expression::IntervalLiteral { .. }
        | Expression::MemberAccess { .. } => expr.clone(),

        // Binary: normalize children, apply optimizations, sort commutative ops
        Expression::Binary { op, left, right } => {
            let l = normalize_expr(left);
            let r = normalize_expr(right);
            normalize_binary(op.clone(), l, r)
        }

        // Unary: normalize child, apply double negation and De Morgan
        Expression::Unary { op, operand } => {
            let inner = normalize_expr(operand);
            normalize_unary(op.clone(), inner)
        }

        // Cast: normalize operand
        Expression::Cast {
            operand,
            target_type,
        } => Expression::Cast {
            operand: Box::new(normalize_expr(operand)),
            target_type: target_type.clone(),
        },

        // Quantifier: normalize collection and condition
        Expression::Quantifier {
            quantifier,
            variable,
            collection,
            condition,
        } => Expression::Quantifier {
            quantifier: quantifier.clone(),
            variable: variable.clone(),
            collection: Box::new(normalize_expr(collection)),
            condition: Box::new(normalize_expr(condition)),
        },

        // GroupBy: normalize all sub-expressions
        Expression::GroupBy {
            variable,
            collection,
            filter,
            key,
            condition,
        } => Expression::GroupBy {
            variable: variable.clone(),
            collection: Box::new(normalize_expr(collection)),
            filter: filter.as_ref().map(|f| Box::new(normalize_expr(f))),
            key: Box::new(normalize_expr(key)),
            condition: Box::new(normalize_expr(condition)),
        },

        // Aggregation: normalize sub-expressions
        Expression::Aggregation {
            function,
            collection,
            field,
            filter,
        } => Expression::Aggregation {
            function: function.clone(),
            collection: Box::new(normalize_expr(collection)),
            field: field.clone(),
            filter: filter.as_ref().map(|f| Box::new(normalize_expr(f))),
        },

        // AggregationComprehension: normalize sub-expressions
        Expression::AggregationComprehension {
            function,
            variable,
            collection,
            window,
            predicate,
            projection,
            target_unit,
        } => Expression::AggregationComprehension {
            function: function.clone(),
            variable: variable.clone(),
            collection: Box::new(normalize_expr(collection)),
            window: window.clone(),
            predicate: Box::new(normalize_expr(predicate)),
            projection: Box::new(normalize_expr(projection)),
            target_unit: target_unit.clone(),
        },
    }
}

// ============================================================================
// Binary Expression Normalization
// ============================================================================

/// Normalize a binary expression with comprehensive optimization rules.
fn normalize_binary(op: BinaryOp, left: Expression, right: Expression) -> Expression {
    // 1. Identity elimination for AND
    if op == BinaryOp::And {
        if is_true_literal(&left) {
            return right;
        }
        if is_true_literal(&right) {
            return left;
        }
    }

    // 2. Identity elimination for OR
    if op == BinaryOp::Or {
        if is_false_literal(&left) {
            return right;
        }
        if is_false_literal(&right) {
            return left;
        }
    }

    // 3. Domination rules (short-circuit)
    if op == BinaryOp::And && (is_false_literal(&left) || is_false_literal(&right)) {
        return Expression::Literal(serde_json::Value::Bool(false));
    }
    if op == BinaryOp::Or && (is_true_literal(&left) || is_true_literal(&right)) {
        return Expression::Literal(serde_json::Value::Bool(true));
    }

    // 4. Idempotence: a AND a → a, a OR a → a
    if (op == BinaryOp::And || op == BinaryOp::Or) && left == right {
        return left;
    }

    // 5. Absorption: a OR (a AND b) → a, a AND (a OR b) → a
    if op == BinaryOp::Or {
        if let Some(result) = try_absorb(&left, &right, BinaryOp::And) {
            return result;
        }
        if let Some(result) = try_absorb(&right, &left, BinaryOp::And) {
            return result;
        }
    }
    if op == BinaryOp::And {
        if let Some(result) = try_absorb(&left, &right, BinaryOp::Or) {
            return result;
        }
        if let Some(result) = try_absorb(&right, &left, BinaryOp::Or) {
            return result;
        }
    }

    // 6. Flatten chains: (a AND b) AND c → collect [a, b, c] and rebuild sorted
    if is_commutative(&op) {
        let mut operands = collect_chain(&op, &left);
        operands.extend(collect_chain(&op, &right));

        // Deduplicate (idempotence across chain)
        operands.sort_by_key(expr_cmp_key);
        operands.dedup();

        // Re-apply absorption across the flattened chain
        // e.g. (A) AND (A OR B) -> A
        let partner_op = match op {
            BinaryOp::Or => Some(BinaryOp::And),
            BinaryOp::And => Some(BinaryOp::Or),
            _ => None,
        };

        if let Some(partner) = partner_op {
            let mut indices_to_remove = std::collections::HashSet::new();
            for i in 0..operands.len() {
                if indices_to_remove.contains(&i) {
                    continue;
                }
                for j in (i + 1)..operands.len() {
                    if indices_to_remove.contains(&j) {
                        continue;
                    }

                    // Check if i absorbs j
                    if try_absorb(&operands[i], &operands[j], partner.clone()).is_some() {
                        indices_to_remove.insert(j);
                        continue;
                    }

                    // Check if j absorbs i
                    if try_absorb(&operands[j], &operands[i], partner.clone()).is_some() {
                        indices_to_remove.insert(i);
                        break; // i is removed, move to next i
                    }
                }
            }

            if !indices_to_remove.is_empty() {
                let mut new_operands = Vec::with_capacity(operands.len() - indices_to_remove.len());
                for (i, op) in operands.into_iter().enumerate() {
                    if !indices_to_remove.contains(&i) {
                        new_operands.push(op);
                    }
                }
                operands = new_operands;
                // Re-sort/dedup not strictly necessary if order preserved and absorption is clean,
                // but good for safety if absorption produced duplicates (unlikely here).
            }
        }

        // Rebuild balanced tree
        return build_balanced_tree(op, operands);
    }

    // 7. Non-commutative: just preserve order
    Expression::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Check for absorption pattern: if `outer` can absorb `inner`.
/// Returns `Some(outer)` if absorption applies, `None` otherwise.
fn try_absorb(outer: &Expression, inner: &Expression, inner_op: BinaryOp) -> Option<Expression> {
    // Check if inner is a binary expression with inner_op containing outer
    if let Expression::Binary { op, left, right } = inner {
        if *op == inner_op && (left.as_ref() == outer || right.as_ref() == outer) {
            return Some(outer.clone());
        }
    }
    None
}

/// Collect all operands from a chain of the same operator.
fn collect_chain(op: &BinaryOp, expr: &Expression) -> Vec<Expression> {
    if let Expression::Binary {
        op: inner_op,
        left,
        right,
    } = expr
    {
        if inner_op == op {
            let mut result = collect_chain(op, left);
            result.extend(collect_chain(op, right));
            return result;
        }
    }
    vec![expr.clone()]
}

/// Build a balanced binary tree from a list of operands.
fn build_balanced_tree(op: BinaryOp, mut operands: Vec<Expression>) -> Expression {
    if operands.is_empty() {
        // Return identity element
        return match op {
            BinaryOp::And => Expression::Literal(serde_json::Value::Bool(true)),
            BinaryOp::Or => Expression::Literal(serde_json::Value::Bool(false)),
            BinaryOp::Plus => Expression::Literal(serde_json::Value::Number(0.into())),
            BinaryOp::Multiply => Expression::Literal(serde_json::Value::Number(1.into())),
            _ => Expression::Literal(serde_json::Value::Null),
        };
    }

    if operands.len() == 1 {
        return operands.remove(0);
    }

    // Build left-to-right for determinism (after sorting)
    let first = operands.remove(0);
    operands
        .into_iter()
        .fold(first, |acc, next| Expression::Binary {
            op: op.clone(),
            left: Box::new(acc),
            right: Box::new(next),
        })
}

// ============================================================================
// Unary Expression Normalization
// ============================================================================

/// Normalize a unary expression with double negation elimination and De Morgan.
fn normalize_unary(op: UnaryOp, operand: Expression) -> Expression {
    match op {
        UnaryOp::Not => {
            // Double negation elimination: NOT NOT x → x
            if let Expression::Unary {
                op: UnaryOp::Not,
                operand: inner,
            } = &operand
            {
                return inner.as_ref().clone();
            }

            // Comparison negation: NOT (a == b) → a != b, NOT (a != b) → a == b
            if let Expression::Binary {
                op: inner_op,
                left,
                right,
            } = &operand
            {
                let maybe_negated = match inner_op {
                    BinaryOp::Equal => Some(BinaryOp::NotEqual),
                    BinaryOp::NotEqual => Some(BinaryOp::Equal),
                    BinaryOp::GreaterThan => Some(BinaryOp::LessThanOrEqual),
                    BinaryOp::LessThan => Some(BinaryOp::GreaterThanOrEqual),
                    BinaryOp::GreaterThanOrEqual => Some(BinaryOp::LessThan),
                    BinaryOp::LessThanOrEqual => Some(BinaryOp::GreaterThan),
                    _ => None,
                };
                if let Some(negated_op) = maybe_negated {
                    return Expression::Binary {
                        op: negated_op,
                        left: left.clone(),
                        right: right.clone(),
                    };
                }
            }

            Expression::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            }
        }
        UnaryOp::Negate => Expression::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(operand),
        },
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if an operator is commutative (operand order doesn't matter).
fn is_commutative(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Plus
            | BinaryOp::Multiply
    )
}

/// Generate a comparison key for lexicographic sorting of expressions.
fn expr_cmp_key(expr: &Expression) -> String {
    // Use canonical serialization for stable comparison
    canonical_serialize(expr)
}

/// Recursively serialized expression to a stable string representation
fn canonical_serialize(expr: &Expression) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    match expr {
        Expression::Literal(v) => write!(out, "Lit({})", v).unwrap(),
        Expression::Variable(s) => write!(out, "Var({})", s).unwrap(),
        Expression::QuantityLiteral { value, unit } => {
            write!(out, "Quant({}, {:?})", value, unit).unwrap()
        }
        Expression::TimeLiteral(s) => write!(out, "Time({})", s).unwrap(),
        Expression::IntervalLiteral { start, end } => {
            write!(out, "Int({}, {})", start, end).unwrap()
        }
        Expression::MemberAccess { object, member } => {
            write!(out, "Mem({}, {})", object, member).unwrap()
        }
        Expression::Binary { op, left, right } => write!(
            out,
            "Bin({:?}, {}, {})",
            op,
            canonical_serialize(left),
            canonical_serialize(right)
        )
        .unwrap(),
        Expression::Unary { op, operand } => {
            write!(out, "Un({:?}, {})", op, canonical_serialize(operand)).unwrap()
        }
        Expression::Cast {
            operand,
            target_type,
        } => write!(
            out,
            "Cast({}, {:?})",
            canonical_serialize(operand),
            target_type
        )
        .unwrap(),
        Expression::Quantifier {
            quantifier,
            variable,
            collection,
            condition,
        } => write!(
            out,
            "Quant({:?}, {}, {}, {})",
            quantifier,
            variable,
            canonical_serialize(collection),
            canonical_serialize(condition)
        )
        .unwrap(),
        Expression::GroupBy {
            variable,
            collection,
            filter,
            key,
            condition,
        } => {
            let f_str = filter
                .as_ref()
                .map(|f| canonical_serialize(f))
                .unwrap_or_else(|| "None".into());
            write!(
                out,
                "Group({}, {}, {}, {}, {})",
                variable,
                canonical_serialize(collection),
                f_str,
                canonical_serialize(key),
                canonical_serialize(condition)
            )
            .unwrap()
        }
        Expression::Aggregation {
            function,
            collection,
            field,
            filter,
        } => {
            let f_str = filter
                .as_ref()
                .map(|f| canonical_serialize(f))
                .unwrap_or_else(|| "None".into());
            let field_str = field.as_ref().map(|s| s.as_str()).unwrap_or("None");
            write!(
                out,
                "Agg({:?}, {}, {}, {})",
                function,
                canonical_serialize(collection),
                field_str,
                f_str
            )
            .unwrap()
        }
        Expression::AggregationComprehension {
            function,
            variable,
            collection,
            window,
            predicate,
            projection,
            target_unit,
        } => {
            let w_str = window
                .as_ref()
                .map(|w| format!("{:?}", w))
                .unwrap_or_else(|| "None".into());
            let tu_str = target_unit.as_deref().unwrap_or("None");
            write!(
                out,
                "AggComp({:?}, {}, {}, {}, {}, {}, {})",
                function,
                variable,
                canonical_serialize(collection),
                w_str,
                canonical_serialize(predicate),
                canonical_serialize(projection),
                tu_str
            )
            .unwrap()
        }
    }
    out
}

/// Check if expression is the literal `true`.
fn is_true_literal(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(v) if v.as_bool() == Some(true))
}

/// Check if expression is the literal `false`.
fn is_false_literal(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(v) if v.as_bool() == Some(false))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::Quantifier;

    #[test]
    fn test_literal_normalization() {
        let expr = Expression::literal(true);
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "true");
    }

    #[test]
    fn test_variable_normalization() {
        let expr = Expression::variable("x");
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "x");
    }

    #[test]
    fn test_commutative_sorting_and() {
        // b AND a should normalize to (a AND b)
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::variable("b"),
            Expression::variable("a"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "(a AND b)");
    }

    #[test]
    fn test_commutative_sorting_or() {
        // z OR y should normalize to (y OR z)
        let expr = Expression::binary(
            BinaryOp::Or,
            Expression::variable("z"),
            Expression::variable("y"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "(y OR z)");
    }

    #[test]
    fn test_identity_elimination_and_true() {
        // true AND x → x
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::literal(true),
            Expression::variable("x"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "x");
    }

    #[test]
    fn test_identity_elimination_and_true_right() {
        // x AND true → x
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::variable("x"),
            Expression::literal(true),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "x");
    }

    #[test]
    fn test_identity_elimination_or_false() {
        // false OR x → x
        let expr = Expression::binary(
            BinaryOp::Or,
            Expression::literal(false),
            Expression::variable("x"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "x");
    }

    #[test]
    fn test_domination_and_false() {
        // false AND x → false
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::literal(false),
            Expression::variable("x"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "false");
    }

    #[test]
    fn test_domination_or_true() {
        // true OR x → true
        let expr = Expression::binary(
            BinaryOp::Or,
            Expression::literal(true),
            Expression::variable("x"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "true");
    }

    #[test]
    fn test_idempotence_and() {
        // a AND a → a
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::variable("a"),
            Expression::variable("a"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "a");
    }

    #[test]
    fn test_idempotence_or() {
        // a OR a → a
        let expr = Expression::binary(
            BinaryOp::Or,
            Expression::variable("a"),
            Expression::variable("a"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "a");
    }

    #[test]
    fn test_absorption_or() {
        // a OR (a AND b) → a
        let expr = Expression::binary(
            BinaryOp::Or,
            Expression::variable("a"),
            Expression::binary(
                BinaryOp::And,
                Expression::variable("a"),
                Expression::variable("b"),
            ),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "a");
    }

    #[test]
    fn test_absorption_and() {
        // a AND (a OR b) → a
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::variable("a"),
            Expression::binary(
                BinaryOp::Or,
                Expression::variable("a"),
                Expression::variable("b"),
            ),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "a");
    }

    #[test]
    fn test_double_negation_elimination() {
        // NOT NOT x → x
        let expr = Expression::unary(
            UnaryOp::Not,
            Expression::unary(UnaryOp::Not, Expression::variable("x")),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "x");
    }

    #[test]
    fn test_de_morgan_equal() {
        // NOT (a == b) → a != b
        let expr = Expression::unary(
            UnaryOp::Not,
            Expression::binary(
                BinaryOp::Equal,
                Expression::variable("a"),
                Expression::variable("b"),
            ),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "(a != b)");
    }

    #[test]
    fn test_de_morgan_not_equal() {
        // NOT (a != b) → a == b
        let expr = Expression::unary(
            UnaryOp::Not,
            Expression::binary(
                BinaryOp::NotEqual,
                Expression::variable("a"),
                Expression::variable("b"),
            ),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "(a == b)");
    }

    #[test]
    fn test_de_morgan_greater_than() {
        // NOT (a > b) → a <= b
        let expr = Expression::unary(
            UnaryOp::Not,
            Expression::binary(
                BinaryOp::GreaterThan,
                Expression::variable("a"),
                Expression::variable("b"),
            ),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "(a <= b)");
    }

    #[test]
    fn test_chain_flattening_and() {
        // (a AND b) AND c → (a AND b AND c) sorted
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::binary(
                BinaryOp::And,
                Expression::variable("c"),
                Expression::variable("a"),
            ),
            Expression::variable("b"),
        );
        let norm = NormalizedExpression::new(&expr);
        // Should be sorted: a AND b AND c
        assert_eq!(norm.to_string(), "((a AND b) AND c)");
    }

    #[test]
    fn test_chain_deduplication() {
        // a AND b AND a → a AND b (sorted)
        let expr = Expression::binary(
            BinaryOp::And,
            Expression::binary(
                BinaryOp::And,
                Expression::variable("a"),
                Expression::variable("b"),
            ),
            Expression::variable("a"),
        );
        let norm = NormalizedExpression::new(&expr);
        assert_eq!(norm.to_string(), "(a AND b)");
    }

    #[test]
    fn test_equivalence_commutative() {
        let a_and_b = Expression::binary(
            BinaryOp::And,
            Expression::variable("a"),
            Expression::variable("b"),
        );
        let b_and_a = Expression::binary(
            BinaryOp::And,
            Expression::variable("b"),
            Expression::variable("a"),
        );
        let norm1 = NormalizedExpression::new(&a_and_b);
        let norm2 = NormalizedExpression::new(&b_and_a);
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_stable_hash_deterministic() {
        let expr = Expression::binary(
            BinaryOp::Or,
            Expression::variable("x"),
            Expression::literal(false),
        );
        let h1 = NormalizedExpression::new(&expr).stable_hash();
        let h2 = NormalizedExpression::new(&expr).stable_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_nested_normalization() {
        // (true AND (b OR a)) → (a OR b)
        let nested = Expression::binary(
            BinaryOp::And,
            Expression::literal(true),
            Expression::binary(
                BinaryOp::Or,
                Expression::variable("b"),
                Expression::variable("a"),
            ),
        );
        let norm = NormalizedExpression::new(&nested);
        assert_eq!(norm.to_string(), "(a OR b)");
    }

    #[test]
    fn test_quantifier_normalization() {
        let expr = Expression::quantifier(
            Quantifier::ForAll,
            "x",
            Expression::variable("items"),
            Expression::binary(
                BinaryOp::And,
                Expression::literal(true),
                Expression::variable("valid"),
            ),
        );
        let norm = NormalizedExpression::new(&expr);
        // Condition should be simplified: true AND valid → valid
        assert!(norm.to_string().contains("valid"));
        assert!(!norm.to_string().contains("true"));
    }
}
