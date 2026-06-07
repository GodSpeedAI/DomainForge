use super::error::AuthorityError;
use super::pack::AuthorityPack;
use super::policy::*;
use super::types::*;

#[derive(Debug, Clone)]
pub struct PolicyEvaluation {
    pub policy: AuthorityPolicy,
    pub condition_result: ThreeValuedResult,
    pub fact_check_results: Vec<(FactRequirement, bool, Option<FactEnvelope>)>,
    pub unknown_handling_applied: bool,
    pub unknown_handling_result: Option<FinalDecision>,
    pub unknown_decision: Option<UnknownDecision>,
}

pub struct AuthorityResolver {
    unknown_handling: UnknownHandlingConfig,
    specificity_profile: SpecificityProfile,
    strict_mode: bool,
    semantics_version: String,
    compatibility_version: String,
}

impl AuthorityResolver {
    pub fn new(
        unknown_handling: UnknownHandlingConfig,
        specificity_profile: SpecificityProfile,
        strict_mode: bool,
        semantics_version: String,
        compatibility_version: String,
    ) -> Self {
        Self {
            unknown_handling,
            specificity_profile,
            strict_mode,
            semantics_version,
            compatibility_version,
        }
    }

    pub fn resolve(
        &self,
        request: &AuthorityRequest,
        packs: &[AuthorityPack],
        facts: &[FactEnvelope],
        fact_trust_decisions: &[FactTrustDecision],
        _derived_lineages: &[DerivedFactLineage],
    ) -> Result<ResolverOutput, AuthorityError> {
        request.validate()?;

        let all_policies: Vec<&AuthorityPolicy> = packs.iter().flat_map(|p| p.policies.iter()).collect();

        let candidates: Vec<&AuthorityPolicy> = all_policies
            .into_iter()
            .filter(|p| p.is_candidate(request))
            .collect();

        if candidates.is_empty() {
            return Ok(ResolverOutput {
                decision_id: generate_decision_id(),
                final_decision: FinalDecision::NotApplicable,
                reason_code: "no_applicable_policy".to_string(),
                candidate_policies: vec![],
                applicable_policies: vec![],
                incomparable_policies: vec![],
                evaluations: vec![],
                conflict_resolution_steps: vec![],
            });
        }

        let mut evaluations = Vec::new();
        for policy in &candidates {
            let fact_checks = policy.check_fact_requirements(facts, &chrono::Utc::now());
            let all_facts_satisfied = fact_checks.iter().all(|(_, satisfied, _)| *satisfied);

            let condition_result = if !all_facts_satisfied {
                ThreeValuedResult::Unknown
            } else {
                policy.evaluate_conditions(facts)?
            };

            let unknown_handling_applied = condition_result == ThreeValuedResult::Unknown;
            let (unknown_handling_result, unknown_decision) = if unknown_handling_applied {
                let default = self.unknown_handling.for_modality(&policy.modality);
                let unsatisfied_facts: Vec<String> = fact_checks
                    .iter()
                    .filter(|(_, s, _)| !s)
                    .map(|(r, _, _)| r.fact_path.clone())
                    .collect();
                let ud = UnknownDecision {
                    policy_id: policy.policy_id.clone(),
                    unknown_handling_applied: true,
                    unknown_modality: policy.modality.clone(),
                    unknown_default_result: default,
                    unknown_reason: classify_unknown_reason(&fact_checks, fact_trust_decisions),
                    affected_fact_paths: unsatisfied_facts,
                    fact_source_ids: vec![],
                    availability_classification: classify_availability(&fact_checks),
                    operator_action_hint: None,
                };
                (Some(default), Some(ud))
            } else {
                (None, None)
            };

            evaluations.push(PolicyEvaluation {
                policy: (*policy).clone(),
                condition_result,
                fact_check_results: fact_checks,
                unknown_handling_applied,
                unknown_handling_result,
                unknown_decision,
            });
        }

        let applicable: Vec<&PolicyEvaluation> = evaluations
            .iter()
            .filter(|e| {
                matches!(
                    e.condition_result,
                    ThreeValuedResult::True
                ) || e.unknown_handling_applied
            })
            .collect();

        if applicable.is_empty() {
            return Ok(ResolverOutput {
                decision_id: generate_decision_id(),
                final_decision: FinalDecision::NotApplicable,
                reason_code: "no_applicable_policy_after_evaluation".to_string(),
                candidate_policies: candidates.into_iter().map(|p| p.policy_id.clone()).collect(),
                applicable_policies: vec![],
                incomparable_policies: vec![],
                evaluations,
                conflict_resolution_steps: vec![],
            });
        }

        let (final_decision, conflict_steps) =
            self.resolve_conflicts(&applicable)?;

        let decision_id = generate_decision_id();
        let reason_code = format!(
            "{}_by_{}",
            match final_decision {
                FinalDecision::Allow => "allowed",
                FinalDecision::Deny => "denied",
                FinalDecision::Escalate => "escalated",
                FinalDecision::NotApplicable => "not_applicable",
                FinalDecision::Reject => "rejected",
            },
            conflict_steps
                .last()
                .map(|s| s.policy_id.clone())
                .unwrap_or_else(|| "resolver".to_string())
        );

        Ok(ResolverOutput {
            decision_id,
            final_decision,
            reason_code,
            candidate_policies: candidates.into_iter().map(|p| p.policy_id.clone()).collect(),
            applicable_policies: applicable.iter().map(|e| e.policy.policy_id.clone()).collect(),
            incomparable_policies: vec![],
            evaluations,
            conflict_resolution_steps: conflict_steps,
        })
    }

    fn resolve_conflicts(
        &self,
        applicable: &[&PolicyEvaluation],
    ) -> Result<(FinalDecision, Vec<ConflictResolutionStep>), AuthorityError> {
        if applicable.len() == 1 {
            let ev = applicable[0];
            let decision = if ev.unknown_handling_applied {
                ev.unknown_handling_result.unwrap_or(FinalDecision::NotApplicable)
            } else {
                modality_to_decision(&ev.policy.modality, ev.condition_result)
            };
            return Ok((
                decision,
                vec![ConflictResolutionStep {
                    step: "single_applicable".to_string(),
                    policy_id: ev.policy.policy_id.clone(),
                    result: format!("{:?}", decision),
                }],
            ));
        }

        // Step 6: Apply modality precedence (spec §10.8)
        // Default: Reject > Deny > Escalate > Allow > NotApplicable
        let mut best_modality_rank = 0u8;
        for ev in applicable {
            let rank = modality_precedence_rank(&ev.policy.modality);
            best_modality_rank = best_modality_rank.max(rank);
        }

        let modal_filtered: Vec<&&PolicyEvaluation> = applicable
            .iter()
            .filter(|e| modality_precedence_rank(&e.policy.modality) == best_modality_rank)
            .collect();

        let mut steps = vec![ConflictResolutionStep {
            step: "modality_precedence".to_string(),
            policy_id: modal_filtered.iter().map(|e| e.policy.policy_id.clone()).collect::<Vec<_>>().join(","),
            result: format!("modality_rank={}", best_modality_rank),
        }];

        if modal_filtered.len() == 1 {
            let ev = *modal_filtered[0];
            let decision = compute_eval_decision(ev);
            steps.push(ConflictResolutionStep {
                step: "modality_resolved".to_string(),
                policy_id: ev.policy.policy_id.clone(),
                result: format!("{:?}", decision),
            });
            return Ok((decision, steps));
        }

        // Step 7: Apply priority
        let mut best_priority = i32::MIN;
        for ev in &modal_filtered {
            best_priority = best_priority.max(ev.policy.priority);
        }

        let top_priority: Vec<&PolicyEvaluation> = modal_filtered
            .iter()
            .filter(|e| e.policy.priority == best_priority)
            .map(|e| **e)
            .collect();

        steps.push(ConflictResolutionStep {
            step: "priority_filter".to_string(),
            policy_id: top_priority.iter().map(|e| e.policy.policy_id.clone()).collect::<Vec<_>>().join(","),
            result: format!("priority={}", best_priority),
        });

        if top_priority.len() == 1 {
            let ev = top_priority[0];
            let decision = compute_eval_decision(ev);
            steps.push(ConflictResolutionStep {
                step: "highest_priority".to_string(),
                policy_id: ev.policy.policy_id.clone(),
                result: format!("{:?}", decision),
            });
            return Ok((decision, steps));
        }

        let mut dominated = vec![false; top_priority.len()];
        for i in 0..top_priority.len() {
            if dominated[i] {
                continue;
            }
            for j in 0..top_priority.len() {
                if i == j || dominated[j] {
                    continue;
                }
                let vec_i = top_priority[i].policy.compute_specificity(&self.specificity_profile);
                let vec_j = top_priority[j].policy.compute_specificity(&self.specificity_profile);
                match vec_i.compare(&vec_j) {
                    SpecificityComparison::AMoreSpecific => {
                        dominated[j] = true;
                    }
                    SpecificityComparison::BMoreSpecific => {
                        dominated[i] = true;
                    }
                    SpecificityComparison::Equal | SpecificityComparison::Incomparable => {}
                }
            }
        }

        let non_dominated: Vec<&PolicyEvaluation> = top_priority
            .iter()
            .enumerate()
            .filter(|(i, _)| !dominated[*i])
            .map(|(_, e)| *e)
            .collect();

        if non_dominated.len() == 1 {
            let ev = non_dominated[0];
            let decision = compute_eval_decision(ev);
            steps.push(ConflictResolutionStep {
                step: "specificity_resolved".to_string(),
                policy_id: ev.policy.policy_id.clone(),
                result: format!("{:?}", decision),
            });
            return Ok((decision, steps));
        }

        steps.push(ConflictResolutionStep {
            step: "specificity_incomparable".to_string(),
            policy_id: non_dominated.iter().map(|e| e.policy.policy_id.clone()).collect::<Vec<_>>().join(","),
            result: "incomparable".to_string(),
        });

        if self.strict_mode {
            let ids: Vec<String> = non_dominated.iter().map(|e| e.policy.policy_id.clone()).collect();
            Err(AuthorityError::specificity_conflict(&ids[0], &ids.get(1).map(|s| s.as_str()).unwrap_or("?")))
        } else {
            Ok((FinalDecision::Escalate, steps))
        }
    }

    pub fn validate_versions(
        &self,
        semantics_version: &str,
        compatibility_version: &str,
    ) -> Result<(), AuthorityError> {
        if self.semantics_version != semantics_version {
            return Err(AuthorityError::invalid_environment(format!(
                "Resolver semantics version '{}' does not match configured semantics version '{}'",
                self.semantics_version, semantics_version
            )));
        }
        if self.compatibility_version != compatibility_version {
            return Err(AuthorityError::invalid_environment(format!(
                "Resolver compatibility version '{}' does not match configured compatibility lowering version '{}'",
                self.compatibility_version, compatibility_version
            )));
        }
        Ok(())
    }
}

fn modality_to_decision(modality: &PolicyModality, condition: ThreeValuedResult) -> FinalDecision {
    match (modality, condition) {
        (PolicyModality::Permission, ThreeValuedResult::True) => FinalDecision::Allow,
        (PolicyModality::Prohibition, ThreeValuedResult::True) => FinalDecision::Deny,
        (PolicyModality::Obligation, ThreeValuedResult::True) => FinalDecision::Allow,
        (PolicyModality::Override, ThreeValuedResult::True) => FinalDecision::Allow,
        _ => FinalDecision::NotApplicable,
    }
}

fn modality_precedence_rank(modality: &PolicyModality) -> u8 {
    match modality {
        PolicyModality::Override => 5,
        PolicyModality::Prohibition => 4,
        PolicyModality::Obligation => 3,
        PolicyModality::Permission => 2,
    }
}

fn compute_eval_decision(ev: &PolicyEvaluation) -> FinalDecision {
    if ev.unknown_handling_applied {
        ev.unknown_handling_result.unwrap_or(FinalDecision::NotApplicable)
    } else {
        modality_to_decision(&ev.policy.modality, ev.condition_result)
    }
}

fn classify_unknown_reason(
    fact_checks: &[(FactRequirement, bool, Option<FactEnvelope>)],
    trust_decisions: &[FactTrustDecision],
) -> String {
    for (req, satisfied, envelope) in fact_checks {
        if !satisfied {
            if envelope.is_none() {
                return "missing".to_string();
            }
            if let Some(env) = envelope {
                if env.is_caller_supplied() {
                    return "caller_omission".to_string();
                }
            }
            for td in trust_decisions {
                if td.fact_path == req.fact_path && !td.trusted {
                    return td.reason.clone();
                }
            }
        }
    }
    "unknown".to_string()
}

fn classify_availability(fact_checks: &[(FactRequirement, bool, Option<FactEnvelope>)]) -> String {
    for (_req, satisfied, envelope) in fact_checks {
        if !satisfied {
            if envelope.is_none() {
                return "caller_omission".to_string();
            }
            if let Some(env) = envelope {
                if env.source_class == SourceClass::CallerSupplied {
                    return "caller_omission".to_string();
                }
            }
        }
    }
    "availability_failure".to_string()
}

#[derive(Debug, Clone)]
pub struct ConflictResolutionStep {
    pub step: String,
    pub policy_id: String,
    pub result: String,
}

#[derive(Debug, Clone)]
pub struct ResolverOutput {
    pub decision_id: String,
    pub final_decision: FinalDecision,
    pub reason_code: String,
    pub candidate_policies: Vec<String>,
    pub applicable_policies: Vec<String>,
    pub incomparable_policies: Vec<String>,
    pub evaluations: Vec<PolicyEvaluation>,
    pub conflict_resolution_steps: Vec<ConflictResolutionStep>,
}
