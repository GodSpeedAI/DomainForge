//! AST-to-contract reference/type resolution and the public entry point
//! (reference §7/§8). Every reference resolves through
//! `ResolvedModuleSet::symbols`; nothing scans the AST by name.

use crate::application::contract::*;
use crate::application::diagnostic::{
    sort_diagnostics, ApplicationDiagnostic, ApplicationDiagnosticCode,
};
use crate::application::validate::{
    check_duplicate_fields, check_entity_key, check_enum, check_record_defaults, failure_kind,
    validate_clause_shape, validate_failures, DeclSite,
};
use crate::concept_id::ConceptId;
use crate::module::resolver::{resolve_source_map, ResolvedModuleSet, ResolvedSymbol, SourceMap};
use crate::parser::ast as past;
use crate::policy::Expression;
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
    let mut doc = ApplicationContractDocument {
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
        // ponytail: placeholder zero closure hash until the envelope packet
        // (Task 12) computes semantic_closure_hash.
        self_hash: zero_hash(),
        semantic_closure_hash: zero_hash(),
        contract,
    };
    doc.self_hash = crate::application::canonical::document_self_hash(&doc);
    Ok(doc)
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

/// Reference §6 source-set hash over the authored source map.
fn source_set_hash(sources: &SourceMap) -> String {
    crate::application::canonical::source_set_hash(
        sources
            .0
            .iter()
            .map(|(id, src)| (id.as_str(), src.as_str())),
    )
    .expect("SourceMap::parse_json already rejected duplicate logical ids")
}

/// Reference §6 semantic-pack-set hash; Milestone 0 uses the empty set.
fn semantic_pack_set_hash() -> String {
    crate::application::canonical::semantic_pack_set_hash(&[]).expect("empty set has no duplicates")
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
            // Operations lower in a second pass, after every record/entity
            // contract they reference has been constructed.
            past::AstNode::Operation(_) => {}
            _ => {}
        }
    }

    let mut closure_codes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut operations = Vec::new();
    for symbol in set.symbols.values() {
        let site = DeclSite {
            logical_module_id: &symbol.origin.logical_module_id,
            line: symbol.origin.line,
            column: symbol.origin.column,
        };
        if let past::AstNode::Operation(decl) = ctx.decl_node(symbol) {
            if let Some(op) = lower_operation(
                &ctx,
                &contract,
                symbol,
                decl,
                site,
                &mut closure_codes,
                &mut diags,
            ) {
                operations.push(op);
            }
        }
    }
    contract.operations = operations;

    if diags.is_empty() {
        Ok(contract)
    } else {
        sort_diagnostics(&mut diags);
        Err(diags)
    }
}

// ---- operation lowering (reference §4) ----

fn field<'a>(fields: &'a [FieldContract], name: &str) -> Option<&'a FieldContract> {
    fields.iter().find(|f| f.name == name)
}

/// Same field type and canonical constraint set (order-free).
fn type_and_constraints_match(a: &FieldContract, b: &FieldContract) -> bool {
    a.field_type == b.field_type
        && a.constraints.len() == b.constraints.len()
        && a.constraints.iter().all(|c| b.constraints.contains(c))
}

/// Same field type, optionality, and canonical constraint set.
fn fields_compatible(a: &FieldContract, b: &FieldContract) -> bool {
    type_and_constraints_match(a, b) && a.optional == b.optional
}

fn is_scalar(f: &FieldContract) -> bool {
    matches!(f.field_type, FieldType::Scalar { .. })
}

#[allow(clippy::too_many_arguments)]
fn lower_operation(
    ctx: &Ctx,
    contract: &ApplicationContract,
    symbol: &ResolvedSymbol,
    decl: &past::OperationDecl,
    site: DeclSite,
    closure_codes: &mut std::collections::HashSet<String>,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Option<OperationContract> {
    use past::OperationClause as C;

    let defects = validate_clause_shape(&decl.clauses);
    if !defects.is_empty() {
        diags.push(site.diag(
            ApplicationDiagnosticCode::App001,
            format!(
                "operation '{}' is not generation-ready: {}",
                decl.name,
                defects.join("; ")
            ),
        ));
        return None;
    }
    let before = diags.len();

    // Clause shape guarantees each of these appears exactly once.
    let mut intent = String::new();
    let mut actor_raw = "";
    let mut access_clause: Option<&C> = None;
    let mut input_raw = "";
    let mut output_raw = "";
    let mut state_raw = "";
    let mut effect_kind_raw = "";
    let mut effect_target_raw = "";
    let mut transaction_raw = "";
    let mut ast_failures: Vec<(String, Vec<FailureKind>, String)> = Vec::new();
    let mut idem_clause: Option<&C> = None;
    let mut conc_clause: Option<&C> = None;
    for clause in &decl.clauses {
        match clause {
            C::Intent(text) => intent = text.clone(),
            C::Actor { actor } => actor_raw = actor,
            C::AccessPublic | C::AccessPolicyGoverned { .. } => access_clause = Some(clause),
            C::Input { reference } => input_raw = reference,
            C::Output { reference } => output_raw = reference,
            C::State { reference } => state_raw = reference,
            C::Effect { kind, reference } => {
                effect_kind_raw = kind;
                effect_target_raw = reference;
            }
            C::Transaction { kind } => transaction_raw = kind,
            C::Failure {
                code,
                kinds,
                message,
            } => {
                let mut mapped = Vec::new();
                for raw in kinds {
                    match failure_kind(raw) {
                        Some(kind) => mapped.push(kind),
                        None => diags.push(site.diag(
                            ApplicationDiagnosticCode::App008,
                            format!(
                                "failure '{code}' of operation '{}' names unknown kind '{raw}'",
                                decl.name
                            ),
                        )),
                    }
                }
                ast_failures.push((code.clone(), mapped, message.clone()));
            }
            C::IdempotencyKeyed { .. }
            | C::IdempotencyInherent
            | C::IdempotencyNotApplicable { .. } => idem_clause = Some(clause),
            C::ConcurrencyUnique { .. }
            | C::ConcurrencyOptimistic { .. }
            | C::ConcurrencyReadSnapshot => conc_clause = Some(clause),
            C::Direction { .. }
            | C::EvidenceOperationTrace
            | C::LifecycleSynchronousRequestResponse => {}
        }
    }

    let effect = match effect_kind_raw {
        "creates" => EffectKind::Creates,
        "mutates" => EffectKind::Mutates,
        "reads" => EffectKind::Reads,
        other => {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App001,
                format!(
                    "effect '{other}' of operation '{}' is unsupported",
                    decl.name
                ),
            ));
            return None;
        }
    };
    let transaction = match transaction_raw {
        "single_aggregate" => TransactionBoundary::SingleAggregate,
        _ => TransactionBoundary::ReadOnly, // pairing already validated
    };

    // Actor.
    let actor = if actor_raw == "anonymous" {
        ActorRef::Anonymous
    } else {
        let (alias, name) = split_symbol_ref(actor_raw);
        match ctx.resolve_ref(symbol.module_index, alias, name) {
            Some((role_symbol, "role")) => ActorRef::Role {
                role: ConceptId::from_concept(
                    &Ctx::namespace_of_symbol(role_symbol, "role", name),
                    name,
                ),
            },
            _ => {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App002,
                    format!(
                        "actor '{actor_raw}' of operation '{}' does not resolve to a role",
                        decl.name
                    ),
                ));
                ActorRef::Anonymous
            }
        }
    };

    // Input / output / state / effect target resolution.
    let resolve_record = |label: &str, raw: &str, diags: &mut Vec<ApplicationDiagnostic>| {
        let (alias, name) = split_symbol_ref(raw);
        match ctx.resolve_ref(symbol.module_index, alias, name) {
            Some((s, "record")) => contract
                .records
                .iter()
                .find(|r| r.id.0 == s.qualified_id)
                .map(|r| (s.qualified_id.clone(), r)),
            Some((_, slug)) => {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App006,
                    format!(
                        "{label} '{raw}' of operation '{}' resolves to {slug}; a record is required",
                        decl.name
                    ),
                ));
                None
            }
            None => {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App002,
                    format!(
                        "{label} '{raw}' of operation '{}' does not resolve in the symbol table",
                        decl.name
                    ),
                ));
                None
            }
        }
    };
    let input = resolve_record("input", input_raw, diags);
    let output = resolve_record("output", output_raw, diags);

    let state = {
        let (alias, name) = split_symbol_ref(state_raw);
        match ctx.resolve_ref(symbol.module_index, alias, name) {
            Some((s, "entity")) => {
                if matches!(ctx.decl_node(s), past::AstNode::Entity { body: None, .. }) {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App011,
                        format!(
                            "state '{state_raw}' of operation '{}' lacks a typed body",
                            decl.name
                        ),
                    ));
                    None
                } else {
                    let namespace = Ctx::namespace_of_symbol(s, "entity", name);
                    let concept_id = ConceptId::from_concept(&namespace, name);
                    contract
                        .entities
                        .iter()
                        .find(|e| e.concept_id == concept_id)
                        .map(|e| (s.qualified_id.clone(), e))
                }
            }
            _ => {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App002,
                    format!(
                        "state '{state_raw}' of operation '{}' does not resolve to an entity",
                        decl.name
                    ),
                ));
                None
            }
        }
    };
    {
        let (alias, name) = split_symbol_ref(effect_target_raw);
        let target = ctx.resolve_ref(symbol.module_index, alias, name);
        match (&state, target) {
            (Some((state_qid, _)), Some((t, "entity"))) if t.qualified_id != *state_qid => {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App011,
                    format!(
                        "effect target '{effect_target_raw}' of operation '{}' differs from state '{state_raw}'",
                        decl.name
                    ),
                ));
            }
            (Some(_), Some((_, "entity"))) => {}
            (Some(_), _) => diags.push(site.diag(
                ApplicationDiagnosticCode::App002,
                format!(
                    "effect target '{effect_target_raw}' of operation '{}' does not resolve to an entity",
                    decl.name
                ),
            )),
            (None, _) => {}
        }
    }

    // Failures.
    let input_has_constraints = input
        .as_ref()
        .is_some_and(|(_, r)| r.fields.iter().any(|f| !f.constraints.is_empty()));
    let mut required_kinds = Vec::new();
    if input_has_constraints {
        required_kinds.push(FailureKind::InputValidation);
    }
    if matches!(effect, EffectKind::Reads | EffectKind::Mutates) {
        required_kinds.push(FailureKind::MissingState);
    }
    if matches!(idem_clause, Some(C::IdempotencyKeyed { .. })) {
        required_kinds.push(FailureKind::IdempotencyConflict);
    }
    if matches!(
        conc_clause,
        Some(C::ConcurrencyUnique { .. } | C::ConcurrencyOptimistic { .. })
    ) {
        required_kinds.push(FailureKind::ConcurrencyConflict);
    }
    validate_failures(
        &decl.name,
        &ast_failures,
        &required_kinds,
        closure_codes,
        site,
        diags,
    );
    let declared_codes: Vec<&str> = ast_failures.iter().map(|(c, _, _)| c.as_str()).collect();

    // Strategies (APP009/APP010).
    let strategy_field = |field_name: &str,
                          want_int: bool,
                          need_state: bool,
                          diags: &mut Vec<ApplicationDiagnostic>| {
        let Some((_, input_rec)) = &input else { return };
        match field(&input_rec.fields, field_name) {
            Some(f) if !f.optional && is_scalar(f) => {
                if want_int
                    && !matches!(
                        f.field_type,
                        FieldType::Scalar {
                            scalar: ScalarType::Int
                        }
                    )
                {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App009,
                        format!(
                            "strategy field '{field_name}' of operation '{}' must be a required int",
                            decl.name
                        ),
                    ));
                }
                if need_state {
                    let compatible = state.as_ref().is_some_and(|(_, e)| {
                        field(&e.fields, field_name).is_some_and(|sf| {
                            sf.field_type == f.field_type && !sf.optional
                        })
                    });
                    if state.is_some() && !compatible {
                        diags.push(site.diag(
                            ApplicationDiagnosticCode::App009,
                            format!(
                                "strategy field '{field_name}' of operation '{}' has no compatible required state field",
                                decl.name
                            ),
                        ));
                    }
                }
            }
            _ => diags.push(site.diag(
                ApplicationDiagnosticCode::App009,
                format!(
                    "strategy field '{field_name}' of operation '{}' must be a required scalar input field",
                    decl.name
                ),
            )),
        }
    };
    let pairing = |what: &str, diags: &mut Vec<ApplicationDiagnostic>| {
        diags.push(site.diag(
            ApplicationDiagnosticCode::App009,
            format!(
                "{what} of operation '{}' falls outside the §4 strategy/effect pairing",
                decl.name
            ),
        ));
    };
    let idempotency = match (effect, idem_clause) {
        (EffectKind::Creates | EffectKind::Mutates, Some(C::IdempotencyKeyed { field: f })) => {
            strategy_field(f, false, false, diags);
            IdempotencyStrategy::KeyedBy { field: f.clone() }
        }
        (EffectKind::Reads, Some(C::IdempotencyInherent)) => IdempotencyStrategy::Inherent,
        (
            EffectKind::Reads,
            Some(C::IdempotencyNotApplicable {
                reason,
                explanation,
            }),
        ) => {
            if reason != "read_only" {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App010,
                    format!(
                        "not_applicable reason '{reason}' on operation '{}' is not proven by its effect",
                        decl.name
                    ),
                ));
            }
            IdempotencyStrategy::NotApplicable(NotApplicable {
                explanation: explanation.clone(),
            })
        }
        (_, Some(C::IdempotencyNotApplicable { .. })) => {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App010,
                format!(
                    "not_applicable idempotency on write operation '{}' has no reachable proof",
                    decl.name
                ),
            ));
            IdempotencyStrategy::Inherent
        }
        _ => {
            pairing("idempotency strategy", diags);
            IdempotencyStrategy::Inherent
        }
    };
    let concurrency = match (effect, conc_clause) {
        (EffectKind::Creates, Some(C::ConcurrencyUnique { field: f })) => {
            strategy_field(f, false, true, diags);
            ConcurrencyStrategy::UniqueKey { field: f.clone() }
        }
        (EffectKind::Mutates, Some(C::ConcurrencyOptimistic { field: f })) => {
            strategy_field(f, true, true, diags);
            ConcurrencyStrategy::OptimisticVersion { field: f.clone() }
        }
        (EffectKind::Reads, Some(C::ConcurrencyReadSnapshot)) => ConcurrencyStrategy::ReadSnapshot,
        _ => {
            pairing("concurrency strategy", diags);
            ConcurrencyStrategy::ReadSnapshot
        }
    };

    // State lowering and output projection (APP011).
    if let (Some((_, input_rec)), Some((_, entity))) = (&input, &state) {
        let app011 = |msg: String, diags: &mut Vec<ApplicationDiagnostic>| {
            diags.push(site.diag(ApplicationDiagnosticCode::App011, msg));
        };
        let key_lookup = |diags: &mut Vec<ApplicationDiagnostic>| match field(
            &input_rec.fields,
            &entity.key_field,
        ) {
            Some(f) if !f.optional => {}
            _ => app011(
                format!(
                    "operation '{}' has no required input field matching entity key '{}'",
                    decl.name, entity.key_field
                ),
                diags,
            ),
        };
        match effect {
            EffectKind::Creates => {
                for sf in &entity.fields {
                    match field(&input_rec.fields, &sf.name) {
                        Some(inf) => {
                            if !type_and_constraints_match(inf, sf) {
                                app011(
                                    format!(
                                        "input field '{}' of operation '{}' is not compatible with its state field",
                                        sf.name, decl.name
                                    ),
                                    diags,
                                );
                            } else if inf.optional && !sf.optional && sf.default.is_none() {
                                app011(
                                    format!(
                                        "optional input field '{}' sources required state field of operation '{}' with no default",
                                        sf.name, decl.name
                                    ),
                                    diags,
                                );
                            }
                        }
                        None if sf.default.is_none() && !sf.optional => app011(
                            format!(
                                "create of operation '{}' cannot construct required state field '{}'",
                                decl.name, sf.name
                            ),
                            diags,
                        ),
                        None => {}
                    }
                }
            }
            EffectKind::Mutates => {
                key_lookup(diags);
                let strategy_fields: Vec<&str> = [
                    match &idempotency {
                        IdempotencyStrategy::KeyedBy { field } => Some(field.as_str()),
                        _ => None,
                    },
                    match &concurrency {
                        ConcurrencyStrategy::OptimisticVersion { field } => Some(field.as_str()),
                        _ => None,
                    },
                ]
                .into_iter()
                .flatten()
                .collect();
                for inf in &input_rec.fields {
                    if inf.name == entity.key_field || strategy_fields.contains(&inf.name.as_str())
                    {
                        continue;
                    }
                    match field(&entity.fields, &inf.name) {
                        Some(sf) if fields_compatible(inf, sf) => {}
                        Some(_) => app011(
                            format!(
                                "input field '{}' of operation '{}' is not compatible with its state field",
                                inf.name, decl.name
                            ),
                            diags,
                        ),
                        None => app011(
                            format!(
                                "mutate of operation '{}' contains unmapped input field '{}'",
                                decl.name, inf.name
                            ),
                            diags,
                        ),
                    }
                }
            }
            EffectKind::Reads => key_lookup(diags),
        }
        if let Some((_, output_rec)) = &output {
            for of in &output_rec.fields {
                match field(&entity.fields, &of.name) {
                    Some(sf) if fields_compatible(of, sf) => {}
                    _ => app011(
                        format!(
                            "output field '{}' of operation '{}' cannot project a same-named compatible state field",
                            of.name, decl.name
                        ),
                        diags,
                    ),
                }
            }
        }
    }

    // Access.
    let access = match access_clause {
        Some(C::AccessPolicyGoverned { bindings }) => {
            let mut lowered = Vec::new();
            for binding in bindings {
                if binding.enforcement_point != "precondition" {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App007,
                        format!(
                            "enforcement point '{}' on operation '{}' is reserved; only precondition is executable",
                            binding.enforcement_point, decl.name
                        ),
                    ));
                    continue;
                }
                let (alias, name) = split_symbol_ref(&binding.policy);
                let policy = match ctx.resolve_ref(symbol.module_index, alias, name) {
                    Some((s, "policy")) => {
                        if let past::AstNode::Policy {
                            metadata,
                            expression,
                            ..
                        } = ctx.decl_node(s)
                        {
                            if metadata.kind != Some(past::PolicyKind::Constraint) {
                                diags.push(site.diag(
                                    ApplicationDiagnosticCode::App007,
                                    format!(
                                        "policy '{}' bound by operation '{}' is not a constraint policy",
                                        binding.policy, decl.name
                                    ),
                                ));
                            }
                            let input_fields: Vec<&str> = input
                                .as_ref()
                                .map(|(_, r)| r.fields.iter().map(|f| f.name.as_str()).collect())
                                .unwrap_or_default();
                            let state_fields: Option<Vec<&str>> = state
                                .as_ref()
                                .map(|(_, e)| e.fields.iter().map(|f| f.name.as_str()).collect());
                            let mut resolve_role = |raw: &str| {
                                let (alias, name) = split_symbol_ref(raw);
                                matches!(
                                    ctx.resolve_ref(s.module_index, alias, name),
                                    Some((_, "role"))
                                )
                            };
                            for defect in crate::application::policy_context::validate_policy_subset(
                                expression,
                                &input_fields,
                                state_fields.as_deref(),
                                &mut resolve_role,
                            ) {
                                diags.push(site.diag(
                                    ApplicationDiagnosticCode::App007,
                                    format!(
                                        "policy '{}' bound by operation '{}' is not evaluable: {defect}",
                                        binding.policy, decl.name
                                    ),
                                ));
                            }
                        }
                        ConceptId::from_concept(&Ctx::namespace_of_symbol(s, "policy", name), name)
                    }
                    _ => {
                        diags.push(site.diag(
                            ApplicationDiagnosticCode::App007,
                            format!(
                                "policy '{}' bound by operation '{}' does not resolve",
                                binding.policy, decl.name
                            ),
                        ));
                        continue;
                    }
                };
                if !declared_codes.contains(&binding.failure_code.as_str()) {
                    diags.push(site.diag(
                        ApplicationDiagnosticCode::App008,
                        format!(
                            "policy binding of operation '{}' names undeclared failure code '{}'",
                            decl.name, binding.failure_code
                        ),
                    ));
                }
                lowered.push(PolicyBinding {
                    policy,
                    enforcement_point: EnforcementPoint::Precondition,
                    failure_code: binding.failure_code.clone(),
                });
            }
            AccessMode::PolicyGoverned { bindings: lowered }
        }
        _ => AccessMode::Public,
    };

    if diags.len() != before {
        return None;
    }
    Some(OperationContract {
        id: ApplicationSymbolId(symbol.qualified_id.clone()),
        name: decl.name.clone(),
        intent,
        direction: Direction::Inbound,
        actor,
        access,
        input: ApplicationSymbolId(input.map(|(id, _)| id).unwrap_or_default()),
        output: ApplicationSymbolId(output.map(|(id, _)| id).unwrap_or_default()),
        state: state
            .map(|(_, e)| e.concept_id.clone())
            .unwrap_or_else(|| ConceptId::from_concept("", "")),
        effect,
        transaction,
        failures: ast_failures
            .into_iter()
            .map(|(code, kinds, meaning)| FailureContract {
                code,
                kinds,
                meaning,
            })
            .collect(),
        idempotency,
        concurrency,
        evidence: EvidenceKind::OperationTrace,
        lifecycle: LifecycleKind::SynchronousRequestResponse,
    })
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
