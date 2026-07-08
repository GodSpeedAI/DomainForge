//! CMMN 1.1 XML renderer for [`CaseIR`].
//!
//! Reuses the generic [`Xml`](crate::projection::bpmn::xml::Xml) writer built
//! for the BPMN projection (elements, attributes, escaped text, pretty
//! indentation) rather than re-implementing one — the writer was deliberately
//! kept free of process semantics for exactly this reuse. All attribute values
//! and text are escaped through [`crate::KnowledgeGraph::escape_xml`] via that
//! writer.
//!
//! Element ordering follows the CMMN 1.1 `CMMN11CaseModel.xsd` content models
//! exactly: `caseFileItemDefinition`s precede `case` inside `tDefinitions`;
//! `caseFileModel` → `casePlanModel` → `caseRoles` inside `tCase`; and inside a
//! `tStage`, `planItem`s precede `sentry`s which precede the plan-item
//! definitions.

use super::ir::{CaseIR, HumanTask, Milestone, PlanItem, Sentry};
use crate::projection::bpmn::xml::Xml;

/// CMMN 1.1 model namespace (element namespace; `elementFormDefault="qualified"`).
const CMMN_NS: &str = "http://www.omg.org/spec/CMMN/20151109/MODEL";
/// Target namespace stamped on generated definitions.
const TARGET_NS: &str = "http://domainforge.ai/cmmn";
/// Expression language of the emitted sentry conditions: DomainForge's own
/// policy-expression surface, not XPath. Declared so consumers do not try to
/// evaluate the condition text as XPath.
const EXPR_LANGUAGE: &str = "http://domainforge.ai/expression";

/// Render `ir` to a complete CMMN 1.1 XML document.
pub fn render(ir: &CaseIR) -> String {
    let mut x = Xml::new();
    x.open(
        "definitions",
        &[
            ("xmlns", CMMN_NS),
            ("id", &ir.definitions_id),
            ("targetNamespace", TARGET_NS),
            ("exporter", "DomainForge"),
            ("exporterVersion", env!("CARGO_PKG_VERSION")),
        ],
    );

    // caseFileItemDefinitions first (tDefinitions content order).
    for def in &ir.case_file_item_defs {
        x.empty(
            "caseFileItemDefinition",
            &[("id", &def.id), ("name", &def.name)],
        );
    }

    x.open("case", &[("id", &ir.case_id), ("name", &ir.case_name)]);

    // caseFileModel (from resources).
    if !ir.case_file_items.is_empty() {
        x.open("caseFileModel", &[]);
        for cfi in &ir.case_file_items {
            x.empty(
                "caseFileItem",
                &[
                    ("id", &cfi.id),
                    ("name", &cfi.name),
                    ("definitionRef", &cfi.definition_ref),
                ],
            );
        }
        x.close("caseFileModel");
    }

    // casePlanModel (a tStage): planItems, then sentries, then plan-item
    // definitions (humanTasks + milestones).
    x.open(
        "casePlanModel",
        &[
            ("id", &ir.plan_model_id),
            ("name", &ir.case_name),
            ("autoComplete", "false"),
        ],
    );
    for pi in &ir.plan_items {
        render_plan_item(&mut x, pi);
    }
    for s in &ir.sentries {
        render_sentry(&mut x, s);
    }
    for t in &ir.human_tasks {
        render_human_task(&mut x, t);
    }
    for m in &ir.milestones {
        render_milestone(&mut x, m);
    }
    x.close("casePlanModel");

    // caseRoles (from roles).
    if !ir.roles.is_empty() {
        x.open("caseRoles", &[]);
        for r in &ir.roles {
            x.empty("role", &[("id", &r.id), ("name", &r.name)]);
        }
        x.close("caseRoles");
    }

    x.close("case");
    x.close("definitions");
    x.finish()
}

fn render_plan_item(x: &mut Xml, pi: &PlanItem) {
    let attrs = [
        ("id", pi.id.as_str()),
        ("name", pi.name.as_str()),
        ("definitionRef", pi.definition_ref.as_str()),
    ];
    match &pi.entry_criterion {
        Some((entry_id, sentry_ref)) => {
            x.open("planItem", &attrs);
            x.empty(
                "entryCriterion",
                &[("id", entry_id), ("sentryRef", sentry_ref)],
            );
            x.close("planItem");
        }
        None => x.empty("planItem", &attrs),
    }
}

fn render_sentry(x: &mut Xml, s: &Sentry) {
    x.open("sentry", &[("id", &s.id)]);
    x.open("ifPart", &[]);
    // condition is a mixed-content tExpression: the policy expression is its text.
    render_condition(x, &s.condition);
    x.close("ifPart");
    x.close("sentry");
}

/// `<condition language="...">expr</condition>` on one line (mixed content).
fn render_condition(x: &mut Xml, expr: &str) {
    x.leaf_attrs("condition", &[("language", EXPR_LANGUAGE)], expr);
}

fn render_human_task(x: &mut Xml, t: &HumanTask) {
    let mut attrs: Vec<(&str, &str)> = vec![("id", &t.id), ("name", &t.name)];
    if let Some(pr) = &t.performer_ref {
        attrs.push(("performerRef", pr));
    }
    x.empty("humanTask", &attrs);
}

fn render_milestone(x: &mut Xml, m: &Milestone) {
    x.empty("milestone", &[("id", &m.id), ("name", &m.name)]);
}
