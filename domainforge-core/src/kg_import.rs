use crate::graph::Graph;
use crate::kg::KnowledgeGraph;

// These types are only used when the 'shacl' feature is enabled; make their
// imports conditional to avoid unused-import warnings when the feature is off.
#[cfg(feature = "shacl")]
use crate::kg::{ShaclProperty, ShaclShape};
use std::fmt;

#[cfg(feature = "shacl")]
use oxigraph::model::{GraphNameRef, Term};
#[cfg(feature = "shacl")]
use oxigraph::sparql::QueryResults;
#[cfg(feature = "shacl")]
use oxigraph::store::Store;

#[derive(Debug)]
pub enum ImportError {
    ShaclValidation(String),
    Other(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::ShaclValidation(msg) => write!(f, "{}", msg),
            ImportError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ImportError {}

pub fn import_kg_turtle(turtle: &str) -> Result<Graph, ImportError> {
    match KnowledgeGraph::from_turtle(turtle) {
        Ok(kg) => validate_and_convert(kg),
        Err(e) => Err(ImportError::Other(format!(
            "Failed to parse Turtle KG: {}",
            e
        ))),
    }
}

fn validate_and_convert(kg: KnowledgeGraph) -> Result<Graph, ImportError> {
    match kg.validate_shacl() {
        Ok(vs) => {
            if !vs.is_empty() {
                let summary = vs
                    .iter()
                    .map(|v| format!("[{:?}] {}", v.severity, v.message))
                    .collect::<Vec<_>>()
                    .join("; ");
                Err(ImportError::ShaclValidation(format!(
                    "SHACL validation failed: {}",
                    summary
                )))
            } else {
                match kg.to_graph() {
                    Ok(graph) => Ok(graph),
                    Err(e) => Err(ImportError::Other(format!(
                        "Failed to convert KG to Graph: {}",
                        e
                    ))),
                }
            }
        }
        Err(e) => Err(ImportError::ShaclValidation(format!(
            "SHACL validation failed: {}",
            e
        ))),
    }
}

pub fn import_kg_rdfxml(xml: &str) -> Result<Graph, ImportError> {
    #[cfg(feature = "shacl")]
    {
        let store = Store::new()
            .map_err(|e| ImportError::Other(format!("Failed to create oxigraph store: {}", e)))?;
        let fmt = oxigraph::io::GraphFormat::RdfXml;

        // Load the RDF/XML into the default graph
        store
            .load_graph(xml.as_bytes(), fmt, GraphNameRef::DefaultGraph, None)
            .map_err(|e| {
                ImportError::Other(format!("Failed to parse RDF/XML with oxigraph: {}", e))
            })?;

        // Serialize the parsed RDF/XML into Turtle so we can reuse KnowledgeGraph::from_turtle
        let mut writer = Vec::new();
        let turtle_fmt = oxigraph::io::GraphFormat::Turtle;
        store
            .dump_graph(&mut writer, turtle_fmt, GraphNameRef::DefaultGraph)
            .map_err(|e| {
                ImportError::Other(format!("Failed to serialize RDF/XML to Turtle: {}", e))
            })?;
        let turtle_str = String::from_utf8(writer)
            .map_err(|e| ImportError::Other(format!("Invalid UTF-8: {}", e)))?;

        let mut kg = KnowledgeGraph::from_turtle(&turtle_str)
            .map_err(|e| ImportError::Other(format!("Failed to convert RDF/XML to KG: {}", e)))?;

        // If we didn't pick up any shapes from the Turtle representation, try to extract
        // SHACL shapes directly from the store via SPARQL.
        if kg.shapes.is_empty() {
            augment_shapes_from_store(&store, &mut kg)?;
        }

        validate_and_convert(kg)
    }
    #[cfg(not(feature = "shacl"))]
    {
        let _ = xml;
        Err(ImportError::Other(
            "RDF/XML import is not supported in this build (enable feature 'shacl')".to_string(),
        ))
    }
}

#[cfg(feature = "shacl")]
fn augment_shapes_from_store(store: &Store, kg: &mut KnowledgeGraph) -> Result<(), ImportError> {
    let q = r#"
        PREFIX sh: <http://www.w3.org/ns/shacl#>
        PREFIX sea: <http://domainforge.ai/sea#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
        SELECT ?shape ?target ?path ?datatype ?minCount ?maxCount ?minExclusive WHERE {
            ?shape a sh:NodeShape .
            OPTIONAL { ?shape sh:targetClass ?target . }
            ?shape sh:property ?prop .
            ?prop sh:path ?path .
            OPTIONAL { ?prop sh:datatype ?datatype . }
            OPTIONAL { ?prop sh:minCount ?minCount . }
            OPTIONAL { ?prop sh:maxCount ?maxCount . }
            OPTIONAL { ?prop sh:minExclusive ?minExclusive . }
        }
    "#;

    match store.query(q) {
        Ok(QueryResults::Solutions(solutions)) => {
            use std::collections::HashMap;

            // Map: shape URI -> (optional target class IRI, properties)
            let mut map: HashMap<String, (Option<String>, Vec<ShaclProperty>)> = HashMap::new();

            for sol_res in solutions {
                let sol = sol_res.map_err(|e| {
                    ImportError::Other(format!("Error reading SHACL SPARQL solution: {}", e))
                })?;

                let term_to_named_str = |t: &Term| match t {
                    Term::NamedNode(nn) => Some(nn.as_str().to_string()),
                    _ => None,
                };
                let term_to_literal_val = |t: &Term| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                };

                let shape_term = sol.get("shape").and_then(term_to_named_str);
                let target_term = sol.get("target").and_then(term_to_named_str);
                let path_term = sol.get("path").and_then(term_to_named_str);
                // datatype can be an IRI like http://www.w3.org/2001/XMLSchema#decimal
                let datatype_term = sol.get("datatype").and_then(term_to_named_str);
                let min_count = sol
                    .get("minCount")
                    .and_then(term_to_literal_val)
                    .and_then(|v| v.parse::<u32>().ok());
                let max_count = sol
                    .get("maxCount")
                    .and_then(term_to_literal_val)
                    .and_then(|v| v.parse::<u32>().ok());
                // minExclusive returns a literal value (e.g., "0"^^xsd:decimal); we only use the lexical form
                let min_exclusive = sol.get("minExclusive").and_then(term_to_literal_val);

                if let (Some(shape), Some(path)) = (shape_term.clone(), path_term.clone()) {
                    let path_pref =
                        if let Some(stripped) = path.strip_prefix("http://domainforge.ai/sea#") {
                            format!("sea:{}", stripped)
                        } else if let Some(stripped) =
                            path.strip_prefix("http://www.w3.org/2000/01/rdf-schema#")
                        {
                            format!("rdfs:{}", stripped)
                        } else {
                            path.clone()
                        };

                    // Normalize datatype IRIs to xsd: prefixes
                    let datatype_pref = datatype_term.as_ref().map(|dt| {
                        if let Some(rest) = dt.strip_prefix("http://www.w3.org/2001/XMLSchema#") {
                            format!("xsd:{}", rest)
                        } else {
                            dt.clone()
                        }
                    });

                    // If min_exclusive present but datatype not specified, assume xsd:decimal
                    let inferred_datatype = match (&datatype_pref, &min_exclusive) {
                        (None, Some(_)) => Some("xsd:decimal".to_string()),
                        _ => datatype_pref.clone(),
                    };

                    let prop = ShaclProperty {
                        path: path_pref,
                        datatype: inferred_datatype,
                        min_count,
                        max_count,
                        min_exclusive,
                    };

                    let entry = map.entry(shape.clone()).or_insert((None, Vec::new()));
                    if let Some(target) = target_term.clone() {
                        entry.0 = Some(target.clone());
                    }
                    entry.1.push(prop);
                }
            }

            let mut extracted_shapes: Vec<ShaclShape> = Vec::new();
            for (shape_uri, (target_opt, props)) in map.into_iter() {
                let target_class = if let Some(target) = target_opt {
                    if let Some(stripped) = target.strip_prefix("http://domainforge.ai/sea#") {
                        format!("sea:{}", stripped)
                    } else {
                        target
                    }
                } else if let Some(pos) = shape_uri.find('#') {
                    let fragment = &shape_uri[pos + 1..];
                    let fragment = fragment.strip_suffix("Shape").unwrap_or(fragment);
                    format!("sea:{}", fragment)
                } else {
                    shape_uri.clone()
                };

                extracted_shapes.push(ShaclShape {
                    target_class,
                    properties: props,
                });
            }

            if !extracted_shapes.is_empty() {
                kg.shapes = extracted_shapes;
            }

            Ok(())
        }
        Ok(_) => Ok(()),
        Err(e) => Err(ImportError::Other(format!(
            "Failed to execute SHACL SPARQL query: {}",
            e
        ))),
    }
}
