use crate::application::diagnostic::{
    ApplicationDiagnostic, APP014_IMPORT_CYCLE, APP014_NOT_EXPORTED, APP014_SYMBOL_COLLISION,
    APP014_UNRESOLVED_ALIAS, APP014_UNRESOLVED_SPECIFIER,
};
use crate::application::ApplicationDiagnosticCode;
use crate::error::fuzzy::levenshtein_distance;
use crate::parser::ast::{Ast, AstNode, ImportDecl, ImportSpecifier};
use crate::parser::{parse_source, ParseError, ParseOptions, ParseResult};
use crate::registry::{NamespaceBinding, NamespaceRegistry};
use indexmap::IndexMap;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub namespace: String,
    pub exports: HashSet<String>,
    pub ast: Ast,
}

#[derive(Debug)]
pub struct ModuleResolver<'a> {
    registry: &'a NamespaceRegistry,
    bindings: Vec<NamespaceBinding>,
    loaded_modules: HashMap<PathBuf, ModuleInfo>,
    visiting: HashSet<PathBuf>,
}

impl<'a> ModuleResolver<'a> {
    pub fn new(registry: &'a NamespaceRegistry) -> ParseResult<Self> {
        let bindings = registry
            .resolve_files()
            .map_err(|e| ParseError::GrammarError(e.to_string()))?;
        Ok(Self {
            registry,
            bindings,
            loaded_modules: HashMap::new(),
            visiting: HashSet::new(),
        })
    }

    pub fn validate_entry(
        &mut self,
        entry_path: impl AsRef<Path>,
        source: &str,
    ) -> ParseResult<Ast> {
        let path = entry_path
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| entry_path.as_ref().to_path_buf());
        let ast = parse_source(source)?;
        self.visit(&path, &ast)?;
        Ok(ast)
    }

    pub fn validate_dependencies(
        &mut self,
        entry_path: impl AsRef<Path>,
        ast: &Ast,
    ) -> ParseResult<()> {
        let path = entry_path
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| entry_path.as_ref().to_path_buf());
        self.visit(&path, ast)
    }

    fn visit(&mut self, path: &Path, ast: &Ast) -> ParseResult<()> {
        let canonical = if path.to_string_lossy().starts_with("__std__") {
            path.to_path_buf()
        } else {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        };

        if self.visiting.contains(&canonical) {
            // Build the cycle path from currently visiting modules
            let cycle: Vec<String> = self
                .visiting
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .chain(std::iter::once(canonical.to_string_lossy().to_string()))
                .collect();
            return Err(ParseError::circular_dependency(cycle));
        }
        if self.loaded_modules.contains_key(&canonical) {
            return Ok(());
        }

        self.visiting.insert(canonical.clone());
        let namespace = ast.metadata.namespace.clone().unwrap_or_else(|| {
            self.registry
                .namespace_for(path)
                .unwrap_or(self.registry.default_namespace())
                .to_string()
        });
        let exports = collect_exports(ast);

        for import in &ast.metadata.imports {
            let dep_path = self.resolve_module_path(&import.from_module)?;
            let dependency_ast = self.parse_file(&dep_path)?;
            self.visit(&dep_path, &dependency_ast)?;
            self.validate_import_targets(import, &dep_path)?;
        }

        self.loaded_modules.insert(
            canonical.clone(),
            ModuleInfo {
                namespace,
                exports,
                ast: ast.clone(),
            },
        );
        self.visiting.remove(&canonical);
        Ok(())
    }

    fn parse_file(&self, path: &Path) -> ParseResult<Ast> {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("__std__") {
            let namespace = path_str.strip_prefix("__std__").unwrap();
            let content = match namespace {
                "std" | "std:core" => include_str!("../../std/core.sea"),
                "std:http" => include_str!("../../std/http.sea"),
                "std:aws" => include_str!("../../std/aws.sea"),
                _ => {
                    return Err(ParseError::GrammarError(format!(
                        "Unknown std module: {}",
                        namespace
                    )))
                }
            };
            return parse_source(content);
        }

        let content = fs::read_to_string(path).map_err(|e| {
            ParseError::GrammarError(format!("Failed to read module {}: {}", path.display(), e))
        })?;
        parse_source(&content)
    }

    fn resolve_module_path(&self, namespace: &str) -> ParseResult<PathBuf> {
        if namespace == "std" || namespace.starts_with("std:") {
            return Ok(PathBuf::from(format!("__std__{}", namespace)));
        }

        self.bindings
            .iter()
            .find(|binding| binding.namespace == namespace)
            .map(|binding| binding.path.clone())
            .ok_or_else(|| {
                // Find similar namespace for suggestion
                let suggestion = self.suggest_similar_namespace(namespace);
                ParseError::namespace_not_found(namespace, 0, 0, suggestion)
            })
    }

    fn validate_import_targets(&self, import: &ImportDecl, dep_path: &Path) -> ParseResult<()> {
        let canonical = if dep_path.to_string_lossy().starts_with("__std__") {
            dep_path.to_path_buf()
        } else {
            dep_path
                .canonicalize()
                .unwrap_or_else(|_| dep_path.to_path_buf())
        };
        let module = self.loaded_modules.get(&canonical).ok_or_else(|| {
            ParseError::GrammarError(format!(
                "Expected module '{}' to be loaded before validating imports",
                dep_path.display()
            ))
        })?;

        match &import.specifier {
            ImportSpecifier::Wildcard(_) => Ok(()),
            ImportSpecifier::Named(items) => {
                for item in items {
                    if !module.exports.contains(&item.name) {
                        return Err(ParseError::symbol_not_exported(
                            &item.name,
                            &module.namespace,
                            0, // TODO: Extract line from import.location if available
                            0,
                            module.exports.iter().cloned().collect(),
                        ));
                    }
                }
                Ok(())
            }
        }
    }

    /// Find a similar namespace name for error suggestions using Levenshtein distance
    fn suggest_similar_namespace(&self, target: &str) -> Option<String> {
        let available: Vec<&str> = self.bindings.iter().map(|b| b.namespace.as_str()).collect();

        available
            .iter()
            .filter_map(|ns| {
                let distance = levenshtein_distance(target, ns);
                if distance <= 2 {
                    Some((*ns, distance))
                } else {
                    None
                }
            })
            .min_by_key(|(_, d)| *d)
            .map(|(ns, _)| ns.to_string())
    }
}

fn collect_exports(ast: &Ast) -> HashSet<String> {
    let mut exports = HashSet::new();
    for node in &ast.declarations {
        if let AstNode::Export(inner) = &node.node {
            if let Some(name) = declaration_name(&inner.node) {
                exports.insert(name.to_string());
            }
        }
    }
    exports
}

fn declaration_name(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::Entity { name, .. }
        | AstNode::Resource { name, .. }
        | AstNode::Flow {
            resource_name: name,
            ..
        }
        | AstNode::Pattern { name, .. }
        | AstNode::Role { name, .. }
        | AstNode::Relation { name, .. }
        | AstNode::Dimension { name }
        | AstNode::UnitDeclaration { symbol: name, .. }
        | AstNode::Policy { name, .. }
        | AstNode::Instance { name, .. }
        | AstNode::ConceptChange { name, .. }
        | AstNode::Metric { name, .. }
        | AstNode::MappingDecl { name, .. }
        | AstNode::ProjectionDecl { name, .. }
        | AstNode::Cell { name, .. }
        | AstNode::SystemDependency { name, .. }
        | AstNode::Runtime { name, .. }
        | AstNode::Tool { name, .. }
        | AstNode::DependencySet { name, .. }
        | AstNode::Service { name, .. }
        | AstNode::Mount { name, .. }
        | AstNode::Endpoint { name, .. }
        | AstNode::NetworkFlow { name, .. }
        | AstNode::Credential { name, .. } => Some(name),
        AstNode::Export(inner) => declaration_name(&inner.node),
        AstNode::Record(r) => Some(&r.name),
        AstNode::Enum(e) => Some(&e.name),
        AstNode::Operation(o) => Some(&o.name),
    }
}

// ---- Source-map closure resolution (ADR-013, reference §7/§8) ----
//
// `ResolvedModuleSet` is the single resolver output consumed by both Graph
// and ApplicationContract construction. It stays crate-private; the public
// boundary is `application::resolve`.

#[derive(Debug, Clone, PartialEq, Eq)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct OriginRef {
    pub logical_module_id: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) enum ResolvedDeclarationKind {
    ExistingConcept,
    Record,
    Enum,
    Operation,
}

#[derive(Debug, Clone)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct ResolvedSymbol {
    pub qualified_id: String,
    pub kind: ResolvedDeclarationKind,
    pub exported: bool,
    pub origin: OriginRef,
    pub module_index: usize,
    pub declaration_index: usize,
}

#[derive(Debug, Clone)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct ResolvedModule {
    pub logical_id: String,
    pub source_hash: String, // "sha256:<lowercase-hex>"
    pub ast: Ast,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct ImportEdge {
    pub importer: String,
    pub imported: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct AliasBinding {
    pub importer: String,
    pub alias: String,
    pub target: String,
}

#[derive(Debug, Clone)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct ResolvedModuleSet {
    pub modules: Vec<ResolvedModule>,
    pub import_graph: Vec<ImportEdge>,
    pub symbols: IndexMap<String, ResolvedSymbol>,
    pub aliases: Vec<AliasBinding>,
}

/// Ordered, validated in-memory source set keyed by normalized logical path.
#[derive(Debug, Clone)]
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) struct SourceMap(pub(crate) IndexMap<String, String>);

impl SourceMap {
    #[allow(dead_code)]
    /// Parse `sources_json` per reference §7: reject non-object input,
    /// duplicate raw keys, duplicate normalized logical IDs, non-string
    /// values, and invalid logical paths.
    pub(crate) fn parse_json(json: &str) -> Result<Self, Vec<ApplicationDiagnostic>> {
        let pairs = raw_json_object_pairs(json).map_err(|msg| {
            vec![ApplicationDiagnostic::closure_error(
                APP014_UNRESOLVED_SPECIFIER,
                format!("sources_json is not a valid JSON object of strings: {msg}"),
            )]
        })?;
        let mut raw_seen = HashSet::new();
        let mut map = IndexMap::new();
        let mut diags = Vec::new();
        for (raw_key, value) in pairs {
            if !raw_seen.insert(raw_key.clone()) {
                diags.push(ApplicationDiagnostic::closure_error(
                    APP014_SYMBOL_COLLISION,
                    format!("duplicate raw source key '{raw_key}'"),
                ));
                continue;
            }
            let normalized = match normalize_logical_path(&raw_key) {
                Ok(p) => p,
                Err(reason) => {
                    diags.push(ApplicationDiagnostic::closure_error(
                        APP014_UNRESOLVED_SPECIFIER,
                        format!("invalid logical path '{raw_key}': {reason}"),
                    ));
                    continue;
                }
            };
            if map.insert(normalized.clone(), value).is_some() {
                diags.push(ApplicationDiagnostic::closure_error(
                    APP014_SYMBOL_COLLISION,
                    format!("duplicate normalized logical ID '{normalized}'"),
                ));
            }
        }
        if diags.is_empty() {
            Ok(Self(map))
        } else {
            Err(diags)
        }
    }
}

/// Deserialize a JSON object into ordered (key, string-value) pairs without
/// silently collapsing duplicate keys the way `serde_json` maps do.
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn raw_json_object_pairs(json: &str) -> Result<Vec<(String, String)>, String> {
    struct Pairs;
    impl<'de> serde::de::Visitor<'de> for Pairs {
        type Value = Vec<(String, String)>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a JSON object mapping logical paths to source strings")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some((k, v)) = map.next_entry::<String, serde_json::Value>()? {
                match v {
                    serde_json::Value::String(s) => out.push((k, s)),
                    other => {
                        return Err(serde::de::Error::custom(format!(
                            "source '{k}' must be a string, found {other}"
                        )))
                    }
                }
            }
            Ok(out)
        }
    }
    let mut de = serde_json::Deserializer::from_str(json);
    let pairs =
        serde::de::Deserializer::deserialize_map(&mut de, Pairs).map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    Ok(pairs)
}

/// Reference §7 logical-path algorithm: NFC, `/` separators, case preserved,
/// `.` segments dropped, `..` resolved without escaping the root; `\`,
/// absolute paths, and empty segments rejected.
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn normalize_logical_path(raw: &str) -> Result<String, String> {
    let path: String = raw.nfc().collect();
    if path.is_empty() {
        return Err("empty path".to_string());
    }
    if path.contains('\\') {
        return Err("backslash separators are not allowed".to_string());
    }
    if path.starts_with('/') {
        return Err("absolute paths are not allowed".to_string());
    }
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" => return Err("empty path segment".to_string()),
            "." => {}
            ".." => {
                if segments.pop().is_none() {
                    return Err("path escapes the source-map root".to_string());
                }
            }
            s => segments.push(s),
        }
    }
    if segments.is_empty() {
        return Err("path resolves to the source-map root".to_string());
    }
    Ok(segments.join("/"))
}

/// The logical directory of a module ID ("flagship/a.sea" -> "flagship").
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn logical_dir(logical_id: &str) -> &str {
    logical_id.rfind('/').map_or("", |i| &logical_id[..i])
}

// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn source_hash(source: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(source.as_bytes()))
}

// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn declaration_kind(node: &AstNode) -> ResolvedDeclarationKind {
    match node {
        AstNode::Export(inner) => declaration_kind(&inner.node),
        AstNode::Record(_) => ResolvedDeclarationKind::Record,
        AstNode::Enum(_) => ResolvedDeclarationKind::Enum,
        AstNode::Operation(_) => ResolvedDeclarationKind::Operation,
        _ => ResolvedDeclarationKind::ExistingConcept,
    }
}

// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn kind_slug(node: &AstNode) -> &'static str {
    match node {
        AstNode::Export(inner) => kind_slug(&inner.node),
        AstNode::Entity { .. } => "entity",
        AstNode::Resource { .. } => "resource",
        AstNode::Flow { .. } => "flow",
        AstNode::Pattern { .. } => "pattern",
        AstNode::Role { .. } => "role",
        AstNode::Relation { .. } => "relation",
        AstNode::Dimension { .. } => "dimension",
        AstNode::UnitDeclaration { .. } => "unit",
        AstNode::Policy { .. } => "policy",
        AstNode::Instance { .. } => "instance",
        AstNode::ConceptChange { .. } => "concept_change",
        AstNode::Metric { .. } => "metric",
        AstNode::MappingDecl { .. } => "mapping",
        AstNode::ProjectionDecl { .. } => "projection",
        AstNode::Cell { .. } => "cell",
        AstNode::SystemDependency { .. } => "system_dependency",
        AstNode::Runtime { .. } => "runtime",
        AstNode::Tool { .. } => "tool",
        AstNode::DependencySet { .. } => "dependency_set",
        AstNode::Service { .. } => "service",
        AstNode::Mount { .. } => "mount",
        AstNode::Endpoint { .. } => "endpoint",
        AstNode::NetworkFlow { .. } => "network_flow",
        AstNode::Credential { .. } => "credential",
        AstNode::Record(_) => "record",
        AstNode::Enum(_) => "enum",
        AstNode::Operation(_) => "operation",
    }
}

// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
fn builtin_std_source(specifier: &str) -> Option<&'static str> {
    match specifier {
        "std" | "std:core" => Some(include_str!("../../std/core.sea")),
        "std:http" => Some(include_str!("../../std/http.sea")),
        "std:aws" => Some(include_str!("../../std/aws.sea")),
        _ => None,
    }
}

/// Resolve one deterministic semantic closure from an in-memory source map
/// (reference §7). All diagnostics are APP014 with a closed reason value,
/// except parse failures which are APP015 (the module fails its language
/// schema before closure identity exists).
// ponytail: consumed by application resolution in Task 7.
#[allow(dead_code)]
pub(crate) fn resolve_source_map(
    entry_logical_path: &str,
    sources: &SourceMap,
) -> Result<ResolvedModuleSet, Vec<ApplicationDiagnostic>> {
    let mut diags: Vec<ApplicationDiagnostic> = Vec::new();

    // Parse every module up front; the set is closed and namespace imports
    // need every module's declared namespace anyway.
    let mut parsed: IndexMap<String, (String, Ast)> = IndexMap::new();
    for (logical_id, source) in &sources.0 {
        match parse_source(source) {
            Ok(ast) => {
                parsed.insert(logical_id.clone(), (source.clone(), ast));
            }
            Err(e) => {
                let mut d = ApplicationDiagnostic::new(
                    ApplicationDiagnosticCode::App015,
                    format!("module '{logical_id}' failed to parse: {e}"),
                );
                d.context.logical_module_id = Some(logical_id.clone());
                diags.push(d);
            }
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }

    let entry = normalize_logical_path(entry_logical_path).map_err(|reason| {
        vec![ApplicationDiagnostic::closure_error(
            APP014_UNRESOLVED_SPECIFIER,
            format!("invalid entry logical path '{entry_logical_path}': {reason}"),
        )]
    })?;
    if !parsed.contains_key(&entry) {
        return Err(vec![ApplicationDiagnostic::closure_error(
            APP014_UNRESOLVED_SPECIFIER,
            format!("entry '{entry}' is not present in sources_json"),
        )]);
    }

    // DFS closure walk from the entry.
    let mut std_modules: IndexMap<String, (String, Ast)> = IndexMap::new();
    let mut reachable: HashSet<String> = HashSet::new();
    let mut visiting: Vec<String> = Vec::new();
    let mut import_graph: Vec<ImportEdge> = Vec::new();
    let mut aliases: Vec<AliasBinding> = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn visit(
        logical_id: &str,
        parsed: &IndexMap<String, (String, Ast)>,
        std_modules: &mut IndexMap<String, (String, Ast)>,
        reachable: &mut HashSet<String>,
        visiting: &mut Vec<String>,
        import_graph: &mut Vec<ImportEdge>,
        aliases: &mut Vec<AliasBinding>,
        diags: &mut Vec<ApplicationDiagnostic>,
    ) {
        if visiting.iter().any(|m| m == logical_id) {
            let cycle: Vec<&str> = visiting
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(logical_id))
                .collect();
            diags.push(ApplicationDiagnostic::closure_error(
                APP014_IMPORT_CYCLE,
                format!("import cycle: {}", cycle.join(" -> ")),
            ));
            return;
        }
        if reachable.contains(logical_id) {
            return;
        }
        reachable.insert(logical_id.to_string());
        let ast = match parsed
            .get(logical_id)
            .or_else(|| std_modules.get(logical_id))
        {
            Some((_, ast)) => ast.clone(),
            None => return,
        };
        visiting.push(logical_id.to_string());
        for import in &ast.metadata.imports {
            let spec = &import.from_module;
            let target: Option<String> = if spec.starts_with("./") || spec.starts_with("../") {
                let joined = if logical_dir(logical_id).is_empty() {
                    spec.clone()
                } else {
                    format!("{}/{}", logical_dir(logical_id), spec)
                };
                match normalize_logical_path(&joined) {
                    Ok(p) if parsed.contains_key(&p) => Some(p),
                    Ok(p) => {
                        diags.push(ApplicationDiagnostic::closure_error(
                            APP014_UNRESOLVED_SPECIFIER,
                            format!(
                                "import '{spec}' in '{logical_id}' resolves to '{p}', which is not in sources_json"
                            ),
                        ));
                        None
                    }
                    Err(reason) => {
                        diags.push(ApplicationDiagnostic::closure_error(
                            APP014_UNRESOLVED_SPECIFIER,
                            format!("import '{spec}' in '{logical_id}' is invalid: {reason}"),
                        ));
                        None
                    }
                }
            } else if spec == "std" || spec.starts_with("std:") {
                match builtin_std_source(spec) {
                    Some(source) => {
                        if !std_modules.contains_key(spec) {
                            // std sources are maintained in-repo and always parse.
                            let ast = parse_source(source).expect("builtin std module parses");
                            std_modules.insert(spec.clone(), (source.to_string(), ast));
                        }
                        Some(spec.clone())
                    }
                    None => {
                        diags.push(ApplicationDiagnostic::closure_error(
                            APP014_UNRESOLVED_SPECIFIER,
                            format!("unknown std module '{spec}' imported by '{logical_id}'"),
                        ));
                        None
                    }
                }
            } else {
                // Namespace import: exactly one source-map module whose declared
                // namespace equals the specifier's exact bytes.
                let matches: Vec<&String> = parsed
                    .iter()
                    .filter(|(_, (_, ast))| ast.metadata.namespace.as_deref() == Some(spec))
                    .map(|(id, _)| id)
                    .collect();
                match matches.as_slice() {
                    [one] => Some((*one).clone()),
                    [] => {
                        diags.push(ApplicationDiagnostic::closure_error(
                            APP014_UNRESOLVED_SPECIFIER,
                            format!(
                                "namespace import '{spec}' in '{logical_id}' matches no source-map module"
                            ),
                        ));
                        None
                    }
                    _ => {
                        diags.push(ApplicationDiagnostic::closure_error(
                            APP014_UNRESOLVED_SPECIFIER,
                            format!(
                                "namespace import '{spec}' in '{logical_id}' matches more than one source-map module"
                            ),
                        ));
                        None
                    }
                }
            };
            let Some(target) = target else { continue };
            import_graph.push(ImportEdge {
                importer: logical_id.to_string(),
                imported: target.clone(),
            });
            if let ImportSpecifier::Wildcard(alias) = &import.specifier {
                aliases.push(AliasBinding {
                    importer: logical_id.to_string(),
                    alias: alias.clone(),
                    target: target.clone(),
                });
            }
            visit(
                &target,
                parsed,
                std_modules,
                reachable,
                visiting,
                import_graph,
                aliases,
                diags,
            );
            // Named import visibility against the resolved target.
            if let ImportSpecifier::Named(items) = &import.specifier {
                if let Some((_, target_ast)) =
                    parsed.get(&target).or_else(|| std_modules.get(&target))
                {
                    let exports = collect_exports(target_ast);
                    let declared: HashSet<&str> = target_ast
                        .declarations
                        .iter()
                        .filter_map(|d| declaration_name(&d.node))
                        .collect();
                    for item in items {
                        if exports.contains(&item.name) {
                            continue;
                        }
                        if declared.contains(item.name.as_str()) {
                            diags.push(ApplicationDiagnostic::closure_error(
                                APP014_NOT_EXPORTED,
                                format!("'{}' exists in '{target}' but is not exported", item.name),
                            ));
                        } else {
                            diags.push(ApplicationDiagnostic::closure_error(
                                APP014_UNRESOLVED_ALIAS,
                                format!(
                                    "named import '{}' in '{logical_id}' binds no declaration in '{target}'",
                                    item.name
                                ),
                            ));
                        }
                    }
                }
            }
        }
        visiting.pop();
    }

    visit(
        &entry,
        &parsed,
        &mut std_modules,
        &mut reachable,
        &mut visiting,
        &mut import_graph,
        &mut aliases,
        &mut diags,
    );

    if !diags.is_empty() {
        crate::application::diagnostic::sort_diagnostics(&mut diags);
        return Err(diags);
    }

    // Deterministic module order: logical ID, bytewise ascending.
    let mut logical_ids: Vec<String> = reachable.into_iter().collect();
    logical_ids.sort();
    let modules: Vec<ResolvedModule> = logical_ids
        .iter()
        .map(|id| {
            let (source, ast) = parsed
                .get(id)
                .or_else(|| std_modules.get(id))
                .expect("reachable module was parsed");
            ResolvedModule {
                logical_id: id.clone(),
                source_hash: source_hash(source),
                ast: ast.clone(),
            }
        })
        .collect();

    import_graph.sort_by(|a, b| (&a.importer, &a.imported).cmp(&(&b.importer, &b.imported)));
    import_graph.dedup();
    aliases.sort_by(|a, b| (&a.importer, &a.alias).cmp(&(&b.importer, &b.alias)));

    // One qualified symbol table across the closure. Application declaration
    // names are NFC-normalized; existing concept names keep exact bytes.
    let mut symbols: IndexMap<String, ResolvedSymbol> = IndexMap::new();
    for (module_index, module) in modules.iter().enumerate() {
        let namespace = module
            .ast
            .metadata
            .namespace
            .clone()
            // ponytail: modules without @namespace use their logical ID as the
            // qualification root; the reference leaves this case to APP014 later.
            .unwrap_or_else(|| module.logical_id.clone());
        let exports = collect_exports(&module.ast);
        for (declaration_index, decl) in module.ast.declarations.iter().enumerate() {
            let Some(name) = declaration_name(&decl.node) else {
                continue;
            };
            // Flows are occurrence-identified (reference §8): equal flows are
            // legal repetitions, not qualified-ID collisions.
            let inner = match &decl.node {
                crate::parser::ast::AstNode::Export(inner) => &inner.node,
                other => other,
            };
            if matches!(inner, crate::parser::ast::AstNode::Flow { .. }) {
                continue;
            }
            let kind = declaration_kind(&decl.node);
            let name = if kind == ResolvedDeclarationKind::ExistingConcept {
                name.to_string()
            } else {
                name.nfc().collect()
            };
            let qualified_id = format!("{namespace}.{}.{name}", kind_slug(&decl.node));
            let symbol = ResolvedSymbol {
                qualified_id: qualified_id.clone(),
                kind,
                exported: exports.contains(name.as_str()),
                origin: OriginRef {
                    logical_module_id: module.logical_id.clone(),
                    line: decl.line,
                    column: decl.column,
                },
                module_index,
                declaration_index,
            };
            if let Some(existing) = symbols.get(&qualified_id) {
                diags.push(ApplicationDiagnostic::closure_error(
                    APP014_SYMBOL_COLLISION,
                    format!(
                        "distinct declarations share qualified ID '{qualified_id}' ('{}' line {} and '{}' line {})",
                        existing.origin.logical_module_id,
                        existing.origin.line,
                        module.logical_id,
                        decl.line
                    ),
                ));
            } else {
                symbols.insert(qualified_id, symbol);
            }
        }
    }

    if diags.is_empty() {
        Ok(ResolvedModuleSet {
            modules,
            import_graph,
            symbols,
            aliases,
        })
    } else {
        crate::application::diagnostic::sort_diagnostics(&mut diags);
        Err(diags)
    }
}

pub fn parse_with_registry(
    path: &Path,
    registry: &NamespaceRegistry,
) -> ParseResult<(Ast, ParseOptions)> {
    let content = fs::read_to_string(path).map_err(|e| {
        ParseError::GrammarError(format!("Failed to read {}: {}", path.display(), e))
    })?;

    // `ParseOptions` are constructed here to be returned to the caller,
    // even though `parse_source` currently doesn't use them. Initialize
    // fields directly in the `ParseOptions` construction to avoid
    // field reassignment lint from clippy (clippy::field-reassign-with-default).
    let options = ParseOptions {
        namespace_registry: Some(registry.clone()),
        entry_path: Some(path.to_path_buf()),
        ..Default::default()
    };

    let ast = parse_source(&content)?;
    Ok((ast, options))
}

#[cfg(test)]
mod source_map_tests {
    use super::*;

    fn resolve(
        entry: &str,
        sources_json: &str,
    ) -> Result<ResolvedModuleSet, Vec<ApplicationDiagnostic>> {
        let sources = SourceMap::parse_json(sources_json)?;
        resolve_source_map(entry, &sources)
    }

    fn reason(err: &[ApplicationDiagnostic]) -> &str {
        err[0].context.reason.as_deref().unwrap()
    }

    #[test]
    fn resolves_named_relative_import_into_one_symbol_table() {
        let sources = serde_json::json!({
            "flagship/command-write.sea": include_str!(
                "../../../fixtures/application_generation/flagship/command-write.sea"),
            "flagship/query-read.sea": include_str!(
                "../../../fixtures/application_generation/flagship/query-read.sea"),
        })
        .to_string();
        let set = resolve("flagship/query-read.sea", &sources).unwrap();
        assert_eq!(
            set.modules
                .iter()
                .map(|m| m.logical_id.as_str())
                .collect::<Vec<_>>(),
            ["flagship/command-write.sea", "flagship/query-read.sea"]
        );
        assert!(set.symbols.contains_key("flagship.orders.entity.Order"));
        assert!(set
            .symbols
            .contains_key("flagship.orders.record.PlaceOrderInput"));
        assert!(set
            .symbols
            .contains_key("flagship.orders.operation.get_order_status"));
        assert_eq!(set.import_graph.len(), 1);
        assert!(set
            .modules
            .iter()
            .all(|m| m.source_hash.starts_with("sha256:")));
    }

    #[test]
    fn relative_import_escaping_root_is_unresolved_specifier() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport { X } from \"../outside.sea\"\n",
        })
        .to_string();
        let err = resolve("a.sea", &sources).unwrap_err();
        assert_eq!(reason(&err), "unresolved_specifier");
        assert!(err[0].message.contains("escapes"));
    }

    #[test]
    fn duplicate_normalized_source_key_is_rejected() {
        let sources = r#"{"a/./x.sea": "@namespace \"n\"", "a/x.sea": "@namespace \"n\""}"#;
        let err = SourceMap::parse_json(sources).unwrap_err();
        assert_eq!(reason(&err), "symbol_collision");
        assert!(err[0].message.contains("duplicate normalized logical ID"));
    }

    #[test]
    fn duplicate_raw_source_key_is_rejected() {
        let sources = r#"{"a.sea": "x", "a.sea": "y"}"#;
        let err = SourceMap::parse_json(sources).unwrap_err();
        assert_eq!(reason(&err), "symbol_collision");
        assert!(err[0].message.contains("duplicate raw source key"));
    }

    #[test]
    fn named_import_of_unexported_symbol_is_not_exported() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport { Hidden } from \"./b.sea\"\n",
            "b.sea": "@namespace \"ns.b\"\nrecord Hidden {\n    x: string\n}\n",
        })
        .to_string();
        let err = resolve("a.sea", &sources).unwrap_err();
        assert_eq!(reason(&err), "not_exported");
    }

    #[test]
    fn named_import_binding_nothing_is_unresolved_alias() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport { Ghost } from \"./b.sea\"\n",
            "b.sea": "@namespace \"ns.b\"\nexport record Real {\n    x: string\n}\n",
        })
        .to_string();
        let err = resolve("a.sea", &sources).unwrap_err();
        assert_eq!(reason(&err), "unresolved_alias");
    }

    #[test]
    fn wildcard_import_records_alias_binding() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport * as b from \"./b.sea\"\n",
            "b.sea": "@namespace \"ns.b\"\nexport record Real {\n    x: string\n}\n",
        })
        .to_string();
        let set = resolve("a.sea", &sources).unwrap();
        assert_eq!(
            set.aliases,
            vec![AliasBinding {
                importer: "a.sea".to_string(),
                alias: "b".to_string(),
                target: "b.sea".to_string(),
            }]
        );
    }

    #[test]
    fn unresolved_relative_specifier_is_reported() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport { X } from \"./missing.sea\"\n",
        })
        .to_string();
        let err = resolve("a.sea", &sources).unwrap_err();
        assert_eq!(reason(&err), "unresolved_specifier");
    }

    #[test]
    fn import_cycle_is_reported() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport * as b from \"./b.sea\"\n",
            "b.sea": "@namespace \"ns.b\"\nimport * as a from \"./a.sea\"\n",
        })
        .to_string();
        let err = resolve("a.sea", &sources).unwrap_err();
        assert_eq!(reason(&err), "import_cycle");
    }

    #[test]
    fn distinct_declarations_with_one_qualified_id_collide() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.shared\"\nimport * as b from \"./b.sea\"\nexport record Same {\n    x: string\n}\n",
            "b.sea": "@namespace \"ns.shared\"\nexport record Same {\n    x: string\n}\n",
        })
        .to_string();
        let err = resolve("a.sea", &sources).unwrap_err();
        assert_eq!(reason(&err), "symbol_collision");
    }

    #[test]
    fn diamond_origin_collapses_to_one_symbol() {
        let sources = serde_json::json!({
            "a.sea": "@namespace \"ns.a\"\nimport * as b from \"./b.sea\"\nimport * as c from \"./c.sea\"\n",
            "b.sea": "@namespace \"ns.b\"\nimport { Shared } from \"./d.sea\"\n",
            "c.sea": "@namespace \"ns.c\"\nimport { Shared } from \"./d.sea\"\n",
            "d.sea": "@namespace \"ns.d\"\nexport record Shared {\n    x: string\n}\n",
        })
        .to_string();
        let set = resolve("a.sea", &sources).unwrap();
        assert_eq!(
            set.symbols
                .values()
                .filter(|s| s.qualified_id.ends_with(".Shared"))
                .count(),
            1
        );
        assert_eq!(set.modules.len(), 4);
    }

    #[test]
    fn entry_missing_from_sources_is_unresolved_specifier() {
        let err = resolve("missing.sea", r#"{"a.sea": "@namespace \"n\""}"#).unwrap_err();
        assert_eq!(reason(&err), "unresolved_specifier");
    }
}
