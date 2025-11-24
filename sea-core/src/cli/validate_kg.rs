use anyhow::{Context, Result};
use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser)]
pub struct ValidateKgArgs {
    pub file: PathBuf,
}

#[allow(unreachable_code)]
pub fn run(args: ValidateKgArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    let src_trim = source.trim_start();

    // More robust RDF/XML detection:
    // 1. Check file extension first (most reliable)
    // 2. Look for XML declaration or RDF namespace markers
    // 3. Check for common XML patterns (allowing for processing instructions/comments)
    let is_rdf_xml = args
        .file
        .extension()
        .is_some_and(|ext| ext == "xml" || ext == "rdf")
        || src_trim.starts_with("<?xml")
        || src_trim.starts_with("<rdf:")
        || src_trim.starts_with("<rdf ")
        || src_trim.contains("xmlns:rdf")
        || src_trim.contains("http://www.w3.org/1999/02/22-rdf-syntax-ns#");

    // Log when format detection is ambiguous (no extension and starts with '<' but not clearly RDF/XML)
    if args.file.extension().is_none() && src_trim.starts_with('<') && !is_rdf_xml {
        eprintln!("Warning: File has no extension and starts with '<' but doesn't match RDF/XML patterns. Attempting Turtle first, will fall back to RDF/XML if parsing fails.");
    }

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
        #[cfg(feature = "shacl")]
        {
            let kg = crate::KnowledgeGraph::from_turtle(&source)
                .map_err(|e| anyhow::anyhow!("Failed to parse Turtle: {}", e))?;

            let violations = kg
                .validate_shacl()
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
        #[cfg(not(feature = "shacl"))]
        {
            anyhow::bail!("Turtle SHACL validation requires 'shacl' feature");
        }
    }

    Ok(())
}
