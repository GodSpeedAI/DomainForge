//! Parse SEA files and output Graph JSON.
//!
//! This command parses a SEA DSL file and outputs the result in the specified format.
//! Use `--format json` to get structured JSON output suitable for tooling.

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct ParseArgs {
    /// Input SEA file to parse
    #[arg(required = true)]
    pub input: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value_t = ParseFormat::Human)]
    pub format: ParseFormat,

    /// Output file (stdout if not specified)
    #[arg(long, short)]
    pub out: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug, Copy, Default)]
pub enum ParseFormat {
    /// Human-readable summary
    #[default]
    Human,
    /// JSON output (Graph structure)
    Json,
}

pub fn run(args: ParseArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read file: {}", args.input.display()))?;

    let output = match args.format {
        ParseFormat::Json => {
            let graph = crate::parser::parse_to_graph(&source)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            serde_json::to_string_pretty(&graph).context("Failed to serialize Graph to JSON")?
        }
        ParseFormat::Human => {
            let graph = crate::parser::parse_to_graph(&source)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            format!(
                "Parsed successfully: {}\n\
                 Entities: {}\n\
                 Resources: {}\n\
                 Flows: {}\n\
                 Roles: {}\n\
                 Relations: {}\n\
                 Instances: {}\n\
                 Policies: {}",
                args.input.display(),
                graph.entity_count(),
                graph.resource_count(),
                graph.flow_count(),
                graph.role_count(),
                graph.relation_count(),
                graph.instance_count(),
                graph.pattern_count()
            )
        }
    };

    // Write output
    match args.out {
        Some(path) => {
            fs::write(&path, &output)
                .with_context(|| format!("Failed to write to: {}", path.display()))?;
            eprintln!("Wrote output to: {}", path.display());
        }
        None => {
            println!("{}", output);
        }
    }

    Ok(())
}
