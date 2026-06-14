#![cfg(feature = "cli")]

use sea_core::parser::parse_to_graph;

const MODEL_WITH_BREAKING_CHANGE_AND_POLICY: &str = r#"
Entity "Vendor" v1.0.0 in procurement

Resource "Payment" USD in finance

Flow "Payment" from "Vendor" to "Vendor" quantity 100

policy vendor_policy per Constraint Obligation priority 1 as: forall f in flows: (f.quantity <= 100)

ConceptChange "VendorMigration"
  @from_version v1.0.0
  @to_version v2.0.0
  @migration_policy mandatory
  @breaking_change true
"#;

const MODEL_WITH_BREAKING_CHANGE_NO_POLICY: &str = r#"
Entity "Vendor" v1.0.0 in procurement

ConceptChange "VendorMigration"
  @from_version v1.0.0
  @to_version v2.0.0
  @migration_policy mandatory
  @breaking_change true
"#;

const MODEL_WITH_NON_BREAKING_CHANGE_AND_POLICY: &str = r#"
Entity "Vendor" v1.0.0 in procurement

Resource "Payment" USD in finance

Flow "Payment" from "Vendor" to "Vendor" quantity 100

policy vendor_policy per Constraint Obligation priority 1 as: forall f in flows: (f.quantity <= 100)

ConceptChange "VendorMinorUpdate"
  @from_version v1.0.0
  @to_version v1.1.0
  @migration_policy optional
  @breaking_change false
"#;

#[test]
fn breaking_concept_change_is_detected() {
    let graph = parse_to_graph(MODEL_WITH_BREAKING_CHANGE_AND_POLICY)
        .expect("parsed model with breaking change + policy");
    let result = graph.validate_concept_change_compatibility();
    assert!(
        result.is_err(),
        "breaking ConceptChange alongside active policies must be flagged"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("VendorMigration"),
        "error must name the breaking change; got: {err}"
    );
}

#[test]
fn non_breaking_concept_change_passes() {
    let graph = parse_to_graph(MODEL_WITH_NON_BREAKING_CHANGE_AND_POLICY)
        .expect("parsed model with non-breaking change + policy");
    assert!(
        graph.validate_concept_change_compatibility().is_ok(),
        "non-breaking ConceptChange must not trigger the gate"
    );
}

#[test]
fn breaking_change_without_policies_passes() {
    let graph = parse_to_graph(MODEL_WITH_BREAKING_CHANGE_NO_POLICY)
        .expect("parsed model with breaking change, no policy");
    assert!(
        graph.validate_concept_change_compatibility().is_ok(),
        "breaking change without active policies must not trigger the gate"
    );
}
