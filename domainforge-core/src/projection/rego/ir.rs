//! Policy IR for the Rego projection, lowered from the resolved graph's
//! `Policy` values (never from raw `.sea` text).
//!
//! The IR is deliberately narrower than `policy::Expression`: it admits only
//! constructs whose evaluator behaviour is reproduced exactly by `rego/lib.rego`
//! and covered by the differential suite (`tests/rego_projection_tests.rs`).
//! Everything else is a typed [`RegoProjectionError::Unrepresentable`].

use super::RegoProjectionError;
use crate::graph::Graph;
use crate::policy::{
    AggregateFunction, BinaryOp, Expression, Policy, PolicyModality, Quantifier, Severity, UnaryOp,
};
use std::collections::BTreeSet;

/// Collections the evaluator can range over (`Expression::get_collection`).
pub const COLLECTIONS: [&str; 6] = [
    "entities",
    "entity_instances",
    "flows",
    "instances",
    "relations",
    "resources",
];

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionIr {
    /// One of [`COLLECTIONS`].
    pub name: String,
    /// `entity_instances:<Type>` narrows by the item's `entity` field.
    pub entity_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueIr {
    /// A JSON scalar (string, number, bool or null).
    Scalar(serde_json::Value),
    /// `var.field` on a quantifier-bound item; a missing field is NULL.
    Field { var: usize, field: String },
    /// `count(collection)` or `count(v in collection where filter: v)`.
    Count {
        var: usize,
        collection: CollectionIr,
        filter: Option<Box<BoolIr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolIr {
    Const(bool),
    Not(Box<BoolIr>),
    And(Box<BoolIr>, Box<BoolIr>),
    Or(Box<BoolIr>, Box<BoolIr>),
    Cmp {
        op: CmpOp,
        left: ValueIr,
        right: ValueIr,
    },
    Quantified {
        quantifier: Quantifier,
        var: usize,
        collection: CollectionIr,
        condition: Box<BoolIr>,
    },
}

/// One policy, with the identity and metadata the decision contract carries.
#[derive(Debug, Clone)]
pub struct PolicyIr {
    /// `namespace::name` — the same pair the policy's `ConceptId` hashes.
    pub key: String,
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub version: String,
    pub modality: String,
    pub kind: String,
    pub priority: i32,
    /// `error` or `info`, from `PolicyModality::to_severity`.
    pub severity: String,
    pub rationale: Option<String>,
    pub tags: Vec<String>,
    pub condition: BoolIr,
    /// Source constructs used, sorted, for the fidelity report.
    pub constructs: Vec<String>,
    /// The policy's own expression, kept so the reference evaluation runs the
    /// DomainForge evaluator rather than a re-implementation.
    pub(crate) source: Expression,
}

#[derive(Debug, Clone)]
pub struct RegoPolicyIr {
    pub policies: Vec<PolicyIr>,
    /// Collections any policy ranges over, sorted. This is the input contract.
    pub collections: Vec<String>,
    /// `(collection, field)` pairs any policy reads, sorted.
    pub fields: Vec<(String, String)>,
}

impl RegoPolicyIr {
    pub fn build(graph: &Graph) -> Result<Self, RegoProjectionError> {
        let mut policies = Vec::new();
        let mut collections = BTreeSet::new();
        let mut fields = BTreeSet::new();
        let mut keys = BTreeSet::new();

        let mut sources: Vec<&Policy> = graph.all_policies();
        sources.sort_by(|a, b| (&a.namespace, &a.name).cmp(&(&b.namespace, &b.name)));

        for policy in sources {
            let key = format!("{}::{}", policy.namespace, policy.name);
            if graph.is_operation_bound_policy(&policy.id) {
                return Err(unrepresentable(
                    &key,
                    "operation_bound_policy",
                    "the policy is bound to an operation's `access policy_governed by` clause and \
                     is only meaningful in that operation's typed context",
                ));
            }
            if !keys.insert(key.clone()) {
                return Err(RegoProjectionError::Model(format!(
                    "duplicate policy key '{key}'"
                )));
            }

            let mut lowering = Lowering {
                key: &key,
                scope: Vec::new(),
                next_var: 0,
                collections: &mut collections,
                fields: &mut fields,
                constructs: BTreeSet::new(),
            };
            let condition = lowering.boolean(policy.expression())?;
            let constructs = lowering.constructs.into_iter().collect();

            policies.push(PolicyIr {
                key,
                id: policy.id.to_string(),
                name: policy.name.clone(),
                namespace: policy.namespace.clone(),
                version: policy.version.to_string(),
                modality: format!("{:?}", policy.modality),
                kind: format!("{:?}", policy.kind),
                priority: policy.priority,
                severity: severity_label(&policy.modality).to_string(),
                rationale: policy.rationale.clone(),
                tags: policy.tags.clone(),
                condition,
                constructs,
                source: policy.expression().clone(),
            });
        }

        Ok(Self {
            policies,
            collections: collections.into_iter().collect(),
            fields: fields.into_iter().collect(),
        })
    }
}

fn severity_label(modality: &PolicyModality) -> &'static str {
    match modality.to_severity() {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn unrepresentable(policy: &str, construct: &str, reason: &str) -> RegoProjectionError {
    RegoProjectionError::Unrepresentable {
        policy: policy.to_string(),
        construct: construct.to_string(),
        reason: reason.to_string(),
    }
}

struct Lowering<'a> {
    key: &'a str,
    /// Bound variables, innermost last: (source name, IR variable, collection).
    scope: Vec<(String, usize, String)>,
    next_var: usize,
    collections: &'a mut BTreeSet<String>,
    fields: &'a mut BTreeSet<(String, String)>,
    constructs: BTreeSet<String>,
}

impl Lowering<'_> {
    fn reject<T>(&self, construct: &str, reason: &str) -> Result<T, RegoProjectionError> {
        Err(unrepresentable(self.key, construct, reason))
    }

    fn note(&mut self, construct: &str) {
        self.constructs.insert(construct.to_string());
    }

    fn collection(&mut self, expr: &Expression) -> Result<CollectionIr, RegoProjectionError> {
        let Expression::Variable(name) = expr else {
            return self.reject(
                "collection_expression",
                "only the named model collections can be ranged over",
            );
        };
        let (base, entity_type) = match name.strip_prefix("entity_instances:") {
            Some(ty) => ("entity_instances", Some(ty.to_string())),
            None => (name.as_str(), None),
        };
        if !COLLECTIONS.contains(&base) {
            return self.reject(
                "unknown_collection",
                &format!("'{name}' is not a collection"),
            );
        }
        self.collections.insert(base.to_string());
        Ok(CollectionIr {
            name: base.to_string(),
            entity_type,
        })
    }

    fn bind(&mut self, variable: &str, collection: &CollectionIr) -> usize {
        let var = self.next_var;
        self.next_var += 1;
        self.scope
            .push((variable.to_string(), var, collection.name.clone()));
        var
    }

    /// Resolve `var.field` against the innermost binding of `var`, mirroring
    /// `Expression::substitute`.
    fn field(&mut self, object: &str, member: &str) -> Option<ValueIr> {
        let (_, var, collection) = self.scope.iter().rev().find(|(n, _, _)| n == object)?;
        let (var, collection) = (*var, collection.clone());
        self.fields.insert((collection, member.to_string()));
        Some(ValueIr::Field {
            var,
            field: member.to_string(),
        })
    }

    fn boolean(&mut self, expr: &Expression) -> Result<BoolIr, RegoProjectionError> {
        match expr {
            Expression::Literal(serde_json::Value::Bool(b)) => Ok(BoolIr::Const(*b)),
            Expression::Literal(_) => self.reject(
                "non_boolean_literal",
                "a non-boolean literal in a boolean position evaluates inconsistently \
                 (NULL or an error depending on context)",
            ),
            Expression::Unary {
                op: UnaryOp::Not,
                operand,
            } => {
                self.note("not");
                Ok(BoolIr::Not(Box::new(self.boolean(operand)?)))
            }
            Expression::Unary { .. } => self.reject("negate", "arithmetic is not projected"),
            Expression::Binary { op, left, right } => match op {
                BinaryOp::And => {
                    self.note("and");
                    Ok(BoolIr::And(
                        Box::new(self.boolean(left)?),
                        Box::new(self.boolean(right)?),
                    ))
                }
                BinaryOp::Or => {
                    self.note("or");
                    Ok(BoolIr::Or(
                        Box::new(self.boolean(left)?),
                        Box::new(self.boolean(right)?),
                    ))
                }
                BinaryOp::Equal => self.compare(CmpOp::Eq, "equality", left, right),
                BinaryOp::NotEqual => self.compare(CmpOp::Ne, "equality", left, right),
                BinaryOp::LessThan => self.compare(CmpOp::Lt, "ordering", left, right),
                BinaryOp::LessThanOrEqual => self.compare(CmpOp::Lte, "ordering", left, right),
                BinaryOp::GreaterThan => self.compare(CmpOp::Gt, "ordering", left, right),
                BinaryOp::GreaterThanOrEqual => self.compare(CmpOp::Gte, "ordering", left, right),
                BinaryOp::Contains => self.compare(CmpOp::Contains, "string_match", left, right),
                BinaryOp::StartsWith => {
                    self.compare(CmpOp::StartsWith, "string_match", left, right)
                }
                BinaryOp::EndsWith => self.compare(CmpOp::EndsWith, "string_match", left, right),
                BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Multiply | BinaryOp::Divide => self
                    .reject(
                        "arithmetic",
                        "decimal arithmetic and its f64 conversion are not projected",
                    ),
                BinaryOp::Matches => self.reject(
                    "matches",
                    "pattern lookups depend on the model's pattern registry",
                ),
                BinaryOp::HasRole => {
                    self.reject("has_role", "role membership is resolved against the graph")
                }
                BinaryOp::Before | BinaryOp::After | BinaryOp::During => {
                    self.reject("temporal", "temporal operators are not projected")
                }
            },
            Expression::Quantifier {
                quantifier,
                variable,
                collection,
                condition,
            } => {
                self.note(match quantifier {
                    Quantifier::ForAll => "forall",
                    Quantifier::Exists => "exists",
                    Quantifier::ExistsUnique => "exists_unique",
                });
                let collection = self.collection(collection)?;
                let var = self.bind(variable, &collection);
                let condition = self.boolean(condition);
                self.scope.pop();
                Ok(BoolIr::Quantified {
                    quantifier: quantifier.clone(),
                    var,
                    collection,
                    condition: Box::new(condition?),
                })
            }
            Expression::Variable(_) | Expression::MemberAccess { .. } => self.reject(
                "bare_reference_as_condition",
                "a reference in a boolean position evaluates inconsistently for non-boolean \
                 values; compare it explicitly",
            ),
            Expression::Aggregation { .. } | Expression::AggregationComprehension { .. } => self
                .reject(
                    "aggregation_as_condition",
                    "an aggregation needs an explicit comparison",
                ),
            Expression::GroupBy { .. } => self.reject("group_by", "group_by is not projected"),
            Expression::Cast { .. } => self.reject("cast", "unit casts are not projected"),
            Expression::QuantityLiteral { .. } => {
                self.reject("quantity", "unit-bearing quantities are not projected")
            }
            Expression::TimeLiteral(_) | Expression::IntervalLiteral { .. } => {
                self.reject("temporal", "temporal literals are not projected")
            }
            Expression::RoleReference { .. } => self.reject(
                "role_reference",
                "role references are opaque outside an application policy context",
            ),
        }
    }

    fn compare(
        &mut self,
        op: CmpOp,
        construct: &str,
        left: &Expression,
        right: &Expression,
    ) -> Result<BoolIr, RegoProjectionError> {
        self.note(construct);
        Ok(BoolIr::Cmp {
            op,
            left: self.value(left)?,
            right: self.value(right)?,
        })
    }

    fn value(&mut self, expr: &Expression) -> Result<ValueIr, RegoProjectionError> {
        match expr {
            Expression::Literal(v) if v.is_array() || v.is_object() => {
                self.reject("composite_literal", "only scalar literals can be compared")
            }
            Expression::Literal(v) => Ok(ValueIr::Scalar(v.clone())),
            Expression::MemberAccess { object, member } => match self.field(object, member) {
                Some(field) => Ok(field),
                None => self.reject(
                    "free_reference",
                    &format!("'{object}.{member}' is not bound by a quantifier"),
                ),
            },
            Expression::Variable(name) => {
                let bound = name
                    .split_once('.')
                    .and_then(|(object, member)| self.field(object, member));
                match bound {
                    Some(field) => Ok(field),
                    None => self.reject(
                        "free_reference",
                        &format!("'{name}' is not a field of a quantifier-bound item"),
                    ),
                }
            }
            Expression::Aggregation {
                function: AggregateFunction::Count,
                collection,
                filter: None,
                ..
            } => {
                self.note("count");
                let collection = self.collection(collection)?;
                let var = self.next_var;
                self.next_var += 1;
                Ok(ValueIr::Count {
                    var,
                    collection,
                    filter: None,
                })
            }
            Expression::Aggregation {
                filter: Some(_), ..
            } => self.reject(
                "legacy_filtered_aggregation",
                "use the comprehension form `count(v in collection where ...: v)`",
            ),
            Expression::AggregationComprehension {
                function: AggregateFunction::Count,
                variable,
                collection,
                window: None,
                predicate,
                projection,
                target_unit: None,
            } => {
                self.note("count");
                if self.scope.iter().any(|(name, _, _)| name == variable) {
                    return self.reject(
                        "shadowed_comprehension_variable",
                        &format!("'{variable}' is already bound by an enclosing quantifier"),
                    );
                }
                let projects_item = match projection.as_ref() {
                    Expression::Variable(name) => {
                        name == variable || name.starts_with(&format!("{variable}."))
                    }
                    Expression::MemberAccess { object, .. } => object == variable,
                    _ => false,
                };
                if !projects_item {
                    return self.reject(
                        "count_projection",
                        "count must project the bound item or one of its fields",
                    );
                }
                let collection = self.collection(collection)?;
                let var = self.bind(variable, &collection);
                let filter = self.boolean(predicate);
                self.scope.pop();
                let filter = match filter? {
                    BoolIr::Const(true) => None,
                    other => Some(Box::new(other)),
                };
                Ok(ValueIr::Count {
                    var,
                    collection,
                    filter,
                })
            }
            Expression::Aggregation { .. } | Expression::AggregationComprehension { .. } => self
                .reject(
                    "aggregation",
                    "only count without a window or target unit is projected",
                ),
            Expression::QuantityLiteral { .. } | Expression::Cast { .. } => {
                self.reject("quantity", "unit-bearing quantities are not projected")
            }
            Expression::TimeLiteral(_) | Expression::IntervalLiteral { .. } => {
                self.reject("temporal", "temporal literals are not projected")
            }
            Expression::Binary {
                op: BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Multiply | BinaryOp::Divide,
                ..
            }
            | Expression::Unary {
                op: UnaryOp::Negate,
                ..
            } => self.reject(
                "arithmetic",
                "decimal arithmetic and its f64 conversion are not projected",
            ),
            _ => self.reject(
                "non_value_operand",
                "a comparison operand must be a scalar, an item field or a count",
            ),
        }
    }
}
