use crate::graph::Graph;
use serde::{Serialize, Deserialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbvrBusinessRule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub expression: String,
    pub severity: String,
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

        for entity in graph.all_entities() {
            model.vocabulary.push(SbvrTerm {
                id: entity.id().to_string(),
                name: entity.name().to_string(),
                term_type: TermType::GeneralConcept,
                definition: Some(format!("Entity: {}", entity.name())),
            });
        }

        for resource in graph.all_resources() {
            model.vocabulary.push(SbvrTerm {
                id: resource.id().to_string(),
                name: resource.name().to_string(),
                term_type: TermType::IndividualConcept,
                definition: Some(format!("Resource: {} ({:?})", resource.name(), resource.unit())),
            });
        }

        model.vocabulary.push(SbvrTerm {
            id: "verb:transfers".to_string(),
            name: "transfers".to_string(),
            term_type: TermType::VerbConcept,
            definition: Some("Transfer of resource between entities".to_string()),
        });

        for flow in graph.all_flows() {
            model.facts.push(SbvrFactType {
                id: flow.id().to_string(),
                subject: flow.from_id().to_string(),
                verb: "transfers".to_string(),
                object: flow.to_id().to_string(),
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
                    xmi.push_str(&format!("    <sbvr:GeneralConcept id=\"{}\" name=\"{}\">\n",
                        Self::escape_xml(&term.id),
                        Self::escape_xml(&term.name)));
                    if let Some(def) = &term.definition {
                        xmi.push_str(&format!("      <sbvr:Definition>{}</sbvr:Definition>\n",
                            Self::escape_xml(def)));
                    }
                    xmi.push_str("    </sbvr:GeneralConcept>\n");
                }
                TermType::IndividualConcept => {
                    xmi.push_str(&format!("    <sbvr:IndividualConcept id=\"{}\" name=\"{}\">\n",
                        Self::escape_xml(&term.id),
                        Self::escape_xml(&term.name)));
                    if let Some(def) = &term.definition {
                        xmi.push_str(&format!("      <sbvr:Definition>{}</sbvr:Definition>\n",
                            Self::escape_xml(def)));
                    }
                    xmi.push_str("    </sbvr:IndividualConcept>\n");
                }
                TermType::VerbConcept => {
                    xmi.push_str(&format!("    <sbvr:VerbConcept id=\"{}\" name=\"{}\">\n",
                        Self::escape_xml(&term.id),
                        Self::escape_xml(&term.name)));
                    if let Some(def) = &term.definition {
                        xmi.push_str(&format!("      <sbvr:Definition>{}</sbvr:Definition>\n",
                            Self::escape_xml(def)));
                    }
                    xmi.push_str("    </sbvr:VerbConcept>\n");
                }
            }
        }

        for fact in &self.facts {
            xmi.push_str(&format!("    <sbvr:FactType id=\"{}\">\n", Self::escape_xml(&fact.id)));
            xmi.push_str(&format!("      <sbvr:Subject>{}</sbvr:Subject>\n", Self::escape_xml(&fact.subject)));
            xmi.push_str(&format!("      <sbvr:Verb>{}</sbvr:Verb>\n", Self::escape_xml(&fact.verb)));
            xmi.push_str(&format!("      <sbvr:Object>{}</sbvr:Object>\n", Self::escape_xml(&fact.object)));
            xmi.push_str("    </sbvr:FactType>\n");
        }

        for rule in &self.rules {
            let rule_element = match rule.rule_type {
                RuleType::Obligation => "Obligation",
                RuleType::Prohibition => "Prohibition",
                RuleType::Permission => "Permission",
                RuleType::Derivation => "Derivation",
            };

            xmi.push_str(&format!("    <sbvr:{} id=\"{}\" name=\"{}\">\n",
                rule_element,
                Self::escape_xml(&rule.id),
                Self::escape_xml(&rule.name)));
            xmi.push_str(&format!("      <sbvr:Expression>{}</sbvr:Expression>\n",
                Self::escape_xml(&rule.expression)));
            xmi.push_str(&format!("      <sbvr:Severity>{}</sbvr:Severity>\n",
                Self::escape_xml(&rule.severity)));
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
    use crate::primitives::{Entity, Resource, Flow};
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
        let resource = Resource::new_with_namespace("Parts", crate::units::unit_from_string("kg"), "supply_chain");

        let entity1_id = entity1.id().clone();
        let entity2_id = entity2.id().clone();
        let resource_id = resource.id().clone();

        graph.add_entity(entity1).unwrap();
        graph.add_entity(entity2).unwrap();
        graph.add_resource(resource).unwrap();

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
}
