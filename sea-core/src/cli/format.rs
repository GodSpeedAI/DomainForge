use crate::parser::parse;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser)]
pub struct FormatArgs {
    pub file: PathBuf,
}

pub fn run(args: FormatArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    // Verify it parses
    parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse failed for {}: {}", args.file.display(), e))?;

    // Formatting is not yet implemented
    anyhow::bail!("Formatting not yet implemented. Only syntax validation is currently supported.");
}
