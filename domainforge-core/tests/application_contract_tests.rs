//! Application contract construction tests (plan Task 7; Tasks 8+ extend).

use domainforge_core::application::{
    resolve_application_contract, AccessMode, ApplicationContractDocument, ApplicationDiagnostic,
    ConcurrencyStrategy, EffectKind, IdempotencyStrategy, OperationContract, TransactionBoundary,
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
    let clauses = READ_CLAUSES_OK.replace("input In", "input Kind");
    let err = resolve_single(&(read_op(&clauses) + "enum Kind {\n    a = \"a\"\n}\n")).unwrap_err();
    assert_eq!(codes(&err), ["APP006"]);
}

#[test]
fn operation_unknown_input_is_app002() {
    let clauses = READ_CLAUSES_OK.replace("input In", "input Missing");
    let err = resolve_single(&read_op(&clauses)).unwrap_err();
    assert_eq!(codes(&err), ["APP002"]);
}

fn operation<'a>(doc: &'a ApplicationContractDocument, name: &str) -> &'a OperationContract {
    doc.contract
        .operations
        .iter()
        .find(|o| o.name == name)
        .unwrap()
}

#[test]
fn command_and_query_lower_to_the_closed_v01_semantics() {
    let command = resolve_fixture_contract("flagship/command-write.sea").unwrap();
    let place = operation(&command, "place_order");
    assert!(matches!(place.effect, EffectKind::Creates));
    assert!(matches!(
        place.transaction,
        TransactionBoundary::SingleAggregate
    ));
    assert!(matches!(place.idempotency,
        IdempotencyStrategy::KeyedBy { ref field } if field == "client_order_id"));
    assert!(matches!(place.concurrency,
        ConcurrencyStrategy::UniqueKey { ref field } if field == "client_order_id"));
    assert!(matches!(place.access, AccessMode::PolicyGoverned { .. }));
    assert_eq!(place.id.0, "flagship.orders.operation.place_order");
    assert_eq!(place.input.0, "flagship.orders.record.PlaceOrderInput");
    assert_eq!(place.failures.len(), 3);

    let query = resolve_fixture_contract("flagship/query-read.sea").unwrap();
    let read = operation(&query, "get_order_status");
    assert!(matches!(read.access, AccessMode::Public));
    assert!(matches!(read.effect, EffectKind::Reads));
    assert!(matches!(read.transaction, TransactionBoundary::ReadOnly));
    assert!(matches!(read.idempotency, IdempotencyStrategy::Inherent));
    assert!(matches!(
        read.concurrency,
        ConcurrencyStrategy::ReadSnapshot
    ));
}

const READ_PREAMBLE: &str = "@namespace \"t\"\n\
entity \"E\" {\n    key id: uuid\n    n: int\n}\n\
record In {\n    id: uuid\n}\n\
record Out {\n    id: uuid\n}\n";

fn read_op(clauses: &str) -> String {
    format!("{READ_PREAMBLE}operation op {{\n{clauses}}}\n")
}

const READ_CLAUSES_OK: &str = "    intent \"read\"\n    direction inbound\n    actor anonymous\n    access public\n    input In\n    output Out\n    state E\n    effect reads E\n    transaction read_only\n    failure not_found for missing_state \"absent\"\n    idempotency inherent\n    concurrency read_snapshot\n    evidence operation_trace\n    lifecycle synchronous_request_response\n";

#[test]
fn valid_inline_read_operation_lowers() {
    resolve_single(&read_op(READ_CLAUSES_OK)).unwrap();
}

#[test]
fn app001_reports_all_clause_defects_together() {
    // Missing intent + outbound direction + wrong transaction: one APP001.
    let clauses = READ_CLAUSES_OK
        .replace("    intent \"read\"\n", "")
        .replace("direction inbound", "direction outbound")
        .replace("transaction read_only", "transaction single_aggregate");
    let err = resolve_single(&read_op(&clauses)).unwrap_err();
    assert_eq!(codes(&err), ["APP001"]);
    let msg = &err[0].message;
    for needle in ["intent", "outbound", "transaction"] {
        assert!(msg.contains(needle), "APP001 message must mention {needle}");
    }
}

#[test]
fn app001_covers_duplicate_and_out_of_order_clauses() {
    let dup = READ_CLAUSES_OK.replace(
        "    direction inbound\n",
        "    direction inbound\n    direction inbound\n",
    );
    assert_eq!(
        codes(&resolve_single(&read_op(&dup)).unwrap_err()),
        ["APP001"]
    );
    let out_of_order = READ_CLAUSES_OK.replace(
        "    intent \"read\"\n    direction inbound\n",
        "    direction inbound\n    intent \"read\"\n",
    );
    assert_eq!(
        codes(&resolve_single(&read_op(&out_of_order)).unwrap_err()),
        ["APP001"]
    );
}

#[test]
fn anonymous_actor_requires_public_access() {
    let src = "@namespace \"t\"\n\
policy p per Constraint Obligation priority 1 as: n >= 0\n\
entity \"E\" {\n    key id: uuid\n    n: int\n}\n\
record In {\n    id: uuid\n}\nrecord Out {\n    id: uuid\n}\n\
operation op {\n    intent \"read\"\n    direction inbound\n    actor anonymous\n    access policy_governed by p at precondition fails with not_found\n    input In\n    output Out\n    state E\n    effect reads E\n    transaction read_only\n    failure not_found for missing_state \"absent\"\n    idempotency inherent\n    concurrency read_snapshot\n    evidence operation_trace\n    lifecycle synchronous_request_response\n}\n";
    assert_eq!(codes(&resolve_single(src).unwrap_err()), ["APP001"]);
}

#[test]
fn failure_defects_are_app008() {
    // Uppercase failure code.
    let bad_code = READ_CLAUSES_OK.replace("failure not_found", "failure NotFound");
    let bad_code = bad_code.replace("with not_found", "with NotFound");
    assert!(
        codes(&resolve_single(&read_op(&bad_code)).unwrap_err()).contains(&"APP008".to_string())
    );
    // Duplicate kind across failures in one operation.
    let dup_kind = READ_CLAUSES_OK.replace(
        "    failure not_found for missing_state \"absent\"\n",
        "    failure not_found for missing_state \"absent\"\n    failure gone for missing_state \"also absent\"\n",
    );
    assert!(
        codes(&resolve_single(&read_op(&dup_kind)).unwrap_err()).contains(&"APP008".to_string())
    );
    // Missing mapped path: reads without a missing_state failure.
    let no_missing_state = READ_CLAUSES_OK.replace(
        "    failure not_found for missing_state \"absent\"\n",
        "    failure oops for input_validation \"bad input\"\n",
    );
    assert!(
        codes(&resolve_single(&read_op(&no_missing_state)).unwrap_err())
            .contains(&"APP008".to_string())
    );
}

#[test]
fn strategy_effect_mismatches_are_app009() {
    // reads with keyed_by idempotency.
    let keyed = READ_CLAUSES_OK.replace("idempotency inherent", "idempotency keyed_by id");
    assert!(codes(&resolve_single(&read_op(&keyed)).unwrap_err()).contains(&"APP009".to_string()));
    // reads with unique_key concurrency.
    let unique = READ_CLAUSES_OK.replace("concurrency read_snapshot", "concurrency unique_key id");
    assert!(codes(&resolve_single(&read_op(&unique)).unwrap_err()).contains(&"APP009".to_string()));
}

#[test]
fn not_applicable_is_legal_only_on_reads() {
    let ok = READ_CLAUSES_OK.replace(
        "idempotency inherent",
        "idempotency not_applicable(read_only, \"no retry semantics\")",
    );
    let doc = resolve_single(&read_op(&ok)).unwrap();
    assert!(matches!(
        operation(&doc, "op").idempotency,
        IdempotencyStrategy::NotApplicable(_)
    ));
}

#[test]
fn app001_missing_semantics_fixture_fails() {
    let sources = serde_json::json!({ "main.sea": include_str!(
        "../../fixtures/application_generation/invalid/app001-missing-application-semantics.sea") })
    .to_string();
    let err = resolve_application_contract("main.sea", &sources).unwrap_err();
    assert_eq!(codes(&err), ["APP001"]);
}

#[test]
fn app011_effect_state_mismatch_fixture_fails() {
    let sources = serde_json::json!({ "main.sea": include_str!(
        "../../fixtures/application_generation/invalid/app011-effect-state-mismatch.sea") })
    .to_string();
    let err = resolve_application_contract("main.sea", &sources).unwrap_err();
    assert!(codes(&err).contains(&"APP011".to_string()));
}

#[test]
fn create_without_required_state_source_is_app011() {
    // Entity field `n` is required with no default and absent from the input.
    let src = "@namespace \"t\"\n\
entity \"E\" {\n    key id: uuid\n    n: int\n}\n\
record In {\n    id: uuid\n    n: int\n}\n\
record Bad {\n    id: uuid\n}\n\
record Out {\n    id: uuid\n}\n\
operation op {\n    intent \"create\"\n    direction inbound\n    actor anonymous\n    access public\n    input Bad\n    output Out\n    state E\n    effect creates E\n    transaction single_aggregate\n    failure conflict for idempotency_conflict, concurrency_conflict \"dup\"\n    idempotency keyed_by id\n    concurrency unique_key id\n    evidence operation_trace\n    lifecycle synchronous_request_response\n}\n";
    assert!(codes(&resolve_single(src).unwrap_err()).contains(&"APP011".to_string()));
}

#[test]
fn output_projection_must_match_state_fields() {
    // Output field `extra` projects nothing in state.
    let bad_out = read_op(&READ_CLAUSES_OK.replace("output Out", "output Wide"))
        + "record Wide {\n    id: uuid\n    extra: string\n}\n";
    assert!(codes(&resolve_single(&bad_out).unwrap_err()).contains(&"APP011".to_string()));
}

// ---- shared Graph/contract ownership (plan Task 13) ----

fn flagship_sources_json() -> String {
    serde_json::json!({
        "flagship/command-write.sea": include_str!(
            "../../fixtures/application_generation/flagship/command-write.sea"),
        "flagship/query-read.sea": include_str!(
            "../../fixtures/application_generation/flagship/query-read.sea"),
    })
    .to_string()
}

#[test]
fn graph_and_contract_share_order_and_role_concept_ids() {
    use domainforge_core::application::{resolve_application_graph, ActorRef};
    let sources = flagship_sources_json();
    let doc = resolve_fixture_contract("flagship/command-write.sea").unwrap();
    let graph = resolve_application_graph("flagship/command-write.sea", &sources).unwrap();
    let order = doc
        .contract
        .entities
        .iter()
        .find(|e| e.name == "Order")
        .unwrap();
    assert!(graph.has_entity(&order.concept_id));
    let place = operation(&doc, "place_order");
    let ActorRef::Role { role } = &place.actor else {
        panic!("place_order must have the Customer actor");
    };
    assert!(graph.get_role(role).is_some());
}

#[test]
fn diamond_imports_build_one_graph_without_duplicates() {
    use domainforge_core::application::resolve_application_graph;
    use domainforge_core::concept_id::ConceptId;
    let sources = serde_json::json!({
        "base.sea": "@namespace \"d\"\nexport entity \"Base\" {\n    key id: uuid\n}\n",
        "left.sea": "@namespace \"d\"\nimport { Base } from \"./base.sea\"\nexport record L {\n    id: uuid\n}\n",
        "right.sea": "@namespace \"d\"\nimport { Base } from \"./base.sea\"\nexport record R {\n    id: uuid\n}\n",
        "top.sea": "@namespace \"d\"\nimport { L } from \"./left.sea\"\nimport { R } from \"./right.sea\"\n",
    })
    .to_string();
    let graph = resolve_application_graph("top.sea", &sources).unwrap();
    assert!(graph.has_entity(&ConceptId::from_concept("d", "Base")));
}

// ---- canonicalization (plan Task 11) ----

#[test]
fn source_set_hash_is_framed_sorted_and_path_independent() {
    use domainforge_core::application::source_set_hash;
    let a = source_set_hash([("b.sea", "Entity \"B\""), ("a.sea", "Entity \"A\"")]).unwrap();
    let b = source_set_hash([("a.sea", "Entity \"A\""), ("b.sea", "Entity \"B\"")]).unwrap();
    assert_eq!(a, b);
    assert!(a.starts_with("sha256:"));
    assert_eq!(a.len(), 71);
    // NFC-equivalent logical IDs hash identically; duplicates are rejected.
    let composed = source_set_hash([("caf\u{e9}.sea", "x")]).unwrap();
    let decomposed = source_set_hash([("cafe\u{301}.sea", "x")]).unwrap();
    assert_eq!(composed, decomposed);
    assert!(source_set_hash([("a.sea", "x"), ("a.sea", "y")]).is_err());
}

#[test]
fn canonical_decimal_strips_trailing_zeros() {
    use domainforge_core::application::canonical_decimal;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    assert_eq!(
        canonical_decimal(Decimal::from_str("125.00").unwrap()),
        "125"
    );
    assert_eq!(canonical_decimal(Decimal::from_str("0.00").unwrap()), "0");
    assert_eq!(canonical_decimal(Decimal::from_str("1.50").unwrap()), "1.5");
}

#[test]
fn input_fingerprint_is_key_order_and_scale_independent() {
    use domainforge_core::application::{input_fingerprint, TypedValue};
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let mut a = indexmap::IndexMap::new();
    a.insert(
        "total".to_string(),
        TypedValue::Decimal(Decimal::from_str("125.00").unwrap()),
    );
    a.insert("id".to_string(), TypedValue::Int(1));
    let mut b = indexmap::IndexMap::new();
    b.insert("id".to_string(), TypedValue::Int(1));
    b.insert(
        "total".to_string(),
        TypedValue::Decimal(Decimal::from_str("125").unwrap()),
    );
    assert_eq!(input_fingerprint(&a), input_fingerprint(&b));
    assert!(input_fingerprint(&a).starts_with("sha256:"));
}

#[test]
fn document_self_hash_omits_only_self_hash() {
    use domainforge_core::application::document_self_hash;
    let doc = resolve_fixture_contract("flagship/command-write.sea").unwrap();
    assert_eq!(doc.self_hash, document_self_hash(&doc));
    assert_ne!(doc.self_hash, format!("sha256:{}", "0".repeat(64)));
    // The self-hash covers every other field, including inputs hashes.
    let mut tampered = doc.clone();
    tampered.inputs.source_set_hash = format!("sha256:{}", "1".repeat(64));
    assert_ne!(document_self_hash(&tampered), doc.self_hash);
}

#[test]
fn invalid_enum_default_wire_is_app003() {
    let err = resolve_single(
        "@namespace \"t\"\nenum Status {\n    a = \"a\"\n}\nentity \"E\" {\n    key id: uuid\n    s: Status default \"nope\"\n}\n",
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["APP003"]);
}
