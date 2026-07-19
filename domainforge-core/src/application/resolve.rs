//! AST-to-contract reference/type resolution and the public entry point
//! (reference §7/§8). Every reference resolves through
//! `ResolvedModuleSet::symbols`; nothing scans the AST by name.

use crate::application::canonical::nfc;
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
    let envelope = crate::application::envelope::build_envelope(&resolved, &contract);
    let closure_hash = crate::application::envelope::semantic_closure_hash(&envelope);
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
        self_hash: zero_hash(),
        semantic_closure_hash: closure_hash,
        contract,
    };
    doc.self_hash = crate::application::canonical::document_self_hash(&doc);
    Ok(doc)
}

/// Build a Graph from the same resolved module set the application contract
/// consumes (D3): one resolution, shared existing-concept identity, no
/// independent module traversal.
pub fn resolve_application_graph(
    entry_logical_path: &str,
    sources_json: &str,
) -> Result<crate::graph::Graph, Vec<ApplicationDiagnostic>> {
    let sources = SourceMap::parse_json(sources_json)?;
    let set = resolve_source_map(entry_logical_path, &sources)?;
    build_graph_from_set(&set)
}

pub(crate) fn build_graph_from_set(
    set: &ResolvedModuleSet,
) -> Result<crate::graph::Graph, Vec<ApplicationDiagnostic>> {
    // One conversion per effective namespace, so every declaration keeps its
    // authoring module's concept identity (D3). Namespace-less modules use
    // their logical ID, exactly like contract resolution.
    let mut groups: indexmap::IndexMap<String, crate::parser::ast::Ast> = indexmap::IndexMap::new();
    for module in &set.modules {
        let namespace = module
            .ast
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| module.logical_id.clone());
        let group = groups.entry(namespace.clone()).or_insert_with(|| {
            let mut ast = module.ast.clone();
            ast.metadata.namespace = Some(namespace);
            ast.declarations = Vec::new();
            ast
        });
        group
            .declarations
            .extend(module.ast.declarations.iter().cloned());
    }
    let mut graph = crate::graph::Graph::new();
    for (_, ast) in groups {
        let converted = crate::parser::ast::ast_to_graph_with_options(
            ast,
            &crate::parser::ParseOptions::default(),
        )
        .map_err(|e| {
            vec![ApplicationDiagnostic::new(
                ApplicationDiagnosticCode::App015,
                format!("resolved closure failed graph construction: {e}"),
            )]
        })?;
        graph.absorb(converted);
    }
    Ok(graph)
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

pub(crate) struct Ctx<'a> {
    set: &'a ResolvedModuleSet,
    bindings: Vec<ModuleBindings>,
}

const REFERENCEABLE_SLUGS: &[&str] = &[
    "record", "enum", "entity", "unit", "pattern", "role", "policy",
];

impl<'a> Ctx<'a> {
    pub(crate) fn new(set: &'a ResolvedModuleSet) -> Self {
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
        // The resolver pushed exactly one import edge per import statement,
        // in statement order; a resolved set therefore maps each authored
        // import to its edge positionally — no specifier re-matching (D3).
        let mut edges_by_importer: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &set.import_graph {
            edges_by_importer
                .entry(edge.importer.as_str())
                .or_default()
                .push(edge.imported.as_str());
        }
        let bindings = set
            .modules
            .iter()
            .map(|module| {
                let mut named = HashMap::new();
                let mut wildcard = HashMap::new();
                let edges = edges_by_importer.get(module.logical_id.as_str());
                for (index, import) in module.ast.metadata.imports.iter().enumerate() {
                    let Some(target_id) = edges.and_then(|e| e.get(index).copied()) else {
                        continue;
                    };
                    let Some(target_ns) = namespace_of.get(target_id) else {
                        continue;
                    };
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
        self.find_symbol_in(namespace, name, REFERENCEABLE_SLUGS)
    }

    fn find_symbol_in(
        &self,
        namespace: &str,
        name: &str,
        slugs: &'static [&'static str],
    ) -> Option<(&ResolvedSymbol, &'static str)> {
        slugs.iter().find_map(|slug| {
            self.set
                .symbols
                .get(&format!("{namespace}.{slug}.{name}"))
                .map(|s| (s, *slug))
        })
    }

    /// Resolve a raw possibly alias-qualified reference against specific
    /// declaration slugs, returning the exact target identity (§8 envelope
    /// references). `None` when the reference does not resolve.
    pub(crate) fn resolve_concept(
        &self,
        module_index: usize,
        raw: &str,
        slugs: &'static [&'static str],
    ) -> Option<ConceptId> {
        let (alias, name) = split_symbol_ref(raw);
        let bindings = &self.bindings[module_index];
        let found = match alias {
            Some(alias) => {
                let ns = bindings.wildcard.get(alias)?;
                self.find_symbol_in(ns, name, slugs)
                    .filter(|(symbol, _)| symbol.exported)
            }
            None => {
                if let Some((ns, original)) = bindings.named.get(name) {
                    self.find_symbol_in(ns, original, slugs)
                } else {
                    self.find_symbol_in(&bindings.namespace, name, slugs)
                }
            }
        };
        found.map(|(symbol, _)| self.symbol_concept(symbol))
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

    pub(crate) fn module_namespace(&self, module_index: usize) -> String {
        let module = &self.set.modules[module_index];
        module
            .ast
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| module.logical_id.clone())
    }

    /// Exact resolved identity of a symbol: its authoring module's namespace
    /// plus its declared name — never a local alias (D3).
    fn symbol_concept(&self, symbol: &ResolvedSymbol) -> ConceptId {
        let name = declared_name(self.decl_node(symbol)).unwrap_or_default();
        ConceptId::from_concept(&self.module_namespace(symbol.module_index), name)
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

/// The name a declaration itself declares (concept identity input).
fn declared_name(node: &past::AstNode) -> Option<&str> {
    match node {
        past::AstNode::Entity { name, .. }
        | past::AstNode::Role { name, .. }
        | past::AstNode::Pattern { name, .. }
        | past::AstNode::Policy { name, .. }
        | past::AstNode::Dimension { name } => Some(name),
        past::AstNode::UnitDeclaration { symbol, .. } => Some(symbol),
        past::AstNode::Record(decl) => Some(&decl.name),
        past::AstNode::Enum(decl) => Some(&decl.name),
        past::AstNode::Operation(decl) => Some(&decl.name),
        _ => None,
    }
}

/// Split a raw `symbol_ref` text (`Name` or `alias.Name`) into parts.
fn split_symbol_ref(raw: &str) -> (Option<&str>, &str) {
    match raw.split_once('.') {
        Some((alias, name)) if !alias.is_empty() && !name.is_empty() => (Some(alias), name),
        _ => (None, raw),
    }
}

pub(crate) fn build_contract(
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
                    name: nfc(&decl.name),
                    members: decl
                        .members
                        .iter()
                        .map(|m| EnumMember {
                            name: nfc(&m.name),
                            wire: nfc(&m.wire),
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
                    name: nfc(&decl.name),
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
                    contract.entities.push(EntityContract {
                        concept_id: ctx.symbol_concept(symbol),
                        name: name.clone(),
                        key_field: nfc(&key_field),
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

    // §6 closure order: contract collections are sets, sorted by canonical
    // identity; only field/member/failure/binding lists keep authored order.
    contract.enums.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    contract.records.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    contract.entities.sort_by_key(|e| e.concept_id.to_string());
    contract.operations.sort_by(|a, b| a.id.0.cmp(&b.id.0));

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
                role: ctx.symbol_concept(role_symbol),
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
                    let concept_id = ctx.symbol_concept(s);
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
                        ctx.symbol_concept(s)
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
        name: nfc(&decl.name),
        intent: nfc(&intent),
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
                meaning: nfc(&meaning),
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
            let constraints =
                resolve_constraints(ctx, symbol.module_index, &field_type, field, site, diags);
            let default = if is_entity {
                match &field.default {
                    Some(expr) => resolve_default(
                        ctx,
                        symbol,
                        &field_type,
                        &constraints,
                        expr,
                        field,
                        site,
                        diags,
                    ),
                    None => None,
                }
            } else {
                None // record defaults already reported by check_record_defaults
            };
            Some(FieldContract {
                name: nfc(&field.name),
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
                    unit: ctx.symbol_concept(symbol),
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
                    entity: ctx.symbol_concept(symbol),
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

/// True when `value` fits the §6 decimal envelope (28 significant digits;
/// rust_decimal already caps scale at 28).
fn decimal_fits(value: &rust_decimal::Decimal) -> bool {
    value.mantissa().unsigned_abs().to_string().len() <= 28
}

fn describe_field_type(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::Scalar { scalar } => match scalar {
            ScalarType::String => "string",
            ScalarType::Int => "int",
            ScalarType::Decimal => "decimal",
            ScalarType::Bool => "bool",
            ScalarType::Timestamp => "timestamp",
            ScalarType::Uuid => "uuid",
        },
        FieldType::Quantity { .. } => "quantity",
        FieldType::EntityRef { .. } => "ref",
        FieldType::Enum { .. } => "enum",
        FieldType::List { .. } => "list",
    }
}

/// Reference §4/§5 constraint semantics (APP012): applicability per field
/// type, at most one member per bound family, non-empty intervals under the
/// inclusivity rules, the 28-digit decimal envelope, and pattern resolution.
/// Stored decimal bounds are scale-normalized so equal values share one
/// canonical spelling.
fn resolve_constraints(
    ctx: &Ctx,
    module_index: usize,
    field_type: &FieldType,
    field: &past::FieldDecl,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Vec<FieldConstraint> {
    use past::FieldConstraintDecl as C;
    let numeric = matches!(
        field_type,
        FieldType::Scalar {
            scalar: ScalarType::Int | ScalarType::Decimal
        } | FieldType::Quantity { .. }
    );
    let stringy = matches!(
        field_type,
        FieldType::Scalar {
            scalar: ScalarType::String
        }
    );
    let listy = matches!(field_type, FieldType::List { .. });
    let app012 = |msg: String, diags: &mut Vec<ApplicationDiagnostic>| {
        diags.push(site.diag(ApplicationDiagnosticCode::App012, msg));
    };
    let inapplicable = |slug: &str, diags: &mut Vec<ApplicationDiagnostic>| {
        app012(
            format!(
                "constraint '{slug}' on field '{}' does not apply to its {} type",
                field.name,
                describe_field_type(field_type)
            ),
            diags,
        );
    };
    let u32_bound = |value: u64, diags: &mut Vec<ApplicationDiagnostic>| -> Option<u32> {
        u32::try_from(value).ok().or_else(|| {
            app012(
                format!(
                    "length/item bound {value} on field '{}' overflows",
                    field.name
                ),
                diags,
            );
            None
        })
    };
    // (slug, applicable) per bound family; > 1 member is a duplicate.
    let mut family_counts: HashMap<&'static str, u32> = HashMap::new();
    let mut lower: Option<(rust_decimal::Decimal, bool)> = None; // (value, inclusive)
    let mut upper: Option<(rust_decimal::Decimal, bool)> = None;
    let mut min_len: Option<u32> = None;
    let mut max_len: Option<u32> = None;
    let mut min_items: Option<u32> = None;
    let mut max_items: Option<u32> = None;
    let mut resolved = Vec::new();

    for constraint in &field.constraints {
        let (slug, family, applicable) = match constraint {
            C::Min(_) => ("min", "lower bound", numeric),
            C::ExclusiveMin(_) => ("exclusive_min", "lower bound", numeric),
            C::Max(_) => ("max", "upper bound", numeric),
            C::ExclusiveMax(_) => ("exclusive_max", "upper bound", numeric),
            C::MinLength(_) => ("min_length", "min_length", stringy),
            C::MaxLength(_) => ("max_length", "max_length", stringy),
            C::MinItems(_) => ("min_items", "min_items", listy),
            C::MaxItems(_) => ("max_items", "max_items", listy),
            C::Pattern(_) => ("pattern", "pattern", stringy),
        };
        if !applicable {
            inapplicable(slug, diags);
            continue;
        }
        let count = family_counts.entry(family).or_insert(0);
        *count += 1;
        if *count > 1 {
            app012(
                format!(
                    "field '{}' declares more than one {family} constraint",
                    field.name
                ),
                diags,
            );
            continue;
        }
        let decimal_bound =
            |v: &rust_decimal::Decimal, diags: &mut Vec<ApplicationDiagnostic>| -> Option<_> {
                if decimal_fits(v) {
                    Some(v.normalize())
                } else {
                    app012(
                        format!(
                            "bound on field '{}' exceeds 28 significant decimal digits",
                            field.name
                        ),
                        diags,
                    );
                    None
                }
            };
        match constraint {
            C::Min(v) => {
                let Some(v) = decimal_bound(v, diags) else {
                    continue;
                };
                lower = Some((v, true));
                resolved.push(FieldConstraint::Min { value: v });
            }
            C::ExclusiveMin(v) => {
                let Some(v) = decimal_bound(v, diags) else {
                    continue;
                };
                lower = Some((v, false));
                resolved.push(FieldConstraint::ExclusiveMin { value: v });
            }
            C::Max(v) => {
                let Some(v) = decimal_bound(v, diags) else {
                    continue;
                };
                upper = Some((v, true));
                resolved.push(FieldConstraint::Max { value: v });
            }
            C::ExclusiveMax(v) => {
                let Some(v) = decimal_bound(v, diags) else {
                    continue;
                };
                upper = Some((v, false));
                resolved.push(FieldConstraint::ExclusiveMax { value: v });
            }
            C::MinLength(v) => {
                if let Some(v) = u32_bound(*v, diags) {
                    min_len = Some(v);
                    resolved.push(FieldConstraint::MinLength { value: v });
                }
            }
            C::MaxLength(v) => {
                if let Some(v) = u32_bound(*v, diags) {
                    max_len = Some(v);
                    resolved.push(FieldConstraint::MaxLength { value: v });
                }
            }
            C::MinItems(v) => {
                if let Some(v) = u32_bound(*v, diags) {
                    min_items = Some(v);
                    resolved.push(FieldConstraint::MinItems { value: v });
                }
            }
            C::MaxItems(v) => {
                if let Some(v) = u32_bound(*v, diags) {
                    max_items = Some(v);
                    resolved.push(FieldConstraint::MaxItems { value: v });
                }
            }
            C::Pattern(raw) => {
                let (alias, name) = split_symbol_ref(raw);
                match ctx.resolve_ref(module_index, alias, name) {
                    Some((symbol, "pattern")) => resolved.push(FieldConstraint::Pattern {
                        pattern: ctx.symbol_concept(symbol),
                    }),
                    _ => app012(
                        format!(
                            "pattern '{raw}' on field '{}' does not resolve to a declared SEA Pattern",
                            field.name
                        ),
                        diags,
                    ),
                }
            }
        }
    }

    // Interval rules: quantity bounds share the field's declared unit, so the
    // §4 base-unit normalization cancels out and values compare directly.
    if let (Some((lo, lo_inclusive)), Some((hi, hi_inclusive))) = (lower, upper) {
        if lo > hi || (lo == hi && !(lo_inclusive && hi_inclusive)) {
            app012(
                format!(
                    "bounds on field '{}' describe an empty interval under the inclusivity rules",
                    field.name
                ),
                diags,
            );
        }
    }
    for (label, min, max) in [("length", min_len, max_len), ("item", min_items, max_items)] {
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                app012(
                    format!(
                        "{label} bounds on field '{}' describe an empty interval",
                        field.name
                    ),
                    diags,
                );
            }
        }
    }
    resolved
}

/// The declared unit of a quantity field: (namespace, factor, base symbol).
fn quantity_unit_info(
    ctx: &Ctx,
    module_index: usize,
    field: &past::FieldDecl,
) -> Option<(String, rust_decimal::Decimal, String)> {
    let past::FieldType::Quantity(r) = &field.field_type else {
        return None;
    };
    let (symbol, _) = ctx.resolve_ref(module_index, r.alias.as_deref(), &r.symbol)?;
    let past::AstNode::UnitDeclaration {
        factor, base_unit, ..
    } = ctx.decl_node(symbol)
    else {
        return None;
    };
    Some((
        ctx.module_namespace(symbol.module_index),
        *factor,
        base_unit.clone(),
    ))
}

/// Numeric magnitude a default contributes to bound checks. Quantity bounds
/// are authored in the field's declared unit, so the stored base value is
/// divided back through the declared factor.
fn default_magnitude(
    typed: &TypedValue,
    unit_factor: Option<rust_decimal::Decimal>,
) -> Option<rust_decimal::Decimal> {
    match typed {
        TypedValue::Int(i) => Some(rust_decimal::Decimal::from(*i)),
        TypedValue::Decimal(d) => Some(*d),
        TypedValue::Quantity { base_value, .. } => {
            let factor = unit_factor?;
            base_value.checked_div(factor)
        }
        _ => None,
    }
}

/// First constraint the default violates, as display text (reference §4:
/// defaults are validated against every field constraint).
fn default_constraint_violation(
    ctx: &Ctx,
    module_index: usize,
    typed: &TypedValue,
    constraints: &[FieldConstraint],
    field: &past::FieldDecl,
) -> Option<String> {
    let magnitude = default_magnitude(
        typed,
        quantity_unit_info(ctx, module_index, field).map(|(_, f, _)| f),
    );
    let text = match typed {
        TypedValue::String(s) => Some(s.as_str()),
        _ => None,
    };
    for constraint in constraints {
        let violated = match constraint {
            FieldConstraint::Min { value } => magnitude.is_some_and(|m| m < *value),
            FieldConstraint::Max { value } => magnitude.is_some_and(|m| m > *value),
            FieldConstraint::ExclusiveMin { value } => magnitude.is_some_and(|m| m <= *value),
            FieldConstraint::ExclusiveMax { value } => magnitude.is_some_and(|m| m >= *value),
            FieldConstraint::MinLength { value } => {
                text.is_some_and(|t| t.chars().count() < *value as usize)
            }
            FieldConstraint::MaxLength { value } => {
                text.is_some_and(|t| t.chars().count() > *value as usize)
            }
            FieldConstraint::MinItems { .. } | FieldConstraint::MaxItems { .. } => false,
            FieldConstraint::Pattern { .. } => text.is_some_and(|t| {
                // Recover the declared regex through the authored pattern ref.
                field.constraints.iter().any(|c| {
                    let past::FieldConstraintDecl::Pattern(raw) = c else {
                        return false;
                    };
                    let (alias, name) = split_symbol_ref(raw);
                    let Some((symbol, "pattern")) = ctx.resolve_ref(module_index, alias, name)
                    else {
                        return false;
                    };
                    let past::AstNode::Pattern { regex, .. } = ctx.decl_node(symbol) else {
                        return false;
                    };
                    !regex::Regex::new(regex).is_ok_and(|re| re.is_match(t))
                })
            }),
        };
        if violated {
            return Some(
                serde_json::to_value(constraint)
                    .ok()
                    .and_then(|v| v["kind"].as_str().map(str::to_string))
                    .unwrap_or_else(|| "constraint".to_string()),
            );
        }
    }
    None
}

/// Convert an authored entity-field default literal into a normalized
/// `TypedValue` that satisfies the resolved field type and every field
/// constraint (reference §4; APP003 on violation). Only required, non-key
/// fields of scalar, quantity, or enum type may carry a default.
#[allow(clippy::too_many_arguments)]
fn resolve_default(
    ctx: &Ctx,
    owner: &ResolvedSymbol,
    field_type: &FieldType,
    constraints: &[FieldConstraint],
    expr: &Expression,
    field: &past::FieldDecl,
    site: DeclSite,
    diags: &mut Vec<ApplicationDiagnostic>,
) -> Option<TypedValue> {
    let app003 = |msg: String, diags: &mut Vec<ApplicationDiagnostic>| {
        diags.push(site.diag(ApplicationDiagnosticCode::App003, msg));
    };
    if field.is_optional {
        app003(
            format!(
                "optional field '{}' declares a default; defaults are required-field-only",
                field.name
            ),
            diags,
        );
        return None;
    }
    if field.is_key {
        return None; // APP005 already reported by check_entity_key
    }
    if matches!(
        field_type,
        FieldType::EntityRef { .. } | FieldType::List { .. }
    ) {
        app003(
            format!(
                "field '{}' declares a default on a {} type; refs and lists must not",
                field.name,
                describe_field_type(field_type)
            ),
            diags,
        );
        return None;
    }
    let mismatch = |diags: &mut Vec<ApplicationDiagnostic>| {
        diags.push(site.diag(
            ApplicationDiagnosticCode::App003,
            format!(
                "default on field '{}' does not satisfy its declared type",
                field.name
            ),
        ));
    };

    // Quantity defaults lower to the base-unit value (§6).
    if let FieldType::Quantity { .. } = field_type {
        let Some((unit_namespace, field_factor, base_symbol)) =
            quantity_unit_info(ctx, owner.module_index, field)
        else {
            mismatch(diags);
            return None;
        };
        let authored = match expr {
            Expression::Literal(serde_json::Value::Number(n)) => Some((
                rust_decimal::Decimal::from_str(&n.to_string()).ok()?,
                field_factor,
            )),
            Expression::QuantityLiteral { value, unit } => {
                // Any unit sharing the field unit's base is convertible.
                match ctx.resolve_ref(owner.module_index, None, unit) {
                    Some((s, "unit")) => match ctx.decl_node(s) {
                        past::AstNode::UnitDeclaration {
                            factor, base_unit, ..
                        } if *base_unit == base_symbol => Some((*value, *factor)),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        };
        let Some((value, factor)) = authored else {
            mismatch(diags);
            return None;
        };
        let Some(base_value) = value.checked_mul(factor).filter(decimal_fits) else {
            diags.push(site.diag(
                ApplicationDiagnosticCode::App012,
                format!(
                    "base-unit normalization of the default on field '{}' overflows the decimal envelope",
                    field.name
                ),
            ));
            return None;
        };
        let typed = TypedValue::Quantity {
            base_value: base_value.normalize(),
            unit: ConceptId::from_concept(&unit_namespace, &base_symbol),
        };
        if let Some(kind) =
            default_constraint_violation(ctx, owner.module_index, &typed, constraints, field)
        {
            app003(
                format!(
                    "default on field '{}' violates its {kind} constraint",
                    field.name
                ),
                diags,
            );
            return None;
        }
        return Some(typed);
    }

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
        ) => TypedValue::String(nfc(s)),
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
        ) => {
            // The parser carries numeric literals as JSON floats; accept any
            // integral spelling that fits i64.
            use rust_decimal::prelude::ToPrimitive;
            let value = rust_decimal::Decimal::from_str(&n.to_string())
                .ok()
                .filter(rust_decimal::Decimal::is_integer)
                .and_then(|d| d.to_i64());
            match value {
                Some(i) => TypedValue::Int(i),
                None => {
                    mismatch(diags);
                    return None;
                }
            }
        }
        (
            FieldType::Scalar {
                scalar: ScalarType::Decimal,
            },
            serde_json::Value::Number(n),
        ) => {
            let d = rust_decimal::Decimal::from_str(&n.to_string()).ok()?;
            if !decimal_fits(&d) {
                diags.push(site.diag(
                    ApplicationDiagnosticCode::App012,
                    format!(
                        "default on field '{}' exceeds 28 significant decimal digits",
                        field.name
                    ),
                ));
                return None;
            }
            TypedValue::Decimal(d.normalize())
        }
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
                    wire: nfc(wire),
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
    if let Some(kind) =
        default_constraint_violation(ctx, owner.module_index, &typed, constraints, field)
    {
        app003(
            format!(
                "default on field '{}' violates its {kind} constraint",
                field.name
            ),
            diags,
        );
        return None;
    }
    Some(typed)
}
