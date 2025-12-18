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
    run_with_writer(args, &mut std::io::stdout())
}

/// Run the normalize command with a specific writer for output capture.
pub fn run_with_writer<W: std::io::Write>(
    args: NormalizeArgs,
    mut writer: W,
) -> anyhow::Result<()> {
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
        writeln!(writer, "{}", serde_json::to_string_pretty(&result)?)?;
    } else {
        // Human-readable output
        writeln!(writer, "{}", normalized1)?;

        if let (Some(is_equiv), Some(other_norm), _) = (equivalent, other_normalized, other_hash) {
            if is_equiv {
                writeln!(
                    writer,
                    "Equivalent (hash: {:#018x})",
                    normalized1.stable_hash()
                )?;
            } else {
                writeln!(writer, "NOT Equivalent")?;
                writeln!(writer, "  First:  {}", normalized1)?;
                writeln!(writer, "  Second: {}", other_norm)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_output() {
        let args = NormalizeArgs {
            expression: "b AND a".to_string(),
            check_equiv: None,
            json: false,
        };
        let mut buffer = Vec::new();
        run_with_writer(args, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output.trim(), "(a AND b)");
    }

    #[test]
    fn test_normalize_equivalence_check_true() {
        let args = NormalizeArgs {
            expression: "a AND b".to_string(),
            check_equiv: Some("b AND a".to_string()),
            json: false,
        };
        let mut buffer = Vec::new();
        run_with_writer(args, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Equivalent"));
        assert!(output.contains("(a AND b)"));
    }

    #[test]
    fn test_normalize_equivalence_check_false() {
        let args = NormalizeArgs {
            expression: "a AND b".to_string(),
            check_equiv: Some("a OR b".to_string()),
            json: false,
        };
        let mut buffer = Vec::new();
        run_with_writer(args, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("NOT Equivalent"));
    }

    #[test]
    fn test_normalize_json_output() {
        let args = NormalizeArgs {
            expression: "true AND x".to_string(),
            check_equiv: Some("x".to_string()),
            json: true,
        };
        let mut buffer = Vec::new();
        run_with_writer(args, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        let json_val: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON");
        assert_eq!(json_val["normalized"], "x");
        assert_eq!(json_val["equivalent"], true);
    }

    #[test]
    fn test_invalid_expression() {
        let args = NormalizeArgs {
            expression: "NOT (a".to_string(),
            check_equiv: None,
            json: false,
        };
        let mut buffer = Vec::new();
        let result = run_with_writer(args, &mut buffer);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }
}
