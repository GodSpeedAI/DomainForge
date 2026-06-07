use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 4.1 SemanticPack
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticPack {
    pub schema_version: String,
    pub pack_id: String,
    pub org_id: String,
    pub domain_id: String,
    pub pack_version: String,
    pub meaning_version: String,
    pub meaning_fingerprint: String,
    pub source_graph_hash: String,
    pub build_config_hash: String,
    pub review_manifest_hash: String,
    pub created_at: String,
    pub generator: GeneratorInfo,
    pub trust: PackTrust,
    pub concepts: Vec<ConceptDef>,
    pub relations: Vec<RelationDef>,
    pub metrics: Vec<MetricDef>,
    pub dimensions: Vec<DimensionDef>,
    pub units: Vec<UnitDef>,
    pub aliases: Vec<AliasDef>,
    pub mapping_rules: Vec<MappingRuleDef>,
    pub compatibility: CompatibilityInfo,
}

// ---------------------------------------------------------------------------
// 4.2 PackTrust
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackTrust {
    pub approval_state: ApprovalState,
    pub signature_state: SignatureState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_alg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Candidate,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureState {
    Unsigned,
    Signed,
    InvalidSignature,
}

// ---------------------------------------------------------------------------
// GeneratorInfo
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratorInfo {
    pub name: String,
    pub version: String,
}

// ---------------------------------------------------------------------------
// 4.3 ConceptDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptDef {
    pub id: String,
    pub canonical_name: String,
    pub kind: ConceptKind,
    pub status: ConceptStatus,
    pub definition: ConceptDefinition,
    pub owner: String,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub counterexamples: Vec<String>,
    #[serde(default)]
    pub allowed_predicates: Vec<String>,
    #[serde(default)]
    pub valid_contexts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptDefinition {
    pub text: String,
    pub definition_hash: String,
    pub decision_ref: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConceptKind {
    Entity,
    Resource,
    Role,
    Flow,
    Policy,
    Metric,
    Dimension,
    Unit,
    External,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConceptStatus {
    Active,
    Proposed,
    Deprecated,
    Rejected,
    ExternalOnly,
}

// ---------------------------------------------------------------------------
// 4.4 AliasDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasDef {
    pub alias: String,
    pub normalized_alias: String,
    pub target_concept_id: String,
    pub status: AliasStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    pub decision_ref: String,
    pub source_ref: SourceRef,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AliasStatus {
    Approved,
    Deprecated,
    Ambiguous,
    Blocked,
}

// ---------------------------------------------------------------------------
// RelationDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationDef {
    pub id: String,
    pub predicate: String,
    pub subject_id: String,
    pub object_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
}

// ---------------------------------------------------------------------------
// MetricDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetricDef {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_id: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
}

// ---------------------------------------------------------------------------
// DimensionDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DimensionDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
}

// ---------------------------------------------------------------------------
// UnitDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnitDef {
    pub id: String,
    pub symbol: String,
    pub dimension_id: String,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
}

// ---------------------------------------------------------------------------
// MappingRuleDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MappingRuleDef {
    pub id: String,
    pub source_term: String,
    pub target_concept_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    pub decision_ref: String,
}

// ---------------------------------------------------------------------------
// CompatibilityInfo (15.2)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityInfo {
    #[serde(default)]
    pub domainforge_min_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domainforge_max_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp_min_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retired_after: Option<String>,
    #[serde(default)]
    pub replaces_pack_ids: Vec<String>,
}

impl Default for CompatibilityInfo {
    fn default() -> Self {
        Self {
            domainforge_min_version: "0.3".to_string(),
            domainforge_max_version: None,
            lsp_min_version: None,
            deprecated_after: None,
            retired_after: None,
            replaces_pack_ids: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// SourceRef (14)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub uri: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl SourceRef {
    pub fn synthetic(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
            start_byte: 0,
            end_byte: 0,
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }

    pub fn workspace_root() -> Self {
        Self::synthetic("domainforge://workspace-root")
    }

    pub fn pack_uri(pack_id: &str) -> Self {
        Self::synthetic(&format!("domainforge://semantic-pack/{}", pack_id))
    }
}

// ---------------------------------------------------------------------------
// PackRef (10.1)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackRef {
    pub pack_id: String,
    pub pack_content_hash: String,
    pub path_or_uri: String,
    pub priority: i32,
}

// ---------------------------------------------------------------------------
// Review records (5.2)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewRecord {
    pub decision_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub decision: ReviewDecision,
    pub rationale: String,
    pub reviewer: String,
    pub reviewed_at: String,
    pub definition_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_definition_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_definition_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approve,
    Reject,
    Deprecate,
    MergeInto,
    SplitRequired,
    NeedsDefinition,
    NeedsOwner,
    MinorAmendmentNoSemanticChange,
}

// ---------------------------------------------------------------------------
// Suggestion (4.6)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Suggestion {
    pub label: String,
    pub rank: u32,
}

// ---------------------------------------------------------------------------
// PackSetRef (4.7)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackSetRef {
    pub merged_pack_hash: String,
    pub pack_refs: Vec<PackRef>,
}
