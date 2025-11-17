use crate::policy::{Severity, Violation};

/// Result object produced by `Graph::validate()` collecting the
/// set of policy violations and a quick summary count for errors
/// to support the CLI and tests.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Total number of policies evaluated
    pub total_policies: usize,

    /// All policy violations collected during evaluation
    pub violations: Vec<Violation>,

    /// Number of ERROR-severity violations
    pub error_count: usize,
}

impl ValidationResult {
    pub fn new(total: usize, violations: Vec<Violation>) -> Self {
        let error_count = violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count();
        Self {
            total_policies: total,
            violations,
            error_count,
        }
    }
}
