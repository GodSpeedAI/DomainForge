use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub policy_name: String,
    pub message: String,
    pub severity: Severity,
    pub context: serde_json::Value,
}

impl Violation {
    pub fn new(
        policy_name: impl Into<String>,
        message: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Self {
            policy_name: policy_name.into(),
            message: message.into(),
            severity,
            context: serde_json::json!({}),
        }
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }
}
