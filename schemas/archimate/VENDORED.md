# Vendored ArchiMate 3.x Model Exchange File Schemas

Pinned copies of the normative Open Group ArchiMate Model Exchange File XSDs
consumed by the ArchiMate projection (`domainforge-core/src/projection/archimate/`
and the `verify-archimate` CI job). Validation entry point is
`archimate3_Diagram.xsd`.

The three schemas form an `xs:redefine` chain: `archimate3_View.xsd` includes
and redefines `archimate3_Model.xsd` (adding `views` to `ModelType`), and
`archimate3_Diagram.xsd` includes and redefines `archimate3_View.xsd` (adding
`diagrams`/`view` to `ViewsType`). The base `archimate3_Model.xsd` alone does
**not** allow `views`, so it cannot be the validation entry point for any model
that emits views; `archimate3_Diagram.xsd` (the top of the chain) is.

| File | Role | Source |
| --- | --- | --- |
| archimate3_Model.xsd | Base: `model`, `elements`, `relationships`, `organizations`, `propertyDefinitions` (no `views`) | The Open Group, ArchiMate 3.x Model Exchange File Format |
| archimate3_View.xsd | Redefines `ModelType` to add `views` (`ViewsType` with `viewpoints`) | The Open Group, ArchiMate 3.x Model Exchange File Format |
| archimate3_Diagram.xsd | Redefines `ViewsType` to add `diagrams` → `view` (`node`, bounds). **Validation entry point.** | The Open Group, ArchiMate 3.x Model Exchange File Format |

**Provenance:** Fetched verbatim from the Open Group's published ArchiMate
Model Exchange File schema set (target namespace
`http://www.opengroup.org/xsd/archimate/3.0/`). These are the unmodified spec
schemas; the Model Exchange File format has been stable since ArchiMate 3.0.

To update: replace these three files with a newer pinned copy of the Open
Group schema set, keep `archimate3_Diagram.xsd` as the validation entry point,
and re-run the `verify-archimate` CI job (and `tests/test_archimate.py`,
which validates the projected fixture against `archimate3_Diagram.xsd` with
`lxml`).
