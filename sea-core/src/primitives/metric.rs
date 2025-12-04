use crate::policy::Expression;
use crate::ConceptId;
use chrono::Duration;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: ConceptId,
    pub name: String,
    pub namespace: String,
    pub expression: Expression,
    pub refresh_interval: Option<Duration>,
    pub unit: Option<String>,
    pub threshold: Option<Decimal>,
    pub severity: Option<Severity>,
    pub target: Option<Decimal>,
    pub window: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Metric {
    pub fn new(name: String, namespace: String, expression: Expression) -> Self {
        let id = ConceptId::from_concept(&namespace, &name);
        Self {
            id,
            name,
            namespace,
            expression,
            refresh_interval: None,
            unit: None,
            threshold: None,
            severity: None,
            target: None,
            window: None,
        }
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn with_refresh_interval(mut self, duration: Duration) -> Self {
        self.refresh_interval = Some(duration);
        self
    }

    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn with_threshold(mut self, threshold: Decimal) -> Self {
        self.threshold = Some(threshold);
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn with_target(mut self, target: Decimal) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_window(mut self, window: Duration) -> Self {
        self.window = Some(window);
        self
    }
}
