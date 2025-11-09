use crate::graph::Graph;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/sea.pest"]
pub struct SeaParser;

pub mod ast;
pub mod error;
pub mod string_utils;

pub use ast::{Ast, AstNode};
pub use error::{ParseError, ParseResult};
pub use string_utils::unescape_string;

/// Parse SEA DSL source code into an AST
pub fn parse(source: &str) -> ParseResult<Ast> {
    ast::parse_source(source)
}

/// Parse SEA DSL source code directly into a Graph
pub fn parse_to_graph(source: &str) -> ParseResult<Graph> {
    let ast = parse(source)?;
    ast::ast_to_graph(ast)
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
