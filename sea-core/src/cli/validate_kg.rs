use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Parser)]
pub struct ValidateKgArgs {
    pub file: PathBuf,
}

pub fn run(args: ValidateKgArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    let is_rdf_xml = args.file.extension().map_or(false, |ext| ext == "xml" || ext == "rdf") 
        || source.trim_start().starts_with('<');

    if is_rdf_xml {
        #[cfg(feature = "shacl")]
        {
             match crate::import_kg_rdfxml(&source) {
                Ok(_) => println!("Validation successful (RDF/XML)"),
                Err(e) => {
                    anyhow::bail!("Validation failed: {}", e);
                }
             }
        }
        #[cfg(not(feature = "shacl"))]
        {
            anyhow::bail!("RDF/XML validation requires 'shacl' feature");
        }
    } else {
        // Assume Turtle
        let kg = crate::KnowledgeGraph::from_turtle(&source)
            .map_err(|e| anyhow::anyhow!("Failed to parse Turtle: {}", e))?;
        
        let violations = kg.validate_shacl()
            .map_err(|e| anyhow::anyhow!("SHACL validation error: {}", e))?;

        if violations.is_empty() {
            println!("Validation successful (Turtle)");
        } else {
            println!("Validation failed: {} violations", violations.len());
            for v in violations {
                // Severity likely implements Display or Debug
                println!("- [{:?}] {}: {}", v.severity, v.policy_name, v.message);
            }
            anyhow::bail!("Validation failed");
        }
    }

    Ok(())
}
