use clap::Parser;
use crate::parser::parse;
use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Parser)]
pub struct FormatArgs {
    #[arg(long)]
    pub check: bool,

    pub file: PathBuf,
}

pub fn run(args: FormatArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    // Verify it parses
    parse(&source).map_err(|e| anyhow::anyhow!("Parse failed: {}", e))?;

    if args.check {
        println!("Format check passed (syntax only)");
    } else {
        println!("Formatting not yet implemented");
    }

    Ok(())
}
