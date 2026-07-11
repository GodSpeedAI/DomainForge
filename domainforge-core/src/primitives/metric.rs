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
    /// Breach condition: the metric is considered in violation when its
    /// value is `>= threshold` (the alerting-industry convention this crate
    /// follows — e.g. `error_rate.threshold = 5.0` breaches at 5% or above).
    /// If a metric's semantics are "breach when below" (e.g. an SLA
    /// on-time-delivery rate), invert the expression itself rather than the
    /// threshold direction, so this convention stays uniform for every
    /// projection target that renders it (M2).
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

    /// M2/M3: catches two silent-default gaps that previously only surfaced
    /// as confusing behavior downstream in projection targets:
    /// - `severity` implies a metric is actionable, which requires a
    ///   `threshold` to compare against; nothing enforced that pairing.
    /// - `refresh_interval`/`window` use signed `chrono::Duration` and
    ///   nothing rejected a negative one.
    ///
    /// `unit` is deliberately *not* checked against the `UnitRegistry` here
    /// (unlike M1's original recommendation): metric units in this codebase
    /// are commonly free-form business labels ("orders", "widgets") rather
    /// than physical/dimensional units, and real fixtures
    /// (`fixtures/projection_cell/basic/model.sea`) rely on that. Forcing
    /// registry membership would reject legitimate domain vocabulary, not
    /// just typos. `unit_from_string`'s fallback (see U1 in units/mod.rs)
    /// now keys unregistered units by their own symbol as a distinct
    /// dimension, so free-form labels no longer collide with each other —
    /// the only remaining unit-shaped defect is an empty/whitespace label,
    /// which is never meaningful and is rejected below.
    pub fn validate(&self) -> Result<(), String> {
        if self.unit.as_deref().is_some_and(|u| u.trim().is_empty()) {
            return Err(format!(
                "Metric '{}': `unit` must not be empty or whitespace-only.",
                self.name
            ));
        }
        if self.severity.is_some() && self.threshold.is_none() {
            return Err(format!(
                "Metric '{}': `severity` requires a `threshold` to compare against.",
                self.name
            ));
        }
        if self.refresh_interval.is_some_and(|d| d < Duration::zero()) {
            return Err(format!(
                "Metric '{}': refresh_interval must not be negative.",
                self.name
            ));
        }
        if self.window.is_some_and(|d| d < Duration::zero()) {
            return Err(format!(
                "Metric '{}': window must not be negative.",
                self.name
            ));
        }
        Ok(())
    }
}
