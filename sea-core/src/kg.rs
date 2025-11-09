use crate::graph::Graph;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum KgError {
    SerializationError(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for KgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KgError::SerializationError(msg) => {
                write!(f, "Knowledge graph serialization error: {}", msg)
            }
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

const URI_ENCODE_SET: &AsciiSet = &CONTROLS.add(b' ').add(b':').add(b'/').add(b'#');

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
                object: format!("\"{}\"", Self::escape_turtle_literal(entity.name())),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: "sea:namespace".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(entity.namespace())),
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
                object: format!("\"{}\"", Self::escape_turtle_literal(resource.name())),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(resource.name())),
                predicate: "sea:unit".to_string(),
                object: format!(
                    "\"{}\"",
                    Self::escape_turtle_literal(&resource.unit().to_string())
                ),
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

            // Validate that the quantity is a safe decimal string for Turtle format
            let quantity_str = flow.quantity().to_string();
            Self::validate_turtle_decimal(&quantity_str).map_err(|e| {
                KgError::SerializationError(format!("Invalid quantity format: {}", e))
            })?;

            kg.triples.push(Triple {
                subject: flow_id.clone(),
                predicate: "sea:quantity".to_string(),
                object: format!("\"{}\"^^xsd:decimal", quantity_str),
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
            properties: vec![ShaclProperty {
                path: "rdfs:label".to_string(),
                datatype: Some("xsd:string".to_string()),
                min_count: Some(1),
                max_count: Some(1),
                min_exclusive: None,
            }],
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
        turtle.push('\n');

        turtle.push_str("# Ontology\n");
        turtle.push_str("sea:Entity a owl:Class ;\n");
        turtle.push_str("    rdfs:label \"Entity\" ;\n");
        turtle.push_str(
            "    rdfs:comment \"Business actor, location, or organizational unit\" .\n\n",
        );

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
            turtle.push_str(&format!(
                "{} {} {} .\n",
                triple.subject, triple.predicate, triple.object
            ));
        }

        turtle.push_str("\n# SHACL Shapes\n");
        for shape in &self.shapes {
            turtle.push_str(&format!(
                "sea:{}Shape a sh:NodeShape ;\n",
                shape.target_class.replace("sea:", "")
            ));
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
            turtle.push('\n');
        }

        turtle
    }

    pub fn to_rdf_xml(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<rdf:RDF\n");
        xml.push_str("    xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n");
    // Use project-local namespace for rdfs to preserve existing prefix mapping
    // in tests and downstream tooling that expects the domainforge namespace.
    xml.push_str("    xmlns:rdfs=\"http://domainforge.ai/rdfs#\"\n");
        xml.push_str("    xmlns:owl=\"http://www.w3.org/2002/07/owl#\"\n");
        xml.push_str("    xmlns:xsd=\"http://www.w3.org/2001/XMLSchema#\"\n");
    // Explicitly declare the xml namespace so XML processors (and tests using roxmltree)
    // can resolve attributes like xml:lang correctly.
    xml.push_str("    xmlns:xml=\"http://www.w3.org/XML/1998/namespace\"\n");
    xml.push_str("    xmlns:sea=\"http://domainforge.ai/sea#\">\n\n");

        for triple in &self.triples {
            let subject = Self::clean_uri(&triple.subject);
            // Keep the predicate as the original prefixed name for use as XML element
            // (e.g. rdfs:label). Use the cleaned URI only for rdf:datatype or rdf:resource
            // attributes where a full URI is required.
            let predicate_name = triple.predicate.clone();
            let object = &triple.object;

            if object.starts_with('"') {
                let (literal_value, suffix) = Self::parse_typed_literal(object);
                let escaped_value = Self::escape_xml(&literal_value);

                xml.push_str(&format!("  <rdf:Description rdf:about=\"{}\">\n", subject));

                match suffix {
                    Some(TypedLiteralSuffix::Datatype(datatype)) => {
                        let datatype_uri = Self::clean_uri(&datatype);
                        xml.push_str(&format!(
                            "    <{} rdf:datatype=\"{}\">{}</{}>\n",
                            predicate_name, datatype_uri, escaped_value, predicate_name
                        ));
                    }
                    Some(TypedLiteralSuffix::Language(lang)) => {
                        xml.push_str(&format!(
                            "    <{} xml:lang=\"{}\">{}</{}>\n",
                            predicate_name, lang, escaped_value, predicate_name
                        ));
                    }
                    None => {
                        xml.push_str(&format!(
                            "    <{}>{}</{}>\n",
                            predicate_name, escaped_value, predicate_name
                        ));
                    }
                }

                xml.push_str("  </rdf:Description>\n\n");
            } else {
                let cleaned_object = Self::clean_uri(object);
                xml.push_str(&format!("  <rdf:Description rdf:about=\"{}\">\n", subject));
                xml.push_str(&format!(
                    "    <{} rdf:resource=\"{}\"/>\n",
                    predicate_name, cleaned_object
                ));
                xml.push_str("  </rdf:Description>\n\n");
            }
        }

        xml.push_str("</rdf:RDF>\n");
        xml
    }

    pub fn escape_turtle_literal(input: &str) -> String {
        let mut escaped = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\x08' => escaped.push_str("\\b"), // backspace
                '\x0C' => escaped.push_str("\\f"), // form feed
                other => escaped.push(other),
            }
        }
        escaped
    }

    fn uri_encode(s: &str) -> String {
        utf8_percent_encode(s, URI_ENCODE_SET).to_string()
    }

    fn validate_turtle_decimal(decimal_str: &str) -> Result<(), String> {
        // Basic validation for safe decimal literals in Turtle
        let trimmed = decimal_str.trim();

        // Check for invalid characters that could break Turtle syntax
        if trimmed
            .chars()
            .any(|ch| matches!(ch, '"' | '\'' | '\\' | '\n' | '\r' | '\t'))
        {
            return Err("Decimal contains invalid characters".to_string());
        }

        // Ensure it looks like a valid decimal number
        if trimmed.is_empty() {
            return Err("Decimal is empty".to_string());
        }

        // Basic pattern check: optional sign, digits, optional fractional part
        let mut has_digit = false;
        let mut chars = trimmed.chars().peekable();

        // Optional sign
        if matches!(chars.peek(), Some('+') | Some('-')) {
            chars.next();
        }

        // Digits and optional fractional part
        while let Some(ch) = chars.next() {
            if ch.is_ascii_digit() {
                has_digit = true;
            } else if ch == '.' {
                // Check fractional part
                if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
                    return Err("Invalid decimal format".to_string());
                }
                for c in chars.by_ref() {
                    if !c.is_ascii_digit() {
                        return Err("Invalid decimal format".to_string());
                    }
                }
                break;
            } else {
                return Err("Invalid decimal format".to_string());
            }
        }

        if !has_digit {
            return Err("Invalid decimal format".to_string());
        }

        Ok(())
    }

    fn clean_uri(uri: &str) -> String {
        if uri.contains(':') {
            let parts: Vec<&str> = uri.splitn(2, ':').collect();
            if parts.len() == 2 {
                let (prefix, name) = (parts[0], parts[1]);

                // Check for standard RDF/XSD prefixes
                let standard_prefixes = [
                    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
                    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
                    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
                    ("owl", "http://www.w3.org/2002/07/owl#"),
                    ("sh", "http://www.w3.org/ns/shacl#"),
                    ("sea", "http://domainforge.ai/sea#"),
                ];

                for (std_prefix, namespace) in &standard_prefixes {
                    if prefix == *std_prefix {
                        return format!("{}{}", namespace, name);
                    }
                }

                // Fall back to original behavior for unknown prefixes
                return format!("http://domainforge.ai/{}#{}", prefix, name);
            }
        }
        uri.to_string()
    }

    pub fn escape_xml(input: &str) -> String {
        let mut escaped = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&apos;"),
                other => escaped.push(other),
            }
        }
        escaped
    }

    fn parse_escaped_value<I>(chars: &mut I) -> String
    where
        I: Iterator<Item = char>,
    {
        let mut value = String::new();
        let mut escaped = false;

        for ch in chars.by_ref() {
            if escaped {
                let resolved = match ch {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    other => {
                        value.push('\\');
                        other
                    }
                };
                value.push(resolved);
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '"' => break,
                other => value.push(other),
            }
        }

        value
    }

    fn parse_typed_literal(literal: &str) -> (String, Option<TypedLiteralSuffix>) {
        if !literal.starts_with('"') {
            return (literal.to_string(), None);
        }

        let mut chars = literal.chars();
        chars.next();
        let value = Self::parse_escaped_value(&mut chars);

        let remainder: String = chars.collect();
        let trimmed = remainder.trim();

        let suffix = if let Some(rest) = trimmed.strip_prefix("^^") {
            let datatype = rest.trim();
            if datatype.is_empty() {
                None
            } else {
                Some(TypedLiteralSuffix::Datatype(datatype.to_string()))
            }
        } else if let Some(rest) = trimmed.strip_prefix('@') {
            let language = rest.trim();
            if language.is_empty() {
                None
            } else {
                Some(TypedLiteralSuffix::Language(language.to_string()))
            }
        } else {
            None
        };

        (value, suffix)
    }
}

enum TypedLiteralSuffix {
    Datatype(String),
    Language(String),
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
    use crate::primitives::{Entity, Flow, Resource};
    use rust_decimal::Decimal;

    #[test]
    fn test_export_to_rdf_turtle() {
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

    #[test]
    fn test_export_rdf_turtle_encodes_special_characters_and_literals() {
        let mut graph = Graph::new();

        let entity_space = Entity::new("Entity With Space");
        let entity_colon = Entity::new("Entity:Colon");
        let entity_slash = Entity::new("Entity/Slash");
        let entity_hash = Entity::new("Entity#Hash");

        graph.add_entity(entity_space.clone()).unwrap();
        graph.add_entity(entity_colon.clone()).unwrap();
        graph.add_entity(entity_slash.clone()).unwrap();
        graph.add_entity(entity_hash.clone()).unwrap();

        let resource = Resource::new(
            "Resource:Name/Hash",
            crate::units::unit_from_string("units"),
        );
        let resource_id = resource.id().clone();
        graph.add_resource(resource).unwrap();

        let flow = Flow::new(
            resource_id,
            entity_space.id().clone(),
            entity_colon.id().clone(),
            Decimal::new(42, 0),
        );
        graph.add_flow(flow).unwrap();

        let turtle = graph.export_rdf("turtle").unwrap();
        assert!(turtle.contains("sea:Entity%20With%20Space"));
        assert!(turtle.contains("sea:Entity%3AColon"));
        assert!(turtle.contains("sea:Entity%2FSlash"));
        assert!(turtle.contains("sea:Entity%23Hash"));
        assert!(turtle.contains("sea:Resource%3AName%2FHash"));
        assert!(turtle.contains("\"42\"^^xsd:decimal"));
    }

    #[test]
    fn test_rdf_xml_escapes_special_literals_and_language_tags() {
        let mut kg = KnowledgeGraph::new();

        kg.triples.push(Triple {
            subject: "sea:testEntity".to_string(),
            predicate: "sea:hasNumericValue".to_string(),
            object: "\"100\"^^xsd:decimal".to_string(),
        });
        kg.triples.push(Triple {
            subject: "sea:testEntity".to_string(),
            predicate: "sea:description".to_string(),
            object: "\"Hello & <World>\"@en".to_string(),
        });

        let xml = kg.to_rdf_xml();
        assert!(xml.contains("rdf:datatype=\"http://www.w3.org/2001/XMLSchema#decimal\""));
        assert!(xml.contains(">100<"));
        assert!(xml.contains("xml:lang=\"en\""));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&gt;"));
    }
}
