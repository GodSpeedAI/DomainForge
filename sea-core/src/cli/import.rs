use clap::{Parser, ValueEnum};
use crate::import_kg_turtle;
use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[cfg(feature = "shacl")]
use crate::kg_import::ImportError;

#[derive(Parser)]
pub struct ImportArgs {
    #[arg(long, value_enum)]
    pub format: ImportFormat,

    pub file: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ImportFormat {
    Sbvr,
    Kg,
}

pub fn run(args: ImportArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    match args.format {
        ImportFormat::Sbvr => match crate::SbvrModel::from_xmi(&source) {
            Ok(model) => match model.to_graph() {
                Ok(graph) => {
                    println!(
                        "Imported SBVR to Graph: entities={} resources={} flows={}",
                        graph.entity_count(),
                        graph.resource_count(),
                        graph.flow_count()
                    );
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Failed to convert SBVR to Graph: {}", e)),
            },
            Err(e) => Err(anyhow::anyhow!("Failed to parse SBVR XMI: {}", e)),
        },
        ImportFormat::Kg => {
            let src_trim = source.trim_start();
            if src_trim.starts_with('<') {
                #[cfg(feature = "shacl")]
                {
                    match crate::import_kg_rdfxml(&source) {
                        Ok(graph) => {
                            println!("Imported KG (RDF/XML) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                            Ok(())
                        }
                        Err(err_rdf) => match err_rdf {
                            ImportError::ShaclValidation(msg) => {
                                Err(anyhow::anyhow!("{}", msg))
                            }
                            other => {
                                let err_rdf_msg = other.to_string();
                                match import_kg_turtle(&source) {
                                    Ok(graph) => {
                                        println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                        Ok(())
                                    }
                                    Err(err_turtle) => {
                                        Err(anyhow::anyhow!("Failed to import KG as RDF/XML: {}; as Turtle: {}", err_rdf_msg, err_turtle))
                                    }
                                }
                            }
                        },
                    }
                }
                #[cfg(not(feature = "shacl"))]
                {
                    match import_kg_turtle(&source) {
                        Ok(graph) => {
                            println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                            Ok(())
                        }
                        Err(err) => {
                            Err(anyhow::anyhow!("Failed to import KG: {}", err))
                        }
                    }
                }
            } else {
                match import_kg_turtle(&source) {
                    Ok(graph) => {
                        println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                        Ok(())
                    }
                    Err(err_turtle) => {
                        #[cfg(feature = "shacl")]
                        {
                            match crate::import_kg_rdfxml(&source) {
                                Ok(graph) => {
                                    println!("Imported KG (RDF/XML) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                    Ok(())
                                }
                                Err(err_rdf) => match err_rdf {
                                    ImportError::ShaclValidation(msg) => {
                                        Err(anyhow::anyhow!("{}", msg))
                                    }
                                    other => {
                                        let err_rdf_msg = other.to_string();
                                        Err(anyhow::anyhow!("Failed to import KG as Turtle: {}; and as RDF/XML: {}", err_turtle, err_rdf_msg))
                                    }
                                },
                            }
                        }
                        #[cfg(not(feature = "shacl"))]
                        {
                            Err(anyhow::anyhow!("Failed to import KG (Turtle): {}. To try RDF/XML, enable feature 'shacl' in the build.", err_turtle))
                        }
                    }
                }
            }
        }
    }
}
