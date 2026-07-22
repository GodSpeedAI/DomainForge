//! Reference §4/§5 field constraint and default semantics (M0 gate finding 1):
//! APP012 applicability/cardinality/interval/precision rules and APP003
//! default rules, table-driven per invariant.

use domainforge_core::application::{
    resolve_application_contract, ApplicationDiagnostic, TypedValue,
};
use serde_json::json;

fn entity_sources(extra_decls: &str, field_lines: &str) -> String {
    json!({
        "main.sea": format!(
            "@namespace \"t\"\n{extra_decls}\nentity \"E\" {{\n    key id: uuid\n{field_lines}\n}}\n"
        )
    })
    .to_string()
}

fn resolve(
    extra_decls: &str,
    field_lines: &str,
) -> Result<Vec<TypedValue>, Vec<ApplicationDiagnostic>> {
    resolve_application_contract("main.sea", &entity_sources(extra_decls, field_lines)).map(|doc| {
        doc.contract.entities[0]
            .fields
            .iter()
            .filter_map(|f| f.default.clone())
            .collect()
    })
}

fn assert_rejects(extra_decls: &str, field_lines: &str, code: &str, fragment: &str) {
    let diags = resolve(extra_decls, field_lines)
        .expect_err(&format!("expected {code} '{fragment}' for: {field_lines}"));
    assert!(
        diags
            .iter()
            .any(|d| d.code.code() == code && d.message.contains(fragment)),
        "expected {code} containing '{fragment}' for '{field_lines}', got {diags:?}"
    );
}

#[test]
fn app012_rejects_inapplicable_constraints() {
    for (field, fragment) in [
        ("    n: int (min_length 1)", "min_length"),
        ("    n: string (min_items 1)", "min_items"),
        ("    n: int (pattern Email)", "pattern"),
        ("    n: string (min 1)", "min"),
        ("    n: bool (max 1)", "max"),
        ("    n: list<string> (max_length 3)", "max_length"),
        ("    n: OrderKind (min 1)", "min"),
    ] {
        let decls = "pattern \"Email\" matches \"^[a-z]+$\"\nenum OrderKind { a = \"a\" }";
        assert_rejects(decls, field, "APP012", fragment);
        assert_rejects(decls, field, "APP012", "does not apply");
    }
}

#[test]
fn app012_rejects_duplicate_bound_family_members() {
    assert_rejects(
        "",
        "    n: int (min 1, exclusive_min 2)",
        "APP012",
        "more than one lower bound",
    );
    assert_rejects(
        "",
        "    n: int (max 5, exclusive_max 4)",
        "APP012",
        "more than one upper bound",
    );
    assert_rejects(
        "",
        "    n: string (max_length 5, max_length 6)",
        "APP012",
        "more than one max_length",
    );
    assert_rejects(
        "",
        "    n: list<int> (min_items 1, min_items 2)",
        "APP012",
        "more than one min_items",
    );
}

#[test]
fn app012_rejects_empty_intervals_under_inclusivity_rules() {
    assert_rejects("", "    n: int (min 5, max 3)", "APP012", "empty interval");
    assert_rejects(
        "",
        "    n: int (exclusive_min 5, max 5)",
        "APP012",
        "empty interval",
    );
    assert_rejects(
        "",
        "    n: int (min 5, exclusive_max 5)",
        "APP012",
        "empty interval",
    );
    assert_rejects(
        "",
        "    n: string (min_length 10, max_length 2)",
        "APP012",
        "empty interval",
    );
    assert_rejects(
        "",
        "    n: list<int> (min_items 5, max_items 1)",
        "APP012",
        "empty interval",
    );
    // Equal inclusive bounds denote a singleton and are valid.
    resolve("", "    n: int (min 5, max 5)").expect("inclusive singleton interval is valid");
}

#[test]
fn app003_rejects_forbidden_default_sites() {
    assert_rejects(
        "",
        "    n: string optional default \"x\"",
        "APP003",
        "optional",
    );
    assert_rejects(
        "",
        "    n: ref<E> default \"x\"",
        "APP003",
        "refs and lists",
    );
    assert_rejects(
        "",
        "    n: list<string> default \"x\"",
        "APP003",
        "refs and lists",
    );
}

#[test]
fn app003_rejects_defaults_violating_constraints() {
    assert_rejects(
        "",
        "    n: int (min 5) default 3",
        "APP003",
        "violates its min constraint",
    );
    assert_rejects(
        "",
        "    n: int (exclusive_max 5) default 5",
        "APP003",
        "violates its exclusive_max constraint",
    );
    assert_rejects(
        "",
        "    n: string (max_length 2) default \"toolong\"",
        "APP003",
        "violates its max_length constraint",
    );
    assert_rejects(
        "",
        "    n: string (min_length 4) default \"ab\"",
        "APP003",
        "violates its min_length constraint",
    );
    assert_rejects(
        "pattern \"Email\" matches \"^[a-z]+$\"",
        "    n: string (pattern Email) default \"NOPE\"",
        "APP003",
        "violates its pattern constraint",
    );
    // A satisfying default passes every applicable constraint.
    resolve(
        "pattern \"Email\" matches \"^[a-z]+$\"",
        "    n: string (min_length 1, max_length 8, pattern Email) default \"ok\"",
    )
    .expect("satisfying default is valid");
}

#[test]
fn quantity_defaults_lower_to_normalized_base_values() {
    let decls = "dimension \"Mass\"\nunit \"g\" of \"Mass\" factor 1 base \"g\"\nunit \"kg\" of \"Mass\" factor 1000 base \"g\"";
    let defaults = resolve(
        decls,
        "    weight: quantity<kg> (exclusive_min 0) default 2.50",
    )
    .expect("quantity default lowers");
    let [TypedValue::Quantity { base_value, .. }] = defaults.as_slice() else {
        panic!("expected one quantity default, got {defaults:?}");
    };
    // 2.50 kg -> 2500 g, canonical decimal form.
    assert_eq!(base_value.to_string(), "2500");

    // A default violating the authored-unit bound is APP003.
    assert_rejects(
        decls,
        "    weight: quantity<kg> (exclusive_min 0) default 0",
        "APP003",
        "violates its exclusive_min constraint",
    );
}

#[test]
fn scalar_defaults_are_canonically_normalized() {
    let defaults = resolve("", "    n: decimal default 1.50").expect("decimal default resolves");
    let [TypedValue::Decimal(d)] = defaults.as_slice() else {
        panic!("expected one decimal default, got {defaults:?}");
    };
    assert_eq!(d.to_string(), "1.5");
}
