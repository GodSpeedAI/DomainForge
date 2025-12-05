use crate::graph::Graph;
use crate::parser::ast::TargetFormat;
use crate::projection::{find_projection_override, ProjectionRegistry};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

const URI_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b':')
    .add(b'/')
    .add(b'#')
    .add(b'?')
    .add(b'&')
    .add(b'=')
    .add(b'+')
    .add(b'$')
    .add(b',')
    .add(b'@')
    .add(b';');

fn tokenize_triple_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();
    let mut in_literal = false;
    let mut escape = false;

    for c in line.chars() {
        if in_literal {
            buffer.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_literal = false;
            }
            continue;
        }

        match c {
            '"' => {
                in_literal = true;
                buffer.push(c);
            }
            c if c.is_whitespace() => {
                if !buffer.is_empty() {
                    tokens.push(buffer.clone());
                    buffer.clear();
                }
            }
            _ => {
                buffer.push(c);
            }
        }
    }

    if !buffer.is_empty() {
        tokens.push(buffer);
    }

    tokens
}

fn extract_local_name(token: &str) -> String {
    let trimmed = token.trim();
    let stripped = trimmed.trim_matches(|c| c == '<' || c == '>');
    stripped
        .rsplit(|c| ['#', ':'].contains(&c))
        .next()
        .unwrap_or(stripped)
        .to_string()
}

fn extract_literal_value(token: &str) -> String {
    let trimmed = token.trim();
    if let Some(stripped) = trimmed.strip_prefix('"') {
        if let Some(end_quote) = stripped.find('"') {
            return stripped[..end_quote].to_string();
        }
        return stripped.trim_end_matches('"').to_string();
    }

    if let Some(idx) = trimmed.find("^^") {
        return trimmed[..idx].trim().to_string();
    }
    trimmed.to_string()
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

        let registry = ProjectionRegistry::new(graph);
        let projections = registry.find_projections_for_target(&TargetFormat::Kg);
        let projection = projections.first().copied();

        for entity in graph.all_entities() {
            let mut rdf_class = "sea:Entity".to_string();
            let mut prop_map = std::collections::HashMap::new();

            if let Some(proj) = projection {
                if let Some(rule) = find_projection_override(proj, "Entity", entity.name()) {
                    if let Some(cls) = rule.fields.get("rdf_class").and_then(|v| v.as_str()) {
                        if Self::is_valid_rdf_term(cls) {
                            rdf_class = cls.to_string();
                        } else {
                            eprintln!("Warning: Invalid RDF term for rdf_class, skipping: {}", cls);
                        }
                    }
                    if let Some(props) = rule.fields.get("properties").and_then(|v| v.as_object()) {
                        for (k, v) in props {
                            if let Some(v_str) = v.as_str() {
                                if Self::is_valid_rdf_term(v_str) {
                                    prop_map.insert(k.clone(), v_str.to_string());
                                } else {
                                    eprintln!(
                                        "Warning: Invalid RDF term for property '{}', skipping: {}",
                                        k, v_str
                                    );
                                }
                            }
                        }
                    }
                }
            }

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: "rdf:type".to_string(),
                object: rdf_class,
            });

            let label_pred = prop_map
                .get("name")
                .cloned()
                .unwrap_or_else(|| "rdfs:label".to_string());
            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: label_pred,
                object: format!("\"{}\"", Self::escape_turtle_literal(entity.name())),
            });

            let ns_pred = prop_map
                .get("namespace")
                .cloned()
                .unwrap_or_else(|| "sea:namespace".to_string());
            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(entity.name())),
                predicate: ns_pred,
                object: format!("\"{}\"", Self::escape_turtle_literal(entity.namespace())),
            });
        }

        for role in graph.all_roles() {
            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(role.name())),
                predicate: "rdf:type".to_string(),
                object: "sea:Role".to_string(),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(role.name())),
                predicate: "rdfs:label".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(role.name())),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", Self::uri_encode(role.name())),
                predicate: "sea:namespace".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(role.namespace())),
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

        for pattern in graph.all_patterns() {
            let subject = format!("sea:pattern_{}", Self::uri_encode(pattern.name()));

            kg.triples.push(Triple {
                subject: subject.clone(),
                predicate: "rdf:type".to_string(),
                object: "sea:Pattern".to_string(),
            });

            kg.triples.push(Triple {
                subject: subject.clone(),
                predicate: "rdfs:label".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(pattern.name())),
            });

            kg.triples.push(Triple {
                subject: subject.clone(),
                predicate: "sea:namespace".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(pattern.namespace())),
            });

            kg.triples.push(Triple {
                subject,
                predicate: "sea:regex".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(pattern.regex())),
            });
        }

        for relation in graph.all_relations() {
            let relation_subject = format!("sea:{}", Self::uri_encode(relation.name()));

            kg.triples.push(Triple {
                subject: relation_subject.clone(),
                predicate: "rdf:type".to_string(),
                object: "sea:Relation".to_string(),
            });

            kg.triples.push(Triple {
                subject: relation_subject.clone(),
                predicate: "rdfs:label".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(relation.name())),
            });

            if let Some(subject_role) = graph.get_role(relation.subject_role()) {
                kg.triples.push(Triple {
                    subject: relation_subject.clone(),
                    predicate: "sea:subjectRole".to_string(),
                    object: format!("sea:{}", Self::uri_encode(subject_role.name())),
                });
            }

            if let Some(object_role) = graph.get_role(relation.object_role()) {
                kg.triples.push(Triple {
                    subject: relation_subject.clone(),
                    predicate: "sea:objectRole".to_string(),
                    object: format!("sea:{}", Self::uri_encode(object_role.name())),
                });
            }

            kg.triples.push(Triple {
                subject: relation_subject.clone(),
                predicate: "sea:predicate".to_string(),
                object: format!("\"{}\"", Self::escape_turtle_literal(relation.predicate())),
            });

            if let Some(flow_id) = relation.via_flow() {
                if let Some(resource) = graph.get_resource(flow_id) {
                    kg.triples.push(Triple {
                        subject: relation_subject.clone(),
                        predicate: "sea:via".to_string(),
                        object: format!("sea:{}", Self::uri_encode(resource.name())),
                    });
                } else {
                    kg.triples.push(Triple {
                        subject: relation_subject.clone(),
                        predicate: "sea:via".to_string(),
                        object: format!("\"{}\"", flow_id),
                    });
                }
            }
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

    /// Parse a simple Turtle snippet into a KnowledgeGraph. This is a best-effort parser
    /// expecting the exact triple format generated by `to_turtle()` in this crate.
    #[allow(clippy::while_let_on_iterator)]
    pub fn from_turtle(turtle: &str) -> Result<Self, KgError> {
        let mut kg = Self::new();
        for line in turtle.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('@') || trimmed.starts_with('#') {
                continue;
            }
            let triple_line = if let Some(stripped) = trimmed.strip_suffix('.') {
                stripped.trim_end()
            } else {
                trimmed
            };
            let tokens = tokenize_triple_line(triple_line);
            if tokens.len() != 3 {
                continue;
            }
            let subject = &tokens[0];
            let predicate = &tokens[1];
            let object = &tokens[2];
            let norm_s = Self::shorten_token(subject);
            let norm_p = Self::shorten_token(predicate);
            let norm_o = Self::shorten_token(object);
            kg.triples.push(Triple {
                subject: norm_s,
                predicate: norm_p,
                object: norm_o,
            });
        }
        // parse shapes: look for NodeShape blocks (start with 'sea:SomethingShape a sh:NodeShape')
        // We scan lines to find blocks terminating with '.' and containing 'sh:property' entries
        let mut lines_iter = turtle.lines();
        while let Some(line) = lines_iter.next() {
            let l = line.trim();
            if l.contains("a sh:NodeShape") {
                // Collect shape block until '.' terminator
                let mut block = l.to_string();
                if !l.ends_with('.') {
                    while let Some(next_line) = lines_iter.next() {
                        block.push(' ');
                        block.push_str(next_line.trim());
                        if next_line.trim().ends_with('.') {
                            break;
                        }
                    }
                }

                // Normalize full URIs into prefixes for easier parsing
                let normalized_block = block
                    .replace("http://www.w3.org/ns/shacl#", "sh:")
                    .replace("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "rdf:")
                    .replace("http://www.w3.org/2000/01/rdf-schema#", "rdfs:")
                    .replace("http://www.w3.org/2001/XMLSchema#", "xsd:")
                    .replace("http://domainforge.ai/sea#", "sea:");

                // Extract target class
                let target_class = if let Some(pos) = normalized_block.find("sh:targetClass") {
                    let rest = &normalized_block[pos + "sh:targetClass".len()..];
                    let tok = rest
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .to_string();
                    tok
                } else {
                    continue; // ignore shapes without targetClass
                };

                let mut shape = ShaclShape {
                    target_class,
                    properties: Vec::new(),
                };

                // Find property blocks: 'sh:property [ ... ]' occurrences
                let mut start_idx = 0;
                while let Some(idx) = normalized_block[start_idx..].find("sh:property") {
                    let local_idx = start_idx + idx;
                    // Find the opening '[' and closing ']' for the property
                    if let Some(open_br) = normalized_block[local_idx..].find('[') {
                        let open_idx = local_idx + open_br + 1;
                        if let Some(close_br) = normalized_block[open_idx..].find(']') {
                            let close_idx = open_idx + close_br;
                            let prop_block = &normalized_block[open_idx..close_idx];
                            // Parse property attributes
                            let mut path = String::new();
                            let mut datatype: Option<String> = None;
                            let mut min_count: Option<u32> = None;
                            let mut max_count: Option<u32> = None;
                            let mut min_exclusive: Option<String> = None;

                            for tok in prop_block.split(';') {
                                let tok = tok.trim();
                                if tok.is_empty() {
                                    continue;
                                }
                                if let Some(s) = tok.strip_prefix("sh:path") {
                                    path = s.trim().to_string();
                                } else if let Some(s) = tok.strip_prefix("sh:datatype") {
                                    datatype = Some(s.trim().to_string());
                                } else if let Some(s) = tok.strip_prefix("sh:minCount") {
                                    let val = s.trim();
                                    if let Ok(n) = val.parse::<u32>() {
                                        min_count = Some(n);
                                    }
                                } else if let Some(s) = tok.strip_prefix("sh:maxCount") {
                                    let val = s.trim();
                                    if let Ok(n) = val.parse::<u32>() {
                                        max_count = Some(n);
                                    }
                                } else if let Some(s) = tok.strip_prefix("sh:minExclusive") {
                                    let val = s.trim();
                                    min_exclusive = Some(val.to_string());
                                }
                            }

                            if !path.is_empty() {
                                shape.properties.push(ShaclProperty {
                                    path,
                                    datatype,
                                    min_count,
                                    max_count,
                                    min_exclusive,
                                });
                            }
                            start_idx = close_idx + 1;
                            continue;
                        }
                    }
                    start_idx = local_idx + 1;
                }

                if !shape.properties.is_empty() {
                    kg.shapes.push(shape);
                }
            }
        }
        Ok(kg)
    }

    /// Convert the knowledge graph back into a Graph by interpreting triples exported by `to_turtle`.
    pub fn to_graph(&self) -> Result<crate::graph::Graph, KgError> {
        use crate::graph::Graph;
        use crate::primitives::{Entity, Flow, Resource};
        use crate::units::unit_from_string;
        use rust_decimal::Decimal;

        let mut graph = Graph::new();

        // First, collect entities and resources
        for t in &self.triples {
            if t.predicate == "rdf:type" && t.object == "sea:Entity" {
                // subject is like sea:Name — extract after colon
                let name = t
                    .subject
                    .split(':')
                    .nth(1)
                    .unwrap_or(&t.subject)
                    .to_string();
                let entity = Entity::new_with_namespace(name.clone(), "default".to_string());
                graph
                    .add_entity(entity)
                    .map_err(|e| KgError::SerializationError(e.to_string()))?;
            }
            if t.predicate == "rdf:type" && t.object == "sea:Resource" {
                let name = t
                    .subject
                    .split(':')
                    .nth(1)
                    .unwrap_or(&t.subject)
                    .to_string();
                let resource = Resource::new_with_namespace(
                    name.clone(),
                    unit_from_string("units"),
                    "default".to_string(),
                );
                graph
                    .add_resource(resource)
                    .map_err(|e| KgError::SerializationError(e.to_string()))?;
            }
        }

        // Now flows: find subjects typed as sea:Flow
        for t in &self.triples {
            if t.predicate == "rdf:type" && t.object == "sea:Flow" {
                let flow_subject = t.subject.clone();
                // we expect sea:flow_uuid etc. collect properties for this subject
                let mut from: Option<String> = None;
                let mut to: Option<String> = None;
                let mut resource_name: Option<String> = None;
                let mut quantity: Option<Decimal> = None;

                for p in &self.triples {
                    if p.subject != flow_subject {
                        continue;
                    }
                    match p.predicate.as_str() {
                        "sea:from" => {
                            from = Some(extract_local_name(&p.object));
                        }
                        "sea:to" => {
                            to = Some(extract_local_name(&p.object));
                        }
                        "sea:hasResource" => {
                            resource_name = Some(extract_local_name(&p.object));
                        }
                        "sea:quantity" => {
                            let lexical = extract_literal_value(&p.object);
                            let parsed = Decimal::from_str(&lexical).map_err(|e| {
                                KgError::SerializationError(format!(
                                    "Invalid quantity literal '{}': {}",
                                    p.object, e
                                ))
                            })?;
                            quantity = Some(parsed);
                        }
                        _ => {}
                    }
                }

                if let (Some(from_name), Some(to_name), Some(resource_name), Some(quantity_val)) =
                    (from, to, resource_name, quantity)
                {
                    let from_id = graph.find_entity_by_name(&from_name).ok_or_else(|| {
                        KgError::SerializationError(format!("Unknown entity: {}", from_name))
                    })?;
                    let to_id = graph.find_entity_by_name(&to_name).ok_or_else(|| {
                        KgError::SerializationError(format!("Unknown entity: {}", to_name))
                    })?;
                    let res_id = graph.find_resource_by_name(&resource_name).ok_or_else(|| {
                        KgError::SerializationError(format!("Unknown resource: {}", resource_name))
                    })?;

                    let flow = Flow::new(res_id, from_id, to_id, quantity_val);
                    graph
                        .add_flow(flow)
                        .map_err(|e| KgError::SerializationError(e.to_string()))?;
                }
            }
        }

        Ok(graph)
    }

    pub fn validate_shacl(&self) -> Result<Vec<crate::policy::Violation>, KgError> {
        use crate::policy::{Severity, Violation};

        // If no shapes are defined, nothing to validate
        if self.shapes.is_empty() {
            return Ok(Vec::new());
        }

        let mut violations: Vec<Violation> = Vec::new();

        // Helper: parse a Turtle literal (optionally typed) into its lexical form
        // and optional datatype suffix (e.g. "\"0\"^^xsd:decimal" -> ("0", Some("xsd:decimal"))).
        fn parse_literal_and_datatype(obj: &str) -> (String, Option<String>) {
            let s = obj.trim();
            if !s.starts_with('"') {
                return (s.to_string(), None);
            }

            // Find the closing quote for the lexical form, respecting simple escapes.
            let bytes = s.as_bytes();
            let mut end_quote = None;
            let mut i = 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    // Skip escaped character
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    end_quote = Some(i);
                    break;
                }
                i += 1;
            }

            if let Some(end) = end_quote {
                let lex = &s[1..end];
                let rest = s[end + 1..].trim();
                let dtype = rest.strip_prefix("^^").map(|s| s.trim().to_string());
                (lex.to_string(), dtype)
            } else {
                // No matching closing quote; fall back to naive trimming
                (s.trim_matches('"').to_string(), None)
            }
        }

        // For each shape, find all subjects in triples typed as the target class
        for shape in &self.shapes {
            match shape.target_class.as_str() {
                "sea:Flow" | "sea:Entity" | "sea:Resource" => {}
                other => {
                    return Err(KgError::SerializationError(format!(
                        "Unsupported SHACL target class: {}",
                        other
                    )))
                }
            }

            // collect all candidate subjects by rdf:type triples
            let mut subjects: Vec<String> = Vec::new();
            for t in &self.triples {
                if t.predicate == "rdf:type" && t.object == shape.target_class {
                    subjects.push(t.subject.clone());
                }
            }

            for subject in subjects {
                for prop in &shape.properties {
                    // check min_count / max_count constraints
                    let count = self
                        .triples
                        .iter()
                        .filter(|tr| tr.subject == subject && tr.predicate == prop.path)
                        .count() as u32;

                    if let Some(min) = prop.min_count {
                        if count < min {
                            let msg = format!(
                                "SHACL violation: subject {} missing required property {} (min_count={} found={})",
                                subject, prop.path, min, count
                            );
                            violations.push(Violation::new(format!("SHACL:{}", shape.target_class), msg, Severity::Error).with_context(serde_json::json!({"subject": subject, "predicate": prop.path, "expected_min": min, "found": count})));
                        }
                    }

                    if let Some(max) = prop.max_count {
                        if count > max {
                            let msg = format!(
                                "SHACL violation: subject {} has {} occurrences of {} (max_count={} found={})",
                                subject, count, prop.path, max, count
                            );
                            violations.push(Violation::new(format!("SHACL:{}", shape.target_class), msg, Severity::Error).with_context(serde_json::json!({"subject": subject, "predicate": prop.path, "expected_max": max, "found": count})));
                        }
                    }

                    // datatype checks (only handle basic types like xsd:decimal and xsd:string)
                    if let Some(dt) = &prop.datatype {
                        for tr in self
                            .triples
                            .iter()
                            .filter(|tr| tr.subject == subject && tr.predicate == prop.path)
                        {
                            let obj = tr.object.trim();
                            let (_lex, dtype_opt) = parse_literal_and_datatype(obj);
                            // look for typed literal like "123"^^xsd:decimal
                            if let Some(dtype) = dtype_opt {
                                if &dtype != dt {
                                    let msg = format!(
                                        "SHACL violation: subject {} property {} expected datatype {} but found {}",
                                        subject, prop.path, dt, dtype
                                    );
                                    violations.push(Violation::new(format!("SHACL:{}", shape.target_class), msg, Severity::Error).with_context(serde_json::json!({"subject": subject, "predicate": prop.path, "expected_type": dt, "found_type": dtype})));
                                }
                            } else if dt != "xsd:string" {
                                // untyped literal but expected typed -> violation
                                let msg = format!(
                                    "SHACL violation: subject {} property {} expected datatype {} but found untyped literal {}",
                                    subject, prop.path, dt, obj
                                );
                                violations.push(Violation::new(format!("SHACL:{}", shape.target_class), msg, Severity::Error).with_context(serde_json::json!({"subject": subject, "predicate": prop.path, "expected_type": dt, "found": obj})));
                            }
                        }
                    }

                    // minExclusive check (e.g. > 0) — interpreted for decimal numbers
                    if let Some(min_ex) = &prop.min_exclusive {
                        if prop.datatype.as_deref() == Some("xsd:decimal") {
                            let threshold =
                                rust_decimal::Decimal::from_str(min_ex).map_err(|e| {
                                    KgError::SerializationError(format!(
                                        "Invalid minExclusive threshold '{}': {}",
                                        min_ex, e
                                    ))
                                })?;
                            for tr in self
                                .triples
                                .iter()
                                .filter(|tr| tr.subject == subject && tr.predicate == prop.path)
                            {
                                let obj = tr.object.trim();
                                let lex = extract_literal_value(obj);
                                if let Ok(val) = rust_decimal::Decimal::from_str(&lex) {
                                    if val <= threshold {
                                        let msg = format!(
                                            "SHACL violation: subject {} property {} must be > {} but found {}",
                                            subject, prop.path, threshold, val
                                        );
                                        violations.push(Violation::new(format!("SHACL:{}", shape.target_class), msg, Severity::Error).with_context(serde_json::json!({"subject": subject, "predicate": prop.path, "threshold": threshold.to_string(), "found": val.to_string()})));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    pub fn to_rdf_xml(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<rdf:RDF\n");
        xml.push_str("    xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n");
        // Use project-local namespace for rdfs to preserve existing prefix mapping
        // in tests and downstream tooling that expects the domainforge namespace.
        xml.push_str("    xmlns:rdfs=\"http://www.w3.org/2000/01/rdf-schema#\"\n");
        xml.push_str("    xmlns:owl=\"http://www.w3.org/2002/07/owl#\"\n");
        xml.push_str("    xmlns:xsd=\"http://www.w3.org/2001/XMLSchema#\"\n");
        // Explicitly declare the xml namespace so XML processors (and tests using roxmltree)
        // can resolve attributes like xml:lang correctly.
        xml.push_str("    xmlns:xml=\"http://www.w3.org/XML/1998/namespace\"\n");
        xml.push_str("    xmlns:sea=\"http://domainforge.ai/sea#\"\n");
        xml.push_str("    xmlns:sh=\"http://www.w3.org/ns/shacl#\">\n\n");

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
        xml.push('\n');
        // Emit SHACL shapes (as RDF/XML)
        for shape in &self.shapes {
            xml.push_str(&Self::write_shacl_shapes_xml(shape));
        }

        xml.push_str("</rdf:RDF>\n");
        xml
    }

    fn write_shacl_shapes_xml(shape: &ShaclShape) -> String {
        let mut xml = String::new();
        let shape_name = shape.target_class.replace("sea:", "") + "Shape";
        xml.push_str(&format!(
            "  <sh:NodeShape rdf:about=\"http://domainforge.ai/sea#{}\">\n",
            shape_name
        ));
        xml.push_str(&format!(
            "    <sh:targetClass rdf:resource=\"http://domainforge.ai/sea#{}\"/>\n",
            shape.target_class.replace("sea:", "")
        ));
        for prop in &shape.properties {
            xml.push_str("    <sh:property>\n");
            xml.push_str("      <rdf:Description>\n");
            // Resolve the full URI for the sh:path based on the prefixed name.
            let (ns, local) = if let Some(rest) = prop.path.strip_prefix("sea:") {
                ("http://domainforge.ai/sea#", rest)
            } else if let Some(rest) = prop.path.strip_prefix("rdfs:") {
                ("http://www.w3.org/2000/01/rdf-schema#", rest)
            } else {
                ("http://domainforge.ai/sea#", prop.path.as_str())
            };
            xml.push_str(&format!(
                "        <sh:path rdf:resource=\"{}{}\"/>\n",
                ns, local
            ));
            if let Some(dt) = &prop.datatype {
                let dt_uri = if dt.starts_with("xsd:") {
                    dt.replace("xsd:", "http://www.w3.org/2001/XMLSchema#")
                } else {
                    dt.clone()
                };
                xml.push_str(&format!(
                    "        <sh:datatype rdf:resource=\"{}\"/>\n",
                    dt_uri
                ));
            }
            if let Some(min) = prop.min_count {
                xml.push_str(&format!("        <sh:minCount>{}</sh:minCount>\n", min));
            }
            if let Some(max) = prop.max_count {
                xml.push_str(&format!("        <sh:maxCount>{}</sh:maxCount>\n", max));
            }
            if let Some(min_ex) = &prop.min_exclusive {
                xml.push_str(&format!("        <sh:minExclusive rdf:datatype=\"http://www.w3.org/2001/XMLSchema#decimal\">{}</sh:minExclusive>\n", min_ex));
            }
            xml.push_str("      </rdf:Description>\n");
            xml.push_str("    </sh:property>\n");
        }
        xml.push_str("  </sh:NodeShape>\n\n");
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

    fn shorten_token(token: &str) -> String {
        let t = token.trim();
        // remove enclosing angle brackets
        let value = if t.starts_with('<') && t.ends_with('>') {
            &t[1..t.len() - 1]
        } else {
            t
        };

        // If contains typed literal with full datatype like "123"^^<http://www.w3.org/2001/XMLSchema#decimal>
        if value.contains("^^<http://www.w3.org/2001/XMLSchema#") {
            // replace the full URI with xsd: prefix
            if let Some(pos) = value.find("^^<http://www.w3.org/2001/XMLSchema#") {
                let (lit, rest) = value.split_at(pos);
                if rest.contains("decimal") {
                    return format!("{}^^xsd:decimal", lit.trim());
                } else if rest.contains("string") {
                    return format!("{}^^xsd:string", lit.trim());
                }
            }
        }

        // Map common vocabularies
        let mappings = [
            ("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "rdf:"),
            ("http://www.w3.org/2000/01/rdf-schema#", "rdfs:"),
            ("http://www.w3.org/2001/XMLSchema#", "xsd:"),
            ("http://www.w3.org/2002/07/owl#", "owl:"),
            ("http://www.w3.org/ns/shacl#", "sh:"),
            ("http://domainforge.ai/sea#", "sea:"),
            ("http://domainforge.ai/rdfs#", "rdfs:"),
        ];

        for (ns, prefix) in &mappings {
            if let Some(stripped) = value.strip_prefix(ns) {
                return format!("{}{}", prefix, stripped);
            }
        }

        // Fall back to original token
        t.to_string()
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

    /// Validates that a string is a safe RDF term for use in triples.
    /// Returns true if the term is valid (no quotes, angle brackets, control chars, backslashes, or illegal colons).
    fn is_valid_rdf_term(term: &str) -> bool {
        // Check for dangerous characters
        if term.contains('"') || term.contains('<') || term.contains('>') || term.contains('\\') {
            return false;
        }

        // Check for control characters
        if term.chars().any(|c| c.is_control()) {
            return false;
        }

        // Check for illegal colons (only allow prefixed names like "sea:Something" or local names without colons)
        // A valid prefixed name has exactly one colon not at the start or end
        let colon_count = term.matches(':').count();
        if colon_count > 1 {
            return false;
        }
        if colon_count == 1 && (term.starts_with(':') || term.ends_with(':')) {
            return false;
        }

        true
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

        let entity = Entity::new_with_namespace("TestEntity", "default".to_string());
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

        let entity_space = Entity::new_with_namespace("Entity With Space", "default".to_string());
        let entity_colon = Entity::new_with_namespace("Entity:Colon", "default".to_string());
        let entity_slash = Entity::new_with_namespace("Entity/Slash", "default".to_string());
        let entity_hash = Entity::new_with_namespace("Entity#Hash", "default".to_string());

        graph.add_entity(entity_space.clone()).unwrap();
        graph.add_entity(entity_colon.clone()).unwrap();
        graph.add_entity(entity_slash.clone()).unwrap();
        graph.add_entity(entity_hash.clone()).unwrap();

        let resource = Resource::new_with_namespace(
            "Resource:Name/Hash",
            crate::units::unit_from_string("units"),
            "default".to_string(),
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
