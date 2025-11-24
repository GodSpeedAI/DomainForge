use crate::parser::ast::Ast;
use std::fmt::Write;

pub struct PrettyPrinter {
    indent_width: usize,
    #[allow(dead_code)]
    max_line_length: usize,
    #[allow(dead_code)]
    trailing_commas: bool,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self {
            indent_width: 4,
            max_line_length: 80,
            trailing_commas: false,
        }
    }

    pub fn print(&self, ast: &Ast) -> String {
        let mut output = String::new();

        // File header
        if let Some(ns) = &ast.metadata.namespace {
            writeln!(output, "namespace {}", ns).unwrap();
            writeln!(output).unwrap();
        }

        // Declarations
        for decl in &ast.declarations {
            match decl {
                crate::parser::ast::AstNode::Entity { name, .. } => {
                    writeln!(output, "Entity \"{}\"", name).unwrap();
                }
                crate::parser::ast::AstNode::Resource { name, unit_name, .. } => {
                    let unit = unit_name.as_deref().unwrap_or("units");
                    writeln!(output, "Resource \"{}\" ({})", name, unit).unwrap();
                }
                crate::parser::ast::AstNode::Flow {
                    resource_name,
                    from_entity,
                    to_entity,
                    quantity,
                } => {
                    let qty = quantity.unwrap_or(0);
                    writeln!(
                        output,
                        "Flow \"{}\" from \"{}\" to \"{}\" quantity {}",
                        resource_name, from_entity, to_entity, qty
                    )
                    .unwrap();
                }
                crate::parser::ast::AstNode::Policy {
                    name,
                    metadata: _,
                    expression,
                    ..
                } => {
                    writeln!(output, "Policy \"{}\" {{", name).unwrap();
                    // Simple indentation for expression
                    // Note: expression printing is simplified here, ideally we'd have an expression printer
                    writeln!(output, "{}{:?}", " ".repeat(self.indent_width), expression).unwrap();
                    writeln!(output, "}}").unwrap();
                }
                _ => {} // Handle other nodes if necessary
            }
            writeln!(output).unwrap();
        }

        output
    }
}
