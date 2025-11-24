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
                    unit,
                } => {
                    let qty = quantity.unwrap_or(0);
                    let flow_str = if let Some(u) = unit {
                        format!(
                            "Flow \"{}\" from \"{}\" to \"{}\" quantity {} \"{}\"",
                            resource_name, from_entity, to_entity, qty, u
                        )
                    } else {
                        format!(
                            "Flow \"{}\" from \"{}\" to \"{}\" quantity {}",
                            resource_name, from_entity, to_entity, qty
                        )
                    };
                    let _ = writeln!(output, "{}", flow_str);
                }
                crate::parser::ast::AstNode::Role { name, entity } => {
                    let _ = writeln!(output, "Role \"{}\" for entity \"{}\"", name, entity);
                }
                crate::parser::ast::AstNode::Relation {
                    name,
                    subject_role,
                    object_role,
                    via_flow,
                } => {
                    let _ = writeln!(output, "Relation \"{}\" {{", name);
                    if let Some(s) = subject_role {
                        let _ = writeln!(
                            output,
                            "{}subject role \"{}\"",
                            " ".repeat(self.indent_width),
                            s
                        );
                    }
                    if let Some(o) = object_role {
                        let _ = writeln!(
                            output,
                            "{}object role \"{}\"",
                            " ".repeat(self.indent_width),
                            o
                        );
                    }
                    if let Some(v) = via_flow {
                        let _ = writeln!(
                            output,
                            "{}via flow \"{}\"",
                            " ".repeat(self.indent_width),
                            v
                        );
                    }
                    let _ = writeln!(output, "}}");
                }
                crate::parser::ast::AstNode::Instance {
                    name,
                    resource,
                    entity,
                    properties,
                } => {
                    let _ = writeln!(
                        output,
                        "Instance \"{}\" of resource \"{}\" at entity \"{}\"",
                        name, resource, entity
                    );
                    if !properties.is_empty() {
                        let _ = writeln!(output, "{{");
                        for (k, v) in properties {
                            let _ = writeln!(
                                output,
                                "{}{} \"{}\"",
                                " ".repeat(self.indent_width),
                                k,
                                v
                            );
                        }
                        let _ = writeln!(output, "}}");
                    }
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
