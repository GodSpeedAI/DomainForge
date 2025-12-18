"""Tests for Expression and NormalizedExpression bindings."""

import pytest
from sea_dsl import (
    Expression,
    NormalizedExpression,
    BinaryOp,
    UnaryOp,
    Quantifier,
    AggregateFunction,
    WindowSpec,
)


class TestExpressionFactoryMethods:
    """Tests for Expression factory methods."""

    def test_literal_bool(self):
        """Test creating boolean literal expressions."""
        expr = Expression.literal(True)
        assert str(expr) == "true"

        expr_false = Expression.literal(False)
        assert str(expr_false) == "false"

    def test_literal_number(self):
        """Test creating number literal expressions."""
        expr = Expression.literal(42)
        assert str(expr) == "42"

        expr_float = Expression.literal(3.14)
        assert "3.14" in str(expr_float)

    def test_literal_string(self):
        """Test creating string literal expressions."""
        expr = Expression.literal("hello")
        assert "hello" in str(expr)

    def test_variable(self):
        """Test creating variable expressions."""
        expr = Expression.variable("x")
        assert str(expr) == "x"

    def test_quantity(self):
        """Test creating quantity literal expressions."""
        expr = Expression.quantity("100", "USD")
        assert "100" in str(expr)
        assert "USD" in str(expr)

    def test_time(self):
        """Test creating time literal expressions."""
        expr = Expression.time("2025-01-01T00:00:00Z")
        assert "2025-01-01" in str(expr)

    def test_interval(self):
        """Test creating interval literal expressions."""
        expr = Expression.interval("2025-01-01T09:00:00Z", "2025-01-01T17:00:00Z")
        assert "09:00:00" in str(expr)
        assert "17:00:00" in str(expr)

    def test_binary(self):
        """Test creating binary expressions."""
        left = Expression.variable("a")
        right = Expression.variable("b")
        expr = Expression.binary(BinaryOp.And, left, right)
        result = str(expr)
        assert "AND" in result
        assert "a" in result
        assert "b" in result

    def test_unary_not(self):
        """Test creating unary NOT expressions."""
        operand = Expression.variable("x")
        expr = Expression.unary(UnaryOp.Not, operand)
        result = str(expr)
        assert "NOT" in result
        assert "x" in result

    def test_unary_negate(self):
        """Test creating unary negation expressions."""
        operand = Expression.variable("x")
        expr = Expression.unary(UnaryOp.Negate, operand)
        assert "x" in str(expr)

    def test_member_access(self):
        """Test creating member access expressions."""
        expr = Expression.member_access("user", "name")
        result = str(expr)
        assert "user" in result
        assert "name" in result

    def test_cast(self):
        """Test creating cast expressions."""
        operand = Expression.variable("x")
        expr = Expression.cast(operand, "Money")
        result = str(expr)
        assert "x" in result
        assert "Money" in result

    def test_quantifier_forall(self):
        """Test creating ForAll quantifier expressions."""
        collection = Expression.variable("items")
        condition = Expression.variable("valid")
        expr = Expression.quantifier(Quantifier.ForAll, "x", collection, condition)
        result = str(expr)
        assert "ForAll" in result or "forall" in result.lower()

    def test_aggregation(self):
        """Test creating aggregation expressions."""
        collection = Expression.variable("items")
        expr = Expression.aggregation(AggregateFunction.Count, collection)
        result = str(expr)
        assert "COUNT" in result or "count" in result.lower()


class TestNormalization:
    """Tests for expression normalization."""

    def test_normalize_returns_normalized_expression(self):
        """Test that normalize() returns a NormalizedExpression."""
        expr = Expression.variable("x")
        normalized = expr.normalize()
        assert isinstance(normalized, NormalizedExpression)

    def test_identity_elimination_and_true(self):
        """Test that `true AND x` normalizes to `x`."""
        true_expr = Expression.literal(True)
        x_expr = Expression.variable("x")
        expr = Expression.binary(BinaryOp.And, true_expr, x_expr)
        normalized = expr.normalize()
        assert str(normalized) == "x"

    def test_identity_elimination_or_false(self):
        """Test that `false OR x` normalizes to `x`."""
        false_expr = Expression.literal(False)
        x_expr = Expression.variable("x")
        expr = Expression.binary(BinaryOp.Or, false_expr, x_expr)
        normalized = expr.normalize()
        assert str(normalized) == "x"

    def test_commutative_sorting(self):
        """Test that commutative operators sort operands."""
        b = Expression.variable("b")
        a = Expression.variable("a")
        expr = Expression.binary(BinaryOp.And, b, a)
        normalized = expr.normalize()
        # After normalization, should be (a AND b)
        result = str(normalized)
        assert result == "(a AND b)"

    def test_idempotence(self):
        """Test that `a AND a` normalizes to `a`."""
        a = Expression.variable("a")
        expr = Expression.binary(BinaryOp.And, a, a)
        normalized = expr.normalize()
        assert str(normalized) == "a"

    def test_double_negation_elimination(self):
        """Test that `NOT NOT x` normalizes to `x`."""
        x = Expression.variable("x")
        not_x = Expression.unary(UnaryOp.Not, x)
        not_not_x = Expression.unary(UnaryOp.Not, not_x)
        normalized = not_not_x.normalize()
        assert str(normalized) == "x"


class TestEquivalence:
    """Tests for expression equivalence checking."""

    def test_equivalent_expressions(self):
        """Test that equivalent expressions are recognized as equivalent."""
        a1 = Expression.variable("a")
        b1 = Expression.variable("b")
        expr1 = Expression.binary(BinaryOp.And, a1, b1)

        a2 = Expression.variable("a")
        b2 = Expression.variable("b")
        expr2 = Expression.binary(BinaryOp.And, b2, a2)

        assert expr1.is_equivalent(expr2)

    def test_non_equivalent_expressions(self):
        """Test that non-equivalent expressions are recognized as non-equivalent."""
        a = Expression.variable("a")
        b = Expression.variable("b")
        c = Expression.variable("c")

        expr1 = Expression.binary(BinaryOp.And, a, b)
        expr2 = Expression.binary(BinaryOp.And, a, c)

        assert not expr1.is_equivalent(expr2)


class TestStableHash:
    """Tests for stable hash functionality."""

    def test_stable_hash_is_deterministic(self):
        """Test that stable hash is deterministic."""
        expr = Expression.variable("x")
        hash1 = expr.normalize().stable_hash()
        hash2 = expr.normalize().stable_hash()
        assert hash1 == hash2

    def test_equivalent_expressions_have_same_hash(self):
        """Test that equivalent expressions have the same hash."""
        a1 = Expression.variable("a")
        b1 = Expression.variable("b")
        expr1 = Expression.binary(BinaryOp.And, a1, b1)

        a2 = Expression.variable("a")
        b2 = Expression.variable("b")
        expr2 = Expression.binary(BinaryOp.And, b2, a2)

        assert expr1.normalize().stable_hash() == expr2.normalize().stable_hash()


class TestNormalizedExpressionMethods:
    """Tests for NormalizedExpression methods."""

    def test_inner_expression(self):
        """Test that inner_expression returns an Expression."""
        expr = Expression.variable("x")
        normalized = expr.normalize()
        inner = normalized.inner_expression()
        assert isinstance(inner, Expression)
        assert str(inner) == "x"

    def test_str_representation(self):
        """Test string representation of NormalizedExpression."""
        expr = Expression.variable("x")
        normalized = expr.normalize()
        assert str(normalized) == "x"

    def test_repr_includes_hash(self):
        """Test that repr includes hash information."""
        expr = Expression.variable("x")
        normalized = expr.normalize()
        repr_str = repr(normalized)
        assert "NormalizedExpression" in repr_str
        assert "hash" in repr_str.lower()


class TestWindowSpec:
    """Tests for WindowSpec."""

    def test_window_spec_creation(self):
        """Test WindowSpec creation."""
        ws = WindowSpec(30, "days")
        assert ws.duration == 30
        assert ws.unit == "days"

    def test_window_spec_repr(self):
        """Test WindowSpec repr."""
        ws = WindowSpec(24, "hours")
        repr_str = repr(ws)
        assert "24" in repr_str
        assert "hours" in repr_str
