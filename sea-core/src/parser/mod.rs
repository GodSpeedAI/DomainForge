use crate::graph::Graph;
use pest_derive::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[grammar = "grammar/sea.pest"]
pub struct SeaParser;

pub mod ast;
#[cfg(test)]
mod ast_schema;
pub mod error;
pub mod lint;
pub mod printer;
pub mod profiles;
pub mod string_utils;

pub use ast::parse_expression_from_str;
pub use ast::parse_source;
pub use ast::Ast;
pub use ast::AstNode;
pub use error::{ParseError, ParseResult};
pub use lint::*;
pub use printer::PrettyPrinter;
pub use profiles::{Profile, ProfileRegistry};
pub use string_utils::unescape_string;

/// Controls how the SEA parser interprets declarations.
///
/// Currently only the `default_namespace` value influences whether entities/resources
/// without an explicit namespace inherit a fallback namespace.
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// When `Some`, unqualified declarations receive this namespace.
    /// When `None`, entities and resources must provide their own namespace.
    pub default_namespace: Option<String>,
    /// Optional namespace registry used for module resolution.
    pub namespace_registry: Option<crate::registry::NamespaceRegistry>,
    /// Path to the entry file being parsed, used for module resolution.
    pub entry_path: Option<PathBuf>,
    /// Active profile to enforce when no profile is declared in the source file.
    pub active_profile: Option<String>,
    /// Downgrades profile violations from hard errors to warnings when set.
    pub tolerate_profile_warnings: bool,
}

/// Parse SEA DSL source code into an AST
pub fn parse(source: &str) -> ParseResult<Ast> {
    ast::parse_source(source)
}

/// Parse SEA DSL source code directly into a Graph
pub fn parse_to_graph(source: &str) -> ParseResult<Graph> {
    let ast = parse(source)?;
    ast::ast_to_graph_with_options(ast, &ParseOptions::default())
}

/// Parses SEA DSL `source` directly into a `Graph`, honoring the provided `options`.
///
/// # Parameters
/// - `source`: DSL text that will be parsed into AST nodes.
/// - `options`: Controls parser behavior (`default_namespace`, module resolution via `namespace_registry`/`entry_path`, `active_profile`, and whether profile violations are warnings).
///
/// # Returns
/// A `ParseResult` containing the constructed `Graph` or a `ParseError`.
///
/// # Errors
/// Returns an error if parsing fails or AST validation (graph construction) rejects the input.
///
/// # Example
/// ```
/// use sea_core::parser::{parse_to_graph_with_options, ParseOptions};
///
/// let options = ParseOptions {
///     default_namespace: Some("logistics".to_string()),
///     ..Default::default()
/// };
/// let graph = parse_to_graph_with_options("Entity \"Warehouse\"", &options).unwrap();
/// ```
pub fn parse_to_graph_with_options(source: &str, options: &ParseOptions) -> ParseResult<Graph> {
    match (&options.namespace_registry, &options.entry_path) {
        (Some(registry), Some(path)) => {
            let mut resolver = crate::module::resolver::ModuleResolver::new(registry)?;
            let ast = resolver.validate_entry(path, source)?;
            resolver.validate_dependencies(path, &ast)?;
            ast::ast_to_graph_with_options(ast, options)
        }
        (Some(_), None) => Err(ParseError::Validation(
            "Namespace registry provided without entry path".to_string(),
        )),
        _ => {
            if let Some(path) = &options.entry_path {
                log::warn!(
                        "Entry path '{}' provided without namespace registry; module resolution skipped",
                        path.display()
                    );
            }
            let ast = parse(source)?;
            ast::ast_to_graph_with_options(ast, options)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_declaration_syntax() {
        let source = r#"
            Entity "Warehouse A" in logistics
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_resource_declaration_syntax() {
        let source = r#"
            Resource "Camera Units" units in inventory
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_flow_declaration_syntax() {
        let source = r#"
            Flow "Camera Units" from "Warehouse" to "Factory" quantity 100
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_policy_simple_syntax() {
        let source = r#"
            Policy check_quantity as: Flow.quantity > 0
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_complex_policy_syntax() {
        let source = r#"
            Policy flow_constraints as:
                (Flow.quantity > 0) and (Entity.name != "")
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_nested_expressions() {
        let source = r#"
            Policy multi_condition as:
                (A or B) and (C or (D and E))
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_comments_ignored() {
        let source = r#"
            // This is a comment
            Entity "Test" in domain
            // Another comment
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_multiple_declarations() {
        let source = r#"
            Entity "Warehouse" in logistics
            Resource "Cameras" units
            Flow "Cameras" from "Warehouse" to "Factory" quantity 50
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }
}
