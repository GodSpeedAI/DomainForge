//! ArchiMate 3.0 Model Exchange File renderer for [`ArchitectureIR`].
//!
//! Reuses the generic [`Xml`](crate::projection::bpmn::xml::Xml) writer built for
//! the BPMN projection (elements, attributes, escaped text, pretty indentation)
//! rather than re-implementing one — the writer was deliberately kept free of
//! process semantics for exactly this reuse, as CMMN already does. All attribute
//! values and text are escaped through [`crate::KnowledgeGraph::escape_xml`] via
//! that writer.
//!
//! Element ordering follows the ArchiMate 3.0 `archimate3_Model.xsd`
//! `ModelType` content model exactly: `name` → `elements` → `relationships` →
//! `views`. Each concrete element/relationship/view carries its ArchiMate type as
//! an `xsi:type` attribute (the exchange format models the type hierarchy through
//! xsi:type on abstract base elements).

use super::ir::{ArchitectureIR, Element, Relation, View, ViewNode};

/// ArchiMate 3.0 Model Exchange File model namespace.
const ARCHIMATE_NS: &str = "http://www.opengroup.org/xsd/archimate/3.0/";
/// XML Schema instance namespace (for `xsi:type` / `xsi:schemaLocation`).
const XSI_NS: &str = "http://www.w3.org/2001/XMLSchema-instance";
/// Schema location hint pointing consumers at the vendored root schema.
const SCHEMA_LOCATION: &str = "http://www.opengroup.org/xsd/archimate/3.0/ \
http://www.opengroup.org/xsd/archimate/3.0/archimate3_Model.xsd";

/// Render `ir` to a complete ArchiMate 3.0 Model Exchange File document.
pub fn render(ir: &ArchitectureIR) -> String {
    let mut x = crate::projection::bpmn::xml::Xml::new();
    x.open(
        "model",
        &[
            ("xmlns", ARCHIMATE_NS),
            ("xmlns:xsi", XSI_NS),
            ("xsi:schemaLocation", SCHEMA_LOCATION),
            ("identifier", &ir.model_id),
        ],
    );

    // ModelType content order: name first.
    x.leaf_attrs("name", &[("xml:lang", "en")], &ir.model_name);

    // elements.
    if !ir.elements.is_empty() {
        x.open("elements", &[]);
        for e in &ir.elements {
            render_element(&mut x, e);
        }
        x.close("elements");
    }

    // relationships.
    if !ir.relations.is_empty() {
        x.open("relationships", &[]);
        for r in &ir.relations {
            render_relation(&mut x, r);
        }
        x.close("relationships");
    }

    // views → diagrams → view.
    if !ir.views.is_empty() {
        x.open("views", &[]);
        x.open("diagrams", &[]);
        for v in &ir.views {
            render_view(&mut x, v);
        }
        x.close("diagrams");
        x.close("views");
    }

    x.close("model");
    x.finish()
}

fn render_element(x: &mut crate::projection::bpmn::xml::Xml, e: &Element) {
    x.open(
        "element",
        &[("identifier", &e.id), ("xsi:type", e.kind.archimate_type())],
    );
    x.leaf_attrs("name", &[("xml:lang", "en")], &e.name);
    x.close("element");
}

fn render_relation(x: &mut crate::projection::bpmn::xml::Xml, r: &Relation) {
    x.empty(
        "relationship",
        &[
            ("identifier", &r.id),
            ("source", &r.source),
            ("target", &r.target),
            ("xsi:type", r.kind.archimate_type()),
        ],
    );
}

fn render_view(x: &mut crate::projection::bpmn::xml::Xml, v: &View) {
    x.open("view", &[("identifier", &v.id), ("xsi:type", "Diagram")]);
    x.leaf_attrs("name", &[("xml:lang", "en")], &v.name);
    for n in &v.nodes {
        render_node(x, n);
    }
    x.close("view");
}

fn render_node(x: &mut crate::projection::bpmn::xml::Xml, n: &ViewNode) {
    let (xs, ys, ws, hs) = (
        n.x.to_string(),
        n.y.to_string(),
        n.w.to_string(),
        n.h.to_string(),
    );
    x.empty(
        "node",
        &[
            ("identifier", &n.id),
            ("elementRef", &n.element_ref),
            ("xsi:type", "Element"),
            ("x", &xs),
            ("y", &ys),
            ("w", &ws),
            ("h", &hs),
        ],
    );
}
