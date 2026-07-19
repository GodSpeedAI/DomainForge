//! APP diagnostic registry contract (plan Task 10; reference §5): stable
//! codes, slugs, severity, sorting, closed APP014 reasons, and APP015 for
//! unparseable sources. Focused per-code cases live in
//! `application_contract_tests.rs` and `application_policy_tests.rs`.

use domainforge_core::application::{
    resolve_application_contract, ApplicationDiagnostic, ApplicationDiagnosticCode,
};
use serde_json::json;

#[test]
fn every_app_code_has_stable_slug_and_error_severity() {
    let expected = [
        ("APP001", "missing_application_semantics"),
        ("APP002", "unknown_contract_reference"),
        ("APP003", "invalid_record_shape"),
        ("APP004", "duplicate_field"),
        ("APP005", "invalid_aggregate_key"),
        ("APP006", "operation_type_mismatch"),
        ("APP007", "policy_scope_error"),
        ("APP008", "invalid_failure_declaration"),
        ("APP009", "invalid_strategy_reference"),
        ("APP010", "invalid_not_applicable"),
        ("APP011", "effect_state_mismatch"),
        ("APP012", "constraint_error"),
        ("APP013", "enum_error"),
        ("APP014", "closure_resolution_error"),
        ("APP015", "artifact_schema_violation"),
    ];
    assert_eq!(ApplicationDiagnosticCode::all_code_slugs(), expected);
    assert!(ApplicationDiagnosticCode::all()
        .iter()
        .all(|c| c.severity() == "error"));
}

fn resolve(entry: &str, sources: serde_json::Value) -> Vec<ApplicationDiagnostic> {
    resolve_application_contract(entry, &sources.to_string()).unwrap_err()
}

fn first(diags: &[ApplicationDiagnostic]) -> &ApplicationDiagnostic {
    diags.first().expect("at least one diagnostic")
}

#[test]
fn app014_symbol_collision_reason() {
    let diags = resolve(
        "main.sea",
        json!({"main.sea": "@namespace \"t\"\nrecord R {\n    x: string\n}\nrecord R {\n    y: string\n}\n"}),
    );
    let d = first(&diags);
    assert_eq!(d.code, ApplicationDiagnosticCode::App014);
    assert_eq!(d.context.reason.as_deref(), Some("symbol_collision"));
}

#[test]
fn app014_import_cycle_reason() {
    let diags = resolve(
        "a.sea",
        json!({
            "a.sea": "@namespace \"a\"\nimport { X } from \"./b.sea\"\nrecord Y {\n    n: int\n}\n",
            "b.sea": "@namespace \"b\"\nimport { Y } from \"./a.sea\"\nexport record X {\n    n: int\n}\n",
        }),
    );
    assert_eq!(
        first(&diags).context.reason.as_deref(),
        Some("import_cycle")
    );
}

#[test]
fn app014_unresolved_specifier_reason() {
    let diags = resolve(
        "a.sea",
        json!({"a.sea": "@namespace \"a\"\nimport { X } from \"./missing.sea\"\n"}),
    );
    assert_eq!(
        first(&diags).context.reason.as_deref(),
        Some("unresolved_specifier")
    );
}

#[test]
fn app014_unresolved_alias_reason() {
    let diags = resolve(
        "a.sea",
        json!({
            "a.sea": "@namespace \"a\"\nimport { Nope } from \"./b.sea\"\n",
            "b.sea": "@namespace \"b\"\nexport record X {\n    n: int\n}\n",
        }),
    );
    assert_eq!(
        first(&diags).context.reason.as_deref(),
        Some("unresolved_alias")
    );
}

#[test]
fn app014_not_exported_reason() {
    let diags = resolve(
        "a.sea",
        json!({
            "a.sea": "@namespace \"a\"\nimport { X } from \"./b.sea\"\n",
            "b.sea": "@namespace \"b\"\nrecord X {\n    n: int\n}\n",
        }),
    );
    assert_eq!(
        first(&diags).context.reason.as_deref(),
        Some("not_exported")
    );
}

#[test]
fn unparseable_source_is_app015() {
    let diags = resolve("a.sea", json!({"a.sea": "this is not SEA"}));
    assert_eq!(first(&diags).code, ApplicationDiagnosticCode::App015);
}

#[test]
fn diagnostics_sort_by_module_line_column_then_code() {
    // Two defective declarations in one module plus one in a second module.
    let diags = resolve(
        "a.sea",
        json!({
            "a.sea": "@namespace \"a\"\nimport { X } from \"./b.sea\"\nrecord R {\n    x: string\n    x: int\n}\nenum E {\n    m = \"w\",\n    m = \"w\"\n}\n",
            "b.sea": "@namespace \"b\"\nexport record X {\n    n: string\n    n: int\n}\n",
        }),
    );
    let keys: Vec<_> = diags
        .iter()
        .map(|d| {
            (
                d.context.logical_module_id.clone(),
                d.context.line,
                d.context.column,
                d.code,
            )
        })
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
    assert!(diags.len() >= 3);
}

#[test]
fn diagnostics_carry_source_evidence_and_error_severity() {
    let diags = resolve(
        "a.sea",
        json!({"a.sea": "@namespace \"a\"\nrecord R {\n    x: Missing\n}\n"}),
    );
    let d = first(&diags);
    assert_eq!(d.severity, "error");
    assert_eq!(d.slug, d.code.slug());
    assert_eq!(d.context.logical_module_id.as_deref(), Some("a.sea"));
    assert!(d.context.line.is_some() && d.context.column.is_some());
}
