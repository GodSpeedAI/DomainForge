use crate::graph::Graph;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub enum KgError {
    SerializationError(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for KgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KgError::SerializationError(msg) => write!(f, "Knowledge graph serialization error: {}", msg),
            KgError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
        }
    }
}

impl std::error::Error for KgError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclShape {
    pub target_class: String,
    pub properties: Vec<ShaclProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclProperty {
    pub path: String,
    pub datatype: Option<String>,
    pub min_count: Option<u32>,
    pub max_count: Option<u32>,
    pub min_exclusive: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub triples: Vec<Triple>,
    pub shapes: Vec<ShaclShape>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            triples: Vec::new(),
            shapes: Vec::new(),
        }
    }

    pub fn from_graph(graph: &Graph) -> Result<Self, KgError> {
        let mut kg = Self::new();

        for entity in graph.all_entities() {
            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: "rdf:type".to_string(),
                object: "sea:Entity".to_string(),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: "rdfs:label".to_string(),
                object: format!("\"{}\"", entity.name()),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: "sea:namespace".to_string(),
                object: format!("\"{}\"", entity.namespace()),
            });
        }

        for resource in graph.all_resources() {
            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(resource.name())),
                predicate: "rdf:type".to_string(),
                object: "sea:Resource".to_string(),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(resource.name())),
                predicate: "rdfs:label".to_string(),
                object: format!("\"{}\"", resource.name()),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(resource.name())),
                predicate: "sea:unit".to_string(),
                object: format!("\"{:?}\"", resource.unit()),
            });
        }

        for flow in graph.all_flows() {
            let flow_id = format!("sea:flow_{}", Self::uri_encode(&flow.id().to_string()));

            kg.triples.push(Triple {
                subject: flow_id.clone(),
                predicate: "rdf:type".to_string(),
                object: "sea:Flow".to_string(),
            });

            if let Some(from_entity) = graph.get_entity(flow.from_id()) {
                kg.triples.push(Triple {
                    subject: flow_id.clone(),
                    predicate: "sea:from".to_string(),
                    object: format!("sea:{}", Self::uri_encode(from_entity.name())),
                });
            }

            if let Some(to_entity) = graph.get_entity(flow.to_id()) {
                kg.triples.push(Triple {
                    subject: flow_id.clone(),
                    predicate: "sea:to".to_string(),
                    object: format!("sea:{}", Self::uri_encode(to_entity.name())),
                });
            }

            if let Some(resource) = graph.get_resource(flow.resource_id()) {
                kg.triples.push(Triple {
                    subject: flow_id.clone(),
                    predicate: "sea:hasResource".to_string(),
                    object: format!("sea:{}", Self::uri_encode(resource.name())),
                });
            }

            kg.triples.push(Triple {
                subject: flow_id.clone(),
                predicate: "sea:quantity".to_string(),
                object: format!("\"{}\"^^xsd:decimal", flow.quantity()),
            });
        }

        kg.shapes.push(ShaclShape {
            target_class: "sea:Flow".to_string(),
            properties: vec![
                ShaclProperty {
                    path: "sea:quantity".to_string(),
                    datatype: Some("xsd:decimal".to_string()),
                    min_count: None,
                    max_count: None,
                    min_exclusive: Some("0".to_string()),
                },
                ShaclProperty {
                    path: "sea:hasResource".to_string(),
                    datatype: None,
                    min_count: Some(1),
                    max_count: Some(1),
                    min_exclusive: None,
                },
                ShaclProperty {
                    path: "sea:from".to_string(),
                    datatype: None,
                    min_count: Some(1),
                    max_count: Some(1),
                    min_exclusive: None,
                },
                ShaclProperty {
                    path: "sea:to".to_string(),
                    datatype: None,
                    min_count: Some(1),
                    max_count: Some(1),
                    min_exclusive: None,
                },
            ],
        });

        kg.shapes.push(ShaclShape {
            target_class: "sea:Entity".to_string(),
            properties: vec![
                ShaclProperty {
                    path: "rdfs:label".to_string(),
                    datatype: Some("xsd:string".to_string()),
                    min_count: Some(1),
                    max_count: Some(1),
                    min_exclusive: None,
                },
            ],
        });

        Ok(kg)
    }

    pub fn to_turtle(&self) -> String {
        let mut turtle = String::new();

        turtle.push_str("@prefix sea: <http://domainforge.ai/sea#> .\n");
        turtle.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        turtle.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        turtle.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        turtle.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
        turtle.push_str("@prefix sh: <http://www.w3.org/ns/shacl#> .\n");
        turtle.push_str("\n");

        turtle.push_str("# Ontology\n");
        turtle.push_str("sea:Entity a owl:Class ;\n");
        turtle.push_str("    rdfs:label \"Entity\" ;\n");
        turtle.push_str("    rdfs:comment \"Business actor, location, or organizational unit\" .\n\n");

        turtle.push_str("sea:Resource a owl:Class ;\n");
        turtle.push_str("    rdfs:label \"Resource\" ;\n");
        turtle.push_str("    rdfs:comment \"Quantifiable subject of value\" .\n\n");

        turtle.push_str("sea:Flow a owl:Class ;\n");
        turtle.push_str("    rdfs:label \"Flow\" ;\n");
        turtle.push_str("    rdfs:comment \"Transfer of resource between entities\" .\n\n");

        turtle.push_str("sea:hasResource a owl:ObjectProperty ;\n");
        turtle.push_str("    rdfs:domain sea:Flow ;\n");
        turtle.push_str("    rdfs:range sea:Resource .\n\n");

        turtle.push_str("sea:from a owl:ObjectProperty ;\n");
        turtle.push_str("    rdfs:domain sea:Flow ;\n");
        turtle.push_str("    rdfs:range sea:Entity .\n\n");

        turtle.push_str("sea:to a owl:ObjectProperty ;\n");
        turtle.push_str("    rdfs:domain sea:Flow ;\n");
        turtle.push_str("    rdfs:range sea:Entity .\n\n");

        turtle.push_str("# Instances\n");
        for triple in &self.triples {
            turtle.push_str(&format!("{} {} {} .\n", triple.subject, triple.predicate, triple.object));
        }

        turtle.push_str("\n# SHACL Shapes\n");
        for shape in &self.shapes {
            turtle.push_str(&format!("sea:{}Shape a sh:NodeShape ;\n",
                shape.target_class.replace("sea:", "")));
            turtle.push_str(&format!("    sh:targetClass {} ;\n", shape.target_class));

            for (i, prop) in shape.properties.iter().enumerate() {
                turtle.push_str("    sh:property [\n");
                turtle.push_str(&format!("        sh:path {} ;\n", prop.path));

                if let Some(dt) = &prop.datatype {
                    turtle.push_str(&format!("        sh:datatype {} ;\n", dt));
                }
                if let Some(min) = prop.min_count {
                    turtle.push_str(&format!("        sh:minCount {} ;\n", min));
                }
                if let Some(max) = prop.max_count {
                    turtle.push_str(&format!("        sh:maxCount {} ;\n", max));
                }
                if let Some(min_ex) = &prop.min_exclusive {
                    turtle.push_str(&format!("        sh:minExclusive {} ;\n", min_ex));
                }

                if i < shape.properties.len() - 1 {
                    turtle.push_str("    ] ;\n");
                } else {
                    turtle.push_str("    ] .\n");
                }
            }
            turtle.push_str("\n");
        }

        turtle
    }

    pub fn to_rdf_xml(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<rdf:RDF\n");
        xml.push_str("    xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n");
        xml.push_str("    xmlns:rdfs=\"http://www.w3.org/2000/01/rdf-schema#\"\n");
        xml.push_str("    xmlns:owl=\"http://www.w3.org/2002/07/owl#\"\n");
        xml.push_str("    xmlns:xsd=\"http://www.w3.org/2001/XMLSchema#\"\n");
        xml.push_str("    xmlns:sea=\"http://domainforge.ai/sea#\">\n\n");

        for triple in &self.triples {
            let subject = Self::clean_uri(&triple.subject);
            let predicate = Self::clean_uri(&triple.predicate);
            let object = &triple.object;

            if object.starts_with('"') {
                let cleaned_object = object.trim_matches('"');
                xml.push_str(&format!("  <rdf:Description rdf:about=\"{}\">\n", subject));
                xml.push_str(&format!("    <{}>", predicate));
                xml.push_str(&Self::escape_xml(cleaned_object));
                xml.push_str(&format!("</{}>\n", predicate));
                xml.push_str("  </rdf:Description>\n\n");
            } else {
                let cleaned_object = Self::clean_uri(object);
                xml.push_str(&format!("  <rdf:Description rdf:about=\"{}\">\n", subject));
                xml.push_str(&format!("    <{} rdf:resource=\"{}\"/>\n", predicate, cleaned_object));
                xml.push_str("  </rdf:Description>\n\n");
            }
        }

        xml.push_str("</rdf:RDF>\n");
        xml
    }

    fn uri_encode(s: &str) -> String {
        s.replace(' ', "_")
            .replace(':', "_")
            .replace('/', "_")
            .replace('#', "_")
    }

    fn clean_uri(uri: &str) -> String {
        if uri.contains(':') {
            let parts: Vec<&str> = uri.split(':').collect();
            if parts.len() == 2 {
                return format!("http://domainforge.ai/{}#{}", parts[0], parts[1]);
            }
        }
        uri.to_string()
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn export_rdf(&self, format: &str) -> Result<String, KgError> {
        let kg = KnowledgeGraph::from_graph(self)?;
        match format {
            "turtle" => Ok(kg.to_turtle()),
            "rdf-xml" => Ok(kg.to_rdf_xml()),
            _ => Err(KgError::UnsupportedFormat(format.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Entity, Resource, Flow};
    use rust_decimal::Decimal;

    #[test]
    fn test_export_to_rdf_turtle() {
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

        let rdf_turtle = graph.export_rdf("turtle").unwrap();

        assert!(rdf_turtle.contains("sea:Entity"));
        assert!(rdf_turtle.contains("sea:hasResource"));
        assert!(rdf_turtle.contains("@prefix"));
    }

    #[test]
    fn test_export_to_rdf_xml() {
        let mut graph = Graph::new();

        let entity = Entity::new("TestEntity");
        graph.add_entity(entity).unwrap();

        let rdf_xml = graph.export_rdf("rdf-xml").unwrap();

        assert!(rdf_xml.contains("<?xml"));
        assert!(rdf_xml.contains("rdf:RDF"));
    }

    #[test]
    fn test_unsupported_format() {
        let graph = Graph::new();
        let result = graph.export_rdf("json-ld");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KgError::UnsupportedFormat(_)));
    }
}
