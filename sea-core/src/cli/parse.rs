//! Parse SEA files and output AST or Graph JSON.
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

    /// Output AST instead of Graph (preserves source structure and line numbers)
    #[arg(long)]
    pub ast: bool,
}

#[derive(ValueEnum, Clone, Debug, Copy, Default)]
pub enum ParseFormat {
    /// Human-readable summary
    #[default]
    Human,
    /// JSON output
    Json,
}

pub fn run(args: ParseArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read file: {}", args.input.display()))?;

    let output = match (args.ast, args.format) {
        // AST output (uses schema types for stable JSON format)
        (true, ParseFormat::Json) => {
            let internal_ast =
                crate::parser::parse(&source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            let schema_ast: crate::parser::ast_schema::Ast = internal_ast.into();
            serde_json::to_string_pretty(&schema_ast).context("Failed to serialize AST to JSON")?
        }
        (true, ParseFormat::Human) => {
            let ast =
                crate::parser::parse(&source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            format!(
                "AST parsed successfully: {}\n\
                 Namespace: {}\n\
                 Version: {}\n\
                 Declarations: {}",
                args.input.display(),
                ast.metadata.namespace.as_deref().unwrap_or("(none)"),
                ast.metadata.version.as_deref().unwrap_or("(none)"),
                ast.declarations.len()
            )
        }
        // Graph output (default)
        (false, ParseFormat::Json) => {
            let graph = crate::parser::parse_to_graph(&source)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            serde_json::to_string_pretty(&graph).context("Failed to serialize Graph to JSON")?
        }
        (false, ParseFormat::Human) => {
            let graph = crate::parser::parse_to_graph(&source)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            format!(
                "Graph parsed successfully: {}\n\
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
