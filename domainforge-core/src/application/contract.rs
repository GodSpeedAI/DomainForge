//! Exact public application contract wire types (reference §8).
//!
//! Public serialized structs use `deny_unknown_fields`; discriminated enums
//! use adjacent tagging (`kind`/`data`, snake_case); newtype IDs are
//! transparent. These attributes, not Serde defaults, define the wire shape.

use crate::concept_id::ConceptId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Stable ID for an application declaration (record/enum/operation only).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ApplicationSymbolId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerIdentity {
    pub name: String, // "domainforge-core"
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContractInputs {
    pub source_set_hash: String, // "sha256:<lowercase-hex>"
    pub semantic_pack_set_hash: String,
    pub language_schema_version: String,
    pub interpretation_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContractDocument {
    pub schema_version: String, // "domainforge-application-contract/v1"
    pub producer: ProducerIdentity,
    pub inputs: ApplicationContractInputs,
    pub self_hash: String,
    pub semantic_closure_hash: String,
    pub contract: ApplicationContract,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContract {
    pub enums: Vec<EnumContract>,
    pub records: Vec<RecordContract>,
    pub entities: Vec<EntityContract>,
    pub operations: Vec<OperationContract>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarType {
    String,
    Int,
    Decimal,
    Bool,
    Timestamp,
    Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FieldType {
    Scalar { scalar: ScalarType },
    Quantity { unit: ConceptId },
    EntityRef { entity: ConceptId },
    Enum { symbol: ApplicationSymbolId },
    List { element: Box<FieldType> }, // element is never List
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FieldConstraint {
    Min { value: rust_decimal::Decimal },
    Max { value: rust_decimal::Decimal },
    ExclusiveMin { value: rust_decimal::Decimal },
    ExclusiveMax { value: rust_decimal::Decimal },
    MinLength { value: u32 },
    MaxLength { value: u32 },
    MinItems { value: u32 },
    MaxItems { value: u32 },
    Pattern { pattern: ConceptId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldContract {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub constraints: Vec<FieldConstraint>,
    /// Entity fields only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<TypedValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumMember {
    pub name: String,
    pub wire: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumContract {
    pub id: ApplicationSymbolId,
    pub name: String,
    pub members: Vec<EnumMember>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordContract {
    pub id: ApplicationSymbolId,
    pub name: String,
    pub fields: Vec<FieldContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityContract {
    pub concept_id: ConceptId,
    pub name: String,
    pub key_field: String,
    pub fields: Vec<FieldContract>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Inbound,
    Outbound,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ActorRef {
    Anonymous,
    Role { role: ConceptId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementPoint {
    Precondition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBinding {
    pub policy: ConceptId,
    pub enforcement_point: EnforcementPoint,
    pub failure_code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum AccessMode {
    Public,
    PolicyGoverned { bindings: Vec<PolicyBinding> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Creates,
    Mutates,
    Reads,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionBoundary {
    SingleAggregate,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    InputValidation,
    Policy,
    MissingState,
    IdempotencyConflict,
    ConcurrencyConflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureContract {
    pub code: String,
    pub kinds: Vec<FailureKind>,
    pub meaning: String,
}

/// Reason is always `read_only` in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotApplicable {
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum IdempotencyStrategy {
    KeyedBy { field: String },
    Inherent,
    NotApplicable(NotApplicable),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ConcurrencyStrategy {
    UniqueKey { field: String },
    OptimisticVersion { field: String },
    ReadSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    OperationTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleKind {
    SynchronousRequestResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationContract {
    pub id: ApplicationSymbolId,
    pub name: String,
    pub intent: String,
    pub direction: Direction,
    pub actor: ActorRef,
    pub access: AccessMode,
    pub input: ApplicationSymbolId,
    pub output: ApplicationSymbolId,
    pub state: ConceptId,
    pub effect: EffectKind,
    pub transaction: TransactionBoundary,
    pub failures: Vec<FailureContract>,
    pub idempotency: IdempotencyStrategy,
    pub concurrency: ConcurrencyStrategy,
    pub evidence: EvidenceKind,
    pub lifecycle: LifecycleKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationPolicyContext {
    pub operation: ApplicationSymbolId,
    pub actor: ActorRef,
    pub input: IndexMap<String, TypedValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_state: Option<IndexMap<String, TypedValue>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TypedValue {
    String(String),
    Int(i64),
    Decimal(rust_decimal::Decimal),
    Bool(bool),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Uuid(uuid::Uuid),
    Quantity {
        base_value: rust_decimal::Decimal,
        unit: ConceptId,
    },
    EntityRef {
        entity: ConceptId,
        key: Box<TypedValue>,
    },
    Enum {
        symbol: ApplicationSymbolId,
        wire: String,
    },
    List(Vec<TypedValue>),
}
