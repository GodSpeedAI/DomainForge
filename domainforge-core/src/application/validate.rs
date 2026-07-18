//! Structural invariant checks for application declarations (reference §5).
//!
//! Task 7 scope: APP003 (record shape), APP004 (duplicate fields), APP005
//! (aggregate keys), APP013 (enums). Operation semantics (APP001 and the
//! strategy/effect family) land with the operation-lowering packet.

use crate::application::diagnostic::{ApplicationDiagnostic, ApplicationDiagnosticCode};
use crate::parser::ast::{EnumDecl, FieldDecl, FieldType, FieldTypeRef};
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
