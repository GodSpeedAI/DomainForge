//! Language-neutral Domain IR encoding the SEA → DDD/CQRS mapping.
//!
//! Every renderer ([`crate::projection::domain`]) consumes exactly this IR;
//! no renderer re-derives a name or reads the [`crate::graph::Graph`]. The
//! mapping is normative (see the plan + `docs/how-tos/project-domain-code.md`):
//!
//! | SEA element        | Domain construct                                                |
//! | ------------------ | --------------------------------------------------------------- |
//! | `@namespace`       | Package name (`slug` / `pascal`)                                |
//! | `Entity`           | DDD Entity (id + name; fields from `instance of`)               |
//! | `Resource`         | Aggregate root + Repository port + Quantity unit                |
//! | `Flow` (default)   | Command + Event + aggregate method                              |
//! | `Flow @cqrs cmd`   | Command + aggregate method (no event)                           |
//! | `Flow @cqrs event` | Event only (no command, no method)                              |
//! | `Policy`           | Domain error `<Pascal>Violation` + `check_<slug>()` guard       |
//! | `Metric`           | Query `get_<slug>()` on `<Ns>ReadModel`                         |
//! | `Pattern`          | Value object validating against the regex                       |
//! | `Role`             | Enum `Role { <Pascal>, ... }`; commands carry `issued_by: Role` |
//! | `Relation`         | Doc-comment line on the Role enum                               |

use crate::graph::Graph;
use crate::projection::flows::ResolvedFlow;
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{pascal, slug, NameRegistrar};
use std::collections::BTreeMap;

/// One aggregate method minted from a flow (the CQRS write path:
/// command → aggregate → event).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuardCheck {
    /// `check_<slug(policy)>` — the no-op guard hook the method calls.
    pub check_method: String,
    /// `<Pascal>Violation` — the error the guard may raise.
    pub error: String,
}

/// A method on an aggregate root, minted from one flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodIr {
    /// `transfer_to_<slug(to)>`.
    pub name: String,
    /// Command this method dispatches (`None` for event-only flows).
    pub command: Option<String>,
    /// Event this method returns (`None` for command-only flows).
    pub event: Option<String>,
    pub from_entity: String,
    pub to_entity: String,
    pub quantity: String,
    /// Every policy guards every method in v1 (no-op bodies).
    pub guard_checks: Vec<GuardCheck>,
}

/// An aggregate root minted from a resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AggregateIr {
    /// `<Pascal(resource)>` (the aggregate root type name).
    pub name: String,
    pub resource_name: String,
    /// Unit symbol carried by the shared Quantity value object.
    pub unit: String,
    pub methods: Vec<MethodIr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityIr {
    /// `<Pascal(entity)>`.
    pub name: String,
    /// `(field, "str")` — v1 is string-typed; fields gathered from `instance of`.
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandIr {
    /// `Transfer<Resource>From<From>To<To>`.
    pub name: String,
    pub aggregate: String,
    pub from_entity: String,
    pub to_entity: String,
    pub quantity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventIr {
    /// `<Resource>TransferredFrom<From>To<To>`.
    pub name: String,
    pub aggregate: String,
    pub from_entity: String,
    pub to_entity: String,
    pub quantity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorIr {
    /// `<Pascal(policy)>Violation`.
    pub name: String,
    pub policy_name: String,
    /// `check_<slug(policy)>` — the no-op guard hook.
    pub check_method: String,
    pub rationale: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryIr {
    /// `get_<slug(metric)>`.
    pub name: String,
    pub metric_name: String,
    pub unit: Option<String>,
    pub threshold: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueObjectIr {
    /// `<Pascal(pattern)>`.
    pub name: String,
    pub regex: String,
    /// `Invalid<Pascal(pattern)>`.
    pub error_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceIr {
    pub name: String,
    /// `<Pascal(entity_type)>`.
    pub entity: String,
    /// `(field, literal-value-as-string)`.
    pub fields: Vec<(String, String)>,
}

/// The complete language-neutral domain model. All names are pre-sanitized
/// through [`NameRegistrar`]; all collections are sorted for determinism.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainIr {
    pub namespace: String,
    /// `slug(namespace)` — package/module directory fragment.
    pub package_slug: String,
    /// `pascal(namespace)` — `<Ns>ReadModel`, Python class prefixes.
    pub package_pascal: String,
    pub entities: Vec<EntityIr>,
    pub aggregates: Vec<AggregateIr>,
    pub commands: Vec<CommandIr>,
    pub events: Vec<EventIr>,
    pub errors: Vec<ErrorIr>,
    pub queries: Vec<QueryIr>,
    pub value_objects: Vec<ValueObjectIr>,
    /// `<Pascal(role)>` variants.
    pub roles: Vec<String>,
    /// `subject —predicate→ object` doc-comment lines.
    pub relations_doc: Vec<String>,
    pub instances: Vec<InstanceIr>,
}

/// Read the fixture file relative to the crate root (CARGO_MANIFEST_DIR).
#[cfg(test)]
fn fixture_source() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/projection_cell/basic/model.sea");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

impl DomainIr {
    /// Build the IR from a graph. The single place DDD semantics are decided.
    pub fn from_graph(graph: &Graph) -> Result<DomainIr, String> {
        let namespace = model_namespace(graph)?;
        let package_slug = slug(&namespace);
        let package_pascal = pascal(&namespace);
        let flows = collect_flows(graph)?;
        let mut reg = NameRegistrar::new();

        // Roles (enum variants).
        let mut roles: Vec<String> = graph
            .all_roles()
            .iter()
            .map(|r| reg.register("ident", &pascal(r.name())))
            .collect();
        roles.sort();

        // Role ConceptId -> display name, for relation doc-comments.
        let role_name: BTreeMap<String, String> = graph
            .all_roles()
            .iter()
            .map(|r| (r.id().to_string(), r.name().to_string()))
            .collect();
        let mut relations_doc: Vec<String> = graph
            .all_relations()
            .iter()
            .map(|rel| {
                let subj = role_name
                    .get(&rel.subject_role().to_string())
                    .cloned()
                    .unwrap_or_else(|| rel.subject_role().to_string());
                let obj = role_name
                    .get(&rel.object_role().to_string())
                    .cloned()
                    .unwrap_or_else(|| rel.object_role().to_string());
                let pred = rel.predicate();
                format!("{subj} \u{2014}{pred}\u{2192} {obj}")
            })
            .collect();
        relations_doc.sort();

        // Errors from policies (each policy -> one domain error + one guard hook).
        let mut errors: Vec<ErrorIr> = graph
            .all_policies()
            .iter()
            .map(|p| {
                let name = reg.register("ident", &format!("{}Violation", pascal(&p.name)));
                ErrorIr {
                    name,
                    policy_name: p.name.clone(),
                    check_method: format!("check_{}", slug(&p.name)),
                    rationale: p.rationale.clone(),
                }
            })
            .collect();
        errors.sort_by(|a, b| a.name.cmp(&b.name));
        let guard_checks: Vec<GuardCheck> = errors
            .iter()
            .map(|e| GuardCheck {
                check_method: e.check_method.clone(),
                error: e.name.clone(),
            })
            .collect();

        // Value objects from patterns.
        let mut value_objects: Vec<ValueObjectIr> = graph
            .all_patterns()
            .iter()
            .map(|p| {
                let pname = pascal(p.name());
                ValueObjectIr {
                    name: reg.register("ident", &pname),
                    regex: p.regex().to_string(),
                    error_name: format!("Invalid{pname}"),
                }
            })
            .collect();
        value_objects.sort_by(|a, b| a.name.cmp(&b.name));

        // Queries from metrics.
        let mut queries: Vec<QueryIr> = graph
            .all_metrics()
            .iter()
            .map(|m| QueryIr {
                name: format!("get_{}", slug(&m.name)),
                metric_name: m.name.clone(),
                unit: m.unit.clone(),
                threshold: m.threshold.map(|d| d.to_string()),
            })
            .collect();
        queries.sort_by(|a, b| a.name.cmp(&b.name));

        // Entities (fields gathered from `instance of <entity>`).
        let mut entity_fields: BTreeMap<String, BTreeMap<String, ()>> = BTreeMap::new();
        for inst in graph.all_entity_instances() {
            let entry = entity_fields
                .entry(inst.entity_type().to_string())
                .or_default();
            for k in inst.fields().keys() {
                // `id` and `name` are the entity's built-in identity fields.
                if k != "id" && k != "name" {
                    entry.insert(k.clone(), ());
                }
            }
        }
        let mut entities: Vec<EntityIr> = graph
            .all_entities()
            .iter()
            .map(|e| {
                let fields: Vec<(String, String)> = entity_fields
                    .get(e.name())
                    .map(|m| {
                        let mut v: Vec<(String, String)> =
                            m.keys().map(|k| (k.clone(), "str".to_string())).collect();
                        v.sort();
                        v
                    })
                    .unwrap_or_default();
                EntityIr {
                    name: reg.register("ident", &pascal(e.name())),
                    fields,
                }
            })
            .collect();
        entities.sort_by(|a, b| a.name.cmp(&b.name));

        // Aggregates from resources; methods from the resource's flows.
        let mut aggregates: Vec<AggregateIr> = Vec::new();
        for resource in graph.all_resources() {
            let agg_name = reg.register("ident", &pascal(resource.name()));
            let mut methods: Vec<MethodIr> = flows
                .iter()
                .filter(|f| f.resource == resource.name() && cqrs_kind(f) != CqrsKind::EventOnly)
                .map(|f| method_from_flow(f, &guard_checks, &mut reg))
                .collect();
            methods.sort_by(|a, b| a.name.cmp(&b.name));
            aggregates.push(AggregateIr {
                name: agg_name,
                resource_name: resource.name().to_string(),
                unit: resource.unit_symbol().to_string(),
                methods,
            });
        }
        aggregates.sort_by(|a, b| a.name.cmp(&b.name));

        // Commands + events from flows (driven by the @cqrs kind switch).
        let mut commands: Vec<CommandIr> = Vec::new();
        let mut events: Vec<EventIr> = Vec::new();
        for f in &flows {
            let kind = cqrs_kind(f);
            let aggregate = pascal(&f.resource);
            let from_entity = pascal(&f.from);
            let to_entity = pascal(&f.to);
            if kind != CqrsKind::EventOnly {
                commands.push(CommandIr {
                    name: reg.register(
                        "ident",
                        &format!("Transfer{aggregate}From{from_entity}To{to_entity}"),
                    ),
                    aggregate: aggregate.clone(),
                    from_entity: from_entity.clone(),
                    to_entity: to_entity.clone(),
                    quantity: f.quantity.clone(),
                });
            }
            if kind != CqrsKind::CommandOnly {
                events.push(EventIr {
                    name: reg.register(
                        "ident",
                        &format!("{aggregate}TransferredFrom{from_entity}To{to_entity}"),
                    ),
                    aggregate,
                    from_entity,
                    to_entity,
                    quantity: f.quantity.clone(),
                });
            }
        }
        commands.sort_by(|a, b| a.name.cmp(&b.name));
        events.sort_by(|a, b| a.name.cmp(&b.name));

        // Instances (example constructions for smoke tests).
        let mut instances: Vec<InstanceIr> = graph
            .all_entity_instances()
            .iter()
            .map(|inst| {
                let mut fields: Vec<(String, String)> = inst
                    .fields()
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_string(v)))
                    .collect();
                fields.sort();
                InstanceIr {
                    name: inst.name().to_string(),
                    entity: pascal(inst.entity_type()),
                    fields,
                }
            })
            .collect();
        instances.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(DomainIr {
            namespace,
            package_slug,
            package_pascal,
            entities,
            aggregates,
            commands,
            events,
            errors,
            queries,
            value_objects,
            roles,
            relations_doc,
            instances,
        })
    }
}

#[derive(PartialEq, Eq)]
enum CqrsKind {
    /// Default (no annotation, or unknown kind): command + event + method.
    CommandAndEvent,
    /// `@cqrs { "kind": "command" }`: command + method, no event.
    CommandOnly,
    /// `@cqrs { "kind": "event" }`: event only, no command, no method.
    EventOnly,
}

fn cqrs_kind(flow: &ResolvedFlow) -> CqrsKind {
    match flow
        .annotations
        .get("cqrs")
        .and_then(|v| v.get("kind"))
        .and_then(|v| v.as_str())
    {
        Some("command") => CqrsKind::CommandOnly,
        Some("event") => CqrsKind::EventOnly,
        _ => CqrsKind::CommandAndEvent,
    }
}

fn method_from_flow(
    flow: &ResolvedFlow,
    guard_checks: &[GuardCheck],
    reg: &mut NameRegistrar,
) -> MethodIr {
    let kind = cqrs_kind(flow);
    let from_entity = pascal(&flow.from);
    let to_entity = pascal(&flow.to);
    let resource = pascal(&flow.resource);
    let command = if kind != CqrsKind::EventOnly {
        Some(reg.register(
            "ident",
            &format!("Transfer{resource}From{from_entity}To{to_entity}"),
        ))
    } else {
        None
    };
    let event = if kind != CqrsKind::CommandOnly {
        Some(reg.register(
            "ident",
            &format!("{resource}TransferredFrom{from_entity}To{to_entity}"),
        ))
    } else {
        None
    };
    MethodIr {
        name: reg.register("ident", &format!("transfer_to_{}", slug(&flow.to))),
        command,
        event,
        from_entity,
        to_entity,
        quantity: flow.quantity.clone(),
        guard_checks: guard_checks.to_vec(),
    }
}

/// Render a JSON instance value as a plain string literal (v1 is string-typed).
fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_to_graph;

    fn ir_from_fixture() -> DomainIr {
        let g = parse_to_graph(&fixture_source()).expect("fixture parses");
        DomainIr::from_graph(&g).expect("IR builds")
    }

    #[test]
    fn fixture_namespace_and_package() {
        let ir = ir_from_fixture();
        assert_eq!(ir.namespace, "procurement");
        assert_eq!(ir.package_slug, "procurement");
        assert_eq!(ir.package_pascal, "Procurement");
    }

    #[test]
    fn fixture_entities() {
        let ir = ir_from_fixture();
        let names: Vec<&str> = ir.entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Approver", "Buyer", "Supplier"]);
        // Fields come from the `instance acme_supplier of "Supplier"`; `name` is the
        // entity's built-in identity field, so only `region` is added here.
        let supplier = ir.entities.iter().find(|e| e.name == "Supplier").unwrap();
        assert_eq!(supplier.fields, [("region".to_string(), "str".to_string())]);
    }

    #[test]
    fn fixture_aggregates() {
        let ir = ir_from_fixture();
        let names: Vec<&str> = ir.aggregates.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Payment", "PurchaseOrder"]);
        let po = ir
            .aggregates
            .iter()
            .find(|a| a.name == "PurchaseOrder")
            .unwrap();
        assert_eq!(po.unit, "units");
        assert_eq!(po.methods.len(), 1);
        assert_eq!(po.methods[0].name, "transfer_to_supplier");
    }

    #[test]
    fn fixture_commands_and_events_exact_names() {
        let ir = ir_from_fixture();
        let cmds: Vec<&str> = ir.commands.iter().map(|c| c.name.as_str()).collect();
        let evs: Vec<&str> = ir.events.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            cmds,
            [
                "TransferPaymentFromBuyerToSupplier",
                "TransferPurchaseOrderFromBuyerToSupplier",
            ]
        );
        assert_eq!(
            evs,
            [
                "PaymentTransferredFromBuyerToSupplier",
                "PurchaseOrderTransferredFromBuyerToSupplier",
            ]
        );
    }

    #[test]
    fn fixture_error_with_rationale() {
        let ir = ir_from_fixture();
        assert_eq!(ir.errors.len(), 1);
        let e = &ir.errors[0];
        assert_eq!(e.name, "RequireApprovalViolation");
        assert_eq!(e.policy_name, "require_approval");
        assert_eq!(e.check_method, "check_require_approval");
        assert_eq!(
            e.rationale.as_deref(),
            Some("every order must be approved before payment")
        );
        // Every method guards against this error.
        let po = ir
            .aggregates
            .iter()
            .find(|a| a.name == "PurchaseOrder")
            .unwrap();
        assert!(po.methods.iter().all(|m| m
            .guard_checks
            .iter()
            .any(|g| g.error == "RequireApprovalViolation"
                && g.check_method == "check_require_approval")));
    }

    #[test]
    fn fixture_query() {
        let ir = ir_from_fixture();
        assert_eq!(ir.queries.len(), 1);
        let q = &ir.queries[0];
        assert_eq!(q.name, "get_order_count");
        assert_eq!(q.metric_name, "order_count");
        assert_eq!(q.unit.as_deref(), Some("orders"));
        assert_eq!(q.threshold.as_deref(), Some("100"));
    }

    #[test]
    fn fixture_value_object() {
        let ir = ir_from_fixture();
        assert_eq!(ir.value_objects.len(), 1);
        let vo = &ir.value_objects[0];
        assert_eq!(vo.name, "OrderNumber");
        assert_eq!(vo.regex, "^[A-Z]{3}-[0-9]+$");
        assert_eq!(vo.error_name, "InvalidOrderNumber");
    }

    #[test]
    fn fixture_roles_and_relation() {
        let ir = ir_from_fixture();
        assert_eq!(ir.roles, ["Authorizer", "Requester"]);
        assert_eq!(
            ir.relations_doc,
            ["Requester \u{2014}orders\u{2192} Authorizer"]
        );
    }

    #[test]
    fn fixture_instance() {
        let ir = ir_from_fixture();
        assert_eq!(ir.instances.len(), 1);
        let inst = &ir.instances[0];
        assert_eq!(inst.name, "acme_supplier");
        assert_eq!(inst.entity, "Supplier");
        assert_eq!(
            inst.fields,
            [
                ("name".to_string(), "Acme Corp".to_string()),
                ("region".to_string(), "NA".to_string())
            ]
        );
    }

    #[test]
    fn fixture_deterministic() {
        assert_eq!(ir_from_fixture(), ir_from_fixture());
    }

    // --- @cqrs kind switch (inline sources; the fixture has no annotations) ---

    #[test]
    fn cqrs_command_kind_emits_command_only() {
        let src = r#"
@namespace "p"
Entity "A" in p
Entity "B" in p
Resource "R" units in p
Flow "R" @cqrs { "kind": "command" } from "A" to "B" quantity 1
"#;
        let ir = DomainIr::from_graph(&parse_to_graph(src).unwrap()).unwrap();
        assert_eq!(ir.commands.len(), 1);
        assert!(ir.events.is_empty(), "command-kind must emit no event");
    }

    #[test]
    fn cqrs_event_kind_emits_event_only() {
        let src = r#"
@namespace "p"
Entity "A" in p
Entity "B" in p
Resource "R" units in p
Flow "R" @cqrs { "kind": "event" } from "A" to "B" quantity 1
"#;
        let ir = DomainIr::from_graph(&parse_to_graph(src).unwrap()).unwrap();
        assert!(ir.commands.is_empty(), "event-kind must emit no command");
        assert_eq!(ir.events.len(), 1);
        // Event-only flow mints no aggregate method.
        let agg = &ir.aggregates[0];
        assert!(agg.methods.is_empty(), "event-kind must mint no method");
    }
}
