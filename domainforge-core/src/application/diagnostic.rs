//! Stable APP diagnostic wire model (reference §5).
//!
//! Diagnostics are machine-readable `APPNNN code_slug` pairs. Every APP
//! diagnostic is `Error` severity and blocks contract emission. Inapplicable
//! context values are omitted, never serialized as empty strings.

use serde::{Deserialize, Serialize};

/// The closed set of APP014 closure-resolution reasons (reference §5).
pub const APP014_SYMBOL_COLLISION: &str = "symbol_collision";
pub const APP014_IMPORT_CYCLE: &str = "import_cycle";
pub const APP014_UNRESOLVED_SPECIFIER: &str = "unresolved_specifier";
pub const APP014_UNRESOLVED_ALIAS: &str = "unresolved_alias";
pub const APP014_NOT_EXPORTED: &str = "not_exported";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ApplicationDiagnosticCode {
    #[serde(rename = "APP001")]
    App001,
    #[serde(rename = "APP002")]
    App002,
    #[serde(rename = "APP003")]
    App003,
    #[serde(rename = "APP004")]
    App004,
    #[serde(rename = "APP005")]
    App005,
    #[serde(rename = "APP006")]
    App006,
    #[serde(rename = "APP007")]
    App007,
    #[serde(rename = "APP008")]
    App008,
    #[serde(rename = "APP009")]
    App009,
    #[serde(rename = "APP010")]
    App010,
    #[serde(rename = "APP011")]
    App011,
    #[serde(rename = "APP012")]
    App012,
    #[serde(rename = "APP013")]
    App013,
    #[serde(rename = "APP014")]
    App014,
    #[serde(rename = "APP015")]
    App015,
}

impl ApplicationDiagnosticCode {
    /// Every registered APP code in numeric order.
    pub fn all() -> [ApplicationDiagnosticCode; 15] {
        use ApplicationDiagnosticCode::*;
        [
            App001, App002, App003, App004, App005, App006, App007, App008, App009, App010, App011,
            App012, App013, App014, App015,
        ]
    }

    /// The stable `("APPNNN", "code_slug")` registry.
    pub fn all_code_slugs() -> [(&'static str, &'static str); 15] {
        Self::all().map(|c| (c.code(), c.slug()))
    }

    /// Stable wire code, e.g. `"APP001"`.
    pub fn code(&self) -> &'static str {
        use ApplicationDiagnosticCode::*;
        match self {
            App001 => "APP001",
            App002 => "APP002",
            App003 => "APP003",
            App004 => "APP004",
            App005 => "APP005",
            App006 => "APP006",
            App007 => "APP007",
            App008 => "APP008",
            App009 => "APP009",
            App010 => "APP010",
            App011 => "APP011",
            App012 => "APP012",
            App013 => "APP013",
            App014 => "APP014",
            App015 => "APP015",
        }
    }

    /// Every APP diagnostic blocks contract emission.
    pub fn severity(&self) -> &'static str {
        "error"
    }

    pub fn slug(&self) -> &'static str {
        use ApplicationDiagnosticCode::*;
        match self {
            App001 => "missing_application_semantics",
            App002 => "unknown_contract_reference",
            App003 => "invalid_record_shape",
            App004 => "duplicate_field",
            App005 => "invalid_aggregate_key",
            App006 => "operation_type_mismatch",
            App007 => "policy_scope_error",
            App008 => "invalid_failure_declaration",
            App009 => "invalid_strategy_reference",
            App010 => "invalid_not_applicable",
            App011 => "effect_state_mismatch",
            App012 => "constraint_error",
            App013 => "enum_error",
            App014 => "closure_resolution_error",
            App015 => "artifact_schema_violation",
        }
    }
}

/// Structured diagnostic context (reference §5). Inapplicable values are
/// omitted from the wire form.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationDiagnosticContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    /// Source module logical ID (APP001–APP014).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_module_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// Stable semantic ID when one has been assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_id: Option<String>,
    /// Import or declaration field path for pre-identity resolver failures,
    /// or the failing JSON Pointer for APP015.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_path: Option<String>,
    /// APP015 only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationDiagnostic {
    pub code: ApplicationDiagnosticCode,
    pub slug: String,
    pub severity: String, // always "error" in interactive and strict modes
    pub message: String,
    pub context: ApplicationDiagnosticContext,
}

impl ApplicationDiagnostic {
    pub fn new(code: ApplicationDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            slug: code.slug().to_string(),
            severity: "error".to_string(),
            message: message.into(),
            context: ApplicationDiagnosticContext::default(),
        }
    }

    /// APP014 with one closed reason value.
    pub fn closure_error(reason: &str, message: impl Into<String>) -> Self {
        let mut d = Self::new(ApplicationDiagnosticCode::App014, message);
        d.context.reason = Some(reason.to_string());
        d
    }

    pub fn at(mut self, logical_module_id: &str, line: usize, column: usize) -> Self {
        self.context.logical_module_id = Some(logical_module_id.to_string());
        self.context.line = Some(line);
        self.context.column = Some(column);
        self
    }

    /// Mark this APP015 as originating from an authored source map (parse,
    /// graph build, pack-set input) rather than from persisted-artifact
    /// validation. The two APP015 paths are distinguishable in context.
    pub fn with_document_kind(mut self, kind: &str) -> Self {
        self.context.document_kind = Some(kind.to_string());
        self
    }
}

/// Deterministic diagnostic order: logical ID, line, column, then APP code.
pub fn sort_diagnostics(diags: &mut [ApplicationDiagnostic]) {
    diags.sort_by(|a, b| {
        (
            &a.context.logical_module_id,
            a.context.line,
            a.context.column,
            a.code,
        )
            .cmp(&(
                &b.context.logical_module_id,
                b.context.line,
                b.context.column,
                b.code,
            ))
    });
}
