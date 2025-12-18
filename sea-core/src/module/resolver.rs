use crate::error::fuzzy::levenshtein_distance;
use crate::parser::ast::{Ast, AstNode, ImportDecl, ImportSpecifier};
use crate::parser::{parse_source, ParseError, ParseOptions, ParseResult};
use crate::registry::{NamespaceBinding, NamespaceRegistry};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

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
        | AstNode::ProjectionDecl { name, .. } => Some(name),
        AstNode::Export(inner) => declaration_name(&inner.node),
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
