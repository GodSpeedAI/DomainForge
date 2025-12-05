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
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if self.visiting.contains(&canonical) {
            return Err(ParseError::GrammarError(
                "Circular dependency detected".to_string(),
            ));
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
        let content = fs::read_to_string(path).map_err(|e| {
            ParseError::GrammarError(format!("Failed to read module {}: {}", path.display(), e))
        })?;
        parse_source(&content)
    }

    fn resolve_module_path(&self, namespace: &str) -> ParseResult<PathBuf> {
        self.bindings
            .iter()
            .find(|binding| binding.namespace == namespace)
            .map(|binding| binding.path.clone())
            .ok_or_else(|| {
                ParseError::GrammarError(format!("Module '{}' could not be resolved", namespace))
            })
    }

    fn validate_import_targets(&self, import: &ImportDecl, dep_path: &Path) -> ParseResult<()> {
        let canonical = dep_path
            .canonicalize()
            .unwrap_or_else(|_| dep_path.to_path_buf());
        let module = self.loaded_modules.get(&canonical).ok_or_else(|| {
            ParseError::GrammarError(format!(
                "Expected module '{}' to be loaded before validating imports",
                dep_path.display()
            ))
        })?;

        match &import.specifier {
            ImportSpecifier::Wildcard => Ok(()),
            ImportSpecifier::Named(items) => {
                for item in items {
                    if !module.exports.contains(&item.name) {
                        return Err(ParseError::GrammarError(format!(
                            "Symbol '{}' is not exported by module '{}'",
                            item.name, module.namespace
                        )));
                    }
                }
                Ok(())
            }
        }
    }
}

fn collect_exports(ast: &Ast) -> HashSet<String> {
    let mut exports = HashSet::new();
    for node in &ast.declarations {
        match node {
            AstNode::Export(inner) => {
                if let Some(name) = declaration_name(inner.as_ref()) {
                    exports.insert(name.to_string());
                }
            }
            other => {
                if let Some(name) = declaration_name(other) {
                    exports.insert(name.to_string());
                }
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
        AstNode::Export(inner) => declaration_name(inner.as_ref()),
    }
}

pub fn parse_with_registry(
    path: &Path,
    registry: &NamespaceRegistry,
) -> ParseResult<(Ast, ParseOptions)> {
    let content = fs::read_to_string(path).map_err(|e| {
        ParseError::GrammarError(format!("Failed to read {}: {}", path.display(), e))
    })?;
    let mut options = ParseOptions::default();
    options.namespace_registry = Some(registry.clone());
    options.entry_path = Some(path.to_path_buf());
    let ast = parse_source(&content)?;
    Ok((ast, options))
}
