//! Application contract construction tests (plan Task 7; Tasks 8+ extend).

use domainforge_core::application::{
    resolve_application_contract, ApplicationContractDocument, ApplicationDiagnostic,
};
use domainforge_core::concept_id::ConceptId;

fn resolve_fixture_contract(
    entry: &str,
) -> Result<ApplicationContractDocument, Vec<ApplicationDiagnostic>> {
    let sources = serde_json::json!({
        "flagship/command-write.sea": include_str!(
            "../../fixtures/application_generation/flagship/command-write.sea"),
        "flagship/query-read.sea": include_str!(
            "../../fixtures/application_generation/flagship/query-read.sea"),
    })
    .to_string();
    resolve_application_contract(entry, &sources)
}

fn resolve_single(source: &str) -> Result<ApplicationContractDocument, Vec<ApplicationDiagnostic>> {
    let sources = serde_json::json!({ "main.sea": source }).to_string();
    resolve_application_contract("main.sea", &sources)
}

fn codes(err: &[ApplicationDiagnostic]) -> Vec<String> {
    err.iter()
        .map(|d| {
            serde_json::to_value(d.code)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn resolves_flagship_types_without_changing_concept_identity() {
    let doc = resolve_fixture_contract("flagship/command-write.sea").unwrap();
    assert_eq!(
        doc.contract.records[0].id.0,
        "flagship.orders.record.PlaceOrderInput"
    );
    let order = doc
        .contract
        .entities
        .iter()
        .find(|e| e.name == "Order")
        .unwrap();
    assert_eq!(
        order.concept_id,
        ConceptId::from_concept("flagship.orders", "Order")
    );
    assert_eq!(order.key_field, "order_id");
    assert_eq!(
        doc.contract.enums[0].id.0,
        "flagship.orders.enum.OrderStatus"
    );
    assert!(doc.inputs.source_set_hash.starts_with("sha256:"));
}

#[test]
fn resolves_flagship_closure_through_relative_import() {
    let doc = resolve_fixture_contract("flagship/query-read.sea").unwrap();
    assert!(doc
        .contract
        .records
        .iter()
        .any(|r| r.id.0 == "flagship.orders.record.GetOrderStatusInput"));
    // The imported module's declarations are part of the same closure.
    assert!(doc.contract.entities.iter().any(|e| e.name == "Order"));
}

#[test]
fn unknown_named_type_is_app002() {
    let err = resolve_single("@namespace \"t\"\nrecord R {\n    x: Missing\n}\n").unwrap_err();
    assert_eq!(codes(&err), ["APP002"]);
}

#[test]
fn entity_used_as_dto_field_type_is_rejected() {
    let err = resolve_single(
        "@namespace \"t\"\nentity \"E\" {\n    key id: uuid\n}\nrecord R {\n    x: E\n}\n",
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["APP002"]);
}

#[test]
fn nested_record_field_is_app003() {
    let err = resolve_single(
        "@namespace \"t\"\nrecord Inner {\n    x: string\n}\nrecord Outer {\n    y: Inner\n}\n",
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["APP003"]);
}

#[test]
fn duplicate_fields_are_app004() {
    let err =
        resolve_single("@namespace \"t\"\nrecord R {\n    x: string\n    x: int\n}\n").unwrap_err();
    assert_eq!(codes(&err), ["APP004"]);
}

#[test]
fn entity_key_violations_are_app005() {
    // no key
    let err = resolve_single("@namespace \"t\"\nentity \"E\" {\n    x: string\n}\n").unwrap_err();
    assert_eq!(codes(&err), ["APP005"]);
    // multiple keys
    let err =
        resolve_single("@namespace \"t\"\nentity \"E\" {\n    key a: uuid\n    key b: uuid\n}\n")
            .unwrap_err();
    assert_eq!(codes(&err), ["APP005"]);
    // invalid key type
    let err = resolve_single("@namespace \"t\"\nentity \"E\" {\n    key a: int\n}\n").unwrap_err();
    assert_eq!(codes(&err), ["APP005"]);
}

#[test]
fn record_defaults_are_app003() {
    let err = resolve_single("@namespace \"t\"\nrecord R {\n    x: string default \"v\"\n}\n")
        .unwrap_err();
    assert_eq!(codes(&err), ["APP003"]);
}

#[test]
fn enum_duplicate_names_and_wires_are_app013() {
    let err = resolve_single("@namespace \"t\"\nenum E {\n    a = \"a\",\n    a = \"b\"\n}\n")
        .unwrap_err();
    assert_eq!(codes(&err), ["APP013"]);
    let err =
        resolve_single("@namespace \"t\"\nenum E {\n    a = \"same\",\n    b = \"same\"\n}\n")
            .unwrap_err();
    assert_eq!(codes(&err), ["APP013"]);
}

#[test]
fn operation_input_resolving_to_enum_is_app006() {
    let err = resolve_single(
        "@namespace \"t\"\nenum E {\n    a = \"a\"\n}\nrecord Out {\n    x: string\n}\noperation op {\n    input E\n    output Out\n}\n",
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["APP006"]);
}

#[test]
fn operation_unknown_input_is_app002() {
    let err = resolve_single(
        "@namespace \"t\"\nrecord Out {\n    x: string\n}\noperation op {\n    input Missing\n    output Out\n}\n",
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["APP002"]);
}

#[test]
fn invalid_enum_default_wire_is_app003() {
    let err = resolve_single(
        "@namespace \"t\"\nenum Status {\n    a = \"a\"\n}\nentity \"E\" {\n    key id: uuid\n    s: Status default \"nope\"\n}\n",
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["APP003"]);
}
