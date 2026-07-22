//! Structural invariant checks for application declarations (reference §5).
//!
//! Task 7 scope: APP003 (record shape), APP004 (duplicate fields), APP005
//! (aggregate keys), APP013 (enums). Operation semantics (APP001 and the
//! strategy/effect family) land with the operation-lowering packet.

use crate::application::contract::FailureKind;
use crate::application::diagnostic::{ApplicationDiagnostic, ApplicationDiagnosticCode};
use crate::parser::ast::{EnumDecl, FieldDecl, FieldType, FieldTypeRef, OperationClause};
use std::collections::HashSet;

/// Source coordinates a check should attach to its diagnostics.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeclSite<'a> {
    pub logical_module_id: &'a str,
    pub line: usize,
    pub column: usize,
}

impl DeclSite<'_> {
    pub(crate) fn diag(
        &self,
        code: ApplicationDiagnosticCode,
        message: String,
    ) -> ApplicationDiagnostic {
        ApplicationDiagnostic::new(code, message).at(self.logical_module_id, self.line, self.column)
    }
}

/// APP004: duplicate field names inside one record or entity body.
pub(crate) fn check_duplicate_fields(
    owner: &str,
    fields: &[FieldDecl],
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) {
    let mut seen = HashSet::new();
    for field in fields {
        if !seen.insert(field.name.as_str()) {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App004,
                format!("duplicate field '{}' in '{owner}'", field.name),
            ));
        }
    }
}

/// APP003 (record part): record fields must not declare defaults.
pub(crate) fn check_record_defaults(
    owner: &str,
    fields: &[FieldDecl],
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) {
    for field in fields {
        if field.default.is_some() {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App003,
                format!(
                    "field '{}' in record '{owner}' declares a default; defaults are entity-only",
                    field.name
                ),
            ));
        }
    }
}

fn is_key_capable(field_type: &FieldType) -> bool {
    matches!(
        field_type,
        FieldType::Scalar(FieldTypeRef { symbol, .. }) if symbol == "uuid" || symbol == "string"
    )
}

/// APP005: exactly one `key` field of type `uuid`/`string`, never optional,
/// never defaulted. Returns the key field name when valid.
pub(crate) fn check_entity_key(
    owner: &str,
    fields: &[FieldDecl],
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Option<String> {
    let keys: Vec<&FieldDecl> = fields.iter().filter(|f| f.is_key).collect();
    match keys.as_slice() {
        [] => {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App005,
                format!("entity '{owner}' has no key field"),
            ));
            None
        }
        [key] => {
            let mut ok = true;
            if !is_key_capable(&key.field_type) {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App005,
                    format!(
                        "key field '{}' of entity '{owner}' must be uuid or string",
                        key.name
                    ),
                ));
                ok = false;
            }
            if key.is_optional {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App005,
                    format!(
                        "key field '{}' of entity '{owner}' must not be optional",
                        key.name
                    ),
                ));
                ok = false;
            }
            if key.default.is_some() {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App005,
                    format!(
                        "key field '{}' of entity '{owner}' must not have a default",
                        key.name
                    ),
                ));
                ok = false;
            }
            ok.then(|| key.name.clone())
        }
        many => {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App005,
                format!(
                    "entity '{owner}' has {} key fields; exactly one is required",
                    many.len()
                ),
            ));
            None
        }
    }
}

/// Canonical clause rank (reference §4 order). Failures share one rank.
fn clause_rank(clause: &OperationClause) -> (usize, &'static str) {
    use OperationClause as C;
    match clause {
        C::Intent(_) => (0, "intent"),
        C::Direction { .. } => (1, "direction"),
        C::Actor { .. } => (2, "actor"),
        C::AccessPublic | C::AccessPolicyGoverned { .. } => (3, "access"),
        C::Input { .. } => (4, "input"),
        C::Output { .. } => (5, "output"),
        C::State { .. } => (6, "state"),
        C::Effect { .. } => (7, "effect"),
        C::Transaction { .. } => (8, "transaction"),
        C::Failure { .. } => (9, "failure"),
        C::IdempotencyKeyed { .. }
        | C::IdempotencyInherent
        | C::IdempotencyNotApplicable { .. } => (10, "idempotency"),
        C::ConcurrencyUnique { .. }
        | C::ConcurrencyOptimistic { .. }
        | C::ConcurrencyReadSnapshot => (11, "concurrency"),
        C::EvidenceOperationTrace => (12, "evidence"),
        C::LifecycleSynchronousRequestResponse => (13, "lifecycle"),
    }
}

const CLAUSE_NAMES: [&str; 14] = [
    "intent",
    "direction",
    "actor",
    "access",
    "input",
    "output",
    "state",
    "effect",
    "transaction",
    "failure",
    "idempotency",
    "concurrency",
    "evidence",
    "lifecycle",
];

/// APP001 clause-shape phase: every §4 defect for one operation is returned
/// together as a defect list (missing, duplicate, out-of-order, unsupported,
/// transaction pairing, actor/access consistency, empty intent).
pub(crate) fn validate_clause_shape(clauses: &[OperationClause]) -> Vec<String> {
    use OperationClause as C;
    let mut defects = Vec::new();
    let mut counts = [0usize; 14];
    let mut last_rank = 0usize;
    let mut ordered = true;
    for clause in clauses {
        let (rank, _) = clause_rank(clause);
        counts[rank] += 1;
        if rank < last_rank {
            ordered = false;
        }
        last_rank = rank;
    }
    for (rank, name) in CLAUSE_NAMES.iter().enumerate() {
        if counts[rank] == 0 {
            defects.push(format!("missing {name} clause"));
        } else if counts[rank] > 1 && rank != 9 {
            defects.push(format!("duplicate {name} clause"));
        }
    }
    if !ordered {
        defects.push("clauses out of canonical order".to_string());
    }

    let mut anonymous = false;
    let mut public = false;
    let mut effect_kind = None;
    let mut transaction_kind = None;
    for clause in clauses {
        match clause {
            C::Intent(text) if text.trim().is_empty() => {
                defects.push("intent text is empty".to_string());
            }
            C::Direction { kind } if kind != "inbound" => {
                defects.push(format!("direction '{kind}' is unsupported in v0.1"));
            }
            C::Actor { actor } if actor == "anonymous" => anonymous = true,
            C::AccessPublic => public = true,
            C::Effect { kind, .. } => effect_kind = Some(kind.as_str()),
            C::Transaction { kind } => transaction_kind = Some(kind.as_str()),
            _ => {}
        }
    }
    if anonymous && !public && counts[3] > 0 {
        defects.push("actor anonymous is legal only with access public".to_string());
    }
    match (effect_kind, transaction_kind) {
        (Some("creates") | Some("mutates"), Some(t)) if t != "single_aggregate" => {
            defects.push(format!(
                "transaction '{t}' does not pair with a write effect; single_aggregate is required"
            ));
        }
        (Some("reads"), Some(t)) if t != "read_only" => {
            defects.push(format!(
                "transaction '{t}' does not pair with effect reads; read_only is required"
            ));
        }
        _ => {}
    }
    defects
}

fn is_lower_snake_case(code: &str) -> bool {
    !code.is_empty()
        && code.split('_').all(|seg| {
            !seg.is_empty()
                && seg
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

pub(crate) fn failure_kind(raw: &str) -> Option<FailureKind> {
    Some(match raw {
        "input_validation" => FailureKind::InputValidation,
        "policy" => FailureKind::Policy,
        "missing_state" => FailureKind::MissingState,
        "idempotency_conflict" => FailureKind::IdempotencyConflict,
        "concurrency_conflict" => FailureKind::ConcurrencyConflict,
        _ => return None,
    })
}

/// APP008 phase: failure code shape, closure-wide code uniqueness, per-operation
/// kind uniqueness, and exactly-one mapping for every reachable failure path.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_failures(
    operation: &str,
    failures: &[(String, Vec<FailureKind>, String)],
    required_kinds: &[FailureKind],
    closure_codes: &mut HashSet<String>,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) {
    let mut kinds_seen: Vec<FailureKind> = Vec::new();
    for (code, kinds, meaning) in failures {
        if !is_lower_snake_case(code) {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App008,
                format!("failure code '{code}' in operation '{operation}' is not lower_snake_case"),
            ));
        }
        if !closure_codes.insert(code.clone()) {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App008,
                format!("failure code '{code}' is not unique in the resolved closure"),
            ));
        }
        if meaning.trim().is_empty() {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App008,
                format!("failure '{code}' in operation '{operation}' has an empty meaning"),
            ));
        }
        for kind in kinds {
            if kinds_seen.contains(kind) {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App008,
                    format!(
                        "failure kind {kind:?} appears on more than one failure of operation '{operation}'"
                    ),
                ));
            } else {
                kinds_seen.push(*kind);
            }
        }
    }
    for required in required_kinds {
        if !kinds_seen.contains(required) {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App008,
                format!(
                    "operation '{operation}' has a reachable {required:?} path with no declared failure code"
                ),
            ));
        }
    }
}

/// APP013: duplicate enum member identifiers or duplicate wire strings.
pub(crate) fn check_enum(decl: &EnumDecl, site: DeclSite, diags: &mut Vec<ApplicationDiagnostic>) {
    let mut names = HashSet::new();
    let mut wires = HashSet::new();
    for member in &decl.members {
        if !names.insert(member.name.as_str()) {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App013,
                format!("duplicate enum member '{}' in '{}'", member.name, decl.name),
            ));
        }
        if !wires.insert(member.wire.as_str()) {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App013,
                format!(
                    "duplicate enum wire value \"{}\" in '{}'",
                    member.wire, decl.name
                ),
            ));
        }
    }
}
