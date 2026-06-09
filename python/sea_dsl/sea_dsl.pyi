from typing import Any, List, Optional, Tuple

__version__: str

# =============================================================================
# Domain Primitives
# =============================================================================

class Entity:
    id: str
    name: str
    namespace: Optional[str]

    def __init__(self, name: str, namespace: Optional[str] = None) -> None: ...
    def set_attribute(self, key: str, value: Any) -> None: ...
    def get_attribute(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Resource:
    id: str
    name: str
    unit: str
    namespace: Optional[str]

    def __init__(self, name: str, unit: str, namespace: Optional[str] = None) -> None: ...
    def set_attribute(self, key: str, value: Any) -> None: ...
    def get_attribute(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Flow:
    id: str
    resource_id: str
    from_id: str
    to_id: str
    quantity: float
    namespace: Optional[str]

    def __init__(self, resource_id: str, from_id: str, to_id: str, quantity: float) -> None: ...
    def set_attribute(self, key: str, value: Any) -> None: ...
    def get_attribute(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...

class ResourceInstance:
    id: str
    resource_id: str
    entity_id: str
    namespace: Optional[str]

    def __init__(self, resource_id: str, entity_id: str, namespace: Optional[str] = None) -> None: ...
    def set_attribute(self, key: str, value: Any) -> None: ...
    def get_attribute(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...

class Instance:
    id: str
    name: str
    entity_type: str
    namespace: Optional[str]

    def __init__(self, name: str, entity_type: str, namespace: Optional[str] = None) -> None: ...
    def set_field(self, key: str, value: Any) -> None: ...
    def get_field(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...

class Role:
    id: str
    name: str
    namespace: Optional[str]

    def __init__(self, name: str, namespace: Optional[str] = None) -> None: ...
    def set_attribute(self, key: str, value: Any) -> None: ...
    def get_attribute(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...

class Relation:
    id: str
    name: str
    namespace: Optional[str]
    subject_role_id: str
    predicate: str
    object_role_id: str
    via_flow_id: Optional[str]

    def __init__(
        self,
        name: str,
        subject_role_id: str,
        predicate: str,
        object_role_id: str,
        namespace: Optional[str] = None,
        via_flow_id: Optional[str] = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

class Mapping:
    name: str
    target_format: str

    def __repr__(self) -> str: ...

class Projection:
    name: str
    target_format: str

    def __repr__(self) -> str: ...

# =============================================================================
# Graph
# =============================================================================

class Graph:
    def __init__(self) -> None: ...
    def add_entity(self, entity: Entity) -> None: ...
    def add_resource(self, resource: Resource) -> None: ...
    def add_flow(self, flow: Flow) -> None: ...
    def add_instance(self, instance: ResourceInstance) -> None: ...
    def add_role(self, role: Role) -> None: ...
    def add_relation(self, relation: Relation) -> None: ...
    def add_policy(self, policy_json: str) -> None: ...
    def add_association(self, owner: str, owned: str, rel_type: str) -> None: ...
    def entity_count(self) -> int: ...
    def resource_count(self) -> int: ...
    def flow_count(self) -> int: ...
    def instance_count(self) -> int: ...
    def role_count(self) -> int: ...
    def relation_count(self) -> int: ...
    def pattern_count(self) -> int: ...
    def has_entity(self, id: str) -> bool: ...
    def has_resource(self, id: str) -> bool: ...
    def has_flow(self, id: str) -> bool: ...
    def has_instance(self, id: str) -> bool: ...
    def get_entity(self, id: str) -> Optional[Entity]: ...
    def get_resource(self, id: str) -> Optional[Resource]: ...
    def get_flow(self, id: str) -> Optional[Flow]: ...
    def get_instance(self, id: str) -> Optional[ResourceInstance]: ...
    def find_entity_by_name(self, name: str) -> Optional[str]: ...
    def find_resource_by_name(self, name: str) -> Optional[str]: ...
    def find_role_by_name(self, name: str) -> Optional[str]: ...
    def flows_from(self, entity_id: str) -> List[Flow]: ...
    def flows_to(self, entity_id: str) -> List[Flow]: ...
    def all_entities(self) -> List[Entity]: ...
    def all_resources(self) -> List[Resource]: ...
    def all_flows(self) -> List[Flow]: ...
    def all_instances(self) -> List[ResourceInstance]: ...
    def all_roles(self) -> List[Role]: ...
    def all_relations(self) -> List[Relation]: ...
    def evaluate_policy(self, policy_json: str) -> EvaluationResult: ...
    def set_evaluation_mode(self, use_three_valued_logic: bool) -> None: ...
    def use_three_valued_logic(self) -> bool: ...
    def export_calm(self) -> str: ...
    def export_protobuf(
        self,
        package: str,
        namespace: str = ...,
        projection_name: str = ...,
        include_governance: bool = ...,
        include_services: bool = ...,
    ) -> str: ...

    @staticmethod
    def parse(source: str) -> Graph: ...

    @staticmethod
    def parse_to_ast_json(source: str) -> str: ...

    @staticmethod
    def import_calm(calm_json: str) -> Graph: ...

    def __repr__(self) -> str: ...

# =============================================================================
# Policy Types
# =============================================================================

class Severity:
    Error: Severity
    Warning: Severity
    Info: Severity

class Violation:
    name: str
    message: str
    severity: Severity

    def __repr__(self) -> str: ...

class EvaluationResult:
    is_satisfied: bool
    is_satisfied_tristate: Optional[bool]
    violations: List[Violation]

    def __repr__(self) -> str: ...

class BinaryOp:
    And: BinaryOp
    Or: BinaryOp
    Equal: BinaryOp
    NotEqual: BinaryOp
    GreaterThan: BinaryOp
    LessThan: BinaryOp
    GreaterThanOrEqual: BinaryOp
    LessThanOrEqual: BinaryOp
    Plus: BinaryOp
    Minus: BinaryOp
    Multiply: BinaryOp
    Divide: BinaryOp
    Contains: BinaryOp
    StartsWith: BinaryOp
    EndsWith: BinaryOp
    Matches: BinaryOp
    HasRole: BinaryOp
    Before: BinaryOp
    After: BinaryOp
    During: BinaryOp

class UnaryOp:
    Not: UnaryOp
    Negate: UnaryOp

class Quantifier:
    ForAll: Quantifier
    Exists: Quantifier
    ExistsUnique: Quantifier

class AggregateFunction:
    Count: AggregateFunction
    Sum: AggregateFunction
    Min: AggregateFunction
    Max: AggregateFunction
    Avg: AggregateFunction

class WindowSpec:
    duration: int
    unit: str

    def __init__(self, duration: int, unit: str) -> None: ...
    def __repr__(self) -> str: ...

class Expression:
    @staticmethod
    def literal(value: Any) -> Expression: ...

    @staticmethod
    def variable(name: str) -> Expression: ...

    @staticmethod
    def quantity(value: str, unit: str) -> Expression: ...

    @staticmethod
    def time(timestamp: str) -> Expression: ...

    @staticmethod
    def interval(start: str, end: str) -> Expression: ...

    @staticmethod
    def binary(op: BinaryOp, left: Expression, right: Expression) -> Expression: ...

    @staticmethod
    def unary(op: UnaryOp, operand: Expression) -> Expression: ...

    @staticmethod
    def cast(operand: Expression, target_type: str) -> Expression: ...

    @staticmethod
    def quantifier(
        q: Quantifier,
        variable: str,
        collection: Expression,
        condition: Expression,
    ) -> Expression: ...

    @staticmethod
    def member_access(object: str, member: str) -> Expression: ...

    @staticmethod
    def aggregation(
        function: AggregateFunction,
        collection: Expression,
        field: Optional[str] = ...,
        filter: Optional[Expression] = ...,
    ) -> Expression: ...

    @staticmethod
    def aggregation_comprehension(
        function: AggregateFunction,
        variable: str,
        collection: Expression,
        predicate: Expression,
        projection: Expression,
        window: Optional[WindowSpec] = ...,
        target_unit: Optional[str] = ...,
    ) -> Expression: ...

    @staticmethod
    def group_by(
        variable: str,
        collection: Expression,
        key: Expression,
        condition: Expression,
        filter: Optional[Expression] = ...,
    ) -> Expression: ...

    def normalize(self) -> NormalizedExpression: ...
    def is_equivalent(self, other: Expression) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: Expression) -> bool: ...

class NormalizedExpression:
    def stable_hash(self) -> int: ...
    def inner_expression(self) -> Expression: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: NormalizedExpression) -> bool: ...
    def __hash__(self) -> int: ...

# =============================================================================
# Authority Module
# =============================================================================

class FinalDecision:
    Allow: FinalDecision
    Deny: FinalDecision
    Escalate: FinalDecision
    NotApplicable: FinalDecision
    Reject: FinalDecision

class PolicyModality:
    Permission: PolicyModality
    Prohibition: PolicyModality
    Obligation: PolicyModality
    Override: PolicyModality

class SourceClass:
    CallerSupplied: SourceClass
    RuntimeObserved: SourceClass
    SystemOfRecord: SourceClass
    Attested: SourceClass
    ManualApproval: SourceClass
    Derived: SourceClass
    UnknownSource: SourceClass

class ClaimLevel:
    AuditBacked: ClaimLevel
    Validated: ClaimLevel
    FormallyProven: ClaimLevel

class AuthorityEnvironment:
    def __init__(self, config_json: str) -> None: ...
    def validate(self) -> None: ...
    def evaluate(self, request_json: str, facts_json: str = ...) -> Tuple[str, str]: ...
    def __repr__(self) -> str: ...

def evaluate_authority(
    config_json: str,
    request_json: str,
    facts_json: str = ...,
) -> Tuple[str, str]: ...

# =============================================================================
# Units Module
# =============================================================================

class Dimension:
    @staticmethod
    def parse(name: str) -> Dimension: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Unit:
    symbol: str
    name: str
    base_unit: str
    base_factor: float

    def __init__(
        self,
        symbol: str,
        name: str,
        dimension: str,
        base_factor: float,
        base_unit: str,
    ) -> None: ...
    def __repr__(self) -> str: ...

# =============================================================================
# Registry Module
# =============================================================================

class NamespaceBinding:
    path: str
    namespace: str

class NamespaceRegistry:
    root: str
    default_namespace: str

    @staticmethod
    def from_file(path: str) -> NamespaceRegistry: ...

    @staticmethod
    def discover(path: str) -> Optional[NamespaceRegistry]: ...

    def resolve_files(self, fail_on_ambiguity: Optional[bool] = ...) -> List[NamespaceBinding]: ...
    def namespace_for(self, path: str, fail_on_ambiguity: Optional[bool] = ...) -> str: ...

# =============================================================================
# Formatter Functions
# =============================================================================

def format_source(
    source: str,
    indent_width: int = ...,
    use_tabs: bool = ...,
    preserve_comments: bool = ...,
    sort_imports: bool = ...,
) -> str: ...

def check_format(
    source: str,
    indent_width: int = ...,
    use_tabs: bool = ...,
) -> bool: ...

# =============================================================================
# Semantic Pack Module
# =============================================================================

class SemanticTruth:
    Valid: SemanticTruth
    Invalid: SemanticTruth
    Unknown: SemanticTruth

class DiagnosticSeverity:
    Error: DiagnosticSeverity
    Warning: DiagnosticSeverity
    Info: DiagnosticSeverity
    Hint: DiagnosticSeverity

class ValidationMode:
    Off: ValidationMode
    Warn: ValidationMode
    Strict: ValidationMode

class ApprovalState:
    Candidate: ApprovalState
    Approved: ApprovalState
    Rejected: ApprovalState

class SignatureState:
    Unsigned: SignatureState
    Signed: SignatureState
    InvalidSignature: SignatureState

class ConceptStatus:
    Active: ConceptStatus
    Proposed: ConceptStatus
    Deprecated: ConceptStatus
    Rejected: ConceptStatus
    ExternalOnly: ConceptStatus

class ConceptKind:
    Entity: ConceptKind
    Resource: ConceptKind
    Role: ConceptKind
    Flow: ConceptKind
    Policy: ConceptKind
    Metric: ConceptKind
    Dimension: ConceptKind
    Unit: ConceptKind
    External: ConceptKind

class AliasStatus:
    Approved: AliasStatus
    Deprecated: AliasStatus
    Ambiguous: AliasStatus
    Blocked: AliasStatus

class SemanticValidationStatus:
    Passed: SemanticValidationStatus
    Failed: SemanticValidationStatus
    Unknown: SemanticValidationStatus
    Blocked: SemanticValidationStatus

class SemanticPack:
    @staticmethod
    def from_json(json: str) -> SemanticPack: ...
    def to_json(self) -> str: ...
    def pack_id(self) -> str: ...
    def schema_version(self) -> str: ...
    def approval_state(self) -> ApprovalState: ...
    def signature_state(self) -> SignatureState: ...
    def concept_count(self) -> int: ...
    def alias_count(self) -> int: ...
    def meaning_version(self) -> str: ...
    def meaning_fingerprint(self) -> str: ...
    def pack_content_hash(self) -> str: ...
    def __repr__(self) -> str: ...

class SemanticValidationResult:
    @staticmethod
    def from_json(json: str) -> SemanticValidationResult: ...
    def status(self) -> SemanticValidationStatus: ...
    def diagnostics_json(self) -> str: ...
    def unsigned_fixture_bypass_used(self) -> bool: ...
    def first_approved_version_bypass_used(self) -> bool: ...
    def __repr__(self) -> str: ...

def build_semantic_pack(input_json: str) -> Tuple[str, List[str]]: ...
def validate_semantic_pack(pack_json: str) -> List[str]: ...
def validate_graph_with_pack(pack_json: str, source: str, options_json: str) -> str: ...
def sign_pack(pack_json: str, private_key_pem: str) -> str: ...
def verify_pack_signature(pack_json: str, public_key_pem: str) -> bool: ...
def diff_packs(old_json: str, new_json: str) -> str: ...
def compute_pack_hash(pack_json: str) -> str: ...
def normalize_lookup_key(text: str) -> str: ...
def resolve_concept(raw_text: str, pack_json: str, options_json: str) -> str: ...
