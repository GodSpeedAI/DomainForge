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
| xml.xsd | The W3C "XML namespace" schema (declares `xml:lang`, `xml:space`, `xml:base`, `xml:id`) imported by `archimate3_Model.xsd` | W3C, http://www.w3.org/2001/xml.xsd (vendored verbatim) |

**Provenance:** Fetched verbatim from the Open Group's published ArchiMate
Model Exchange File schema set (target namespace
`http://www.opengroup.org/xsd/archimate/3.0/`). These are the unmodified spec
schemas; the Model Exchange File format has been stable since ArchiMate 3.0.

**One deviation from the upstream spec set:** `archimate3_Model.xsd` originally
imported the XML namespace with a remote `schemaLocation="http://www.w3.org/2001/xml.xsd"`.
That remote reference is unresolvable by offline validators (lxml/libxml2), so
`xml:lang` (used by `LangStringType`) failed to resolve and schema loading
aborted. The vendored copy points the import at the local `xml.xsd`
(`schemaLocation="xml.xsd"`), and `xml.xsd` itself is vendored verbatim from
W3C. This is the only modification to the upstream files; the schema content is
otherwise identical. Updating to a newer Open Group schema set must preserve
this local import (or re-vendor `xml.xsd`).

To update: replace these three files with a newer pinned copy of the Open
Group schema set, keep `archimate3_Diagram.xsd` as the validation entry point,
keep the local `xml.xsd` import, and re-run the `verify-archimate` CI job (and
`tests/test_archimate.py`, which validates the projected fixture against
`archimate3_Diagram.xsd` with `lxml`).
