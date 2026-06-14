use crate::graph::Graph;
use crate::policy::{PolicyKind, PolicyModality, Severity, Violation};

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

/// Produce the canonical `sea validate --format json` aggregate as a
/// [`serde_json::Value`].
///
/// This is the **single producer** for validate-format JSON across the CLI and
/// every binding (Python / TypeScript / WASM). Bindings wrap the result with
/// `serde_json::to_string_pretty`; they never reconstruct the aggregate from
/// binding-native objects. This guarantees byte-identical output across
/// surfaces (audit goal G5).
///
/// The shape is:
/// ```json
/// {
///   "evaluation_mode": "three_valued" | "boolean",
///   "error_count": <usize>,
///   "policies": [ { name, namespace, modality, kind, is_satisfied,
///                    is_satisfied_tristate, evaluation_mode, violations },
///                 { ..., "error": "<string>" } ],
///   "violations": [ { severity, policy_name, message, context } ]
/// }
/// ```
pub fn validate_to_canonical_json(graph: &Graph) -> serde_json::Value {
    let result = graph.validate();
    let graph_mode = if graph.use_three_valued_logic() {
        "three_valued"
    } else {
        "boolean"
    };

    let policy_results: Vec<serde_json::Value> = graph
        .all_policies()
        .iter()
        .map(
            |policy| match policy.evaluate_with_mode(graph, graph.use_three_valued_logic()) {
                Ok(eval) => serde_json::json!({
                    "name": policy.name,
                    "namespace": policy.namespace,
                    "modality": modality_str(&policy.modality),
                    "kind": kind_str(policy.kind()),
                    "is_satisfied": eval.is_satisfied,
                    "is_satisfied_tristate": eval.is_satisfied_tristate,
                    "evaluation_mode": eval.evaluation_mode.as_str(),
                    "violations": eval
                        .violations
                        .iter()
                        .map(|v| v.message.clone())
                        .collect::<Vec<_>>(),
                }),
                Err(err) => serde_json::json!({
                    "name": policy.name,
                    "namespace": policy.namespace,
                    "modality": modality_str(&policy.modality),
                    "kind": kind_str(policy.kind()),
                    "error": err,
                }),
            },
        )
        .collect();

    serde_json::json!({
        "evaluation_mode": graph_mode,
        "error_count": result.error_count,
        "policies": policy_results,
        "violations": result.violations.iter().map(|v| {
            serde_json::json!({
                "severity": severity_str(&v.severity),
                "policy_name": v.policy_name,
                "message": v.message,
                "context": v.context,
            })
        }).collect::<Vec<_>>(),
    })
}

/// Stable lowercase string for a policy modality used in canonical JSON output.
fn modality_str(m: &PolicyModality) -> &'static str {
    match m {
        PolicyModality::Obligation => "obligation",
        PolicyModality::Prohibition => "prohibition",
        PolicyModality::Permission => "permission",
    }
}

/// Stable lowercase string for a policy kind used in canonical JSON output.
fn kind_str(k: &PolicyKind) -> &'static str {
    match k {
        PolicyKind::Constraint => "constraint",
        PolicyKind::Derivation => "derivation",
        PolicyKind::Obligation => "obligation",
    }
}

/// Stable lowercase string for a violation severity used in canonical JSON output.
fn severity_str(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}
