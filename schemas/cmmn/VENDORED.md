# Vendored CMMN 1.1 Schemas

Pinned copies of the normative OMG CMMN 1.1 XSDs consumed by the CMMN
projection (`domainforge-core/src/projection/cmmn/` and the `verify-cmmn` CI
job). Validation entry point is `CMMN11.xsd`, which `include`s
`CMMN11CaseModel.xsd` and `import`s the diagram-interchange schema.

| File | Role | Source |
| --- | --- | --- |
| CMMN11.xsd | Root: `definitions`, includes the case model and imports the DI | OMG CMMN 1.1 (`20151109`) |
| CMMN11CaseModel.xsd | Case model (tCase, tStage, tHumanTask, tMilestone, tSentry, caseFileItem, caseRoles, entry/exit criteria) | OMG CMMN 1.1 (`20151109`) |
| CMMNDI11.xsd | Diagram interchange (imported; unused by the v1 subset — CMMN DI is a declared Non-goal) | OMG CMMN 1.1 (`20151109`) |
| DI.xsd | Diagram-interchange base types | OMG CMMN 1.1 (`20151109`) |
| DC.xsd | Diagram-common types (Bounds, Point) | OMG CMMN 1.1 (`20151109`) |

**Provenance:** Fetched verbatim from the OMG CMMN 1.1 specification schema set
(`https://www.omg.org/spec/CMMN/20151109/`, target namespace
`http://www.omg.org/spec/CMMN/20151109/MODEL`). These are the unmodified spec
schemas; the CMMN 1.1 schema set has been stable since 2015.

To update: replace these five files with a newer pinned copy of the OMG schema
set, keep `CMMN11.xsd` as the validation entry point, and re-run the
`verify-cmmn` CI job (and `tests/test_cmmn.py`, which validates the projected
fixture against `CMMN11.xsd` with `lxml`).
