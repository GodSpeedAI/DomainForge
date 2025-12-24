//! Core formatter implementation.

use crate::formatter::config::FormatConfig;
use crate::parser::ast::{
    parse_source, Ast, AstNode, FileMetadata, ImportDecl, ImportItem, ImportSpecifier,
};
use crate::policy::Expression;
use std::fmt;

/// Error type for formatting operations.
#[derive(Debug)]
pub enum FormatError {
    /// Failed to parse the source code.
    ParseError(String),
    /// Internal formatting error.
    InternalError(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            FormatError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}

/// Format SEA source code.
///
/// Parses the source, then pretty-prints it with consistent formatting.
///
/// # Arguments
///
/// * `source` - The SEA source code to format
/// * `config` - Formatting configuration options
///
/// # Returns
///
/// The formatted source code, or an error if parsing fails.
///
/// # Example
///
/// ```rust,ignore
/// use sea_core::formatter::{format, FormatConfig};
///
/// let source = r#"Entity   "Foo"  in    bar"#;
/// let formatted = format(source, FormatConfig::default()).unwrap();
/// assert_eq!(formatted, "Entity \"Foo\" in bar\n");
/// ```
pub fn format(source: &str, config: FormatConfig) -> Result<String, FormatError> {
    // Check if we should preserve comments
    if config.preserve_comments {
        format_preserving_comments(source, config)
    } else {
        format_without_comments(source, config)
    }
}

/// Format SEA source code without preserving comments.
fn format_without_comments(source: &str, config: FormatConfig) -> Result<String, FormatError> {
    let ast = parse_source(source).map_err(|e| FormatError::ParseError(e.to_string()))?;

    let mut formatter = Formatter::new(config, None);
    formatter.format_ast(&ast);

    Ok(formatter.output)
}

/// Format SEA source code while preserving comments.
///
/// This function:
/// 1. Extracts all comments from the source with their positions
/// 2. Parses and formats the code
/// 3. Re-inserts comments at appropriate positions
pub fn format_preserving_comments(
    source: &str,
    config: FormatConfig,
) -> Result<String, FormatError> {
    use crate::formatter::comments::CommentedSource;

    let commented = CommentedSource::new(source);
    let ast = parse_source(source).map_err(|e| FormatError::ParseError(e.to_string()))?;

    let mut formatter = Formatter::new(config, Some(commented));
    formatter.format_ast(&ast);

    Ok(formatter.output)
}

/// Internal formatter state.
struct Formatter {
    config: FormatConfig,
    output: String,
    indent_level: usize,
    /// Optional source with comments for preservation
    commented_source: Option<crate::formatter::comments::CommentedSource>,
}

impl Formatter {
    fn new(
        config: FormatConfig,
        commented_source: Option<crate::formatter::comments::CommentedSource>,
    ) -> Self {
        Self {
            config,
            output: String::new(),
            indent_level: 0,
            commented_source,
        }
    }

    /// Format the entire AST.
    fn format_ast(&mut self, ast: &Ast) {
        // Output file header comments if we have them
        // Clone to avoid borrow conflict with self.write()
        let header_comments: Vec<_> = self
            .commented_source
            .as_ref()
            .map(|cs| cs.file_header_comments.clone())
            .unwrap_or_default();

        for comment in &header_comments {
            self.write("// ");
            self.write(&comment.text);
            self.newline();
        }
        if !header_comments.is_empty() {
            self.newline();
        }

        // Format file header
        self.format_file_metadata(&ast.metadata);

        // Format declarations
        for (i, decl) in ast.declarations.iter().enumerate() {
            if i > 0 || !ast.metadata.imports.is_empty() || ast.metadata.namespace.is_some() {
                self.newline();
            }
            self.format_declaration(&decl.node);
        }

        // Ensure trailing newline
        if self.config.trailing_newline && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    /// Format file-level metadata (annotations and imports).
    fn format_file_metadata(&mut self, meta: &FileMetadata) {
        // Annotations in canonical order
        if let Some(ref ns) = meta.namespace {
            self.write("@namespace ");
            self.write_string_literal(ns);
            self.newline();
        }
        if let Some(ref version) = meta.version {
            self.write("@version ");
            self.write_string_literal(version);
            self.newline();
        }
        if let Some(ref owner) = meta.owner {
            self.write("@owner ");
            self.write_string_literal(owner);
            self.newline();
        }
        if let Some(ref profile) = meta.profile {
            self.write("@profile ");
            self.write_string_literal(profile);
            self.newline();
        }

        // Blank line after annotations if any and before imports
        if (meta.namespace.is_some()
            || meta.version.is_some()
            || meta.owner.is_some()
            || meta.profile.is_some())
            && !meta.imports.is_empty()
        {
            self.newline();
        }

        // Imports (sorted if configured)
        let mut imports = meta.imports.clone();
        if self.config.sort_imports {
            imports.sort_by(|a, b| a.from_module.cmp(&b.from_module));
        }

        for import in &imports {
            self.format_import(import);
            self.newline();
        }
    }

    /// Format an import declaration.
    fn format_import(&mut self, import: &ImportDecl) {
        self.write("import ");
        match &import.specifier {
            ImportSpecifier::Named(items) => {
                self.write("{ ");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_import_item(item);
                }
                self.write(" }");
            }
            ImportSpecifier::Wildcard(alias) => {
                self.write("* as ");
                self.write(alias);
            }
        }
        self.write(" from ");
        self.write_string_literal(&import.from_module);
    }

    /// Format an import item.
    fn format_import_item(&mut self, item: &ImportItem) {
        self.write(&item.name);
        if let Some(ref alias) = item.alias {
            self.write(" as ");
            self.write(alias);
        }
    }

    /// Format a declaration node.
    fn format_declaration(&mut self, node: &AstNode) {
        match node {
            AstNode::Export(inner) => {
                self.write("export ");
                self.format_declaration(&inner.node);
            }
            AstNode::Entity {
                name,
                version,
                annotations,
                domain,
            } => {
                self.write("Entity ");
                self.write_string_literal(name);
                if let Some(v) = version {
                    self.write(" v");
                    self.write(v);
                }
                // Format annotations
                if let Some(replaces) = annotations.get("replaces") {
                    if let Some(s) = replaces.as_str() {
                        self.newline();
                        self.indent();
                        self.write_indent();
                        self.write("@replaces ");
                        // Parse the replaces value which may include version
                        if s.contains(" v") {
                            let parts: Vec<&str> = s.splitn(2, " v").collect();
                            self.write_string_literal(parts[0]);
                            if parts.len() > 1 {
                                self.write(" v");
                                self.write(parts[1]);
                            }
                        } else {
                            self.write_string_literal(s);
                        }
                        self.dedent();
                    }
                }
                if let Some(changes) = annotations.get("changes") {
                    if let Some(arr) = changes.as_array() {
                        self.newline();
                        self.indent();
                        self.write_indent();
                        self.write("@changes [");
                        for (i, change) in arr.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            if let Some(s) = change.as_str() {
                                self.write_string_literal(s);
                            }
                        }
                        self.write("]");
                        self.dedent();
                    }
                }
                if let Some(d) = domain {
                    self.write(" in ");
                    self.write(d);
                }
                self.newline();
            }
            AstNode::Resource {
                name,
                annotations,
                unit_name,
                domain,
            } => {
                self.write("Resource ");
                self.write_string_literal(name);
                // Format annotations
                if let Some(replaces) = annotations.get("replaces") {
                    if let Some(s) = replaces.as_str() {
                        self.newline();
                        self.indent();
                        self.write_indent();
                        self.write("@replaces ");
                        // Parse the replaces value which may include version
                        if s.contains(" v") {
                            let parts: Vec<&str> = s.splitn(2, " v").collect();
                            self.write_string_literal(parts[0]);
                            if parts.len() > 1 {
                                self.write(" v");
                                self.write(parts[1]);
                            }
                        } else {
                            self.write_string_literal(s);
                        }
                        self.dedent();
                    }
                }
                if let Some(changes) = annotations.get("changes") {
                    if let Some(arr) = changes.as_array() {
                        self.newline();
                        self.indent();
                        self.write_indent();
                        self.write("@changes [");
                        for (i, change) in arr.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            if let Some(s) = change.as_str() {
                                self.write_string_literal(s);
                            }
                        }
                        self.write("]");
                        self.dedent();
                    }
                }
                if let Some(u) = unit_name {
                    self.write(" ");
                    self.write(u);
                }
                if let Some(d) = domain {
                    self.write(" in ");
                    self.write(d);
                }
                self.newline();
            }
            AstNode::Flow {
                resource_name,
                annotations,
                from_entity,
                to_entity,
                quantity,
            } => {
                self.write("Flow ");
                self.write_string_literal(resource_name);
                // Format annotations
                if let Some(replaces) = annotations.get("replaces") {
                    if let Some(s) = replaces.as_str() {
                        self.newline();
                        self.indent();
                        self.write_indent();
                        self.write("@replaces ");
                        if s.contains(" v") {
                            let parts: Vec<&str> = s.splitn(2, " v").collect();
                            self.write_string_literal(parts[0]);
                            if parts.len() > 1 {
                                self.write(" v");
                                self.write(parts[1]);
                            }
                        } else {
                            self.write_string_literal(s);
                        }
                        self.dedent();
                    }
                }
                if let Some(changes) = annotations.get("changes") {
                    if let Some(arr) = changes.as_array() {
                        self.newline();
                        self.indent();
                        self.write_indent();
                        self.write("@changes [");
                        for (i, change) in arr.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            if let Some(s) = change.as_str() {
                                self.write_string_literal(s);
                            }
                        }
                        self.write("]");
                        self.dedent();
                    }
                }
                self.write(" from ");
                self.write_string_literal(from_entity);
                self.write(" to ");
                self.write_string_literal(to_entity);
                if let Some(q) = quantity {
                    self.write(" quantity ");
                    self.write(&q.to_string());
                }
                self.newline();
            }
            AstNode::Pattern { name, regex } => {
                self.write("Pattern ");
                self.write_string_literal(name);
                self.write(" matches ");
                self.write_string_literal(regex);
                self.newline();
            }
            AstNode::Role { name, domain } => {
                self.write("Role ");
                self.write_string_literal(name);
                if let Some(d) = domain {
                    self.write(" in ");
                    self.write(d);
                }
                self.newline();
            }
            AstNode::Relation {
                name,
                subject_role,
                predicate,
                object_role,
                via_flow,
            } => {
                self.write("Relation ");
                self.write_string_literal(name);
                self.newline();
                self.indent();
                self.write_indent();
                self.write("subject: ");
                self.write_string_literal(subject_role);
                self.newline();
                self.write_indent();
                self.write("predicate: ");
                self.write_string_literal(predicate);
                self.newline();
                self.write_indent();
                self.write("object: ");
                self.write_string_literal(object_role);
                if let Some(flow) = via_flow {
                    self.newline();
                    self.write_indent();
                    self.write("via: flow ");
                    self.write_string_literal(flow);
                }
                self.dedent();
                self.newline();
            }
            AstNode::Dimension { name } => {
                self.write("Dimension ");
                self.write_string_literal(name);
                self.newline();
            }
            AstNode::UnitDeclaration {
                symbol,
                dimension,
                factor,
                base_unit,
            } => {
                self.write("Unit ");
                self.write_string_literal(symbol);
                self.write(" of ");
                self.write_string_literal(dimension);
                self.write(" factor ");
                self.write(&factor.to_string());
                self.write(" base ");
                self.write_string_literal(base_unit);
                self.newline();
            }
            AstNode::Policy {
                name,
                version,
                metadata,
                expression,
            } => {
                self.write("Policy ");
                self.write(name);
                if let (Some(kind), Some(modality), Some(priority)) =
                    (&metadata.kind, &metadata.modality, metadata.priority)
                {
                    self.write(" per ");
                    self.write(&format!("{}", kind));
                    self.write(" ");
                    self.write(&format!("{}", modality));
                    self.write(" priority ");
                    self.write(&priority.to_string());
                }
                // Policy annotations
                if let Some(ref rationale) = metadata.rationale {
                    self.newline();
                    self.indent();
                    self.write_indent();
                    self.write("@rationale ");
                    self.write_string_literal(rationale);
                    self.dedent();
                }
                if !metadata.tags.is_empty() {
                    self.newline();
                    self.indent();
                    self.write_indent();
                    self.write("@tags [");
                    for (i, tag) in metadata.tags.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write_string_literal(tag);
                    }
                    self.write("]");
                    self.dedent();
                }
                if let Some(v) = version {
                    self.newline();
                    self.indent();
                    self.write_indent();
                    self.write("v");
                    self.write(v);
                    self.dedent();
                }
                self.newline();
                self.indent();
                self.write_indent();
                self.write("as: ");
                self.format_expression(expression);
                self.dedent();
                self.newline();
            }
            AstNode::Instance {
                name,
                entity_type,
                fields,
            } => {
                self.write("Instance ");
                self.write(name);
                self.write(" of ");
                self.write_string_literal(entity_type);
                if !fields.is_empty() {
                    self.write(" {");
                    self.newline();
                    self.indent();
                    let mut sorted_fields: Vec<_> = fields.iter().collect();
                    sorted_fields.sort_by_key(|(k, _)| *k);
                    for (i, (field_name, expr)) in sorted_fields.iter().enumerate() {
                        self.write_indent();
                        self.write(field_name);
                        self.write(": ");
                        self.format_expression(expr);
                        if i < sorted_fields.len() - 1 {
                            self.write(",");
                        }
                        self.newline();
                    }
                    self.dedent();
                    self.write_indent();
                    self.write("}");
                }
                self.newline();
            }
            AstNode::ConceptChange {
                name,
                from_version,
                to_version,
                migration_policy,
                breaking_change,
            } => {
                self.write("ConceptChange ");
                self.write_string_literal(name);
                self.newline();
                self.indent();
                self.write_indent();
                self.write("@from_version v");
                self.write(from_version);
                self.newline();
                self.write_indent();
                self.write("@to_version v");
                self.write(to_version);
                self.newline();
                self.write_indent();
                self.write("@migration_policy ");
                self.write(migration_policy);
                self.newline();
                self.write_indent();
                self.write("@breaking_change ");
                self.write(if *breaking_change { "true" } else { "false" });
                self.dedent();
                self.newline();
            }
            AstNode::Metric {
                name,
                expression,
                metadata,
            } => {
                self.write("Metric ");
                self.write_string_literal(name);
                self.write(" as:");
                self.newline();
                self.indent();
                self.write_indent();
                self.format_expression(expression);
                // Metric annotations
                if let Some(ref interval) = metadata.refresh_interval {
                    self.newline();
                    self.write_indent();
                    self.write("@refresh_interval ");
                    self.write(&interval.num_seconds().to_string());
                    self.write(" \"seconds\"");
                }
                if let Some(ref unit) = metadata.unit {
                    self.newline();
                    self.write_indent();
                    self.write("@unit ");
                    self.write_string_literal(unit);
                }
                if let Some(threshold) = &metadata.threshold {
                    self.newline();
                    self.write_indent();
                    self.write("@threshold ");
                    self.write(&threshold.to_string());
                }
                if let Some(ref severity) = metadata.severity {
                    self.newline();
                    self.write_indent();
                    self.write("@severity ");
                    self.write_string_literal(&format!("{:?}", severity).to_lowercase());
                }
                if let Some(target) = &metadata.target {
                    self.newline();
                    self.write_indent();
                    self.write("@target ");
                    self.write(&target.to_string());
                }
                if let Some(ref window) = metadata.window {
                    self.newline();
                    self.write_indent();
                    self.write("@window ");
                    self.write(&window.num_seconds().to_string());
                    self.write(" \"seconds\"");
                }
                self.dedent();
                self.newline();
            }
            AstNode::MappingDecl {
                name,
                target,
                rules,
            } => {
                self.write("Mapping ");
                self.write_string_literal(name);
                self.write(" for ");
                self.write(&format!("{}", target).to_lowercase());
                self.write(" {");
                self.newline();
                self.indent();
                for rule in rules {
                    self.write_indent();
                    self.write(&rule.primitive_type);
                    self.write(" ");
                    self.write_string_literal(&rule.primitive_name);
                    self.write(" -> ");
                    self.write(&rule.target_type);
                    self.write(" { ");
                    let mut first = true;
                    for (k, v) in &rule.fields {
                        if !first {
                            self.write(", ");
                        }
                        self.write_string_literal(k);
                        self.write(": ");
                        self.write(&v.to_string());
                        first = false;
                    }
                    self.write(" }");
                    self.newline();
                }
                self.dedent();
                self.write_indent();
                self.write("}");
                self.newline();
            }
            AstNode::ProjectionDecl {
                name,
                target,
                overrides,
            } => {
                self.write("Projection ");
                self.write_string_literal(name);
                self.write(" for ");
                self.write(&format!("{}", target).to_lowercase());
                self.write(" {");
                self.newline();
                self.indent();
                for over in overrides {
                    self.write_indent();
                    self.write(&over.primitive_type);
                    self.write(" ");
                    self.write_string_literal(&over.primitive_name);
                    self.write(" { ");
                    let mut first = true;
                    for (k, v) in &over.fields {
                        if !first {
                            self.write(", ");
                        }
                        self.write_string_literal(k);
                        self.write(": ");
                        self.write(&v.to_string());
                        first = false;
                    }
                    self.write(" }");
                    self.newline();
                }
                self.dedent();
                self.write_indent();
                self.write("}");
                self.newline();
            }
        }
    }

    /// Format an expression using the Expression's Display implementation.
    ///
    /// The Expression type has a proper Display that outputs SEA-compatible syntax.
    fn format_expression(&mut self, expr: &Expression) {
        self.write(&format!("{}", expr));
    }

    // Helper methods

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_string_literal(&mut self, s: &str) {
        self.output.push('"');
        // Escape special characters
        for c in s.chars() {
            match c {
                '"' => self.output.push_str("\\\""),
                '\\' => self.output.push_str("\\\\"),
                '\n' => self.output.push_str("\\n"),
                '\r' => self.output.push_str("\\r"),
                '\t' => self.output.push_str("\\t"),
                _ => self.output.push(c),
            }
        }
        self.output.push('"');
    }

    fn newline(&mut self) {
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        let indent = self.config.indent_string();
        for _ in 0..self.indent_level {
            self.output.push_str(&indent);
        }
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_entity_basic() {
        let input = r#"Entity "Foo""#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("Entity \"Foo\""));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_format_entity_with_domain() {
        let input = r#"Entity   "Bar"   in   test"#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("Entity \"Bar\" in test"));
    }

    #[test]
    fn test_format_resource() {
        let input = r#"Resource "Money" USD in finance"#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("Resource \"Money\" USD in finance"));
    }

    #[test]
    fn test_format_flow() {
        let input = r#"Flow "Money" from "A" to "B" quantity 100"#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("Flow \"Money\" from \"A\" to \"B\" quantity 100"));
    }

    #[test]
    fn test_format_idempotent() {
        let input = r#"Entity "Test" in foo"#;
        let once = format(input, FormatConfig::default()).unwrap();
        let twice = format(&once, FormatConfig::default()).unwrap();
        assert_eq!(once, twice, "Formatting should be idempotent");
    }

    #[test]
    fn test_format_multiple_declarations() {
        let input = r#"
Entity "A"
Entity "B"
Resource "R" units
"#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("Entity \"A\""));
        assert!(result.contains("Entity \"B\""));
        assert!(result.contains("Resource \"R\""));
    }

    #[test]
    fn test_format_with_tabs() {
        let input = r#"
Relation "Test"
    subject: "A"
    predicate: "rel"
    object: "B"
"#;
        let config = FormatConfig::default().with_tabs();
        let result = format(input, config).unwrap();
        assert!(result.contains('\t'), "Should use tabs for indentation");
    }

    #[test]
    fn test_format_imports_sorted() {
        let input = r#"
import { B } from "z.sea"
import { A } from "a.sea"
Entity "Test"
"#;
        let config = FormatConfig::default();
        let result = format(input, config).unwrap();
        // A should come before B in sorted output
        let a_pos = result.find("a.sea").unwrap();
        let z_pos = result.find("z.sea").unwrap();
        assert!(a_pos < z_pos, "Imports should be sorted alphabetically");
    }

    #[test]
    fn test_format_policy_with_expression() {
        let input = r#"Policy test as: x = 5"#;
        let result = format(input, FormatConfig::default()).unwrap();
        // Should contain the expression, not Debug format
        assert!(result.contains("Policy test"));
        assert!(result.contains("as:"));
        // Expression should use Display format, not Debug
        assert!(!result.contains("Binary {"), "Should not use Debug format");
    }

    #[test]
    fn test_format_instance_with_fields() {
        let input = r#"
Instance test_user of "User" {
    name: "Alice",
    age: 30
}
"#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("Instance test_user of \"User\""));
        assert!(result.contains("name:"));
        assert!(result.contains("age:"));
    }

    #[test]
    fn test_format_preserves_header_comments() {
        let input = r#"// This is a header comment
// Second line
Entity "Foo"
"#;
        let result = format(input, FormatConfig::default()).unwrap();
        assert!(result.contains("// This is a header comment"));
        assert!(result.contains("// Second line"));
        assert!(result.contains("Entity \"Foo\""));
    }

    #[test]
    fn test_format_without_comments() {
        let input = r#"// Comment
Entity "Foo"
"#;
        let config = FormatConfig {
            preserve_comments: false,
            ..Default::default()
        };
        let result = format(input, config).unwrap();
        // Comments should not be preserved when disabled
        assert!(!result.contains("// Comment"));
        assert!(result.contains("Entity \"Foo\""));
    }
}
