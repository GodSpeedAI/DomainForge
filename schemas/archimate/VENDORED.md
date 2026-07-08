# Vendored ArchiMate 3.x Model Exchange File Schemas

Pinned copies of the normative Open Group ArchiMate Model Exchange File XSDs
consumed by the ArchiMate projection (`domainforge-core/src/projection/archimate/`
and the `verify-archimate` CI job). Validation entry point is
`archimate3_Model.xsd`, which imports the view/diagram schema.

| File | Role | Source |
| --- | --- | --- |
| archimate3_Model.xsd | Root: `model`, `elements`, `relationships`, `organizations`, `propertydefs`, imports the view schema | The Open Group, ArchiMate 3.x Model Exchange File Format |
| archimate3_View.xsd | Views container (`views`, `diagrams`, `view`) | The Open Group, ArchiMate 3.x Model Exchange File Format |
| archimate3_Diagram.xsd | Diagram/visual-notation types (`node`, bounds) referenced from views | The Open Group, ArchiMate 3.x Model Exchange File Format |

**Provenance:** Fetched verbatim from the Open Group's published ArchiMate
Model Exchange File schema set (target namespace
`http://www.opengroup.org/xsd/archimate/3.0/`). These are the unmodified spec
schemas; the Model Exchange File format has been stable since ArchiMate 3.0.

To update: replace these three files with a newer pinned copy of the Open
Group schema set, keep `archimate3_Model.xsd` as the validation entry point,
and re-run the `verify-archimate` CI job (and `tests/test_archimate.py`,
which validates the projected fixture against `archimate3_Model.xsd` with
`lxml`).
