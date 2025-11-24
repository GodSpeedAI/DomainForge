use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
pub struct TestArgs {
    pub pattern: String,
}

pub fn run(args: TestArgs) -> Result<()> {
    println!("Test runner not yet implemented. Pattern: {}", args.pattern);
    Ok(())
}
