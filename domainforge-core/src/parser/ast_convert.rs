//! Conversion functions from internal AST types to schema types.
//!
//! This module provides `From` implementations to convert between
//! the internal AST representation and the serializable schema types.

use crate::parser::ast::{self, Spanned};
use crate::parser::ast_schema as schema;
use crate::policy::Expression as CoreExpression;

// =========================================================================
// FileMetadata Conversion
// =========================================================================

impl From<&ast::FileMetadata> for schema::FileMetadata {
    fn from(m: &ast::FileMetadata) -> Self {
        schema::FileMetadata {
            namespace: m.namespace.clone(),
            version: m.version.clone(),
            owner: m.owner.clone(),
            profile: m.profile.clone(),
            imports: m.imports.iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<&ast::ImportDecl> for schema::ImportDecl {
    fn from(i: &ast::ImportDecl) -> Self {
        schema::ImportDecl {
            specifier: (&i.specifier).into(),
            from_module: i.from_module.clone(),
        }
    }
}

impl From<&ast::ImportSpecifier> for schema::ImportSpecifier {
    fn from(s: &ast::ImportSpecifier) -> Self {
        match s {
            ast::ImportSpecifier::Named(items) => schema::ImportSpecifier::Named {
                items: items.iter().map(|i| i.into()).collect(),
            },
            ast::ImportSpecifier::Wildcard(alias) => schema::ImportSpecifier::Wildcard {
                alias: alias.clone(),
            },
        }
    }
}

impl From<&ast::ImportItem> for schema::ImportItem {
    fn from(i: &ast::ImportItem) -> Self {
        schema::ImportItem {
            name: i.name.clone(),
            alias: i.alias.clone(),
        }
    }
}

// =========================================================================
// Policy Metadata Conversion
// =========================================================================

impl From<&ast::PolicyMetadata> for schema::PolicyMetadata {
    fn from(m: &ast::PolicyMetadata) -> Self {
        schema::PolicyMetadata {
            kind: m.kind.as_ref().map(|k| k.into()),
            modality: m.modality.as_ref().map(|m| m.into()),
            priority: m.priority,
            rationale: m.rationale.clone(),
            tags: m.tags.clone(),
        }
    }
}

impl From<&ast::PolicyKind> for schema::PolicyKind {
    fn from(k: &ast::PolicyKind) -> Self {
        match k {
            ast::PolicyKind::Constraint => schema::PolicyKind::Constraint,
            ast::PolicyKind::Derivation => schema::PolicyKind::Derivation,
            ast::PolicyKind::Obligation => schema::PolicyKind::Obligation,
        }
    }
}

impl From<&ast::PolicyModality> for schema::PolicyModality {
    fn from(m: &ast::PolicyModality) -> Self {
        match m {
            ast::PolicyModality::Obligation => schema::PolicyModality::Obligation,
            ast::PolicyModality::Prohibition => schema::PolicyModality::Prohibition,
            ast::PolicyModality::Permission => schema::PolicyModality::Permission,
        }
    }
}

// =========================================================================
// Metric Metadata Conversion
// =========================================================================

impl From<&ast::MetricMetadata> for schema::MetricMetadata {
    fn from(m: &ast::MetricMetadata) -> Self {
        schema::MetricMetadata {
            refresh_interval: m
                .refresh_interval
                .map(|d| format!("PT{}S", d.num_seconds())),
            unit: m.unit.clone(),
            threshold: m.threshold.map(|d| d.to_string()),
            severity: m.severity.as_ref().map(|s| s.into()),
            target: m.target.map(|d| d.to_string()),
            window: m.window.map(|d| format!("PT{}S", d.num_seconds())),
        }
    }
}

impl From<&crate::primitives::Severity> for schema::Severity {
    fn from(s: &crate::primitives::Severity) -> Self {
        match s {
            crate::primitives::Severity::Info => schema::Severity::Info,
            crate::primitives::Severity::Warning => schema::Severity::Warning,
            crate::primitives::Severity::Error => schema::Severity::Error,
            // Handle any additional variants
            _ => schema::Severity::Error,
        }
    }
}

// =========================================================================
// Target Format Conversion
// =========================================================================

impl From<&ast::TargetFormat> for schema::TargetFormat {
    fn from(t: &ast::TargetFormat) -> Self {
        match t {
            ast::TargetFormat::Calm => schema::TargetFormat::Calm,
            ast::TargetFormat::Kg => schema::TargetFormat::Kg,
            ast::TargetFormat::Sbvr => schema::TargetFormat::Sbvr,
            ast::TargetFormat::Protobuf => schema::TargetFormat::Protobuf,
        }
    }
}

impl From<&ast::MappingRule> for schema::MappingRule {
    fn from(r: &ast::MappingRule) -> Self {
        schema::MappingRule {
            primitive_type: r.primitive_type.clone(),
            primitive_name: r.primitive_name.clone(),
            target_type: r.target_type.clone(),
            fields: r.fields.clone(),
        }
    }
}

impl From<&ast::ProjectionOverride> for schema::ProjectionOverride {
    fn from(o: &ast::ProjectionOverride) -> Self {
        schema::ProjectionOverride {
            primitive_type: o.primitive_type.clone(),
            primitive_name: o.primitive_name.clone(),
            fields: o.fields.clone(),
        }
    }
}

// =========================================================================
// Expression Conversion
// =========================================================================

impl From<&CoreExpression> for schema::Expression {
    fn from(e: &CoreExpression) -> Self {
        match e {
            CoreExpression::Literal(v) => schema::Expression::Literal { value: v.clone() },
            CoreExpression::QuantityLiteral { value, unit } => {
                schema::Expression::QuantityLiteral {
                    value: value.to_string(),
                    unit: unit.clone(),
                }
            }
            CoreExpression::TimeLiteral(ts) => schema::Expression::TimeLiteral {
                timestamp: ts.clone(),
            },
            CoreExpression::IntervalLiteral { start, end } => schema::Expression::IntervalLiteral {
                start: start.clone(),
                end: end.clone(),
            },
            CoreExpression::Variable(name) => schema::Expression::Variable { name: name.clone() },
            CoreExpression::GroupBy {
                variable,
                collection,
                filter,
                key,
                condition,
            } => schema::Expression::GroupBy {
                variable: variable.clone(),
                collection: Box::new(collection.as_ref().into()),
                filter: filter.as_ref().map(|f| Box::new(f.as_ref().into())),
                key: Box::new(key.as_ref().into()),
                condition: Box::new(condition.as_ref().into()),
            },
            CoreExpression::Binary { op, left, right } => schema::Expression::Binary {
                op: op.into(),
                left: Box::new(left.as_ref().into()),
                right: Box::new(right.as_ref().into()),
            },
            CoreExpression::Unary { op, operand } => schema::Expression::Unary {
                op: op.into(),
                operand: Box::new(operand.as_ref().into()),
            },
            CoreExpression::Cast {
                operand,
                target_type,
            } => schema::Expression::Cast {
                operand: Box::new(operand.as_ref().into()),
                target_type: target_type.clone(),
            },
            CoreExpression::Quantifier {
                quantifier,
                variable,
                collection,
                condition,
            } => schema::Expression::Quantifier {
                quantifier: quantifier.into(),
                variable: variable.clone(),
                collection: Box::new(collection.as_ref().into()),
                condition: Box::new(condition.as_ref().into()),
            },
            CoreExpression::MemberAccess { object, member } => schema::Expression::MemberAccess {
                object: object.clone(),
                member: member.clone(),
            },
            CoreExpression::Aggregation {
                function,
                collection,
                field,
                filter,
            } => schema::Expression::Aggregation {
                function: function.into(),
                collection: Box::new(collection.as_ref().into()),
                field: field.clone(),
                filter: filter.as_ref().map(|f| Box::new(f.as_ref().into())),
            },
            CoreExpression::AggregationComprehension {
                function,
                variable,
                collection,
                window,
                predicate,
                projection,
                target_unit,
            } => schema::Expression::AggregationComprehension {
                function: function.into(),
                variable: variable.clone(),
                collection: Box::new(collection.as_ref().into()),
                window: window.as_ref().map(|w| w.into()),
                predicate: Box::new(predicate.as_ref().into()),
                projection: Box::new(projection.as_ref().into()),
                target_unit: target_unit.clone(),
            },
            CoreExpression::RoleReference { role } => {
                schema::Expression::RoleReference { role: role.clone() }
            }
        }
    }
}

impl From<&crate::policy::BinaryOp> for schema::BinaryOp {
    fn from(op: &crate::policy::BinaryOp) -> Self {
        match op {
            crate::policy::BinaryOp::And => schema::BinaryOp::And,
            crate::policy::BinaryOp::Or => schema::BinaryOp::Or,
            crate::policy::BinaryOp::Equal => schema::BinaryOp::Equal,
            crate::policy::BinaryOp::NotEqual => schema::BinaryOp::NotEqual,
            crate::policy::BinaryOp::GreaterThan => schema::BinaryOp::GreaterThan,
            crate::policy::BinaryOp::LessThan => schema::BinaryOp::LessThan,
            crate::policy::BinaryOp::GreaterThanOrEqual => schema::BinaryOp::GreaterThanOrEqual,
            crate::policy::BinaryOp::LessThanOrEqual => schema::BinaryOp::LessThanOrEqual,
            crate::policy::BinaryOp::Plus => schema::BinaryOp::Plus,
            crate::policy::BinaryOp::Minus => schema::BinaryOp::Minus,
            crate::policy::BinaryOp::Multiply => schema::BinaryOp::Multiply,
            crate::policy::BinaryOp::Divide => schema::BinaryOp::Divide,
            crate::policy::BinaryOp::Contains => schema::BinaryOp::Contains,
            crate::policy::BinaryOp::StartsWith => schema::BinaryOp::StartsWith,
            crate::policy::BinaryOp::EndsWith => schema::BinaryOp::EndsWith,
            crate::policy::BinaryOp::Matches => schema::BinaryOp::Matches,
            crate::policy::BinaryOp::HasRole => schema::BinaryOp::HasRole,
            crate::policy::BinaryOp::Before => schema::BinaryOp::Before,
            crate::policy::BinaryOp::After => schema::BinaryOp::After,
            crate::policy::BinaryOp::During => schema::BinaryOp::During,
        }
    }
}

impl From<&crate::policy::UnaryOp> for schema::UnaryOp {
    fn from(op: &crate::policy::UnaryOp) -> Self {
        match op {
            crate::policy::UnaryOp::Not => schema::UnaryOp::Not,
            crate::policy::UnaryOp::Negate => schema::UnaryOp::Negate,
        }
    }
}

impl From<&crate::policy::Quantifier> for schema::Quantifier {
    fn from(q: &crate::policy::Quantifier) -> Self {
        match q {
            crate::policy::Quantifier::ForAll => schema::Quantifier::ForAll,
            crate::policy::Quantifier::Exists => schema::Quantifier::Exists,
            crate::policy::Quantifier::ExistsUnique => schema::Quantifier::ExistsUnique,
        }
    }
}

impl From<&crate::policy::AggregateFunction> for schema::AggregateFunction {
    fn from(f: &crate::policy::AggregateFunction) -> Self {
        match f {
            crate::policy::AggregateFunction::Count => schema::AggregateFunction::Count,
            crate::policy::AggregateFunction::Sum => schema::AggregateFunction::Sum,
            crate::policy::AggregateFunction::Min => schema::AggregateFunction::Min,
            crate::policy::AggregateFunction::Max => schema::AggregateFunction::Max,
            crate::policy::AggregateFunction::Avg => schema::AggregateFunction::Avg,
        }
    }
}

impl From<&crate::policy::WindowSpec> for schema::WindowSpec {
    fn from(w: &crate::policy::WindowSpec) -> Self {
        schema::WindowSpec {
            duration: w.duration,
            unit: "seconds".to_string(),
        }
    }
}

// =========================================================================
// AST Node Conversion
// =========================================================================

impl From<&Spanned<ast::AstNode>> for schema::SpannedAstNode {
    fn from(s: &Spanned<ast::AstNode>) -> Self {
        schema::SpannedAstNode {
            node: (&s.node).into(),
            line: s.line,
            column: s.column,
        }
    }
}

impl From<&ast::AstNode> for schema::AstNode {
    fn from(n: &ast::AstNode) -> Self {
        match n {
            ast::AstNode::Export(inner) => schema::AstNode::Export {
                declaration: Box::new(inner.as_ref().into()),
            },
            ast::AstNode::Entity {
                name,
                version,
                annotations,
                domain,
                body,
            } => schema::AstNode::Entity {
                name: name.clone(),
                version: version.clone(),
                annotations: annotations.clone(),
                domain: domain.clone(),
                body: body.as_ref().map(|b| b.into()),
            },
            ast::AstNode::Record(r) => schema::AstNode::Record {
                declaration: r.into(),
            },
            ast::AstNode::Enum(e) => schema::AstNode::Enum {
                declaration: e.into(),
            },
            ast::AstNode::Operation(o) => schema::AstNode::Operation {
                declaration: o.into(),
            },
            ast::AstNode::Resource {
                name,
                annotations,
                unit_name,
                domain,
            } => schema::AstNode::Resource {
                name: name.clone(),
                annotations: annotations.clone(),
                unit_name: unit_name.clone(),
                domain: domain.clone(),
            },
            ast::AstNode::Flow {
                resource_name,
                annotations,
                from_entity,
                to_entity,
                quantity,
            } => schema::AstNode::Flow {
                resource_name: resource_name.clone(),
                annotations: annotations.clone(),
                from_entity: from_entity.clone(),
                to_entity: to_entity.clone(),
                quantity: quantity.map(|d| d.to_string()),
            },
            ast::AstNode::Pattern { name, regex } => schema::AstNode::Pattern {
                name: name.clone(),
                regex: regex.clone(),
            },
            ast::AstNode::Role { name, domain } => schema::AstNode::Role {
                name: name.clone(),
                domain: domain.clone(),
            },
            ast::AstNode::Relation {
                name,
                subject_role,
                predicate,
                object_role,
                via_flow,
            } => schema::AstNode::Relation {
                name: name.clone(),
                subject_role: subject_role.clone(),
                predicate: predicate.clone(),
                object_role: object_role.clone(),
                via_flow: via_flow.clone(),
            },
            ast::AstNode::Dimension { name } => schema::AstNode::Dimension { name: name.clone() },
            ast::AstNode::UnitDeclaration {
                symbol,
                dimension,
                factor,
                base_unit,
            } => schema::AstNode::UnitDeclaration {
                symbol: symbol.clone(),
                dimension: dimension.clone(),
                factor: factor.to_string(),
                base_unit: base_unit.clone(),
            },
            ast::AstNode::Policy {
                name,
                version,
                metadata,
                expression,
            } => schema::AstNode::Policy {
                name: name.clone(),
                version: version.clone(),
                metadata: metadata.into(),
                expression: expression.into(),
            },
            ast::AstNode::Instance {
                name,
                entity_type,
                fields,
            } => schema::AstNode::Instance {
                name: name.clone(),
                entity_type: entity_type.clone(),
                fields: fields.iter().map(|(k, v)| (k.clone(), v.into())).collect(),
            },
            ast::AstNode::ConceptChange {
                name,
                from_version,
                to_version,
                migration_policy,
                breaking_change,
            } => schema::AstNode::ConceptChange {
                name: name.clone(),
                from_version: from_version.clone(),
                to_version: to_version.clone(),
                migration_policy: migration_policy.clone(),
                breaking_change: *breaking_change,
            },
            ast::AstNode::Metric {
                name,
                expression,
                metadata,
            } => schema::AstNode::Metric {
                name: name.clone(),
                expression: expression.into(),
                metadata: metadata.into(),
            },
            ast::AstNode::MappingDecl {
                name,
                target,
                rules,
            } => schema::AstNode::MappingDecl {
                name: name.clone(),
                target: target.into(),
                rules: rules.iter().map(|r| r.into()).collect(),
            },
            ast::AstNode::ProjectionDecl {
                name,
                target,
                overrides,
            } => schema::AstNode::ProjectionDecl {
                name: name.clone(),
                target: target.into(),
                overrides: overrides.iter().map(|o| o.into()).collect(),
            },
            ast::AstNode::Cell { name, annotations } => schema::AstNode::Cell {
                name: name.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::SystemDependency {
                name,
                version,
                annotations,
            } => schema::AstNode::SystemDependency {
                name: name.clone(),
                version: version.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::Runtime {
                name,
                version,
                annotations,
            } => schema::AstNode::Runtime {
                name: name.clone(),
                version: version.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::Tool {
                name,
                version,
                annotations,
            } => schema::AstNode::Tool {
                name: name.clone(),
                version: version.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::DependencySet { name, annotations } => schema::AstNode::DependencySet {
                name: name.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::Service {
                name,
                version,
                annotations,
            } => schema::AstNode::Service {
                name: name.clone(),
                version: version.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::Mount { name, annotations } => schema::AstNode::Mount {
                name: name.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::Endpoint { name, annotations } => schema::AstNode::Endpoint {
                name: name.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::NetworkFlow {
                name,
                from_ref,
                to_ref,
                annotations,
            } => schema::AstNode::NetworkFlow {
                name: name.clone(),
                from_ref: from_ref.clone(),
                to_ref: to_ref.clone(),
                annotations: annotations.clone(),
            },
            ast::AstNode::Credential { name, annotations } => schema::AstNode::Credential {
                name: name.clone(),
                annotations: annotations.clone(),
            },
        }
    }
}

// =========================================================================
// Root AST Conversion
// =========================================================================

impl From<&ast::Ast> for schema::Ast {
    fn from(a: &ast::Ast) -> Self {
        schema::Ast {
            metadata: (&a.metadata).into(),
            declarations: a.declarations.iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<ast::Ast> for schema::Ast {
    fn from(a: ast::Ast) -> Self {
        (&a).into()
    }
}

// ---- SEA application contract conversion (ADR-013) ----

impl From<&ast::EntityBody> for schema::EntityBody {
    fn from(b: &ast::EntityBody) -> Self {
        schema::EntityBody {
            fields: b.fields.iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<&ast::FieldDecl> for schema::FieldDecl {
    fn from(f: &ast::FieldDecl) -> Self {
        schema::FieldDecl {
            is_key: f.is_key,
            name: f.name.clone(),
            field_type: (&f.field_type).into(),
            is_optional: f.is_optional,
            constraints: f.constraints.iter().map(|c| c.into()).collect(),
            default: f.default.as_ref().map(|e| e.into()),
        }
    }
}

impl From<&ast::FieldType> for schema::FieldType {
    fn from(t: &ast::FieldType) -> Self {
        match t {
            ast::FieldType::Scalar(s) => schema::FieldType::Scalar { symbol: s.into() },
            ast::FieldType::Quantity(s) => schema::FieldType::Quantity { symbol: s.into() },
            ast::FieldType::Ref(s) => schema::FieldType::Ref { symbol: s.into() },
            ast::FieldType::Named(s) => schema::FieldType::Named { symbol: s.into() },
            ast::FieldType::List(elem) => schema::FieldType::List {
                element: Box::new(elem.as_ref().into()),
            },
        }
    }
}

impl From<&ast::FieldTypeRef> for schema::FieldTypeRef {
    fn from(r: &ast::FieldTypeRef) -> Self {
        schema::FieldTypeRef {
            alias: r.alias.clone(),
            symbol: r.symbol.clone(),
        }
    }
}

impl From<&ast::FieldConstraintDecl> for schema::FieldConstraintDecl {
    fn from(c: &ast::FieldConstraintDecl) -> Self {
        match c {
            ast::FieldConstraintDecl::MinLength(v) => {
                schema::FieldConstraintDecl::MinLength { value: *v }
            }
            ast::FieldConstraintDecl::MaxLength(v) => {
                schema::FieldConstraintDecl::MaxLength { value: *v }
            }
            ast::FieldConstraintDecl::MinItems(v) => {
                schema::FieldConstraintDecl::MinItems { value: *v }
            }
            ast::FieldConstraintDecl::MaxItems(v) => {
                schema::FieldConstraintDecl::MaxItems { value: *v }
            }
            ast::FieldConstraintDecl::ExclusiveMin(v) => {
                schema::FieldConstraintDecl::ExclusiveMin {
                    value: v.to_string(),
                }
            }
            ast::FieldConstraintDecl::ExclusiveMax(v) => {
                schema::FieldConstraintDecl::ExclusiveMax {
                    value: v.to_string(),
                }
            }
            ast::FieldConstraintDecl::Min(v) => schema::FieldConstraintDecl::Min {
                value: v.to_string(),
            },
            ast::FieldConstraintDecl::Max(v) => schema::FieldConstraintDecl::Max {
                value: v.to_string(),
            },
            ast::FieldConstraintDecl::Pattern(s) => {
                schema::FieldConstraintDecl::Pattern { symbol: s.clone() }
            }
        }
    }
}

impl From<&ast::RecordDecl> for schema::RecordDecl {
    fn from(r: &ast::RecordDecl) -> Self {
        schema::RecordDecl {
            name: r.name.clone(),
            fields: r.fields.iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<&ast::EnumDecl> for schema::EnumDecl {
    fn from(e: &ast::EnumDecl) -> Self {
        schema::EnumDecl {
            name: e.name.clone(),
            members: e
                .members
                .iter()
                .map(|m| schema::EnumMember {
                    name: m.name.clone(),
                    wire: m.wire.clone(),
                })
                .collect(),
        }
    }
}

impl From<&ast::OperationDecl> for schema::OperationDecl {
    fn from(o: &ast::OperationDecl) -> Self {
        schema::OperationDecl {
            name: o.name.clone(),
            clauses: o.clauses.iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<&ast::OperationClause> for schema::OperationClause {
    fn from(c: &ast::OperationClause) -> Self {
        match c {
            ast::OperationClause::Intent(text) => {
                schema::OperationClause::Intent { text: text.clone() }
            }
            ast::OperationClause::Direction { kind } => {
                schema::OperationClause::Direction { kind: kind.clone() }
            }
            ast::OperationClause::Actor { actor } => schema::OperationClause::Actor {
                actor: actor.clone(),
            },
            ast::OperationClause::AccessPublic => schema::OperationClause::AccessPublic,
            ast::OperationClause::AccessPolicyGoverned { bindings } => {
                schema::OperationClause::AccessPolicyGoverned {
                    bindings: bindings.iter().map(|b| b.into()).collect(),
                }
            }
            ast::OperationClause::Input { reference } => schema::OperationClause::Input {
                reference: reference.clone(),
            },
            ast::OperationClause::Output { reference } => schema::OperationClause::Output {
                reference: reference.clone(),
            },
            ast::OperationClause::State { reference } => schema::OperationClause::State {
                reference: reference.clone(),
            },
            ast::OperationClause::Effect { kind, reference } => schema::OperationClause::Effect {
                kind: kind.clone(),
                reference: reference.clone(),
            },
            ast::OperationClause::Transaction { kind } => {
                schema::OperationClause::Transaction { kind: kind.clone() }
            }
            ast::OperationClause::Failure {
                code,
                kinds,
                message,
            } => schema::OperationClause::Failure {
                code: code.clone(),
                kinds: kinds.clone(),
                message: message.clone(),
            },
            ast::OperationClause::IdempotencyKeyed { field } => {
                schema::OperationClause::IdempotencyKeyed {
                    field: field.clone(),
                }
            }
            ast::OperationClause::IdempotencyInherent => {
                schema::OperationClause::IdempotencyInherent
            }
            ast::OperationClause::IdempotencyNotApplicable {
                reason,
                explanation,
            } => schema::OperationClause::IdempotencyNotApplicable {
                reason: reason.clone(),
                explanation: explanation.clone(),
            },
            ast::OperationClause::ConcurrencyUnique { field } => {
                schema::OperationClause::ConcurrencyUnique {
                    field: field.clone(),
                }
            }
            ast::OperationClause::ConcurrencyOptimistic { field } => {
                schema::OperationClause::ConcurrencyOptimistic {
                    field: field.clone(),
                }
            }
            ast::OperationClause::ConcurrencyReadSnapshot => {
                schema::OperationClause::ConcurrencyReadSnapshot
            }
            ast::OperationClause::EvidenceOperationTrace => {
                schema::OperationClause::EvidenceOperationTrace
            }
            ast::OperationClause::LifecycleSynchronousRequestResponse => {
                schema::OperationClause::LifecycleSynchronousRequestResponse
            }
        }
    }
}

impl From<&ast::PolicyBinding> for schema::PolicyBinding {
    fn from(b: &ast::PolicyBinding) -> Self {
        schema::PolicyBinding {
            policy: b.policy.clone(),
            enforcement_point: b.enforcement_point.clone(),
            failure_code: b.failure_code.clone(),
        }
    }
}
