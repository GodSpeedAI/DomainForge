//! AST-to-contract reference/type resolution and the public entry point
//! (reference §7/§8). Every reference resolves through
//! `ResolvedModuleSet::symbols`; nothing scans the AST by name.

use crate::application::contract::*;
use crate::application::diagnostic::{
    sort_diagnostics, ApplicationDiagnostic, ApplicationDiagnosticCode,
};
use crate::application::validate::{
    check_duplicate_fields, check_entity_key, check_enum, check_record_defaults, DeclSite,
};
use crate::concept_id::ConceptId;
use crate::module::resolver::{resolve_source_map, ResolvedModuleSet, ResolvedSymbol, SourceMap};
use crate::parser::ast as past;
use crate::policy::Expression;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;

pub const APPLICATION_CONTRACT_SCHEMA_VERSION: &str = "domainforge-application-contract/v1";
pub const LANGUAGE_SCHEMA_VERSION: &str = "domainforge-ast/v3";
pub const INTERPRETATION_VERSION: &str = "domainforge-interpretation/v1";

/// Resolve an in-memory source map into a validated application contract
/// document, or the complete deterministic diagnostic list.
pub fn resolve_application_contract(
    entry_logical_path: &str,
    sources_json: &str,
) -> Result<ApplicationContractDocument, Vec<ApplicationDiagnostic>> {
    let sources = SourceMap::parse_json(sources_json)?;
    let resolved = resolve_source_map(entry_logical_path, &sources)?;
    let contract = build_contract(&resolved)?;
    Ok(ApplicationContractDocument {
        schema_version: APPLICATION_CONTRACT_SCHEMA_VERSION.to_string(),
        producer: ProducerIdentity {
            name: "domainforge-core".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        inputs: ApplicationContractInputs {
            source_set_hash: source_set_hash(&sources),
            semantic_pack_set_hash: semantic_pack_set_hash(),
            language_schema_version: LANGUAGE_SCHEMA_VERSION.to_string(),
            interpretation_version: INTERPRETATION_VERSION.to_string(),
        },
        // ponytail: placeholder zero hashes until the canonicalization packet
        // (Tasks 11/12) computes self_hash and semantic_closure_hash.
        self_hash: zero_hash(),
        semantic_closure_hash: zero_hash(),
        contract,
    })
}

/// JSON boundary twin of [`resolve_application_contract`].
pub fn resolve_application_contract_json(
    entry_logical_path: &str,
    sources_json: &str,
) -> Result<String, Vec<ApplicationDiagnostic>> {
    let doc = resolve_application_contract(entry_logical_path, sources_json)?;
    serde_json::to_string(&doc).map_err(|e| {
        vec![ApplicationDiagnostic::new(
            ApplicationDiagnosticCode::App015,
            format!("failed to serialize application contract document: {e}"),
        )]
    })
}

fn zero_hash() -> String {
    format!("sha256:{}", "0".repeat(64))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// Reference §6 source-set hash over the authored source map.
fn source_set_hash(sources: &SourceMap) -> String {
    #[derive(Serialize)]
    struct Entry<'a> {
        logical_id: &'a str,
        source_utf8: &'a str,
    }
    #[derive(Serialize)]
    struct SourceSet<'a> {
        schema_version: &'a str,
        sources: Vec<Entry<'a>>,
    }
    let mut entries: Vec<Entry> = sources
        .0
        .iter()
        .map(|(id, source)| Entry {
            logical_id: id,
            source_utf8: source,
        })
        .collect();
    entries.sort_by(|a, b| a.logical_id.cmp(b.logical_id));
    let set = SourceSet {
        schema_version: "domainforge-source-set/v1",
        sources: entries,
    };
    sha256_prefixed(
        serde_json::to_string(&set)
            .expect("source set serializes")
            .as_bytes(),
    )
}

/// Reference §6 semantic-pack-set hash; Milestone 0 uses the empty set.
fn semantic_pack_set_hash() -> String {
    sha256_prefixed(br#"{"packs":[],"schema_version":"domainforge-semantic-pack-set/v1"}"#)
}

// ---- contract construction ----

struct ModuleBindings {
    namespace: String,
    /// Locally visible name -> (target namespace, original name).
    named: HashMap<String, (String, String)>,
    /// Wildcard alias -> target namespace.
    wildcard: HashMap<String, String>,
}

struct Ctx<'a> {
    set: &'a ResolvedModuleSet,
    bindings: Vec<ModuleBindings>,
}

const REFERENCEABLE_SLUGS: &[&str] = &[
    "record", "enum", "entity", "unit", "pattern", "role", "policy",
];

impl<'a> Ctx<'a> {
    fn new(set: &'a ResolvedModuleSet) -> Self {
        let namespace_of: HashMap<&str, String> = set
            .modules
            .iter()
            .map(|m| {
                (
                    m.logical_id.as_str(),
                    m.ast
                        .metadata
                        .namespace
                        .clone()
                        .unwrap_or_else(|| m.logical_id.clone()),
                )
            })
            .collect();
        let bindings = set
            .modules
            .iter()
            .map(|module| {
                let mut named = HashMap::new();
                let mut wildcard = HashMap::new();
                for import in &module.ast.metadata.imports {
                    // Resolve the import target through the already-validated
                    // import graph edge; specifiers were checked by the resolver.
                    let target_ns = set
                        .import_graph
                        .iter()
                        .find(|e| e.importer == module.logical_id)
                        .and_then(|_| {
                            resolve_specifier_namespace(
                                set,
                                &module.logical_id,
                                &import.from_module,
                                &namespace_of,
                            )
                        });
                    let Some(target_ns) = target_ns else { continue };
                    match &import.specifier {
                        past::ImportSpecifier::Named(items) => {
                            for item in items {
                                let local = item.alias.clone().unwrap_or_else(|| item.name.clone());
                                named.insert(local, (target_ns.clone(), item.name.clone()));
                            }
                        }
                        past::ImportSpecifier::Wildcard(alias) => {
                            wildcard.insert(alias.clone(), target_ns.clone());
                        }
                    }
                }
                ModuleBindings {
                    namespace: namespace_of[module.logical_id.as_str()].clone(),
                    named,
                    wildcard,
                }
            })
            .collect();
        Self { set, bindings }
    }

    fn find_symbol(&self, namespace: &str, name: &str) -> Option<(&ResolvedSymbol, &'static str)> {
        REFERENCEABLE_SLUGS.iter().find_map(|slug| {
            self.set
                .symbols
                .get(&format!("{namespace}.{slug}.{name}"))
                .map(|s| (s, *slug))
        })
    }

    /// Resolve a possibly alias-qualified reference from `module_index`.
    fn resolve_ref(
        &self,
        module_index: usize,
        alias: Option<&str>,
        name: &str,
    ) -> Option<(&ResolvedSymbol, &'static str)> {
        let bindings = &self.bindings[module_index];
        match alias {
            Some(alias) => {
                let ns = bindings.wildcard.get(alias)?;
                let (symbol, slug) = self.find_symbol(ns, name)?;
                symbol.exported.then_some((symbol, slug))
            }
            None => {
                if let Some((ns, original)) = bindings.named.get(name) {
                    self.find_symbol(ns, original)
                } else {
                    self.find_symbol(&bindings.namespace, name)
                }
            }
        }
    }

    fn namespace_of_symbol(symbol: &ResolvedSymbol, slug: &str, name: &str) -> String {
        symbol
            .qualified_id
            .strip_suffix(&format!(".{slug}.{name}"))
            .unwrap_or(&symbol.qualified_id)
            .to_string()
    }

    fn decl_node(&self, symbol: &ResolvedSymbol) -> &past::AstNode {
        let node =
            &self.set.modules[symbol.module_index].ast.declarations[symbol.declaration_index].node;
        match node {
            past::AstNode::Export(inner) => &inner.node,
            other => other,
        }
    }
}

fn resolve_specifier_namespace(
    set: &ResolvedModuleSet,
    importer: &str,
    specifier: &str,
    namespace_of: &HashMap<&str, String>,
) -> Option<String> {
    // The resolver recorded exactly one edge per resolved import; recover the
    // target by re-checking which imported module the specifier denotes.
    set.import_graph
        .iter()
        .filter(|e| e.importer == importer)
        .map(|e| e.imported.as_str())
        .find(|imported| {
            if specifier.starts_with("./") || specifier.starts_with("../") {
                imported.ends_with(specifier.trim_start_matches("./").trim_start_matches("../"))
            } else if specifier == "std" || specifier.starts_with("std:") {
                *imported == specifier
            } else {
                namespace_of.get(imported).map(String::as_str) == Some(specifier)
            }
        })
        .and_then(|id| namespace_of.get(id).cloned())
}

/// Split a raw `symbol_ref` text (`Name` or `alias.Name`) into parts.
fn split_symbol_ref(raw: &str) -> (Option<&str>, &str) {
    match raw.split_once('.') {
        Some((alias, name)) if !alias.is_empty() && !name.is_empty() => (Some(alias), name),
        _ => (None, raw),
    }
}

fn build_contract(
    set: &ResolvedModuleSet,
) -> Result<ApplicationContract, Vec<ApplicationDiagnostic>> {
    let ctx = Ctx::new(set);
    let mut contract = ApplicationContract::default();
    let mut diags: Vec<ApplicationDiagnostic> = Vec::new();

    for symbol in set.symbols.values() {
        let site = DeclSite {
            logical_module_id: &symbol.origin.logical_module_id,
            line: symbol.origin.line,
            column: symbol.origin.column,
        };
        match ctx.decl_node(symbol) {
            past::AstNode::Enum(decl) => {
                check_enum(decl, site, &mut diags);
                contract.enums.push(EnumContract {
                    id: ApplicationSymbolId(symbol.qualified_id.clone()),
                    name: decl.name.clone(),
                    members: decl
                        .members
                        .iter()
                        .map(|m| EnumMember {
                            name: m.name.clone(),
                            wire: m.wire.clone(),
                        })
                        .collect(),
                });
            }
            past::AstNode::Record(decl) => {
                check_duplicate_fields(&decl.name, &decl.fields, site, &mut diags);
                check_record_defaults(&decl.name, &decl.fields, site, &mut diags);
                let fields = resolve_fields(&ctx, symbol, &decl.fields, false, site, &mut diags);
                contract.records.push(RecordContract {
                    id: ApplicationSymbolId(symbol.qualified_id.clone()),
                    name: decl.name.clone(),
                    fields,
                });
            }
            past::AstNode::Entity {
                name,
                body: Some(body),
                ..
            } => {
                check_duplicate_fields(name, &body.fields, site, &mut diags);
                let key_field = check_entity_key(name, &body.fields, site, &mut diags);
                let fields = resolve_fields(&ctx, symbol, &body.fields, true, site, &mut diags);
                if let Some(key_field) = key_field {
                    let namespace = Ctx::namespace_of_symbol(symbol, "entity", name);
                    contract.entities.push(EntityContract {
                        concept_id: ConceptId::from_concept(&namespace, name),
                        name: name.clone(),
                        key_field,
                        fields,
                    });
                }
            }
            past::AstNode::Operation(decl) => {
                // Task 7 scope: resolve input/output references (APP002/APP006).
                // Full clause semantics and OperationContract lowering follow
                // in the operation packet.
                for clause in &decl.clauses {
                    let (label, raw) = match clause {
                        past::OperationClause::Input { reference } => ("input", reference),
                        past::OperationClause::Output { reference } => ("output", reference),
                        _ => continue,
                    };
                    let (alias, name) = split_symbol_ref(raw);
                    match ctx.resolve_ref(symbol.module_index, alias, name) {
                        None => diags.push(site.diag(
                            ApplicationDiagnosticCode::App002,
                            format!(
                                "{label} '{raw}' of operation '{}' does not resolve in the symbol table",
                                decl.name
                            ),
                        )),
                        Some((_, "record")) => {}
                        Some((_, slug)) => diags.push(site.diag(
                            ApplicationDiagnosticCode::App006,
                            format!(
                                "{label} '{raw}' of operation '{}' resolves to {slug}; a record is required",
                                decl.name
                            ),
                        )),
                    }
                }
            }
            _ => {}
        }
    }

    if diags.is_empty() {
        Ok(contract)
    } else {
        sort_diagnostics(&mut diags);
        Err(diags)
    }
}

fn resolve_fields(
    ctx: &Ctx,
    symbol: &ResolvedSymbol,
    fields: &[past::FieldDecl],
    is_entity: bool,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Vec<FieldContract> {
    fields
        .iter()
        .filter_map(|field| {
            let field_type = resolve_field_type(
                ctx,
                symbol.module_index,
                &field.field_type,
                field,
                site,
                diags,
            )?;
            let constraints = field
                .constraints
                .iter()
                .filter_map(|c| resolve_constraint(ctx, symbol.module_index, c, field, site, diags))
                .collect();
            let default = if is_entity {
                match &field.default {
                    Some(expr) => Some(resolve_default(
                        ctx,
                        symbol,
                        &field_type,
                        expr,
                        field,
                        site,
                        diags,
                    )?),
                    None => None,
                }
            } else {
                None // record defaults already reported by check_record_defaults
            };
            Some(FieldContract {
                name: field.name.clone(),
                field_type,
                optional: field.is_optional,
                constraints,
                default,
            })
        })
        .collect()
}

fn scalar_type(symbol: &str) -> Option<ScalarType> {
    Some(match symbol {
        "string" => ScalarType::String,
        "int" => ScalarType::Int,
        "decimal" => ScalarType::Decimal,
        "bool" => ScalarType::Bool,
        "timestamp" => ScalarType::Timestamp,
        "uuid" => ScalarType::Uuid,
        _ => return None,
    })
}

fn resolve_field_type(
    ctx: &Ctx,
    module_index: usize,
    ty: &past::FieldType,
    field: &past::FieldDecl,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Option<FieldType> {
    let unknown = |diags: &mut Vec<ApplicationDiagnostic>, what: &str| {
        diags.push(site.diag(
            ApplicationDiagnosticCode::App002,
            format!(
                "{what} on field '{}' does not resolve in the symbol table",
                field.name
            ),
        ));
    };
    match ty {
        past::FieldType::Scalar(r) => match scalar_type(&r.symbol) {
            Some(scalar) => Some(FieldType::Scalar { scalar }),
            None => {
                unknown(diags, &format!("scalar type '{}'", r.symbol));
                None
            }
        },
        past::FieldType::Quantity(r) => {
            match ctx.resolve_ref(module_index, r.alias.as_deref(), &r.symbol) {
                Some((symbol, "unit")) => Some(FieldType::Quantity {
                    unit: ConceptId::from_concept(
                        &Ctx::namespace_of_symbol(symbol, "unit", &r.symbol),
                        &r.symbol,
                    ),
                }),
                _ => {
                    unknown(diags, &format!("quantity unit '{}'", r.symbol));
                    None
                }
            }
        }
        past::FieldType::Ref(r) => {
            match ctx.resolve_ref(module_index, r.alias.as_deref(), &r.symbol) {
                Some((symbol, "entity")) => Some(FieldType::EntityRef {
                    entity: ConceptId::from_concept(
                        &Ctx::namespace_of_symbol(symbol, "entity", &r.symbol),
                        &r.symbol,
                    ),
                }),
                Some(_) => {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App002,
                        format!(
                            "ref '{}' on field '{}' resolves to a non-entity declaration",
                            r.symbol, field.name
                        ),
                    ));
                    None
                }
                None => {
                    unknown(diags, &format!("ref '{}'", r.symbol));
                    None
                }
            }
        }
        past::FieldType::Named(r) => {
            match ctx.resolve_ref(module_index, r.alias.as_deref(), &r.symbol) {
                Some((symbol, "enum")) => Some(FieldType::Enum {
                    symbol: ApplicationSymbolId(symbol.qualified_id.clone()),
                }),
                Some((_, "record")) => {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App003,
                        format!(
                            "field '{}' nests record '{}'; the closed type model forbids nested records",
                            field.name, r.symbol
                        ),
                    ));
                    None
                }
                Some((_, slug)) => {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App002,
                        format!(
                            "named type '{}' on field '{}' resolves to {slug}, which is not a field type",
                            r.symbol, field.name
                        ),
                    ));
                    None
                }
                None => {
                    unknown(diags, &format!("named type '{}'", r.symbol));
                    None
                }
            }
        }
        past::FieldType::List(inner) => {
            if matches!(**inner, past::FieldType::List(_)) {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App003,
                    format!("field '{}' nests a list inside a list", field.name),
                ));
                return None;
            }
            let element = resolve_field_type(ctx, module_index, inner, field, site, diags)?;
            Some(FieldType::List {
                element: Box::new(element),
            })
        }
    }
}

fn resolve_constraint(
    ctx: &Ctx,
    module_index: usize,
    constraint: &past::FieldConstraintDecl,
    field: &past::FieldDecl,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Option<FieldConstraint> {
    use past::FieldConstraintDecl as C;
    let bound = |value: u64, diags: &mut Vec<ApplicationDiagnostic>| -> Option<u32> {
        u32::try_from(value).ok().or_else(|| {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App012,
                format!(
                    "length/item bound {value} on field '{}' overflows",
                    field.name
                ),
            ));
            None
        })
    };
    Some(match constraint {
        C::Min(v) => FieldConstraint::Min { value: *v },
        C::Max(v) => FieldConstraint::Max { value: *v },
        C::ExclusiveMin(v) => FieldConstraint::ExclusiveMin { value: *v },
        C::ExclusiveMax(v) => FieldConstraint::ExclusiveMax { value: *v },
        C::MinLength(v) => FieldConstraint::MinLength {
            value: bound(*v, diags)?,
        },
        C::MaxLength(v) => FieldConstraint::MaxLength {
            value: bound(*v, diags)?,
        },
        C::MinItems(v) => FieldConstraint::MinItems {
            value: bound(*v, diags)?,
        },
        C::MaxItems(v) => FieldConstraint::MaxItems {
            value: bound(*v, diags)?,
        },
        C::Pattern(raw) => {
            let (alias, name) = split_symbol_ref(raw);
            match ctx.resolve_ref(module_index, alias, name) {
                Some((symbol, "pattern")) => FieldConstraint::Pattern {
                    pattern: ConceptId::from_concept(
                        &Ctx::namespace_of_symbol(symbol, "pattern", name),
                        name,
                    ),
                },
                _ => {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App012,
                        format!(
                            "pattern '{raw}' on field '{}' does not resolve to a declared SEA Pattern",
                            field.name
                        ),
                    ));
                    return None;
                }
            }
        }
    })
}

/// Convert an authored entity-field default literal into a `TypedValue` that
/// satisfies the resolved field type (APP003 on mismatch).
fn resolve_default(
    ctx: &Ctx,
    owner: &ResolvedSymbol,
    field_type: &FieldType,
    expr: &Expression,
    field: &past::FieldDecl,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Option<TypedValue> {
    let mismatch = |diags: &mut Vec<ApplicationDiagnostic>| {
        diags.push(site.diag(
            ApplicationDiagnosticCode::App003,
            format!(
                "default on field '{}' does not satisfy its declared type",
                field.name
            ),
        ));
    };
    let Expression::Literal(value) = expr else {
        mismatch(diags);
        return None;
    };
    let typed = match (field_type, value) {
        (
            FieldType::Scalar {
                scalar: ScalarType::String,
            },
            serde_json::Value::String(s),
        ) => TypedValue::String(s.clone()),
        (
            FieldType::Scalar {
                scalar: ScalarType::Bool,
            },
            serde_json::Value::Bool(b),
        ) => TypedValue::Bool(*b),
        (
            FieldType::Scalar {
                scalar: ScalarType::Int,
            },
            serde_json::Value::Number(n),
        ) => TypedValue::Int(n.as_i64()?),
        (
            FieldType::Scalar {
                scalar: ScalarType::Decimal,
            },
            serde_json::Value::Number(n),
        ) => TypedValue::Decimal(rust_decimal::Decimal::from_str(&n.to_string()).ok()?),
        (
            FieldType::Scalar {
                scalar: ScalarType::Uuid,
            },
            serde_json::Value::String(s),
        ) => TypedValue::Uuid(uuid::Uuid::from_str(s).ok().or_else(|| {
            mismatch(diags);
            None
        })?),
        (
            FieldType::Scalar {
                scalar: ScalarType::Timestamp,
            },
            serde_json::Value::String(s),
        ) => TypedValue::Timestamp(
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|t| t.with_timezone(&chrono::Utc))
                .or_else(|| {
                    mismatch(diags);
                    None
                })?,
        ),
        (FieldType::Enum { symbol }, serde_json::Value::String(wire)) => {
            let enum_symbol = ctx.set.symbols.get(&symbol.0)?;
            let past::AstNode::Enum(decl) = ctx.decl_node(enum_symbol) else {
                return None;
            };
            if decl.members.iter().any(|m| m.wire == *wire) {
                TypedValue::Enum {
                    symbol: symbol.clone(),
                    wire: wire.clone(),
                }
            } else {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App003,
                    format!(
                        "default \"{wire}\" on field '{}' of '{}' is not a wire value of the referenced enum",
                        field.name, owner.qualified_id
                    ),
                ));
                return None;
            }
        }
        _ => {
            mismatch(diags);
            return None;
        }
    };
    Some(typed)
}
