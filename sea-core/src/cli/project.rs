use clap::{Parser, ValueEnum};
use crate::parser::{parse_to_graph_with_options, ParseOptions};
use crate::NamespaceRegistry;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Parser)]
pub struct ProjectArgs {
    #[arg(long, value_enum)]
    pub format: ProjectFormat,

    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ProjectFormat {
    Calm,
    Kg,
}

pub fn run(args: ProjectArgs) -> Result<()> {
    let source = read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file {}", args.input.display()))?;

    // Parse input
    let registry = NamespaceRegistry::discover(&args.input)
        .context("discovering namespace registry")?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(&args.input).map(|ns| ns.to_string()));
    let options = ParseOptions { default_namespace };
    let graph = parse_to_graph_with_options(&source, &options)
        .map_err(|e| anyhow::anyhow!("Parse failed for {}: {}", args.input.display(), e))?;

    match args.format {
        ProjectFormat::Calm => {
            let value = crate::calm::export(&graph)
                .map_err(|e| anyhow::anyhow!("Failed to export to CALM: {}", e))?;
            let json = serde_json::to_string_pretty(&value)
                .context("Failed to serialize CALM JSON")?;
            write(&args.output, json)
                .with_context(|| format!("Failed to write output to {}", args.output.display()))?;
            println!("Projected to CALM: {}", args.output.display());
        }
        ProjectFormat::Kg => {
            let kg = crate::KnowledgeGraph::from_graph(&graph)
                .map_err(|e| anyhow::anyhow!("Failed to convert to Knowledge Graph: {}", e))?;
            
            let output_str = if args.output.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("xml") || ext.eq_ignore_ascii_case("rdf")) {
                kg.to_rdf_xml()
            } else {
                kg.to_turtle()
            };

            write(&args.output, output_str)
                .with_context(|| format!("Failed to write output to {}", args.output.display()))?;
            println!("Projected to KG: {}", args.output.display());
        }
    }

    Ok(())
}
