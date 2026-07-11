use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::error::AuthorityError;
use super::pack::AuthorityPack;
use super::policy::ObligationSpec;
use super::resolver::{ConflictResolutionStep, ResolverOutput};
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityTrace {
    pub decision_id: String,
    pub request_id: String,
    pub ir_hash: String,
    pub pack_hashes: Vec<String>,
    pub resolver_version: String,
    pub resolver_semantics_version: String,
    pub resolver_semantics_hash: String,
    pub specificity_profile_id: String,
    pub specificity_profile_hash: String,
    pub specificity_profile_source: String,
    pub specificity_profile_conflicts: Vec<String>,
    pub unknown_handling_config_hash: String,
    pub compatibility_lowering_version: String,
    pub action_request_hash: String,
    pub fact_envelopes: Vec<FactEnvelopeSummary>,
    pub fact_trust_decisions: Vec<FactTrustDecision>,
    pub derived_fact_lineage: Vec<DerivedFactLineage>,
    pub candidate_policies: Vec<String>,
    pub applicable_policies: Vec<String>,
    pub incomparable_policies: Vec<String>,
    pub unknown_decisions: Vec<UnknownDecision>,
    pub compatibility_lowering_decisions: Vec<CompatibilityLoweringDecision>,
    pub conflict_resolution_steps: Vec<ConflictStepRecord>,
    pub final_decision: FinalDecision,
    pub claim_level: ClaimLevel,
    pub validation_suite_ref: Option<String>,
    pub created_at: String,
    /// Obligations incurred alongside `final_decision` (A8).
    pub obligations: Vec<ObligationSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactEnvelopeSummary {
    pub path: String,
    pub source_class: SourceClass,
    pub source_id: String,
    pub trusted: bool,
}

impl From<&FactEnvelope> for FactEnvelopeSummary {
    fn from(f: &FactEnvelope) -> Self {
        Self {
            path: f.path.clone(),
            source_class: f.source_class,
            source_id: f.source_id.clone(),
            trusted: f.is_trusted(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStepRecord {
    pub step: String,
    pub policy_id: String,
    pub result: String,
}

impl From<&ConflictResolutionStep> for ConflictStepRecord {
    fn from(s: &ConflictResolutionStep) -> Self {
        Self {
            step: s.step.clone(),
            policy_id: s.policy_id.clone(),
            result: s.result.clone(),
        }
    }
}

pub struct AuthorityTraceEmitter {
    resolver_version: String,
    semantics_version: String,
    compatibility_version: String,
    evidence_sink: EvidenceSink,
    memory_store: Arc<Mutex<Vec<AuthorityTrace>>>,
}

pub enum EvidenceSink {
    Memory,
    Discard,
}

impl AuthorityTraceEmitter {
    pub fn new(
        resolver_version: String,
        semantics_version: String,
        compatibility_version: String,
        evidence_sink: EvidenceSink,
    ) -> Self {
        Self {
            resolver_version,
            semantics_version,
            compatibility_version,
            evidence_sink,
            memory_store: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_traces(&self) -> Vec<AuthorityTrace> {
        self.memory_store
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn clear_traces(&self) {
        if let Ok(mut s) = self.memory_store.lock() {
            s.clear();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        &self,
        request: &AuthorityRequest,
        packs: &[AuthorityPack],
        facts: &[FactEnvelope],
        fact_trust_decisions: &[FactTrustDecision],
        derived_lineages: &[DerivedFactLineage],
        output: &ResolverOutput,
        specificity_profile: &SpecificityProfile,
        unknown_handling: &UnknownHandlingConfig,
        compatibility_decisions: &[CompatibilityLoweringDecision],
    ) -> Result<(AuthorityTrace, AuthorityDecision), AuthorityError> {
        let trace = AuthorityTrace {
            decision_id: output.decision_id.clone(),
            request_id: request.request_id.clone(),
            ir_hash: compute_hash(
                &serde_json::to_string(&output.candidate_policies).unwrap_or_default(),
            ),
            pack_hashes: packs.iter().map(|p| p.hash.clone()).collect(),
            resolver_version: self.resolver_version.clone(),
            resolver_semantics_version: self.semantics_version.clone(),
            resolver_semantics_hash: compute_hash(&self.semantics_version),
            specificity_profile_id: specificity_profile.id.clone(),
            specificity_profile_hash: specificity_profile.hash.clone(),
            specificity_profile_source: "environment".to_string(),
            specificity_profile_conflicts: vec![],
            unknown_handling_config_hash: compute_hash(
                &serde_json::to_string(unknown_handling).unwrap_or_default(),
            ),
            compatibility_lowering_version: self.compatibility_version.clone(),
            action_request_hash: compute_hash(&request.action_hash_input()),
            fact_envelopes: facts.iter().map(FactEnvelopeSummary::from).collect(),
            fact_trust_decisions: fact_trust_decisions.to_vec(),
            derived_fact_lineage: derived_lineages.to_vec(),
            candidate_policies: output.candidate_policies.clone(),
            applicable_policies: output.applicable_policies.clone(),
            incomparable_policies: output.incomparable_policies.clone(),
            unknown_decisions: output
                .evaluations
                .iter()
                .filter_map(|e| e.unknown_decision.clone())
                .collect(),
            compatibility_lowering_decisions: compatibility_decisions.to_vec(),
            conflict_resolution_steps: output
                .conflict_resolution_steps
                .iter()
                .map(ConflictStepRecord::from)
                .collect(),
            final_decision: output.final_decision,
            claim_level: ClaimLevel::AuditBacked,
            validation_suite_ref: None,
            created_at: Utc::now().to_rfc3339(),
            obligations: output.obligations.clone(),
        };

        match self.persist_trace(&trace) {
            Ok(()) => {}
            Err(_) => {
                if matches!(output.final_decision, FinalDecision::Allow) {
                    return Err(AuthorityError::trace_persistence_failure(
                        "Cannot confirm high-impact decision without trace persistence",
                    ));
                }
            }
        }

        let decision = AuthorityDecision {
            decision_id: output.decision_id.clone(),
            request_id: request.request_id.clone(),
            final_decision: output.final_decision,
            reason_code: output.reason_code.clone(),
            trace_ref: output.decision_id.clone(),
            operator_action_hint: output
                .evaluations
                .iter()
                .filter_map(|e| e.unknown_decision.as_ref())
                .filter_map(|ud| ud.operator_action_hint.clone())
                .next(),
        };

        Ok((trace, decision))
    }

    fn persist_trace(&self, trace: &AuthorityTrace) -> Result<(), AuthorityError> {
        match self.evidence_sink {
            EvidenceSink::Memory => {
                let mut store = self.memory_store.lock().map_err(|e| {
                    AuthorityError::new(
                        super::error::AuthorityErrorCode::TracePersistenceFailure,
                        format!("Mutex poisoned: {}", e),
                    )
                })?;
                store.push(trace.clone());
                Ok(())
            }
            EvidenceSink::Discard => Ok(()),
        }
    }
}
