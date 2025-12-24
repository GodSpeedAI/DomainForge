use crate::parser::ast::{
    Ast, AstNode, FileMetadata, ImportDecl, ImportItem, ImportSpecifier, MappingRule,
    ProjectionOverride,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Copy, Clone)]
enum ObjectStyle {
    ColonSeparated,
    ArrowSeparated,
}

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

    /// Returns a configured PrettyPrinter with trailing commas enabled/disabled.
    pub fn with_trailing_commas(mut self, trailing: bool) -> Self {
        self.trailing_commas = trailing;
        self
    }

    pub fn print(&self, ast: &Ast) -> String {
        let mut sections = Vec::new();
        let mut header = String::new();
        self.write_metadata(&mut header, &ast.metadata);
        if !header.trim().is_empty() {
            sections.push(header.trim_end().to_string());
        }

        for decl in &ast.declarations {
            sections.push(self.format_node(&decl.node, 0));
        }

        let mut output = sections.join("\n\n");
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output
    }

    fn indent(&self, level: usize) -> String {
        " ".repeat(self.indent_width * level)
    }

    fn quote(&self, value: &str) -> String {
        serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value))
    }

    fn write_metadata(&self, output: &mut String, metadata: &FileMetadata) {
        let mut wrote_header = false;

        if let Some(ns) = &metadata.namespace {
            let _ = writeln!(output, "@namespace {}", self.quote(ns));
            wrote_header = true;
        }
        if let Some(version) = &metadata.version {
            let _ = writeln!(output, "@version {}", self.quote(version));
            wrote_header = true;
        }
        if let Some(owner) = &metadata.owner {
            let _ = writeln!(output, "@owner {}", self.quote(owner));
            wrote_header = true;
        }

        for import in &metadata.imports {
            let _ = writeln!(output, "{}", self.format_import(import));
            wrote_header = true;
        }

        if wrote_header {
            let _ = writeln!(output);
        }
    }

    fn format_import(&self, import: &ImportDecl) -> String {
        let specifier = match &import.specifier {
            ImportSpecifier::Named(items) => {
                let rendered: Vec<String> = items
                    .iter()
                    .map(|item| self.render_import_item(item))
                    .collect();
                format!("{{ {} }}", rendered.join(", "))
            }
            ImportSpecifier::Wildcard(alias) => format!("* as {}", alias),
        };
        format!(
            "Import {} from {}",
            specifier,
            self.quote(&import.from_module)
        )
    }

    fn render_import_item(&self, item: &ImportItem) -> String {
        match &item.alias {
            Some(alias) => format!("{} as {}", item.name, alias),
            None => item.name.clone(),
        }
    }

    fn format_node(&self, node: &AstNode, indent_level: usize) -> String {
        match node {
            AstNode::Export(inner) => self.format_export(&inner.node, indent_level),
            AstNode::Entity {
                name,
                version,
                annotations,
                domain,
            } => self.format_entity(name, version, annotations, domain),
            AstNode::Resource {
                name,
                annotations,
                unit_name,
                domain,
            } => self.format_resource(name, annotations, unit_name.as_deref(), domain.as_deref()),
            AstNode::Flow {
                resource_name,
                annotations,
                from_entity,
                to_entity,
                quantity,
            } => self.format_flow(
                resource_name,
                annotations,
                from_entity,
                to_entity,
                *quantity,
            ),
            AstNode::Pattern { name, regex } => self.format_pattern(name, regex),
            AstNode::Role { name, domain } => self.format_role(name, domain),
            AstNode::Relation {
                name,
                subject_role,
                predicate,
                object_role,
                via_flow,
            } => self.format_relation(name, subject_role, predicate, object_role, via_flow),
            AstNode::Dimension { name } => format!("Dimension {}", self.quote(name)),
            AstNode::UnitDeclaration {
                symbol,
                dimension,
                factor,
                base_unit,
            } => self.format_unit(symbol, dimension, factor, base_unit),
            AstNode::Policy {
                name,
                version,
                metadata,
                expression,
            } => self.format_policy(name, version, metadata, expression),
            AstNode::Instance {
                name,
                entity_type,
                fields,
            } => self.format_instance(name, entity_type, fields),
            AstNode::ConceptChange {
                name,
                from_version,
                to_version,
                migration_policy,
                breaking_change,
            } => self.format_concept_change(
                name,
                from_version,
                to_version,
                migration_policy,
                *breaking_change,
            ),
            AstNode::Metric {
                name,
                expression,
                metadata,
            } => self.format_metric(name, expression, metadata),
            AstNode::MappingDecl {
                name,
                target,
                rules,
            } => self.format_mapping(name, target, rules),
            AstNode::ProjectionDecl {
                name,
                target,
                overrides,
            } => self.format_projection(name, target, overrides),
        }
    }

    fn format_export(&self, node: &AstNode, indent_level: usize) -> String {
        let inner = self.format_node(node, indent_level);
        let mut lines = inner.lines();
        if let Some(first) = lines.next() {
            let mut rendered = vec![format!("Export {}", first)];
            rendered.extend(lines.map(|line| line.to_string()));
            rendered.join("\n")
        } else {
            String::new()
        }
    }

    fn format_entity(
        &self,
        name: &str,
        version: &Option<String>,
        annotations: &HashMap<String, JsonValue>,
        domain: &Option<String>,
    ) -> String {
        let mut lines = Vec::new();
        let mut head = format!("Entity {}", self.quote(name));
        if let Some(v) = version {
            head.push_str(&format!(" v{}", v));
        }
        lines.push(head);

        if let Some(replaces) = annotations.get("replaces").and_then(JsonValue::as_str) {
            lines.push(format!(
                "{}@replaces {}",
                self.indent(1),
                self.format_replaces_annotation(replaces)
            ));
        }
        if let Some(changes) = annotations.get("changes").and_then(JsonValue::as_array) {
            let rendered = changes
                .iter()
                .filter_map(JsonValue::as_str)
                .map(|c| self.quote(c))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("{}@changes [{}]", self.indent(1), rendered));
        }
        if let Some(ns) = domain {
            lines.push(format!("{}in {}", self.indent(1), ns));
        }

        lines.join("\n")
    }

    fn format_replaces_annotation(&self, value: &str) -> String {
        if let Some((name, version)) = value.rsplit_once(" v") {
            format!("{} v{}", self.quote(name), version)
        } else {
            self.quote(value)
        }
    }

    fn format_resource(
        &self,
        name: &str,
        annotations: &HashMap<String, JsonValue>,
        unit: Option<&str>,
        domain: Option<&str>,
    ) -> String {
        let mut lines = Vec::new();
        let head = format!("Resource {}", self.quote(name));
        lines.push(head);

        if let Some(replaces) = annotations.get("replaces").and_then(JsonValue::as_str) {
            lines.push(format!(
                "{}@replaces {}",
                self.indent(1),
                self.format_replaces_annotation(replaces)
            ));
        }
        if let Some(changes) = annotations.get("changes").and_then(JsonValue::as_array) {
            let rendered = changes
                .iter()
                .filter_map(JsonValue::as_str)
                .map(|c| self.quote(c))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("{}@changes [{}]", self.indent(1), rendered));
        }

        // Add unit and domain to first line if present
        if unit.is_some() || domain.is_some() {
            if lines.len() == 1 {
                // No annotations, put on single line
                if let Some(u) = unit {
                    lines[0].push_str(&format!(" {}", u));
                }
                if let Some(ns) = domain {
                    lines[0].push_str(&format!(" in {}", ns));
                }
            } else {
                // Has annotations, add unit/domain on separate line
                let mut suffix = String::new();
                if let Some(u) = unit {
                    suffix.push_str(u);
                }
                if let Some(ns) = domain {
                    if !suffix.is_empty() {
                        suffix.push(' ');
                    }
                    suffix.push_str(&format!("in {}", ns));
                }
                if !suffix.is_empty() {
                    lines.push(format!("{}{}", self.indent(1), suffix));
                }
            }
        }

        lines.join("\n")
    }

    fn format_flow(
        &self,
        resource: &str,
        annotations: &HashMap<String, JsonValue>,
        from: &str,
        to: &str,
        quantity: Option<i32>,
    ) -> String {
        let mut lines = Vec::new();
        let head = format!("Flow {}", self.quote(resource));
        lines.push(head);

        if let Some(replaces) = annotations.get("replaces").and_then(JsonValue::as_str) {
            lines.push(format!(
                "{}@replaces {}",
                self.indent(1),
                self.format_replaces_annotation(replaces)
            ));
        }
        if let Some(changes) = annotations.get("changes").and_then(JsonValue::as_array) {
            let rendered = changes
                .iter()
                .filter_map(JsonValue::as_str)
                .map(|c| self.quote(c))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("{}@changes [{}]", self.indent(1), rendered));
        }

        // Add from/to/quantity
        let mut suffix = format!("from {} to {}", self.quote(from), self.quote(to));
        if let Some(qty) = quantity {
            suffix.push_str(&format!(" quantity {}", qty));
        }

        if lines.len() == 1 {
            // No annotations, put on single line
            lines[0].push_str(&format!(" {}", suffix));
        } else {
            // Has annotations, add on separate line
            lines.push(format!("{}{}", self.indent(1), suffix));
        }

        lines.join("\n")
    }

    fn format_pattern(&self, name: &str, regex: &str) -> String {
        format!("Pattern {} matches {}", self.quote(name), self.quote(regex))
    }

    fn format_role(&self, name: &str, domain: &Option<String>) -> String {
        match domain {
            Some(ns) => format!("Role {} in {}", self.quote(name), ns),
            None => format!("Role {}", self.quote(name)),
        }
    }

    fn format_relation(
        &self,
        name: &str,
        subject: &str,
        predicate: &str,
        object: &str,
        via_flow: &Option<String>,
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Relation {}", self.quote(name)));
        lines.push(format!(
            "{}subject: {}",
            self.indent(1),
            self.quote(subject)
        ));
        lines.push(format!(
            "{}predicate: {}",
            self.indent(1),
            self.quote(predicate)
        ));
        lines.push(format!("{}object: {}", self.indent(1), self.quote(object)));
        if let Some(flow) = via_flow {
            lines.push(format!("{}via: flow {}", self.indent(1), self.quote(flow)));
        }
        lines.join("\n")
    }

    fn format_unit(
        &self,
        symbol: &str,
        dimension: &str,
        factor: &rust_decimal::Decimal,
        base_unit: &str,
    ) -> String {
        format!(
            "Unit {} of {} factor {} base {}",
            self.quote(symbol),
            self.quote(dimension),
            factor,
            self.quote(base_unit)
        )
    }

    fn format_policy(
        &self,
        name: &str,
        version: &Option<String>,
        metadata: &crate::parser::ast::PolicyMetadata,
        expression: &crate::policy::Expression,
    ) -> String {
        let mut header = format!("Policy {}", name);
        if let Some(kind) = &metadata.kind {
            header.push_str(&format!(" per {}", kind));
        }
        if let Some(modality) = &metadata.modality {
            header.push_str(&format!(" {}", modality));
        }
        if let Some(priority) = metadata.priority {
            header.push_str(&format!(" priority {}", priority));
        }
        if let Some(rationale) = &metadata.rationale {
            header.push_str(&format!(" @rationale {}", self.quote(rationale)));
        }
        if !metadata.tags.is_empty() {
            let tags = metadata
                .tags
                .iter()
                .map(|t| self.quote(t))
                .collect::<Vec<_>>()
                .join(", ");
            header.push_str(&format!(" @tags [{}]", tags));
        }
        if let Some(v) = version {
            header.push_str(&format!(" v{}", v));
        }
        header.push_str(" as:");

        let mut lines = Vec::new();
        lines.push(header);
        lines.push(format!("{}{}", self.indent(1), expression));
        lines.join("\n")
    }

    fn format_instance(
        &self,
        name: &str,
        entity_type: &str,
        fields: &HashMap<String, crate::policy::Expression>,
    ) -> String {
        if fields.is_empty() {
            return format!("Instance {} of {}", name, self.quote(entity_type));
        }

        let mut lines = Vec::new();
        lines.push(format!(
            "Instance {} of {} {{",
            name,
            self.quote(entity_type)
        ));

        let mut entries: Vec<_> = fields.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (idx, (field, value)) in entries.iter().enumerate() {
            let is_last = idx == entries.len() - 1;
            let suffix = if self.trailing_commas {
                ","
            } else if is_last {
                ""
            } else {
                ","
            };
            lines.push(format!("{}{}: {}{}", self.indent(1), field, value, suffix));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    fn format_concept_change(
        &self,
        name: &str,
        from_version: &str,
        to_version: &str,
        migration_policy: &str,
        breaking_change: bool,
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("ConceptChange {}", self.quote(name)));
        lines.push(format!("{}@from_version v{}", self.indent(1), from_version));
        lines.push(format!("{}@to_version v{}", self.indent(1), to_version));
        lines.push(format!(
            "{}@migration_policy {}",
            self.indent(1),
            migration_policy
        ));
        lines.push(format!(
            "{}@breaking_change {}",
            self.indent(1),
            breaking_change
        ));
        lines.join("\n")
    }

    fn format_metric(
        &self,
        name: &str,
        expression: &crate::policy::Expression,
        metadata: &crate::parser::ast::MetricMetadata,
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Metric {} as:", self.quote(name)));
        lines.push(format!("{}{}", self.indent(1), expression));

        if let Some(refresh) = metadata.refresh_interval {
            lines.push(format!(
                "{}@refresh_interval {} \"seconds\"",
                self.indent(1),
                refresh.num_seconds()
            ));
        }
        if let Some(unit) = &metadata.unit {
            lines.push(format!("{}@unit {}", self.indent(1), self.quote(unit)));
        }
        if let Some(threshold) = metadata.threshold {
            lines.push(format!("{}@threshold {}", self.indent(1), threshold));
        }
        if let Some(severity) = &metadata.severity {
            lines.push(format!(
                "{}@severity {}",
                self.indent(1),
                self.quote(&format!("{:?}", severity))
            ));
        }
        if let Some(target) = metadata.target {
            lines.push(format!("{}@target {}", self.indent(1), target));
        }
        if let Some(window) = metadata.window {
            lines.push(format!(
                "{}@window {} \"seconds\"",
                self.indent(1),
                window.num_seconds()
            ));
        }

        lines.join("\n")
    }

    fn format_mapping(
        &self,
        name: &str,
        target: &crate::parser::ast::TargetFormat,
        rules: &[MappingRule],
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Mapping {} for {} {{", self.quote(name), target));

        for rule in rules {
            let mut field_lines = Vec::new();
            field_lines.push(format!(
                "{}{} {} -> {} {{",
                self.indent(1),
                rule.primitive_type,
                self.quote(&rule.primitive_name),
                rule.target_type
            ));

            let mut fields: Vec<_> = rule.fields.iter().collect();
            fields.sort_by(|a, b| a.0.cmp(b.0));
            for (idx, (field, value)) in fields.iter().enumerate() {
                let is_last = idx == fields.len() - 1;
                let suffix = if self.trailing_commas {
                    ","
                } else if is_last {
                    ""
                } else {
                    ","
                };
                field_lines.push(format!(
                    "{}{}: {}{}",
                    self.indent(2),
                    field,
                    self.format_mapping_value(value, ObjectStyle::ColonSeparated),
                    suffix
                ));
            }
            field_lines.push(format!("{}}}", self.indent(1)));
            lines.push(field_lines.join("\n"));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    fn format_projection(
        &self,
        name: &str,
        target: &crate::parser::ast::TargetFormat,
        overrides: &[ProjectionOverride],
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Projection {} for {} {{", self.quote(name), target));

        for override_entry in overrides {
            let mut override_lines = Vec::new();
            override_lines.push(format!(
                "{}{} {} {{",
                self.indent(1),
                override_entry.primitive_type,
                self.quote(&override_entry.primitive_name)
            ));

            let mut fields: Vec<_> = override_entry.fields.iter().collect();
            fields.sort_by(|a, b| a.0.cmp(b.0));
            for (idx, (field, value)) in fields.iter().enumerate() {
                let is_last = idx == fields.len() - 1;
                let suffix = if self.trailing_commas {
                    ","
                } else if is_last {
                    ""
                } else {
                    ","
                };
                override_lines.push(format!(
                    "{}{}: {}{}",
                    self.indent(2),
                    field,
                    self.format_mapping_value(value, ObjectStyle::ArrowSeparated),
                    suffix
                ));
            }
            override_lines.push(format!("{}}}", self.indent(1)));
            lines.push(override_lines.join("\n"));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    fn format_mapping_value(&self, value: &JsonValue, object_style: ObjectStyle) -> String {
        match value {
            JsonValue::String(s) => self.quote(s),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Object(map) => {
                let mut parts: Vec<_> = map.iter().collect();
                parts.sort_by(|a, b| a.0.cmp(b.0));
                let rendered = parts
                    .into_iter()
                    .map(|(k, v)| {
                        let rendered_value = match v {
                            JsonValue::String(s) => self.quote(s),
                            JsonValue::Bool(b) => b.to_string(),
                            JsonValue::Number(n) => n.to_string(),
                            JsonValue::Object(_) | JsonValue::Array(_) => {
                                // Recursively format nested objects/arrays so they are rendered correctly
                                self.format_mapping_value(v, object_style)
                            }
                            _ => self.quote(&v.to_string()),
                        };
                        let separator = match object_style {
                            ObjectStyle::ColonSeparated => ":",
                            ObjectStyle::ArrowSeparated => "->",
                        };
                        format!("{} {} {}", self.quote(k), separator, rendered_value)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", rendered)
            }
            JsonValue::Array(arr) => {
                let items = arr
                    .iter()
                    .map(|v| self.format_mapping_value(v, object_style))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items)
            }
            _ => self.quote(&value.to_string()),
        }
    }
}
