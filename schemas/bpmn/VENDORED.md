# Vendored BPMN 2.0 Schemas

Pinned copies of the normative OMG BPMN 2.0 XSDs consumed by the BPMN
Projection (`domainforge-core/src/projection/bpmn/` and the `verify-bpmn` CI
job). Validation entry point is `BPMN20.xsd`, which `include`s `Semantic.xsd`
and `import`s the diagram-interchange schemas.

| File | Role | Source |
| --- | --- | --- |
| BPMN20.xsd | Root: `definitions`, imports/includes the rest | OMG BPMN 2.0 (`20100524`) |
| Semantic.xsd | Process/collaboration semantic model (tProcess, tTask, gateways, sequenceFlow, laneSet, dataObject) | OMG BPMN 2.0 (`20100524`) |
| BPMNDI.xsd | Diagram interchange (imported; unused by the v1 subset — BPMNDI is a declared Non-goal) | OMG BPMN 2.0 (`20100524`) |
| DI.xsd | Diagram-interchange base types | OMG BPMN 2.0 (`20100524`) |
| DC.xsd | Diagram-common types (Bounds, Point) | OMG BPMN 2.0 (`20100524`) |

**Provenance:** Extracted verbatim from the `SpiffWorkflow` distribution
(`spiffworkflow==3.1.2`, `SpiffWorkflow/bpmn/parser/schema/`), which vendors the
unmodified OMG BPMN 2.0 spec schemas (target namespace
`http://www.omg.org/spec/BPMN/20100524/MODEL`). These are the same XSDs
published with the BPMN 2.0 specification; the spec schema set has been stable
since 2011.

To update: replace these five files with a newer pinned copy of the OMG schema
set, keep `BPMN20.xsd` as the validation entry point, and re-run the
`verify-bpmn` CI job (and `tests/test_bpmn.py`, which validates the projected
fixture against `BPMN20.xsd` with `lxml`).
