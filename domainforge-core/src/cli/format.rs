use crate::formatter::{format, FormatConfig, IndentStyle};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::{read_to_string, write};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
pub struct FormatArgs {
    /// SEA file to format
    pub file: PathBuf,

    /// Output file (defaults to stdout)
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Number of spaces per indentation level
    #[arg(long, default_value = "4")]
    pub indent_width: usize,

    /// Use tabs instead of spaces for indentation
    #[arg(long)]
    pub use_tabs: bool,

    /// Check if file would be reformatted (exit 1 if changes needed)
    #[arg(long)]
    pub check: bool,

    /// Sort imports alphabetically
    #[arg(long, default_value = "true")]
    pub sort_imports: bool,
}

pub fn run(args: FormatArgs) -> Result<()> {
    let source = read_to_string(&args.file)
        .with_context(|| format!("Failed to read file {}", args.file.display()))?;

    let config = FormatConfig {
        indent_width: args.indent_width,
        indent_style: if args.use_tabs {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces
        },
        sort_imports: args.sort_imports,
        ..Default::default()
    };

    let formatted = format(&source, config)
        .map_err(|e| anyhow::anyhow!("Format failed for {}: {}", args.file.display(), e))?;

    if args.check {
        // Check mode: exit with error if file would change
        if source != formatted {
            // Show diff summary
            let source_lines = source.lines().count();
            let formatted_lines = formatted.lines().count();
            eprintln!(
                "Would reformat {} ({} -> {} lines)",
                args.file.display(),
                source_lines,
                formatted_lines
            );
            anyhow::bail!("File would be reformatted: {}", args.file.display());
        } else {
            println!("{}: already formatted", args.file.display());
        }
    } else if let Some(out_path) = args.out {
        // Write to specified output file
        write(&out_path, &formatted)
            .with_context(|| format!("Failed to write to {}", out_path.display()))?;
        println!(
            "Formatted {} -> {}",
            args.file.display(),
            out_path.display()
        );
    } else {
        // Write to stdout
        io::stdout()
            .write_all(formatted.as_bytes())
            .context("Failed to write to stdout")?;
    }

    Ok(())
}
