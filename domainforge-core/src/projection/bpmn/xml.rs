//! BPMN 2.0 XML renderer for [`ProcessIR`].
//!
//! The generic [`Xml`] writer (elements, attributes, escaped text, pretty
//! indentation) is deliberately un-clever and kept separate from BPMN-specific
//! rendering, so Tasks 4/5 (CMMN, ArchiMate) can lift it into a shared module
//! without untangling it from process semantics. All attribute values and text
//! are escaped through [`crate::KnowledgeGraph::escape_xml`] — the one XML
//! escaper in the tree.
//!
//! Element ordering follows the BPMN 2.0 `Semantic.xsd` content models exactly:
//! `laneSet` precedes flow elements inside `tProcess`; `incoming` precedes
//! `outgoing` inside every `tFlowNode`.

use super::ir::{DataObject, Lane, Node, NodeKind, ProcessIR, SequenceFlow};

/// BPMN model namespace (element namespace; `elementFormDefault="qualified"`).
const BPMN_NS: &str = "http://www.omg.org/spec/BPMN/20100524/MODEL";
/// Target namespace stamped on generated definitions.
const TARGET_NS: &str = "http://domainforge.ai/bpmn";

/// Minimal, dependency-free XML writer. Generic across XML-family projections.
pub(crate) struct Xml {
    buf: String,
    depth: usize,
}

impl Xml {
    pub(crate) fn new() -> Self {
        Self {
            buf: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"),
            depth: 0,
        }
    }

    fn pad(&mut self) {
        for _ in 0..self.depth {
            self.buf.push_str("  ");
        }
    }

    fn attrs(&mut self, attrs: &[(&str, &str)]) {
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            self.buf.push_str(&crate::KnowledgeGraph::escape_xml(v));
            self.buf.push('"');
        }
    }

    /// `<name .../>`
    pub(crate) fn empty(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(name);
        self.attrs(attrs);
        self.buf.push_str("/>\n");
    }

    /// `<name ...>` and indent.
    pub(crate) fn open(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(name);
        self.attrs(attrs);
        self.buf.push_str(">\n");
        self.depth += 1;
    }

    /// Dedent and `</name>`.
    pub(crate) fn close(&mut self, name: &str) {
        self.depth -= 1;
        self.pad();
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push_str(">\n");
    }

    /// `<name attrs...>escaped text</name>` on one line (mixed content with
    /// attributes). Used by CMMN for `<condition language="...">expr</condition>`.
    pub(crate) fn leaf_attrs(&mut self, name: &str, attrs: &[(&str, &str)], text: &str) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(name);
        self.attrs(attrs);
        self.buf.push('>');
        self.buf.push_str(&crate::KnowledgeGraph::escape_xml(text));
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push_str(">\n");
    }

    /// `<name>escaped text</name>` on one line.
    pub(crate) fn leaf(&mut self, name: &str, text: &str) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(name);
        self.buf.push('>');
        self.buf.push_str(&crate::KnowledgeGraph::escape_xml(text));
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push_str(">\n");
    }

    pub(crate) fn finish(self) -> String {
        self.buf
    }
}

/// Render `ir` to a complete BPMN 2.0 XML document.
pub fn render(ir: &ProcessIR) -> String {
    let mut x = Xml::new();
    x.open(
        "definitions",
        &[
            ("xmlns", BPMN_NS),
            ("id", &ir.definitions_id),
            ("targetNamespace", TARGET_NS),
            ("exporter", "DomainForge"),
            ("exporterVersion", env!("CARGO_PKG_VERSION")),
        ],
    );

    x.open(
        "process",
        &[
            ("id", &ir.process_id),
            ("name", &ir.process_name),
            ("isExecutable", "false"),
        ],
    );

    // laneSet first (Semantic.xsd tProcess content order).
    if let Some(lane_set_id) = &ir.lane_set_id {
        x.open("laneSet", &[("id", lane_set_id)]);
        for lane in &ir.lanes {
            render_lane(&mut x, lane);
        }
        x.close("laneSet");
    }

    // Flow elements: data objects, then nodes, then sequence flows. All are in
    // the `flowElement` substitution group, so their relative order is free; we
    // fix it for determinism.
    for obj in &ir.data_objects {
        render_data_object(&mut x, obj);
    }
    for node in &ir.nodes {
        render_node(&mut x, node);
    }
    for flow in &ir.sequence_flows {
        render_sequence_flow(&mut x, flow);
    }

    x.close("process");
    x.close("definitions");
    x.finish()
}

fn render_lane(x: &mut Xml, lane: &Lane) {
    if lane.flow_node_refs.is_empty() {
        x.empty("lane", &[("id", &lane.id), ("name", &lane.name)]);
        return;
    }
    x.open("lane", &[("id", &lane.id), ("name", &lane.name)]);
    for r in &lane.flow_node_refs {
        x.leaf("flowNodeRef", r);
    }
    x.close("lane");
}

fn render_data_object(x: &mut Xml, obj: &DataObject) {
    x.empty("dataObject", &[("id", &obj.id), ("name", &obj.name)]);
}

fn render_node(x: &mut Xml, node: &Node) {
    let (tag, direction): (&str, Option<&str>) = match node.kind {
        NodeKind::StartEvent => ("startEvent", None),
        NodeKind::EndEvent => ("endEvent", None),
        NodeKind::Task => ("task", None),
        NodeKind::GatewayDiverging => ("parallelGateway", Some("Diverging")),
        NodeKind::GatewayConverging => ("parallelGateway", Some("Converging")),
    };

    let mut attrs: Vec<(&str, &str)> = vec![("id", &node.id)];
    if let Some(name) = &node.name {
        attrs.push(("name", name));
    }
    if let Some(dir) = direction {
        attrs.push(("gatewayDirection", dir));
    }

    if node.incoming.is_empty() && node.outgoing.is_empty() {
        x.empty(tag, &attrs);
        return;
    }
    x.open(tag, &attrs);
    // incoming before outgoing (tFlowNode content order).
    for r in &node.incoming {
        x.leaf("incoming", r);
    }
    for r in &node.outgoing {
        x.leaf("outgoing", r);
    }
    x.close(tag);
}

fn render_sequence_flow(x: &mut Xml, flow: &SequenceFlow) {
    x.empty(
        "sequenceFlow",
        &[
            ("id", &flow.id),
            ("sourceRef", &flow.source),
            ("targetRef", &flow.target),
        ],
    );
}
