//! Gate evidence for ADR-017: formatter fidelity for quantifiers,
//! `entity_instances of` collections, and `//` comments.
//!
//! The 16 unit tests in `formatter::printer::tests` cover these fixes in
//! depth; these integration tests prove the same behavior through the
//! public API (`formatter::format` + `parser::parse`).

use domainforge_core::formatter::{format, FormatConfig};
use domainforge_core::parser::parse;

fn round_trip(input: &str) -> String {
    let once = format(input, FormatConfig::default()).expect("fmt must succeed");
    parse(&once).unwrap_or_else(|e| panic!("formatted output must reparse: {once}\nerror: {e}"));
    let twice = format(&once, FormatConfig::default()).expect("re-fmt must succeed");
    assert_eq!(once, twice, "fmt(fmt(x)) must equal fmt(x):\n{once}");
    once
}

const QUANTIFIER_PREAMBLE: &str =
    "Entity \"A\"\nEntity \"B\"\nResource \"R\" units\nFlow \"R\" from \"A\" to \"B\" quantity 1\n";

#[test]
fn test_fmt_forall_round_trip() {
    let input =
        format!("{QUANTIFIER_PREAMBLE}\nPolicy p as:\n    forall f in flows: (f.quantity > 0)\n");
    let once = round_trip(&input);
    assert!(
        once.contains("forall f in flows:"),
        "quantifier keeps SEA spelling: {once}"
    );
    assert!(!once.contains("ForAll("), "no debug-style wrapper: {once}");
}

#[test]
fn test_fmt_exists_entity_instances_of_round_trip() {
    let input = "Entity \"A\"\n\nInstance a1 of \"A\" {\n    x: 5\n}\n\nPolicy p as:\n    exists e in entity_instances of \"A\": (e.x > 0)\n";
    let once = round_trip(input);
    assert!(
        once.contains("entity_instances of \"A\""),
        "internal encoding decoded back to SEA syntax: {once}"
    );
    assert!(
        !once.contains("entity_instances:A"),
        "no internal encoding leaks: {once}"
    );
}

#[test]
fn test_fmt_comments_preserved() {
    let input = "Entity \"A\" // trailing on entity\n// between declarations\nEntity \"B\"\n// end of file comment\n";
    let once = round_trip(input);
    for text in [
        "// trailing on entity",
        "// between declarations",
        "// end of file comment",
    ] {
        assert_eq!(
            once.matches(text).count(),
            1,
            "expected exactly one occurrence of {text:?} in:\n{once}"
        );
    }
    let entity_a_line = once
        .lines()
        .find(|l| l.contains("Entity \"A\""))
        .expect("Entity A line not found");
    assert!(
        entity_a_line.contains("// trailing on entity"),
        "trailing comment stays on its declaration's line: {entity_a_line}"
    );
}

#[test]
fn test_fmt_string_literal_slashes_not_a_comment() {
    let input = format!(
        "{QUANTIFIER_PREAMBLE}\nPolicy p as:\n    f.resource CONTAINS \"see https://example.com for details\"\n"
    );
    let once = round_trip(&input);
    assert!(
        once.contains("https://example.com"),
        "string literal content survives: {once}"
    );
    assert_eq!(
        once.lines()
            .filter(|l| l.trim_start().starts_with("//"))
            .count(),
        0,
        "no comment manufactured from a string literal:\n{once}"
    );
}
