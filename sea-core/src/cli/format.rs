use clap::Parser;
use crate::parser::parse;
use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Parser)]
pub struct FormatArgs {
    pub file: PathBuf,
}

pub fn run(args: FormatArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    // Verify it parses
    parse(&source).map_err(|e| anyhow::anyhow!("Parse failed for {}: {}", args.file.display(), e))?;

    // Formatting is not yet implemented
    anyhow::bail!("Formatting not yet implemented. Only syntax validation is currently supported.");
}
