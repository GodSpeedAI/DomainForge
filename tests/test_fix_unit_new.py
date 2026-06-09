"""
Tests for fix_unit_new.py — the fix_unit_new() function that rewrites
Unit::new(...) calls in Rust source to add .unwrap().
"""

import sys
import os

# Add the repo root to sys.path so we can import fix_unit_new directly
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from fix_unit_new import fix_unit_new


# ---------------------------------------------------------------------------
# Basic transformation cases
# ---------------------------------------------------------------------------

def test_adds_unwrap_to_bare_unit_new():
    """Unit::new(...) without any chained call gets .unwrap() appended."""
    source = 'let u = Unit::new("m", "meter");'
    result = fix_unit_new(source)
    assert 'Unit::new("m", "meter").unwrap()' in result


def test_does_not_double_wrap_already_unwrapped():
    """Unit::new(...).unwrap() is left unchanged."""
    source = 'let u = Unit::new("m", "meter").unwrap();'
    result = fix_unit_new(source)
    # Should appear exactly once and not become .unwrap().unwrap()
    assert result.count(".unwrap()") == 1
    assert ".unwrap().unwrap()" not in result


def test_does_not_modify_already_expected():
    """Unit::new(...).expect("...") is left unchanged."""
    source = 'let u = Unit::new("m", "meter").expect("invalid unit");'
    result = fix_unit_new(source)
    assert ".expect(" in result
    assert ".unwrap()" not in result


def test_handles_nested_parens_in_arguments():
    """Unit::new with nested function call arguments is handled correctly."""
    source = 'let u = Unit::new(get_symbol(), "meter");'
    result = fix_unit_new(source)
    assert "Unit::new(get_symbol(), \"meter\").unwrap()" in result


def test_multiple_unit_new_calls_in_one_file():
    """All Unit::new calls in the content get .unwrap() added."""
    source = (
        'let a = Unit::new("m", "meter");\n'
        'let b = Unit::new("kg", "kilogram");\n'
        'let c = Unit::new("s", "second");\n'
    )
    result = fix_unit_new(source)
    assert result.count(".unwrap()") == 3


def test_mixed_already_fixed_and_bare():
    """Only bare Unit::new calls are modified; already-fixed calls are left alone."""
    source = (
        'let a = Unit::new("m", "meter");\n'
        'let b = Unit::new("kg", "kilogram").unwrap();\n'
    )
    result = fix_unit_new(source)
    assert result.count(".unwrap()") == 2  # one original + one added
    # The already-unwrapped one should not be double-wrapped
    assert ".unwrap().unwrap()" not in result


def test_no_modification_when_no_unit_new():
    """Content without Unit::new is returned unchanged."""
    source = "let x = 42;\nfn foo() {}"
    result = fix_unit_new(source)
    assert result == source


def test_empty_string_returns_empty():
    """Empty string input returns empty string."""
    assert fix_unit_new("") == ""


def test_unit_new_with_numeric_arguments():
    """Unit::new with numeric arguments is handled."""
    source = "let u = Unit::new(1.0, 0.001);"
    result = fix_unit_new(source)
    assert "Unit::new(1.0, 0.001).unwrap()" in result


def test_unit_new_inline_in_expression():
    """Unit::new appearing inline in a larger expression gets .unwrap()."""
    source = "let u = foo(Unit::new(\"m\", \"meter\"));"
    result = fix_unit_new(source)
    assert "Unit::new(\"m\", \"meter\").unwrap()" in result


def test_transformation_is_idempotent():
    """Applying fix_unit_new twice produces the same result as applying it once."""
    source = 'let a = Unit::new("m", "meter");\nlet b = Unit::new("kg", "kilogram");'
    once = fix_unit_new(source)
    twice = fix_unit_new(once)
    assert once == twice


def test_preserves_surrounding_code():
    """fix_unit_new only modifies Unit::new calls, leaving everything else intact."""
    source = (
        "// A comment\n"
        'let x = Unit::new("m", "meter");\n'
        "fn other_function() { return 42; }\n"
    )
    result = fix_unit_new(source)
    assert "// A comment" in result
    assert "fn other_function() { return 42; }" in result
    assert 'Unit::new("m", "meter").unwrap()' in result


def test_unit_new_at_end_of_line_without_semicolon():
    """Unit::new at end of content without semicolon still gets .unwrap()."""
    source = 'Unit::new("m", "meter")'
    result = fix_unit_new(source)
    assert result == 'Unit::new("m", "meter").unwrap()'


def test_does_not_affect_other_new_calls():
    """Other::new() calls that are not Unit::new are not modified."""
    source = 'let x = Other::new("arg");'
    result = fix_unit_new(source)
    assert result == source


def test_unit_new_with_single_argument():
    """Unit::new with a single argument is handled."""
    source = 'Unit::new("m")'
    result = fix_unit_new(source)
    assert 'Unit::new("m").unwrap()' in result