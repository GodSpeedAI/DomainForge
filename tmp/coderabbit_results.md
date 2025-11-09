Starting CodeRabbit review in plain text mode...

Connecting to review service
Setting up
Analyzing
Reviewing

============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 529 to 542
Type: potential_issue

[x] Task:
In docs/plans/tmp/task-06-rdf-sbvr-escaping.md around lines 529–542, the test function test_language_tag_preserved is a scaffold with no verification; either mark it explicitly as TODO or implement the assertion. Fix by directing to implement the test: parse input_xml with the project’s RDF/XML parser, locate the rdfs:label for the TestEntity, assert its language tag equals "en", optionally re-serialize and assert the serialized output still contains rdfs:label xml:lang="en". Ensure the test includes an assert so it fails/alerts until implemented.



============================================================================
File: docs/plans/tmp/task-06-rdf-sbvr-escaping.md
Line: 727 to 851
Type: potential_issue

[x] Task:
docs/plans/tmp/task-06-rdf-sbvr-escaping.md lines 727–851: The SBVR struct change is backward-incompatible because the object field semantic changed from to_entity to resource_id and no versioning/migration is specified; fix by adding an explicit schema_version field (with serde default), make destination optional for v1 compatibility and implement deserialization logic that accepts v1 facts (no destination) by mapping legacy object->destination when needed or populating resource_id appropriately, ensure all SBVR serializers emit schema_version (e.g., "2.0") for new facts, add unit tests covering v1->v2 upgrade and round-trip, and update Cycle C/GREEN and Risk & Rollback notes to state the migration window and fallback policy (how long v1 is supported and whether auto-upgrade or rejection is used).



Review completed ✔
