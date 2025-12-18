//! CLI module for the `sea normalize` command.
//!
//! Provides expression normalization and equivalence checking from the command line.

use crate::parser::parse_expression_from_str;
use clap::Args;

/// Arguments for the `normalize` subcommand.
#[derive(Args, Debug)]
pub struct NormalizeArgs {
    /// Expression to normalize (in SEA DSL syntax)
    #[arg(help = "Expression to normalize, e.g., \"b AND a\"")]
    pub expression: String,

    /// Second expression to compare for equivalence
    #[arg(
        long,
        value_name = "EXPR",
        help = "Compare with another expression for equivalence"
    )]
    pub check_equiv: Option<String>,

    /// Output result as JSON
    #[arg(long, help = "Output as JSON object")]
    pub json: bool,
}

/// Result of normalization for JSON output.
#[derive(serde::Serialize)]
struct NormalizeResult {
    normalized: String,
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    equivalent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    other_normalized: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    other_hash: Option<String>,
}

/// Run the normalize command.
pub fn run(args: NormalizeArgs) -> anyhow::Result<()> {
    // Parse the first expression
    let expr1 = parse_expression_from_str(&args.expression)
        .map_err(|e| anyhow::anyhow!("Failed to parse expression: {}", e))?;

    // Normalize the expression
    let normalized1 = expr1.normalize();

    // Check equivalence if a second expression is provided
    let (equivalent, other_normalized, other_hash) = if let Some(ref other_expr) = args.check_equiv
    {
        let expr2 = parse_expression_from_str(other_expr)
            .map_err(|e| anyhow::anyhow!("Failed to parse second expression: {}", e))?;

        let normalized2 = expr2.normalize();
        let is_equiv = normalized1 == normalized2;

        (
            Some(is_equiv),
            Some(normalized2.to_string()),
            Some(format!("{:#018x}", normalized2.stable_hash())),
        )
    } else {
        (None, None, None)
    };

    if args.json {
        // JSON output
        let result = NormalizeResult {
            normalized: normalized1.to_string(),
            hash: format!("{:#018x}", normalized1.stable_hash()),
            equivalent,
            other_normalized,
            other_hash,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        // Human-readable output
        println!("{}", normalized1);

        if let (Some(is_equiv), Some(other_norm), _) = (equivalent, other_normalized, other_hash) {
            if is_equiv {
                println!("Equivalent (hash: {:#018x})", normalized1.stable_hash());
            } else {
                println!("NOT Equivalent");
                println!("  First:  {}", normalized1);
                println!("  Second: {}", other_norm);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_simple() {
        let args = NormalizeArgs {
            expression: "b AND a".to_string(),
            check_equiv: None,
            json: false,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_normalize_equivalence() {
        let args = NormalizeArgs {
            expression: "a AND b".to_string(),
            check_equiv: Some("b AND a".to_string()),
            json: false,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_normalize_json() {
        let args = NormalizeArgs {
            expression: "true AND x".to_string(),
            check_equiv: None,
            json: true,
        };
        assert!(run(args).is_ok());
    }
}
