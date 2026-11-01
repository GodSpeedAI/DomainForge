use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser)]
pub struct TestArgs {
    pub pattern: String,
}

/// The test runner is not implemented. Fail loudly so a script or CI step that
/// invokes `domainforge test <pattern>` cannot report success while testing nothing.
pub fn run(args: TestArgs) -> Result<()> {
    bail!(
        "`domainforge test` is not implemented (pattern: {}); no tests were run",
        args.pattern
    )
}
