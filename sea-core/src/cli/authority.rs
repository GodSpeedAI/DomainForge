//! CLI module for the `sea authority` command.
//!
//! Provides authority evaluation from the command line.

use crate::authority::{
    AuthorityEnvironment, AuthorityEnvironmentConfig, AuthorityRequest, FactEnvelope,
};
use clap::Args;

/// Arguments for the `authority` subcommand.
#[derive(Args, Debug)]
pub struct AuthorityArgs {
    /// Path to authority environment config JSON file
    #[arg(help = "Path to authority environment config JSON")]
    pub config: String,

    /// Path to authority request JSON file
    #[arg(help = "Path to authority request JSON")]
    pub request: String,

    /// Path to facts JSON file (optional)
    #[arg(long, value_name = "FILE", help = "Path to facts JSON file")]
    pub facts: Option<String>,

    /// Output result as JSON
    #[arg(long, help = "Output as JSON object")]
    pub json: bool,
}

/// Result of authority evaluation for JSON output.
#[derive(serde::Serialize)]
struct AuthorityResult {
    decision_id: String,
    request_id: String,
    final_decision: String,
    reason_code: String,
    trace_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    operator_action_hint: Option<String>,
}

/// Run the authority command.
pub fn run(args: AuthorityArgs) -> anyhow::Result<()> {
    run_with_writer(args, &mut std::io::stdout())
}

/// Run the authority command with a specific writer for output capture.
pub fn run_with_writer<W: std::io::Write>(
    args: AuthorityArgs,
    mut writer: W,
) -> anyhow::Result<()> {
    // Load config
    let config_str = std::fs::read_to_string(&args.config)
        .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", args.config, e))?;
    let config: AuthorityEnvironmentConfig = serde_json::from_str(&config_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse config JSON: {}", e))?;

    // Create and validate environment
    let mut env = AuthorityEnvironment::new(config)
        .map_err(|e| anyhow::anyhow!("Failed to create authority environment: {}", e))?;
    env.validate()
        .map_err(|e| anyhow::anyhow!("Environment validation failed: {}", e))?;

    // Load request
    let request_str = std::fs::read_to_string(&args.request)
        .map_err(|e| anyhow::anyhow!("Failed to read request file '{}': {}", args.request, e))?;
    let request: AuthorityRequest = serde_json::from_str(&request_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse request JSON: {}", e))?;

    // Load facts (optional)
    let facts: Vec<FactEnvelope> = if let Some(ref facts_path) = args.facts {
        let facts_str = std::fs::read_to_string(facts_path)
            .map_err(|e| anyhow::anyhow!("Failed to read facts file '{}': {}", facts_path, e))?;
        serde_json::from_str(&facts_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse facts JSON: {}", e))?
    } else {
        vec![]
    };

    // Evaluate
    let (trace, decision) = env
        .evaluate(&request, &facts)
        .map_err(|e| anyhow::anyhow!("Authority evaluation failed: {}", e))?;

    if args.json {
        let result = AuthorityResult {
            decision_id: decision.decision_id,
            request_id: decision.request_id,
            final_decision: format!("{:?}", decision.final_decision),
            reason_code: decision.reason_code,
            trace_ref: decision.trace_ref,
            operator_action_hint: decision.operator_action_hint,
        };
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize result: {}", e))?;
        writeln!(writer, "{}", json)?;
    } else {
        writeln!(writer, "Authority Decision")?;
        writeln!(writer, "  Decision ID:  {}", decision.decision_id)?;
        writeln!(writer, "  Request ID:   {}", decision.request_id)?;
        writeln!(writer, "  Decision:     {:?}", decision.final_decision)?;
        writeln!(writer, "  Reason:       {}", decision.reason_code)?;
        writeln!(writer, "  Trace Ref:    {}", decision.trace_ref)?;
        if let Some(ref hint) = decision.operator_action_hint {
            writeln!(writer, "  Action Hint:  {}", hint)?;
        }
        writeln!(writer)?;
        writeln!(writer, "Trace Details")?;
        writeln!(
            writer,
            "  Semantics:    {} ({})",
            trace.resolver_semantics_version, trace.resolver_semantics_hash
        )?;
        writeln!(
            writer,
            "  Profile:      {} ({})",
            trace.specificity_profile_id, trace.specificity_profile_hash
        )?;
        writeln!(writer, "  Packs:        {:?}", trace.pack_hashes)?;
        writeln!(writer, "  Claim Level:  {:?}", trace.claim_level)?;
        writeln!(writer, "  Created:      {}", trace.created_at)?;
    }

    Ok(())
}
