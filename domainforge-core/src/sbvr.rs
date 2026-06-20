use crate::graph::Graph;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum SbvrError {
    SerializationError(String),
    UnsupportedConstruct(String),
}

impl std::fmt::Display for SbvrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SbvrError::SerializationError(msg) => write!(f, "SBVR serialization error: {}", msg),
            SbvrError::UnsupportedConstruct(msg) => write!(f, "Unsupported construct: {}", msg),
        }
    }
}

impl std::error::Error for SbvrError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbvrTerm {
    pub id: String,
    pub name: String,
    pub term_type: TermType,
    pub definition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TermType {
    GeneralConcept,
    IndividualConcept,
    VerbConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbvrFactType {
    pub id: String,
    pub subject: String,
    pub verb: String,
    pub object: String,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default = "SbvrFactType::default_schema_version")]
    pub schema_version: String,
}

impl SbvrFactType {
    pub fn default_schema_version() -> String {
        "2.0".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbvrBusinessRule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub expression: String,
    pub severity: String,
    #[serde(default)]
    pub priority: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Obligation,
    Prohibition,
    Permission,
    Derivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbvrModel {
    pub vocabulary: Vec<SbvrTerm>,
    pub facts: Vec<SbvrFactType>,
    pub rules: Vec<SbvrBusinessRule>,
}

impl SbvrModel {
    pub fn new() -> Self {
        Self {
            vocabulary: Vec::new(),
            facts: Vec::new(),
            rules: Vec::new(),
        }
    }

    pub fn from_graph(graph: &Graph) -> Result<Self, SbvrError> {
        let mut model = Self::new();
        let mut relation_predicates: HashSet<String> = HashSet::new();

        for entity in graph.all_entities() {
            model.vocabulary.push(SbvrTerm {
                id: entity.id().to_string(),
                name: entity.name().to_string(),
                term_type: TermType::GeneralConcept,
                definition: Some(format!("Entity: {}", entity.name())),
            });
        }

        for role in graph.all_roles() {
            model.vocabulary.push(SbvrTerm {
                id: role.id().to_string(),
                name: role.name().to_string(),
                term_type: TermType::GeneralConcept,
                definition: Some(format!("Role: {}", role.name())),
            });
        }

        for resource in graph.all_resources() {
            model.vocabulary.push(SbvrTerm {
                id: resource.id().to_string(),
                name: resource.name().to_string(),
                term_type: TermType::IndividualConcept,
                definition: Some(format!(
                    "Resource: {} ({})",
                    resource.name(),
                    resource.unit()
                )),
            });
        }

        for pattern in graph.all_patterns() {
            model.vocabulary.push(SbvrTerm {
                id: pattern.id().to_string(),
                name: pattern.name().to_string(),
                term_type: TermType::IndividualConcept,
                definition: Some(format!(
                    "Pattern '{}' matches {}",
                    pattern.name(),
                    pattern.regex()
                )),
            });
        }

        model.vocabulary.push(SbvrTerm {
            id: "verb:transfers".to_string(),
            name: "transfers".to_string(),
            term_type: TermType::VerbConcept,
            definition: Some("Transfer of resource between entities".to_string()),
        });

        for relation in graph.all_relations() {
            if relation_predicates.insert(relation.predicate().to_string()) {
                model.vocabulary.push(SbvrTerm {
                    id: format!("verb:{}", relation.predicate()),
                    name: relation.predicate().to_string(),
                    term_type: TermType::VerbConcept,
                    definition: Some(format!(
                        "Fact type predicate '{}' connecting declared roles",
                        relation.predicate()
                    )),
                });
            }

            model.facts.push(SbvrFactType {
                id: relation.id().to_string(),
                subject: graph
                    .get_role(relation.subject_role())
                    .map(|role| role.name().to_string())
                    .unwrap_or_else(|| relation.subject_role().to_string()),
                verb: relation.predicate().to_string(),
                object: graph
                    .get_role(relation.object_role())
                    .map(|role| role.name().to_string())
                    .unwrap_or_else(|| relation.object_role().to_string()),
                destination: relation.via_flow().map(|id| {
                    graph
                        .get_resource(id)
                        .map(|resource| resource.name().to_string())
                        .unwrap_or_else(|| id.to_string())
                }),
                schema_version: SbvrFactType::default_schema_version(),
            });
        }

        for flow in graph.all_flows() {
            model.facts.push(SbvrFactType {
                id: flow.id().to_string(),
                subject: flow.from_id().to_string(),
                verb: "transfers".to_string(),
                object: flow.resource_id().to_string(),
                destination: Some(flow.to_id().to_string()),
                schema_version: SbvrFactType::default_schema_version(),
            });
        }

        Ok(model)
    }

    pub fn to_xmi(&self) -> Result<String, SbvrError> {
        let mut xmi = String::new();

        xmi.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xmi.push_str("<xmi:XMI xmlns:xmi=\"http://www.omg.org/XMI\" xmlns:sbvr=\"http://www.omg.org/spec/SBVR/20080801\">\n");
        xmi.push_str("  <sbvr:Vocabulary name=\"SEA_Model\">\n");

        for term in &self.vocabulary {
            match term.term_type {
                TermType::GeneralConcept => {
                    xmi.push_str(&format!(
                        "    <sbvr:GeneralConcept id=\"{}\" name=\"{}\">\n",
                        Self::escape_xml(&term.id),
                        Self::escape_xml(&term.name)
                    ));
                    if let Some(def) = &term.definition {
                        xmi.push_str(&format!(
                            "      <sbvr:Definition>{}</sbvr:Definition>\n",
                            Self::escape_xml(def)
                        ));
                    }
                    xmi.push_str("    </sbvr:GeneralConcept>\n");
                }
                TermType::IndividualConcept => {
                    xmi.push_str(&format!(
                        "    <sbvr:IndividualConcept id=\"{}\" name=\"{}\">\n",
                        Self::escape_xml(&term.id),
                        Self::escape_xml(&term.name)
                    ));
                    if let Some(def) = &term.definition {
                        xmi.push_str(&format!(
                            "      <sbvr:Definition>{}</sbvr:Definition>\n",
                            Self::escape_xml(def)
                        ));
                    }
                    xmi.push_str("    </sbvr:IndividualConcept>\n");
                }
                TermType::VerbConcept => {
                    xmi.push_str(&format!(
                        "    <sbvr:VerbConcept id=\"{}\" name=\"{}\">\n",
                        Self::escape_xml(&term.id),
                        Self::escape_xml(&term.name)
                    ));
                    if let Some(def) = &term.definition {
                        xmi.push_str(&format!(
                            "      <sbvr:Definition>{}</sbvr:Definition>\n",
                            Self::escape_xml(def)
                        ));
                    }
                    xmi.push_str("    </sbvr:VerbConcept>\n");
                }
            }
        }

        for fact in &self.facts {
            xmi.push_str(&format!(
                "    <sbvr:FactType id=\"{}\">\n",
                Self::escape_xml(&fact.id)
            ));
            xmi.push_str(&format!(
                "      <sbvr:SchemaVersion>{}</sbvr:SchemaVersion>\n",
                Self::escape_xml(&fact.schema_version)
            ));
            xmi.push_str(&format!(
                "      <sbvr:Subject>{}</sbvr:Subject>\n",
                Self::escape_xml(&fact.subject)
            ));
            xmi.push_str(&format!(
                "      <sbvr:Verb>{}</sbvr:Verb>\n",
                Self::escape_xml(&fact.verb)
            ));
            xmi.push_str(&format!(
                "      <sbvr:Object>{}</sbvr:Object>\n",
                Self::escape_xml(&fact.object)
            ));
            if let Some(dest) = &fact.destination {
                xmi.push_str(&format!(
                    "      <sbvr:Destination>{}</sbvr:Destination>\n",
                    Self::escape_xml(dest)
                ));
            }
            xmi.push_str("    </sbvr:FactType>\n");
        }

        for rule in &self.rules {
            let rule_element = match rule.rule_type {
                RuleType::Obligation => "Obligation",
                RuleType::Prohibition => "Prohibition",
                RuleType::Permission => "Permission",
                RuleType::Derivation => "Derivation",
            };

            xmi.push_str(&format!(
                "    <sbvr:{} id=\"{}\" name=\"{}\">\n",
                rule_element,
                Self::escape_xml(&rule.id),
                Self::escape_xml(&rule.name)
            ));
            xmi.push_str(&format!(
                "      <sbvr:Expression>{}</sbvr:Expression>\n",
                Self::escape_xml(&rule.expression)
            ));
            xmi.push_str(&format!(
                "      <sbvr:Severity>{}</sbvr:Severity>\n",
                Self::escape_xml(&rule.severity)
            ));
            if let Some(p) = rule.priority {
                xmi.push_str(&format!("      <sbvr:Priority>{}</sbvr:Priority>\n", p));
            }
            xmi.push_str(&format!("    </sbvr:{}>\n", rule_element));
        }

        xmi.push_str("  </sbvr:Vocabulary>\n");
        xmi.push_str("</xmi:XMI>\n");

        Ok(xmi)
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Parse an SBVR XMI document and return a SbvrModel
    pub fn from_xmi(xmi: &str) -> Result<Self, SbvrError> {
        let doc = roxmltree::Document::parse(xmi)
            .map_err(|e| SbvrError::SerializationError(format!("Failed to parse XMI: {}", e)))?;

        let mut model = SbvrModel::new();

        // Find vocabulary node
        for node in doc.descendants() {
            if node.has_tag_name("GeneralConcept") {
                let id = node.attribute("id").unwrap_or_default().to_string();
                let name = node.attribute("name").unwrap_or_default().to_string();
                let mut definition = None;
                for child in node.children() {
                    if child.has_tag_name("Definition") {
                        definition = Some(child.text().unwrap_or_default().to_string());
                    }
                }
                model.vocabulary.push(SbvrTerm {
                    id,
                    name,
                    term_type: TermType::GeneralConcept,
                    definition,
                });
            }

            if node.has_tag_name("IndividualConcept") {
                let id = node.attribute("id").unwrap_or_default().to_string();
                let name = node.attribute("name").unwrap_or_default().to_string();
                let mut definition = None;
                for child in node.children() {
                    if child.has_tag_name("Definition") {
                        definition = Some(child.text().unwrap_or_default().to_string());
                    }
                }
                model.vocabulary.push(SbvrTerm {
                    id,
                    name,
                    term_type: TermType::IndividualConcept,
                    definition,
                });
            }

            if node.has_tag_name("VerbConcept") {
                let id = node.attribute("id").unwrap_or_default().to_string();
                let name = node.attribute("name").unwrap_or_default().to_string();
                let mut definition = None;
                for child in node.children() {
                    if child.has_tag_name("Definition") {
                        definition = Some(child.text().unwrap_or_default().to_string());
                    }
                }
                model.vocabulary.push(SbvrTerm {
                    id,
                    name,
                    term_type: TermType::VerbConcept,
                    definition,
                });
            }

            if node.has_tag_name("FactType") {
                let id = node.attribute("id").unwrap_or_default().to_string();
                let mut subject = String::new();
                let mut verb = String::new();
                let mut object = String::new();
                let mut destination = None;
                let mut schema_version = SbvrFactType::default_schema_version();

                for child in node.children() {
                    if child.has_tag_name("SchemaVersion") {
                        schema_version = child.text().unwrap_or_default().to_string();
                    }
                    if child.has_tag_name("Subject") {
                        subject = child.text().unwrap_or_default().to_string();
                    }
                    if child.has_tag_name("Verb") {
                        verb = child.text().unwrap_or_default().to_string();
                    }
                    if child.has_tag_name("Object") {
                        object = child.text().unwrap_or_default().to_string();
                    }
                    if child.has_tag_name("Destination") {
                        destination = Some(child.text().unwrap_or_default().to_string());
                    }
                }

                model.facts.push(SbvrFactType {
                    id,
                    subject,
                    verb,
                    object,
                    destination,
                    schema_version,
                });
            }

            // Parse basic rules (Obligation/Prohibition/Permission/Derivation)
            if node.has_tag_name("Obligation")
                || node.has_tag_name("Prohibition")
                || node.has_tag_name("Permission")
                || node.has_tag_name("Derivation")
            {
                let id = node.attribute("id").unwrap_or_default().to_string();
                let name = node.attribute("name").unwrap_or_default().to_string();
                let kind = if node.has_tag_name("Obligation") {
                    RuleType::Obligation
                } else if node.has_tag_name("Prohibition") {
                    RuleType::Prohibition
                } else if node.has_tag_name("Permission") {
                    RuleType::Permission
                } else {
                    RuleType::Derivation
                };
                let mut expression = String::new();
                let mut severity = String::from("Info");
                let mut parsed_priority: Option<u8> = None;
                for child in node.children() {
                    if child.has_tag_name("Expression") {
                        expression = child.text().unwrap_or_default().to_string();
                    }
                    if child.has_tag_name("Severity") {
                        severity = child.text().unwrap_or_default().to_string();
                    }
                    if child.has_tag_name("Priority") {
                        if let Some(text) = child.text() {
                            if let Ok(value) = text.trim().parse::<u8>() {
                                parsed_priority = Some(value);
                            }
                        }
                    }
                }
                let mut rule = SbvrBusinessRule {
                    id,
                    name,
                    rule_type: kind,
                    expression,
                    severity,
                    priority: parsed_priority,
                };
                if rule.priority.is_none() {
                    rule.priority = Some(match rule.rule_type {
                        RuleType::Obligation => 5,
                        RuleType::Prohibition => 5,
                        RuleType::Permission => 1,
                        RuleType::Derivation => 3,
                    });
                }
                model.rules.push(rule);
            }
        }

        Ok(model)
    }

    /// Convert parsed SbvrModel into a Graph
    pub fn to_graph(&self) -> Result<crate::graph::Graph, SbvrError> {
        use crate::graph::Graph;
        use crate::primitives::{Entity, Flow, Resource};
        use crate::units::unit_from_string;
        use rust_decimal::Decimal;

        let mut graph = Graph::new();

        // Create vocabulary terms first
        for term in &self.vocabulary {
            match term.term_type {
                TermType::GeneralConcept => {
                    let entity =
                        Entity::new_with_namespace(term.name.clone(), "default".to_string());
                    graph
                        .add_entity(entity)
                        .map_err(SbvrError::SerializationError)?;
                }
                TermType::IndividualConcept => {
                    let unit_symbol = term.definition.as_deref().and_then(|def| {
                        if let Some(open) = def.rfind('(') {
                            if let Some(close_offset) = def[open..].find(')') {
                                let close = open + close_offset;
                                let candidate = def[open + 1..close].trim();
                                if !candidate.is_empty() {
                                    return Some(candidate.to_string());
                                }
                            }
                        }
                        None
                    });
                    let unit_symbol = unit_symbol.unwrap_or_else(|| {
                        // SBVR definitions currently omit explicit unit metadata, so default to "units".
                        // Extend the SBVR model if richer unit information becomes available.
                        "units".to_string()
                    });
                    let res = Resource::new_with_namespace(
                        term.name.clone(),
                        unit_from_string(&unit_symbol),
                        "default".to_string(),
                    );
                    graph
                        .add_resource(res)
                        .map_err(SbvrError::SerializationError)?;
                }
                TermType::VerbConcept => {
                    // We don't represent verbs directly as primitives
                }
            }
        }

        // Now create flows
        for fact in &self.facts {
            // find subject (entity)
            let subject_name = self
                .vocabulary
                .iter()
                .find(|t| t.id == fact.subject)
                .map(|t| t.name.clone())
                .unwrap_or(fact.subject.clone());

            let object_name = self
                .vocabulary
                .iter()
                .find(|t| t.id == fact.object)
                .map(|t| t.name.clone())
                .unwrap_or(fact.object.clone());

            let destination_name = fact
                .destination
                .clone()
                .and_then(|d| {
                    self.vocabulary
                        .iter()
                        .find(|t| t.id == d)
                        .map(|t| t.name.clone())
                })
                .unwrap_or_default();

            let subject_id = graph.find_entity_by_name(&subject_name).ok_or_else(|| {
                SbvrError::UnsupportedConstruct(format!("Unknown subject entity: {}", subject_name))
            })?;

            let destination_id = graph
                .find_entity_by_name(&destination_name)
                .ok_or_else(|| {
                    SbvrError::UnsupportedConstruct(format!(
                        "Unknown destination entity: {}",
                        destination_name
                    ))
                })?;

            let resource_id = graph.find_resource_by_name(&object_name).ok_or_else(|| {
                SbvrError::UnsupportedConstruct(format!("Unknown resource: {}", object_name))
            })?;

            // SBVR FactType does not expose an explicit quantity, so default flows to 1.
            // This is intentional until the SBVR model is extended with quantity metadata.
            let quantity = Decimal::from(1);

            let flow = Flow::new(resource_id, subject_id, destination_id, quantity);
            graph
                .add_flow(flow)
                .map_err(SbvrError::SerializationError)?;
        }

        // Map SBVR Business Rules into Graph Policies
        for rule in &self.rules {
            // Parse expression using SEA DSL expression parser
            let expr = crate::parser::parse_expression_from_str(rule.expression.as_str()).map_err(
                |e| {
                    SbvrError::SerializationError(format!("Failed to parse rule expression: {}", e))
                },
            )?;

            let mut policy = crate::policy::Policy::new_with_namespace(
                rule.name.clone(),
                "default".to_string(),
                expr,
            );

            // Map rule type to modality/kind
            match rule.rule_type {
                RuleType::Obligation => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Obligation);
                    policy = policy.with_kind(crate::policy::PolicyKind::Constraint);
                }
                RuleType::Prohibition => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Prohibition);
                    policy = policy.with_kind(crate::policy::PolicyKind::Constraint);
                }
                RuleType::Permission => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Permission);
                    policy = policy.with_kind(crate::policy::PolicyKind::Constraint);
                }
                RuleType::Derivation => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Permission);
                    policy = policy.with_kind(crate::policy::PolicyKind::Derivation);
                }
            }

            // Priority mapping
            if let Some(p) = rule.priority {
                policy = policy.with_priority(p as i32);
            } else {
                // Default based on rule type
                let default_p = match rule.rule_type {
                    RuleType::Obligation => 5,
                    RuleType::Prohibition => 5,
                    RuleType::Permission => 1,
                    RuleType::Derivation => 3,
                };
                policy = policy.with_priority(default_p);
            }

            graph
                .add_policy(policy)
                .map_err(SbvrError::SerializationError)?;
        }

        Ok(graph)
    }
}

impl Default for SbvrModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn export_sbvr(&self) -> Result<String, SbvrError> {
        let model = SbvrModel::from_graph(self)?;
        model.to_xmi()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Entity, Flow, Resource};
    use rust_decimal::Decimal;

    #[test]
    fn test_sbvr_model_creation() {
        let model = SbvrModel::new();
        assert_eq!(model.vocabulary.len(), 0);
        assert_eq!(model.facts.len(), 0);
        assert_eq!(model.rules.len(), 0);
    }

    #[test]
    fn test_export_to_sbvr() {
        let mut graph = Graph::new();

        let entity1 = Entity::new_with_namespace("Supplier", "supply_chain");
        let entity2 = Entity::new_with_namespace("Manufacturer", "supply_chain");
        let resource = Resource::new_with_namespace(
            "Parts",
            crate::units::unit_from_string("kg"),
            "supply_chain",
        );

        let entity1_id = entity1.id().clone();
        let entity2_id = entity2.id().clone();
        let resource_id = resource.id().clone();

        graph.add_entity(entity1).unwrap();
        graph.add_entity(entity2).unwrap();
        graph.add_resource(resource).unwrap();

        #[allow(deprecated)]
        let flow = Flow::new(resource_id, entity1_id, entity2_id, Decimal::new(100, 0));
        graph.add_flow(flow).unwrap();

        let sbvr_xml = graph.export_sbvr().unwrap();

        assert!(sbvr_xml.contains("<sbvr:FactType"));
        assert!(sbvr_xml.contains("<sbvr:GeneralConcept"));
        assert!(sbvr_xml.contains("Supplier"));
        assert!(sbvr_xml.contains("Manufacturer"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(SbvrModel::escape_xml("A&B"), "A&amp;B");
        assert_eq!(SbvrModel::escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(SbvrModel::escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_sbvr_rule_to_policy() {
        let mut model = SbvrModel::new();

        // minimal vocabulary
        model.vocabulary.push(SbvrTerm {
            id: "e1".to_string(),
            name: "Warehouse".to_string(),
            term_type: TermType::GeneralConcept,
            definition: None,
        });
        model.vocabulary.push(SbvrTerm {
            id: "e2".to_string(),
            name: "Factory".to_string(),
            term_type: TermType::GeneralConcept,
            definition: None,
        });
        model.vocabulary.push(SbvrTerm {
            id: "r1".to_string(),
            name: "Cameras".to_string(),
            term_type: TermType::IndividualConcept,
            definition: None,
        });

        model.rules.push(SbvrBusinessRule {
            id: "rule1".to_string(),
            name: "MustHavePositiveQuantity".to_string(),
            rule_type: RuleType::Obligation,
            expression: "forall f in flows: (f.quantity > 0)".to_string(),
            severity: "Info".to_string(),
            priority: None,
        });

        let graph = model.to_graph().expect("SBVR to Graph conversion failed");
        assert_eq!(graph.policy_count(), 1);
        let policy = graph.all_policies().into_iter().next().unwrap();
        assert_eq!(policy.name, "MustHavePositiveQuantity");
        assert_eq!(policy.priority, 5); // default for Obligation
        assert_eq!(policy.modality, crate::policy::PolicyModality::Obligation);
    }
}
