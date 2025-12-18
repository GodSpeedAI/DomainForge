import { describe, it, expect } from 'vitest';
import {
  Expression,
  NormalizedExpression,
  BinaryOp,
  UnaryOp,
  Quantifier,
  AggregateFunction,
  WindowSpec,
} from '..';

describe('Expression Factory Methods', () => {
  it('creates boolean literal expressions', () => {
    const exprTrue = Expression.literalBool(true);
    expect(exprTrue.toStringRepr()).toBe('true');

    const exprFalse = Expression.literalBool(false);
    expect(exprFalse.toStringRepr()).toBe('false');
  });

  it('creates number literal expressions', () => {
    const expr = Expression.literalNumber(42);
    expect(expr.toStringRepr()).toContain('42');
  });

  it('creates string literal expressions', () => {
    const expr = Expression.literalString('hello');
    expect(expr.toStringRepr()).toContain('hello');
  });

  it('creates variable expressions', () => {
    const expr = Expression.variable('x');
    expect(expr.toStringRepr()).toBe('x');
  });

  it('creates quantity expressions', () => {
    const expr = Expression.quantity('100', 'USD');
    const str = expr.toStringRepr();
    expect(str).toContain('100');
    expect(str).toContain('USD');
  });

  it('creates time expressions', () => {
    const expr = Expression.time('2025-01-01T00:00:00Z');
    expect(expr.toStringRepr()).toContain('2025-01-01');
  });

  it('creates interval expressions', () => {
    const expr = Expression.interval('2025-01-01T09:00:00Z', '2025-01-01T17:00:00Z');
    const str = expr.toStringRepr();
    expect(str).toContain('09:00:00');
    expect(str).toContain('17:00:00');
  });

  it('creates binary expressions', () => {
    const left = Expression.variable('a');
    const right = Expression.variable('b');
    const expr = Expression.binary(BinaryOp.And, left, right);
    const str = expr.toStringRepr();
    expect(str).toContain('AND');
    expect(str).toContain('a');
    expect(str).toContain('b');
  });

  it('creates unary NOT expressions', () => {
    const operand = Expression.variable('x');
    const expr = Expression.unary(UnaryOp.Not, operand);
    const str = expr.toStringRepr();
    expect(str).toContain('NOT');
    expect(str).toContain('x');
  });

  it('creates member access expressions', () => {
    const expr = Expression.memberAccess('user', 'name');
    const str = expr.toStringRepr();
    expect(str).toContain('user');
    expect(str).toContain('name');
  });

  it('creates cast expressions', () => {
    const operand = Expression.variable('x');
    const expr = Expression.cast(operand, 'Money');
    const str = expr.toStringRepr();
    expect(str).toContain('x');
    expect(str).toContain('Money');
  });

  it('creates quantifier expressions', () => {
    const collection = Expression.variable('items');
    const condition = Expression.variable('valid');
    const expr = Expression.quantifier(Quantifier.ForAll, 'x', collection, condition);
    const str = expr.toStringRepr().toLowerCase();
    expect(str).toContain('forall');
  });

  it('creates aggregation expressions', () => {
    const collection = Expression.variable('items');
    const expr = Expression.aggregation(AggregateFunction.Count, collection, null, null);
    const str = expr.toStringRepr().toLowerCase();
    expect(str).toContain('count');
  });
});

describe('Normalization', () => {
  it('returns a NormalizedExpression', () => {
    const expr = Expression.variable('x');
    const normalized = expr.normalize();
    expect(normalized).toBeDefined();
    expect(typeof normalized.stableHash).toBe('function');
  });

  it('eliminates identity (true AND x → x)', () => {
    const trueExpr = Expression.literalBool(true);
    const xExpr = Expression.variable('x');
    const expr = Expression.binary(BinaryOp.And, trueExpr, xExpr);
    const normalized = expr.normalize();
    expect(normalized.toStringRepr()).toBe('x');
  });

  it('eliminates identity (false OR x → x)', () => {
    const falseExpr = Expression.literalBool(false);
    const xExpr = Expression.variable('x');
    const expr = Expression.binary(BinaryOp.Or, falseExpr, xExpr);
    const normalized = expr.normalize();
    expect(normalized.toStringRepr()).toBe('x');
  });

  it('sorts commutative operands', () => {
    const b = Expression.variable('b');
    const a = Expression.variable('a');
    const expr = Expression.binary(BinaryOp.And, b, a);
    const normalized = expr.normalize();
    expect(normalized.toStringRepr()).toBe('(a AND b)');
  });

  it('applies idempotence (a AND a → a)', () => {
    const a = Expression.variable('a');
    const expr = Expression.binary(BinaryOp.And, a, a);
    const normalized = expr.normalize();
    expect(normalized.toStringRepr()).toBe('a');
  });

  it('eliminates double negation (NOT NOT x → x)', () => {
    const x = Expression.variable('x');
    const notX = Expression.unary(UnaryOp.Not, x);
    const notNotX = Expression.unary(UnaryOp.Not, notX);
    const normalized = notNotX.normalize();
    expect(normalized.toStringRepr()).toBe('x');
  });
});

describe('Equivalence', () => {
  it('recognizes equivalent expressions', () => {
    const a1 = Expression.variable('a');
    const b1 = Expression.variable('b');
    const expr1 = Expression.binary(BinaryOp.And, a1, b1);

    const a2 = Expression.variable('a');
    const b2 = Expression.variable('b');
    const expr2 = Expression.binary(BinaryOp.And, b2, a2);

    expect(expr1.isEquivalent(expr2)).toBe(true);
  });

  it('recognizes non-equivalent expressions', () => {
    const a = Expression.variable('a');
    const b = Expression.variable('b');
    const c = Expression.variable('c');

    const expr1 = Expression.binary(BinaryOp.And, a, b);
    const expr2 = Expression.binary(BinaryOp.And, a, c);

    expect(expr1.isEquivalent(expr2)).toBe(false);
  });
});

describe('Stable Hash', () => {
  it('is deterministic', () => {
    const expr = Expression.variable('x');
    const hash1 = expr.normalize().stableHash();
    const hash2 = expr.normalize().stableHash();
    expect(hash1).toBe(hash2);
  });

  it('is the same for equivalent expressions', () => {
    const a1 = Expression.variable('a');
    const b1 = Expression.variable('b');
    const expr1 = Expression.binary(BinaryOp.And, a1, b1);

    const a2 = Expression.variable('a');
    const b2 = Expression.variable('b');
    const expr2 = Expression.binary(BinaryOp.And, b2, a2);

    expect(expr1.normalize().stableHash()).toBe(expr2.normalize().stableHash());
  });

  it('has hex representation', () => {
    const expr = Expression.variable('x');
    const hashHex = expr.normalize().stableHashHex();
    expect(hashHex).toMatch(/^0x[0-9a-f]+$/);
  });
});

describe('NormalizedExpression Methods', () => {
  it('returns inner expression', () => {
    const expr = Expression.variable('x');
    const normalized = expr.normalize();
    const inner = normalized.innerExpression();
    expect(inner.toStringRepr()).toBe('x');
  });

  it('has string representation', () => {
    const expr = Expression.variable('x');
    const normalized = expr.normalize();
    expect(normalized.toStringRepr()).toBe('x');
  });

  it('supports equality check', () => {
    const a = Expression.variable('a');
    const b = Expression.variable('b');
    const expr1 = Expression.binary(BinaryOp.And, a, b);
    const expr2 = Expression.binary(BinaryOp.And, b, a);

    const norm1 = expr1.normalize();
    const norm2 = expr2.normalize();
    expect(norm1.equals(norm2)).toBe(true);
  });
});
