//! Canonical semantic envelope (reference §6/§8): every normalized semantic
//! declaration in one resolved closure, its import meaning, and the
//! application contract — formatting- and comment-independent.

use crate::application::canonical::{canonical_decimal, nfc};
use crate::application::contract::*;
use crate::application::diagnostic::{ApplicationDiagnostic, ApplicationDiagnosticCode};
use crate::application::resolve::{
    Ctx, APPLICATION_CONTRACT_SCHEMA_VERSION, INTERPRETATION_VERSION, LANGUAGE_SCHEMA_VERSION,
};
use crate::concept_id::ConceptId;
use crate::module::resolver::ResolvedModuleSet;
use crate::parser::ast as past;
use crate::parser::ast_schema as schema;
use crate::semantic_pack::{canonical_json, compute_sha256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ENVELOPE_SCHEMA_VERSION: &str = "domainforge-semantic-envelope/v1";
pub const CANONICALIZATION_VERSION: &str = "domainforge-canonicalization/v1";

// ---- reference §8 envelope types ----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalSemanticEnvelopeDocument {
    pub schema_version: String,
    pub producer: ProducerIdentity,
    pub inputs: ApplicationContractInputs,
    pub self_hash: String,
    pub semantic_closure_hash: String,
    pub envelope: CanonicalSemanticEnvelope,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalSemanticEnvelope {
    pub language_schema_version: String,
    pub canonicalization_version: String,
    pub compiler_interpretation_version: String,
    pub modules: Vec<CanonicalModuleRef>,
    pub import_graph: Vec<CanonicalImportEdge>,
    pub namespace_bindings: Vec<CanonicalNamespaceBinding>,
    pub semantic_packs: Vec<CanonicalSemanticPackRef>,
    pub semantic_declarations: Vec<CanonicalSemanticDeclaration>,
    pub resolved_references: Vec<CanonicalResolvedReference>,
    pub application_contract: ApplicationContract,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalModuleRef {
    pub logical_id: String,
    pub semantic_content_hash: String,
    pub namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalImportEdge {
    pub importer: String,
    pub imported: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalNamespaceBinding {
    pub namespace: String,
    pub logical_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalSemanticPackRef {
    pub pack_id: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalSemanticDeclaration {
    pub id: CanonicalDeclarationId,
    pub logical_module_id: String,
    pub declaration: CanonicalSemanticPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CanonicalDeclarationId {
    Concept {
        id: ConceptId,
    },
    Application {
        id: ApplicationSymbolId,
    },
    Occurrence {
        kind: String,
        content_hash: String,
        duplicate_index: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CanonicalReferenceTarget {
    Declaration {
        id: CanonicalDeclarationId,
    },
    Namespace {
        exact_name: String,
        logical_module_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalResolvedReference {
    pub source: CanonicalDeclarationId,
    pub field_path: String,
    pub target: CanonicalReferenceTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CanonicalSemanticPayload {
    Dimension(CanonicalDimensionDecl),
    Unit(CanonicalUnitDecl),
    Record(RecordContract),
    Enum(EnumContract),
    Operation(OperationContract),
    Entity(CanonicalEntityDecl),
    Resource(CanonicalResourceDecl),
    Flow(CanonicalFlowDecl),
    Pattern(CanonicalPatternDecl),
    Role(CanonicalRoleDecl),
    Relation(CanonicalRelationDecl),
    Instance(CanonicalInstanceDecl),
    Policy(CanonicalPolicyDecl),
    ConceptChange(CanonicalConceptChangeDecl),
    Metric(CanonicalMetricDecl),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDimensionDecl {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalUnitDecl {
    pub symbol: String,
    pub dimension: CanonicalReferenceTarget,
    pub factor: String,
    pub base_unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalEntityDecl {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub annotations: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<EntityContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalResourceDecl {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<CanonicalReferenceTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub annotations: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalFlowDecl {
    pub resource: CanonicalReferenceTarget,
    pub from_entity: CanonicalReferenceTarget,
    pub to_entity: CanonicalReferenceTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    pub annotations: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalPatternDecl {
    pub name: String,
    pub regex: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalRoleDecl {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalRelationDecl {
    pub name: String,
    pub subject_role: CanonicalReferenceTarget,
    pub predicate: String,
    pub object_role: CanonicalReferenceTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via_flow: Option<CanonicalReferenceTarget>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalInstanceDecl {
    pub name: String,
    pub entity_type: CanonicalReferenceTarget,
    /// Field name -> serialized normalized expression (ast-v3 schema form).
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalPolicyDecl {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Serialized ast-v3 schema policy metadata.
    pub metadata: serde_json::Value,
    /// Serialized ast-v3 schema expression with every `role<...>` leaf
    /// rewritten to its resolved role `ConceptId`.
    pub expression: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalConceptChangeDecl {
    pub name: String,
    pub from_version: String,
    pub to_version: String,
    pub migration_policy: String,
    pub breaking_change: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalMetricDecl {
    pub name: String,
    pub expression: serde_json::Value,
    pub metadata: serde_json::Value,
}

// ---- construction ----

const KIND_RANK: [&str; 15] = [
    "dimension",
    "unit",
    "record",
    "enum",
    "operation",
    "entity",
    "resource",
    "flow",
    "pattern",
    "role",
    "relation",
    "instance",
    "policy",
    "concept_change",
    "metric",
];

fn kind_rank(kind: &str) -> usize {
    KIND_RANK
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(usize::MAX)
}

/// Resolve a reference through the shared symbol table (imports and aliases
/// included); an unresolved reference keeps deterministic local identity so
/// classification never invents a third form.
fn resolved_target(
    ctx: &Ctx,
    module_index: usize,
    namespace: &str,
    raw: &str,
    slugs: &'static [&'static str],
) -> CanonicalReferenceTarget {
    let id = ctx
        .resolve_concept(module_index, raw, slugs)
        .unwrap_or_else(|| ConceptId::from_concept(namespace, raw));
    CanonicalReferenceTarget::Declaration {
        id: CanonicalDeclarationId::Concept { id },
    }
}

/// Rewrite every `role<...>` leaf in a serialized policy expression to the
/// resolved role `ConceptId` (§8: envelopes carry resolved identities).
/// Unresolvable references keep their authored text; APP007 reports them at
/// binding validation.
fn rewrite_role_references(value: &mut serde_json::Value, ctx: &Ctx, module_index: usize) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(serde_json::Value::as_str) == Some("RoleReference") {
                if let Some(serde_json::Value::String(role)) = map.get_mut("role") {
                    if let Some(id) = ctx.resolve_concept(module_index, role, &["role"]) {
                        *role = id.to_string();
                    }
                }
            }
            for (_, nested) in map.iter_mut() {
                rewrite_role_references(nested, ctx, module_index);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                rewrite_role_references(item, ctx, module_index);
            }
        }
        _ => {}
    }
}

fn sorted_annotations(
    annotations: &std::collections::HashMap<String, serde_json::Value>,
) -> BTreeMap<String, serde_json::Value> {
    annotations
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn expr_value(expr: &crate::policy::Expression) -> serde_json::Value {
    serde_json::to_value(schema::Expression::from(expr)).expect("schema expression serializes")
}

/// Classify one non-export declaration. Returns `None` for realization and
/// Cell-environment declarations, which are configuration inputs excluded from
/// the envelope (§6). The match is exhaustive by design: a new AST variant
/// fails compilation until classified.
#[allow(clippy::too_many_lines)]
fn classify(
    node: &past::AstNode,
    namespace: &str,
    contract: &ApplicationContract,
    ctx: &Ctx,
    module_index: usize,
) -> Option<(
    &'static str,
    CanonicalDeclarationId,
    CanonicalSemanticPayload,
)> {
    use past::AstNode as N;
    let target = |raw: &str, slugs: &'static [&'static str]| {
        resolved_target(ctx, module_index, namespace, raw, slugs)
    };
    let concept_id = |name: &str| CanonicalDeclarationId::Concept {
        id: ConceptId::from_concept(namespace, name),
    };
    let app_id = |slug: &str, name: &str| CanonicalDeclarationId::Application {
        id: ApplicationSymbolId(format!("{namespace}.{slug}.{}", nfc(name))),
    };
    Some(match node {
        N::Export(_) => unreachable!("export wrappers are unwrapped before classification"),
        N::Dimension { name } => (
            "dimension",
            concept_id(name),
            CanonicalSemanticPayload::Dimension(CanonicalDimensionDecl { name: name.clone() }),
        ),
        N::UnitDeclaration {
            symbol,
            dimension,
            factor,
            base_unit,
        } => (
            "unit",
            concept_id(symbol),
            CanonicalSemanticPayload::Unit(CanonicalUnitDecl {
                symbol: symbol.clone(),
                dimension: target(dimension, &["dimension"]),
                factor: canonical_decimal(*factor),
                base_unit: base_unit.clone(),
            }),
        ),
        N::Record(decl) => {
            let id = app_id("record", &decl.name);
            let payload = contract
                .records
                .iter()
                .find(|r| matches!(&id, CanonicalDeclarationId::Application { id } if *id == r.id))?
                .clone();
            ("record", id, CanonicalSemanticPayload::Record(payload))
        }
        N::Enum(decl) => {
            let id = app_id("enum", &decl.name);
            let payload = contract
                .enums
                .iter()
                .find(|e| matches!(&id, CanonicalDeclarationId::Application { id } if *id == e.id))?
                .clone();
            ("enum", id, CanonicalSemanticPayload::Enum(payload))
        }
        N::Operation(decl) => {
            let id = app_id("operation", &decl.name);
            let payload = contract
                .operations
                .iter()
                .find(|o| matches!(&id, CanonicalDeclarationId::Application { id } if *id == o.id))?
                .clone();
            (
                "operation",
                id,
                CanonicalSemanticPayload::Operation(payload),
            )
        }
        N::Entity {
            name,
            version,
            annotations,
            domain,
            body,
        } => {
            let entity_concept = ConceptId::from_concept(namespace, name);
            let entity_contract = body.as_ref().and_then(|_| {
                contract
                    .entities
                    .iter()
                    .find(|e| e.concept_id == entity_concept)
                    .cloned()
            });
            (
                "entity",
                concept_id(name),
                CanonicalSemanticPayload::Entity(CanonicalEntityDecl {
                    name: name.clone(),
                    version: version.clone(),
                    domain: domain.clone(),
                    annotations: sorted_annotations(annotations),
                    contract: entity_contract,
                }),
            )
        }
        N::Resource {
            name,
            annotations,
            unit_name,
            domain,
        } => (
            "resource",
            concept_id(name),
            CanonicalSemanticPayload::Resource(CanonicalResourceDecl {
                name: name.clone(),
                unit: unit_name.as_ref().map(|u| target(u, &["unit"])),
                domain: domain.clone(),
                annotations: sorted_annotations(annotations),
            }),
        ),
        N::Flow {
            resource_name,
            annotations,
            from_entity,
            to_entity,
            quantity,
        } => {
            let payload = CanonicalSemanticPayload::Flow(CanonicalFlowDecl {
                resource: target(resource_name, &["resource"]),
                from_entity: target(from_entity, &["entity"]),
                to_entity: target(to_entity, &["entity"]),
                quantity: quantity.map(canonical_decimal),
                annotations: sorted_annotations(annotations),
            });
            // duplicate_index is assigned after grouping equal payload hashes.
            let content_hash = compute_sha256(
                canonical_json(&serde_json::to_value(&payload).expect("flow serializes"))
                    .as_bytes(),
            );
            (
                "flow",
                CanonicalDeclarationId::Occurrence {
                    kind: "flow".to_string(),
                    content_hash,
                    duplicate_index: 0,
                },
                payload,
            )
        }
        N::Pattern { name, regex } => (
            "pattern",
            concept_id(name),
            CanonicalSemanticPayload::Pattern(CanonicalPatternDecl {
                name: name.clone(),
                regex: regex.clone(),
            }),
        ),
        N::Role { name, domain } => (
            "role",
            concept_id(name),
            CanonicalSemanticPayload::Role(CanonicalRoleDecl {
                name: name.clone(),
                domain: domain.clone(),
            }),
        ),
        N::Relation {
            name,
            subject_role,
            predicate,
            object_role,
            via_flow,
        } => (
            "relation",
            concept_id(name),
            CanonicalSemanticPayload::Relation(CanonicalRelationDecl {
                name: name.clone(),
                subject_role: target(subject_role, &["role"]),
                predicate: predicate.clone(),
                object_role: target(object_role, &["role"]),
                via_flow: via_flow.as_ref().map(|f| target(f, &["flow"])),
            }),
        ),
        N::Instance {
            name,
            entity_type,
            fields,
        } => (
            "instance",
            CanonicalDeclarationId::Concept {
                // Deterministic instance identity matches Graph.
                id: ConceptId::from_concept(namespace, &format!("{entity_type}:{name}")),
            },
            CanonicalSemanticPayload::Instance(CanonicalInstanceDecl {
                name: name.clone(),
                entity_type: target(entity_type, &["entity"]),
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), expr_value(v)))
                    .collect(),
            }),
        ),
        N::Policy {
            name,
            version,
            metadata,
            expression,
        } => {
            let mut expression = expr_value(expression);
            rewrite_role_references(&mut expression, ctx, module_index);
            (
                "policy",
                concept_id(name),
                CanonicalSemanticPayload::Policy(CanonicalPolicyDecl {
                    name: name.clone(),
                    version: version.clone(),
                    metadata: serde_json::to_value(schema::PolicyMetadata::from(metadata))
                        .expect("policy metadata serializes"),
                    expression,
                }),
            )
        }
        N::ConceptChange {
            name,
            from_version,
            to_version,
            migration_policy,
            breaking_change,
        } => (
            "concept_change",
            concept_id(name),
            CanonicalSemanticPayload::ConceptChange(CanonicalConceptChangeDecl {
                name: name.clone(),
                from_version: from_version.clone(),
                to_version: to_version.clone(),
                migration_policy: migration_policy.clone(),
                breaking_change: *breaking_change,
            }),
        ),
        N::Metric {
            name,
            expression,
            metadata,
        } => (
            "metric",
            concept_id(name),
            CanonicalSemanticPayload::Metric(CanonicalMetricDecl {
                name: name.clone(),
                expression: expr_value(expression),
                metadata: serde_json::to_value(schema::MetricMetadata::from(metadata))
                    .expect("metric metadata serializes"),
            }),
        ),
        // Realization/configuration declarations: excluded from the envelope
        // per §6; their hashes belong to the later resolution/plan lock.
        N::MappingDecl { .. }
        | N::ProjectionDecl { .. }
        | N::Cell { .. }
        | N::SystemDependency { .. }
        | N::Runtime { .. }
        | N::Tool { .. }
        | N::DependencySet { .. }
        | N::Service { .. }
        | N::Mount { .. }
        | N::Endpoint { .. }
        | N::NetworkFlow { .. }
        | N::Credential { .. } => return None,
    })
}

fn id_sort_key(id: &CanonicalDeclarationId) -> String {
    canonical_json(&serde_json::to_value(id).expect("declaration id serializes"))
}

pub(crate) fn build_envelope(
    set: &ResolvedModuleSet,
    contract: &ApplicationContract,
) -> CanonicalSemanticEnvelope {
    let ctx = Ctx::new(set);
    let mut declarations: Vec<(usize, CanonicalSemanticDeclaration)> = Vec::new();
    let mut namespace_bindings = Vec::new();
    let mut module_decl_hashes: Vec<(String, String, Vec<serde_json::Value>)> = Vec::new();

    for (module_index, module) in set.modules.iter().enumerate() {
        let namespace = module
            .ast
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| module.logical_id.clone());
        namespace_bindings.push(CanonicalNamespaceBinding {
            namespace: namespace.clone(),
            logical_id: module.logical_id.clone(),
        });
        let mut module_payloads = Vec::new();
        for spanned in &module.ast.declarations {
            let node = match &spanned.node {
                past::AstNode::Export(inner) => &inner.node,
                other => other,
            };
            if let Some((kind, id, payload)) =
                classify(node, &namespace, contract, &ctx, module_index)
            {
                module_payloads.push(serde_json::to_value(&payload).expect("payload serializes"));
                declarations.push((
                    kind_rank(kind),
                    CanonicalSemanticDeclaration {
                        id,
                        logical_module_id: module.logical_id.clone(),
                        declaration: payload,
                    },
                ));
            }
        }
        module_decl_hashes.push((module.logical_id.clone(), namespace, module_payloads));
    }

    // Assign flow occurrence duplicate indexes within equal-hash groups.
    let mut occurrence_counts: BTreeMap<String, u32> = BTreeMap::new();
    for (_, decl) in &mut declarations {
        if let CanonicalDeclarationId::Occurrence {
            content_hash,
            duplicate_index,
            ..
        } = &mut decl.id
        {
            let count = occurrence_counts.entry(content_hash.clone()).or_default();
            *duplicate_index = *count;
            *count += 1;
        }
    }

    declarations.sort_by(|a, b| (a.0, id_sort_key(&a.1.id)).cmp(&(b.0, id_sort_key(&b.1.id))));
    let semantic_declarations: Vec<CanonicalSemanticDeclaration> =
        declarations.into_iter().map(|(_, d)| d).collect();

    let modules = module_decl_hashes
        .into_iter()
        .map(|(logical_id, namespace, payloads)| CanonicalModuleRef {
            logical_id,
            semantic_content_hash: compute_sha256(
                canonical_json(&serde_json::Value::Array(payloads)).as_bytes(),
            ),
            namespace,
        })
        .collect();

    let resolved_references = collect_references(&semantic_declarations);

    CanonicalSemanticEnvelope {
        language_schema_version: LANGUAGE_SCHEMA_VERSION.to_string(),
        canonicalization_version: CANONICALIZATION_VERSION.to_string(),
        compiler_interpretation_version: INTERPRETATION_VERSION.to_string(),
        modules,
        import_graph: set
            .import_graph
            .iter()
            .map(|e| CanonicalImportEdge {
                importer: e.importer.clone(),
                imported: e.imported.clone(),
            })
            .collect(),
        namespace_bindings,
        semantic_packs: Vec::new(), // Milestone 0: empty pack set
        semantic_declarations,
        resolved_references,
        application_contract: contract.clone(),
    }
}

/// Duplicate the application contract's typed reference edges for indexing
/// and audit (§8). Field paths are canonical schema paths, never source
/// coordinates.
fn collect_references(
    declarations: &[CanonicalSemanticDeclaration],
) -> Vec<CanonicalResolvedReference> {
    let mut refs = Vec::new();
    let mut push =
        |source: &CanonicalDeclarationId, path: String, target: CanonicalReferenceTarget| {
            refs.push(CanonicalResolvedReference {
                source: source.clone(),
                field_path: path,
                target,
            });
        };
    let field_refs =
        |source: &CanonicalDeclarationId,
         fields: &[FieldContract],
         refs: &mut dyn FnMut(&CanonicalDeclarationId, String, CanonicalReferenceTarget)| {
            for field in fields {
                let base = format!("fields.{}.field_type", field.name);
                match &field.field_type {
                    FieldType::Quantity { unit } => refs(
                        source,
                        format!("{base}.unit"),
                        CanonicalReferenceTarget::Declaration {
                            id: CanonicalDeclarationId::Concept { id: unit.clone() },
                        },
                    ),
                    FieldType::EntityRef { entity } => refs(
                        source,
                        format!("{base}.entity"),
                        CanonicalReferenceTarget::Declaration {
                            id: CanonicalDeclarationId::Concept { id: entity.clone() },
                        },
                    ),
                    FieldType::Enum { symbol } => refs(
                        source,
                        format!("{base}.symbol"),
                        CanonicalReferenceTarget::Declaration {
                            id: CanonicalDeclarationId::Application { id: symbol.clone() },
                        },
                    ),
                    FieldType::Scalar { .. } | FieldType::List { .. } => {}
                }
                for (index, constraint) in field.constraints.iter().enumerate() {
                    if let FieldConstraint::Pattern { pattern } = constraint {
                        refs(
                            source,
                            format!("fields.{}.constraints.{index}.pattern", field.name),
                            CanonicalReferenceTarget::Declaration {
                                id: CanonicalDeclarationId::Concept {
                                    id: pattern.clone(),
                                },
                            },
                        );
                    }
                }
            }
        };
    for decl in declarations {
        match &decl.declaration {
            CanonicalSemanticPayload::Record(record) => {
                field_refs(&decl.id, &record.fields, &mut push)
            }
            CanonicalSemanticPayload::Entity(CanonicalEntityDecl {
                contract: Some(entity),
                ..
            }) => field_refs(&decl.id, &entity.fields, &mut push),
            CanonicalSemanticPayload::Operation(op) => {
                push(
                    &decl.id,
                    "input".to_string(),
                    CanonicalReferenceTarget::Declaration {
                        id: CanonicalDeclarationId::Application {
                            id: op.input.clone(),
                        },
                    },
                );
                push(
                    &decl.id,
                    "output".to_string(),
                    CanonicalReferenceTarget::Declaration {
                        id: CanonicalDeclarationId::Application {
                            id: op.output.clone(),
                        },
                    },
                );
                push(
                    &decl.id,
                    "state".to_string(),
                    CanonicalReferenceTarget::Declaration {
                        id: CanonicalDeclarationId::Concept {
                            id: op.state.clone(),
                        },
                    },
                );
                if let ActorRef::Role { role } = &op.actor {
                    push(
                        &decl.id,
                        "actor.role".to_string(),
                        CanonicalReferenceTarget::Declaration {
                            id: CanonicalDeclarationId::Concept { id: role.clone() },
                        },
                    );
                }
                if let AccessMode::PolicyGoverned { bindings } = &op.access {
                    for (index, binding) in bindings.iter().enumerate() {
                        push(
                            &decl.id,
                            format!("access.bindings.{index}.policy"),
                            CanonicalReferenceTarget::Declaration {
                                id: CanonicalDeclarationId::Concept {
                                    id: binding.policy.clone(),
                                },
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }
    refs
}

/// Resolve an in-memory source map into a validated canonical semantic
/// envelope document, or the complete deterministic diagnostic list.
pub fn resolve_semantic_envelope(
    entry_logical_path: &str,
    sources_json: &str,
) -> Result<CanonicalSemanticEnvelopeDocument, Vec<ApplicationDiagnostic>> {
    use crate::module::resolver::{resolve_source_map, SourceMap};
    let sources = SourceMap::parse_json(sources_json)?;
    let resolved = resolve_source_map(entry_logical_path, &sources)?;
    let contract = crate::application::resolve::build_contract(&resolved)?;
    let envelope = build_envelope(&resolved, &contract);
    let closure_hash = semantic_closure_hash(&envelope);
    let mut doc = CanonicalSemanticEnvelopeDocument {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
        producer: ProducerIdentity {
            name: "domainforge-core".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        inputs: ApplicationContractInputs {
            source_set_hash: crate::application::canonical::source_set_hash(
                sources
                    .0
                    .iter()
                    .map(|(id, src)| (id.as_str(), src.as_str())),
            )
            .expect("duplicates already rejected"),
            semantic_pack_set_hash: crate::application::canonical::semantic_pack_set_hash(&[])
                .expect("empty set has no duplicates"),
            language_schema_version: LANGUAGE_SCHEMA_VERSION.to_string(),
            interpretation_version: INTERPRETATION_VERSION.to_string(),
        },
        self_hash: format!("sha256:{}", "0".repeat(64)),
        semantic_closure_hash: closure_hash,
        envelope,
    };
    doc.self_hash = envelope_document_self_hash(&doc);
    Ok(doc)
}

// ---- hashing and validation ----

/// Reference §6 semantic-closure hash over the canonical envelope payload.
pub fn semantic_closure_hash(envelope: &CanonicalSemanticEnvelope) -> String {
    compute_sha256(
        canonical_json(&serde_json::to_value(envelope).expect("envelope serializes")).as_bytes(),
    )
}

pub fn envelope_document_self_hash(doc: &CanonicalSemanticEnvelopeDocument) -> String {
    let mut value = serde_json::to_value(doc).expect("envelope document serializes");
    value
        .as_object_mut()
        .expect("document is an object")
        .remove("self_hash");
    compute_sha256(canonical_json(&value).as_bytes())
}

fn is_hash(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    })
}

fn app015(document_kind: &str, field_path: &str, message: String) -> ApplicationDiagnostic {
    let mut d = ApplicationDiagnostic::new(ApplicationDiagnosticCode::App015, message);
    d.context.document_kind = Some(document_kind.to_string());
    d.context.field_path = Some(field_path.to_string());
    d
}

/// Validate schema, inputs, and self-hash of a persisted application contract
/// document before use (APP015 on violation).
pub fn validate_application_contract_document_json(
    json: &str,
) -> Result<ApplicationContractDocument, Vec<ApplicationDiagnostic>> {
    let kind = "application_contract";
    let doc: ApplicationContractDocument = serde_json::from_str(json)
        .map_err(|e| vec![app015(kind, "", format!("document fails its schema: {e}"))])?;
    let mut diags = Vec::new();
    if doc.schema_version != APPLICATION_CONTRACT_SCHEMA_VERSION {
        let mut d = app015(
            kind,
            "/schema_version",
            format!("unexpected schema_version '{}'", doc.schema_version),
        );
        d.context.schema_version = Some(doc.schema_version.clone());
        diags.push(d);
    }
    for (path, value) in [
        ("/self_hash", &doc.self_hash),
        ("/semantic_closure_hash", &doc.semantic_closure_hash),
        ("/inputs/source_set_hash", &doc.inputs.source_set_hash),
        (
            "/inputs/semantic_pack_set_hash",
            &doc.inputs.semantic_pack_set_hash,
        ),
    ] {
        if !is_hash(value) {
            diags.push(app015(
                kind,
                path,
                format!("'{value}' is not a lowercase sha256:<hex> hash"),
            ));
        }
    }
    if diags.is_empty() && doc.self_hash != crate::application::canonical::document_self_hash(&doc)
    {
        diags.push(app015(
            kind,
            "/self_hash",
            "self_hash does not match the canonical document bytes".to_string(),
        ));
    }
    if diags.is_empty() {
        Ok(doc)
    } else {
        Err(diags)
    }
}

/// Validate schema, inputs, and hashes of a persisted canonical semantic
/// envelope document before use (APP015 on violation).
pub fn validate_semantic_envelope_document_json(
    json: &str,
) -> Result<CanonicalSemanticEnvelopeDocument, Vec<ApplicationDiagnostic>> {
    let kind = "canonical_semantic_envelope";
    let doc: CanonicalSemanticEnvelopeDocument = serde_json::from_str(json)
        .map_err(|e| vec![app015(kind, "", format!("document fails its schema: {e}"))])?;
    let mut diags = Vec::new();
    if doc.schema_version != ENVELOPE_SCHEMA_VERSION {
        let mut d = app015(
            kind,
            "/schema_version",
            format!("unexpected schema_version '{}'", doc.schema_version),
        );
        d.context.schema_version = Some(doc.schema_version.clone());
        diags.push(d);
    }
    for (path, value) in [
        ("/self_hash", &doc.self_hash),
        ("/semantic_closure_hash", &doc.semantic_closure_hash),
    ] {
        if !is_hash(value) {
            diags.push(app015(
                kind,
                path,
                format!("'{value}' is not a lowercase sha256:<hex> hash"),
            ));
        }
    }
    if diags.is_empty() {
        if doc.semantic_closure_hash != semantic_closure_hash(&doc.envelope) {
            diags.push(app015(
                kind,
                "/semantic_closure_hash",
                "semantic_closure_hash does not match the canonical envelope bytes".to_string(),
            ));
        }
        if doc.self_hash != envelope_document_self_hash(&doc) {
            diags.push(app015(
                kind,
                "/self_hash",
                "self_hash does not match the canonical document bytes".to_string(),
            ));
        }
    }
    if diags.is_empty() {
        Ok(doc)
    } else {
        Err(diags)
    }
}
