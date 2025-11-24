use crate::parser::ast::Ast;
use std::fmt::Write;

pub struct PrettyPrinter {
    indent_width: usize,
    #[allow(dead_code)]
    max_line_length: usize,
    #[allow(dead_code)]
    trailing_commas: bool,
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self {
            indent_width: 4,
            max_line_length: 80,
            trailing_commas: false,
        }
    }
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn print(&self, ast: &Ast) -> String {
        let mut output = String::new();

        // File header
        if let Some(ns) = &ast.metadata.namespace {
            let _ = writeln!(output, "namespace {}", ns);
            let _ = writeln!(output);
        }

        // Declarations
        for decl in &ast.declarations {
            match decl {
                crate::parser::ast::AstNode::Entity { name, .. } => {
                    let _ = writeln!(output, "Entity \"{}\"", name);
                }
                crate::parser::ast::AstNode::Resource {
                    name, unit_name, ..
                } => {
                    let unit = unit_name.as_deref().unwrap_or("units");
                    let _ = writeln!(output, "Resource \"{}\" ({})", name, unit);
                }
                crate::parser::ast::AstNode::Flow {
                    resource_name,
                    from_entity,
                    to_entity,
                    quantity,
                } => {
                    let qty = quantity.unwrap_or(0);
                    let _ = writeln!(
                        output,
                        "Flow \"{}\" from \"{}\" to \"{}\" quantity {}",
                        resource_name, from_entity, to_entity, qty
                    );
                }
                crate::parser::ast::AstNode::Policy {
                    name,
                    metadata: _,
                    expression,
                    ..
                } => {
                    let _ = writeln!(output, "Policy \"{}\" {{", name);
                    // Simple indentation for expression
                    let _ = writeln!(output, "{}{}", " ".repeat(self.indent_width), expression);
                    let _ = writeln!(output, "}}");
                }
                _ => {} // Handle other nodes if necessary
            }
            let _ = writeln!(output);
        }

        output
    }
}
