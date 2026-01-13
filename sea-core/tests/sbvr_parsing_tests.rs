use sea_core::sbvr::{RuleType, SbvrModel, TermType};

#[test]
fn test_sbvr_parsing_from_xml() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmi:XMI xmlns:xmi="http://www.omg.org/XMI" xmlns:sbvr="http://www.omg.org/spec/SBVR/20080801">
  <sbvr:Vocabulary name="Test_Model">
    <sbvr:GeneralConcept id="c1" name="Customer">
      <sbvr:Definition>A person who buys things</sbvr:Definition>
    </sbvr:GeneralConcept>
    <sbvr:IndividualConcept id="r1" name="GoldParams">
      <sbvr:Definition>Parameters for gold status</sbvr:Definition>
    </sbvr:IndividualConcept>
    <sbvr:FactType id="f1">
       <sbvr:Subject>c1</sbvr:Subject>
       <sbvr:Verb>has</sbvr:Verb>
       <sbvr:Object>r1</sbvr:Object>
    </sbvr:FactType>
    <sbvr:Obligation id="rule1" name="MustBeValid">
      <sbvr:Expression>It is obligatory that...</sbvr:Expression>
      <sbvr:Severity>Error</sbvr:Severity>
    </sbvr:Obligation>
  </sbvr:Vocabulary>
</xmi:XMI>"#;

    let model = SbvrModel::from_xmi(xml).expect("Failed to parse SBVR XML");

    // Check Vocabulary
    assert_eq!(model.vocabulary.len(), 2);
    let term1 = model
        .vocabulary
        .iter()
        .find(|t| t.name == "Customer")
        .unwrap();
    assert!(matches!(term1.term_type, TermType::GeneralConcept));
    assert_eq!(
        term1.definition.as_deref(),
        Some("A person who buys things")
    );

    // Check Facts
    assert_eq!(model.facts.len(), 1);
    let fact = &model.facts[0];
    assert_eq!(fact.subject, "c1");
    assert_eq!(fact.verb, "has");

    // Check Rules
    assert_eq!(model.rules.len(), 1);
    let rule = &model.rules[0];
    assert_eq!(rule.name, "MustBeValid");
    assert!(matches!(rule.rule_type, RuleType::Obligation));
    assert_eq!(rule.expression, "It is obligatory that...");
}
