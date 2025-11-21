use wasm_bindgen::prelude::*;

/// Severity level for policy violations
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Error = 0,
    Warning = 1,
    Info = 2,
}

impl From<crate::policy::Severity> for Severity {
    fn from(severity: crate::policy::Severity) -> Self {
        match severity {
            crate::policy::Severity::Error => Severity::Error,
            crate::policy::Severity::Warning => Severity::Warning,
            crate::policy::Severity::Info => Severity::Info,
        }
    }
}

/// A policy violation
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Violation {
    #[wasm_bindgen(readonly)]
    pub name: String,
    #[wasm_bindgen(readonly)]
    pub message: String,
    #[wasm_bindgen(readonly)]
    pub severity: Severity,
}

impl From<crate::policy::Violation> for Violation {
    fn from(v: crate::policy::Violation) -> Self {
        Self {
            name: v.policy_name,
            message: v.message,
            severity: v.severity.into(),
        }
    }
}

/// Result of evaluating a policy against a graph
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct EvaluationResult {
    /// Backward-compatible boolean result (false if evaluation is NULL)
    #[wasm_bindgen(readonly, js_name = isSatisfied)]
    pub is_satisfied: bool,
    
    /// Three-valued result: true, false, or undefined (NULL)
    /// Note: In WASM, Option<bool> where None becomes undefined in JS
    #[wasm_bindgen(readonly, js_name = isSatisfiedTristate)]
    pub is_satisfied_tristate: Option<bool>,
    
    /// List of violations
    #[wasm_bindgen(readonly)]
    pub violations: Vec<Violation>,
}

impl From<crate::policy::EvaluationResult> for EvaluationResult {
    fn from(result: crate::policy::EvaluationResult) -> Self {
        Self {
            is_satisfied: result.is_satisfied,
            is_satisfied_tristate: result.is_satisfied_tristate,
            violations: result.violations.into_iter().map(|v| v.into()).collect(),
        }
    }
}
