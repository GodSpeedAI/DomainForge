//! BAML capability IR: the intermediate representation the renderer lowers to
//! `.baml` source. Built entirely from an [`AiLearningContext`] so BAML reuses
//! the same resolver-grounded authority cases and policy information the
//! ai-learning datasets are built from — the examples/tests are the resolver's
//! own decisions, never re-derived here.
//!
//! Every id is minted through [`crate::projection::ids`] so identity is uniform
//! across projection families and stable run-to-run. All collections are built
//! in sorted order (BTreeMap/BTreeSet only) so the rendered output is
//! byte-deterministic for a fixed `created_at`.

use crate::projection::ai_learning::AiLearningContext;
use crate::projection::ids::{element_id, sanitize_qname};
use std::collections::{BTreeMap, BTreeSet};

/// Family tag for [`element_id`]: keeps BAML ids disjoint from every other
/// projection family's ids for the same part tuple.
pub const BAML_FAMILY: &str = "baml";

/// Pinned BAML language target. The generated `.baml` is written against this
/// syntax revision. Bump procedure: change this constant, regenerate the
/// fixture, and confirm the `verify-baml` CI job (structural well-formedness
/// check) stays green. See `docs/baml-projections.md` for what is and is not
/// live-validated.
pub const BAML_TARGET_VERSION: &str = "0.68";

/// One enum variant: the BAML identifier plus the raw model string it came from
/// (kept for doc comments and so the raw name appears verbatim in the source).
#[derive(Debug, Clone)]
pub struct Variant {
    pub ident: String,
    pub raw: String,
}

/// A BAML `enum` lowered from a closed model domain.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub doc: String,
    pub variants: Vec<Variant>,
}

/// One class field: `name type` plus an optional doc line.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_ref: String,
    pub doc: String,
}

/// A BAML `class` (a structured input or output schema).
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub doc: String,
    pub fields: Vec<FieldDef>,
}

/// One BAML `test` block seeded from a resolver-grounded authority case.
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Deterministic BAML test identifier (valid ident, starts with a letter).
    pub name: String,
    pub role: String,
    pub operation: String,
    pub resource_type: String,
    /// Resolver decision label: "allow" | "deny" | "escalate" | "reject".
    pub decision: String,
    pub reason_code: String,
    pub applicable_policies: Vec<String>,
    /// Deterministic element id for the (role, operation, resource) tuple.
    pub id: String,
}

/// The whole BAML capability: type defs, one policy-decision function, its
/// tests, and the placeholder client — everything a renderer needs.
#[derive(Debug, Clone)]
pub struct AICapabilityIR {
    pub model_ref: String,
    pub model_hash: String,
    pub created_at: String,
    pub generator_version: String,
    /// Deterministic id for the capability (function) itself.
    pub capability_id: String,
    pub function_name: String,
    pub client_name: String,
    pub entity_enum: EnumDef,
    pub role_enum: EnumDef,
    pub resource_enum: EnumDef,
    pub operation_enum: EnumDef,
    pub decision_enum: EnumDef,
    pub request_class: ClassDef,
    pub decision_class: ClassDef,
    /// One human-readable line per authority policy, embedded in the prompt so
    /// the function carries the policy semantics it must respect.
    pub policy_lines: Vec<String>,
    pub tests: Vec<TestCase>,
}

/// Build a distinct-variant enum from an iterator of raw model names, sorted and
/// de-duplicated on the sanitized identifier so the output is deterministic and
/// no two variants collide.
fn build_enum(name: &str, doc: &str, raws: impl IntoIterator<Item = String>) -> EnumDef {
    let mut by_ident: BTreeMap<String, String> = BTreeMap::new();
    for raw in raws {
        let ident = sanitize_qname(&raw);
        // First raw wins for a given sanitized ident (deterministic: inputs are
        // pre-sorted by the caller / BTree keeps ident order).
        by_ident.entry(ident).or_insert(raw);
    }
    let variants = by_ident
        .into_iter()
        .map(|(ident, raw)| Variant { ident, raw })
        .collect();
    EnumDef {
        name: name.to_string(),
        doc: doc.to_string(),
        variants,
    }
}

impl AICapabilityIR {
    /// Lower an [`AiLearningContext`] into the BAML capability IR. Requires a
    /// resolver-grounded authority environment: without it there are no
    /// policy-decision capabilities to represent.
    pub fn build(ctx: &AiLearningContext) -> Result<Self, String> {
        ctx.require_authority("baml")?;
        let cfg = &ctx.recipe.projections.baml;

        // Closed domains, sorted for determinism.
        let mut entity_names: Vec<String> = ctx
            .graph
            .all_entities()
            .into_iter()
            .map(|e| e.name().to_string())
            .collect();
        entity_names.sort();
        let mut role_names: Vec<String> = ctx
            .graph
            .all_roles()
            .into_iter()
            .map(|r| r.name().to_string())
            .collect();
        role_names.sort();
        let mut resource_names: Vec<String> = ctx
            .graph
            .all_resources()
            .into_iter()
            .map(|r| r.name().to_string())
            .collect();
        resource_names.sort();

        // Operations come from the authority pack action vocabulary, surfaced
        // via the resolver-enumerated cases (single source of truth).
        let operations: BTreeSet<String> = ctx
            .authority_cases
            .iter()
            .map(|c| c.operation.clone())
            .collect();
        if operations.is_empty() {
            return Err(
                "baml projection requires at least one authority policy action; the \
                 provided authority environment declares none"
                    .to_string(),
            );
        }
        if role_names.is_empty() || resource_names.is_empty() {
            return Err(
                "baml projection requires the model to declare at least one role and one \
                 resource (the authority request shape references both)"
                    .to_string(),
            );
        }

        let entity_enum = build_enum(
            "DomainEntity",
            "Closed domain of entities declared in the model.",
            entity_names,
        );
        let role_enum = build_enum(
            "ActorRole",
            "Closed domain of actor roles declared in the model.",
            role_names,
        );
        let resource_enum = build_enum(
            "ResourceType",
            "Closed domain of resource types declared in the model.",
            resource_names,
        );
        let operation_enum = build_enum(
            "Operation",
            "Operations governed by the authority pack (its action vocabulary).",
            operations.into_iter().collect::<Vec<_>>(),
        );
        // Fixed output vocabulary: the resolver's five decision outcomes.
        let decision_enum = EnumDef {
            name: "AuthorityDecision".to_string(),
            doc: "The authorization outcome, matching the DomainForge resolver's decision set."
                .to_string(),
            variants: ["Allow", "Deny", "Escalate", "Unknown", "Reject"]
                .iter()
                .map(|v| Variant {
                    ident: v.to_string(),
                    raw: v.to_lowercase(),
                })
                .collect(),
        };

        let request_class = ClassDef {
            name: "AuthorityRequest".to_string(),
            doc: "The authorization question: which actor role wants to perform which \
                  operation on which resource type."
                .to_string(),
            fields: vec![
                FieldDef {
                    name: "actor_role".to_string(),
                    type_ref: role_enum.name.clone(),
                    doc: "The role the requesting actor holds.".to_string(),
                },
                FieldDef {
                    name: "operation".to_string(),
                    type_ref: operation_enum.name.clone(),
                    doc: "The operation being attempted.".to_string(),
                },
                FieldDef {
                    name: "resource_type".to_string(),
                    type_ref: resource_enum.name.clone(),
                    doc: "The type of resource the operation targets.".to_string(),
                },
            ],
        };
        let decision_class = ClassDef {
            name: "AuthorityRuling".to_string(),
            doc: "The decision plus its rationale and the policy ids that justify it.".to_string(),
            fields: vec![
                FieldDef {
                    name: "decision".to_string(),
                    type_ref: decision_enum.name.clone(),
                    doc: "The authorization outcome.".to_string(),
                },
                FieldDef {
                    name: "rationale".to_string(),
                    type_ref: "string".to_string(),
                    doc: "A short justification citing the governing policy.".to_string(),
                },
                FieldDef {
                    name: "policy_refs".to_string(),
                    type_ref: "string[]".to_string(),
                    doc: "Ids of the authority policies the decision relies on.".to_string(),
                },
            ],
        };

        // Policy semantics for the prompt, deterministic via the BTreeMap.
        let policy_lines = ctx
            .policy_info
            .values()
            .map(|p| {
                let scope = [
                    p.actor_role.as_deref().map(|r| format!("role={r}")),
                    p.action.as_deref().map(|a| format!("action={a}")),
                    p.resource_type.as_deref().map(|r| format!("resource={r}")),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
                let desc = p.description.as_deref().unwrap_or("");
                format!(
                    "{} [{}] (priority {}) applies to {{ {} }}{}",
                    p.policy_id,
                    p.modality,
                    p.priority,
                    scope,
                    if desc.is_empty() {
                        String::new()
                    } else {
                        format!(": {desc}")
                    }
                )
            })
            .collect();

        // Tests seeded from the resolver's decisions. "unknown" cases (no
        // applicable policy) are excluded by default: they add many rows with
        // no governing policy and drown the meaningful examples. Sorted for a
        // stable, readable order.
        let mut cases: Vec<&crate::projection::ai_learning::AuthorityCase> = ctx
            .authority_cases
            .iter()
            .filter(|c| cfg.include_unknown || c.label != "unknown")
            .collect();
        cases.sort_by(|a, b| {
            (a.label, &a.role, &a.operation, &a.resource_type).cmp(&(
                b.label,
                &b.role,
                &b.operation,
                &b.resource_type,
            ))
        });
        let tests = cases
            .into_iter()
            .map(|c| {
                let id = element_id(
                    BAML_FAMILY,
                    &["test", &c.role, &c.operation, &c.resource_type],
                );
                let name = format!(
                    "t_{}_{}_{}_{}_{}",
                    c.label,
                    sanitize_qname(&c.role),
                    sanitize_qname(&c.operation),
                    sanitize_qname(&c.resource_type),
                    &id[..8],
                );
                TestCase {
                    name,
                    role: c.role.clone(),
                    operation: c.operation.clone(),
                    resource_type: c.resource_type.clone(),
                    decision: c.label.to_string(),
                    reason_code: c.reason_code.clone(),
                    applicable_policies: c.applicable_policies.clone(),
                    id,
                }
            })
            .collect();

        let function_name = cfg.function_name.clone();
        let capability_id = element_id(BAML_FAMILY, &["capability", &function_name]);

        Ok(Self {
            model_ref: ctx.model_ref.clone(),
            model_hash: ctx.model_hash.clone(),
            created_at: ctx.created_at.clone(),
            generator_version: crate::projection::ai_learning::GENERATOR_VERSION.to_string(),
            capability_id,
            function_name,
            client_name: "DomainForgeAuthorityClient".to_string(),
            entity_enum,
            role_enum,
            resource_enum,
            operation_enum,
            decision_enum,
            request_class,
            decision_class,
            policy_lines,
            tests,
        })
    }
}
