# graph — Canonical In-Memory Semantic Model

## Overview
The `Graph` aggregate: insertion-ordered maps of every declared primitive keyed
by `ConceptId`, plus graph-level config, declared dimensions/units, and
entity-role assignments. It owns policy validation (`Graph::validate`), atomic
graph merging, and AST round-tripping.

## Interface
- `[mod.rs:Graph]` — the aggregate. Storage is `IndexMap<ConceptId, _>` per collection so source order is preserved.
- `[mod.rs:Graph::new]`, `[mod.rs:Graph::is_empty]` — construction and emptiness.
- `[mod.rs:Graph::add_entity]`, `[mod.rs:Graph::has_entity]`, `[mod.rs:Graph::get_entity]`, `[mod.rs:Graph::get_entity_mut]`, `[mod.rs:Graph::remove_entity]`, `[mod.rs:Graph::entity_count]`, `[mod.rs:Graph::all_entities]`, `[mod.rs:Graph::find_entity_by_name]`, `[mod.rs:Graph::find_entity_by_name_and_namespace]` — entity access.
- `[mod.rs:Graph::add_role]`, `[mod.rs:Graph::get_role]`, `[mod.rs:Graph::has_role]`, `[mod.rs:Graph::role_count]`, `[mod.rs:Graph::all_roles]`, `[mod.rs:Graph::find_role_by_name]` — role access.
- `[mod.rs:Graph::assign_role_to_entity]`, `[mod.rs:Graph::roles_for_entity]`, `[mod.rs:Graph::role_names_for_entity]` — entity-role assignment.
- `[mod.rs:Graph::add_resource]`, `[mod.rs:Graph::has_resource]`, `[mod.rs:Graph::get_resource]`, `[mod.rs:Graph::remove_resource]`, `[mod.rs:Graph::resource_count]`, `[mod.rs:Graph::all_resources]`, `[mod.rs:Graph::find_resource_by_name]` — resource access.
- `[mod.rs:Graph::add_flow]`, `[mod.rs:Graph::has_flow]`, `[mod.rs:Graph::get_flow]`, `[mod.rs:Graph::remove_flow]`, `[mod.rs:Graph::flow_count]`, `[mod.rs:Graph::all_flows]`, `[mod.rs:Graph::flows_from]`, `[mod.rs:Graph::flows_to]`, `[mod.rs:Graph::upstream_entities]`, `[mod.rs:Graph::downstream_entities]` — flow access and traversal.
- `[mod.rs:Graph::add_relation_type]`, `[mod.rs:Graph::all_relations]`, `[mod.rs:Graph::relation_count]` — relation types.
- `[mod.rs:Graph::add_pattern]`, `[mod.rs:Graph::find_pattern]`, `[mod.rs:Graph::all_patterns]`, `[mod.rs:Graph::pattern_count]` — patterns.
- `[mod.rs:Graph::add_concept_change]`, `[mod.rs:Graph::get_concept_change]`, `[mod.rs:Graph::all_concept_changes]` — concept-change records.
- `[mod.rs:Graph::add_instance]`, `[mod.rs:Graph::instance_count]`, `[mod.rs:Graph::all_instances]` — resource-instance access.
- `[mod.rs:Graph::add_entity_instance]`, `[mod.rs:Graph::all_entity_instances]`, `[mod.rs:Graph::has_entity_instance_by_id]`, `[mod.rs:Graph::get_entity_instance_by_id]`, `[mod.rs:Graph::remove_entity_instance_by_id]`, `[mod.rs:Graph::find_entity_instance_by_name]`, `[mod.rs:Graph::find_entity_instance_by_name_and_namespace]` — entity-instance access.
- `[mod.rs:Graph::validate_entity_instances]` — validates entity instances against their `EntityContract` schemas.
- `[mod.rs:Graph::add_policy]`, `[mod.rs:Graph::has_policy]`, `[mod.rs:Graph::get_policy]`, `[mod.rs:Graph::remove_policy]`, `[mod.rs:Graph::all_policies]`, `[mod.rs:Graph::policy_count]`, `[mod.rs:Graph::is_operation_bound_policy]` — policies.
- `[mod.rs:Graph::add_metric]`, `[mod.rs:Graph::has_metric]`, `[mod.rs:Graph::get_metric]`, `[mod.rs:Graph::all_metrics]`, `[mod.rs:Graph::metric_count]` — metrics.
- `[mod.rs:Graph::add_mapping]`, `[mod.rs:Graph::has_mapping]`, `[mod.rs:Graph::get_mapping]`, `[mod.rs:Graph::all_mappings]`, `[mod.rs:Graph::mapping_count]` — mapping contracts.
- `[mod.rs:Graph::add_projection]`, `[mod.rs:Graph::has_projection]`, `[mod.rs:Graph::get_projection]`, `[mod.rs:Graph::all_projections]`, `[mod.rs:Graph::projection_count]` — projection contracts.
- `[mod.rs:Graph::add_declared_dimension]`, `[mod.rs:Graph::add_declared_unit]`, `[mod.rs:Graph::all_declared_dimensions]`, `[mod.rs:Graph::all_declared_units]` — source-declared units, kept out of the ambient `UnitRegistry`.
- `[mod.rs:Graph::set_evaluation_mode]`, `[mod.rs:Graph::use_three_valued_logic]`, `[mod.rs:Graph::config]`, `[mod.rs:Graph::config_mut]`, `[mod.rs:GraphConfig]` — evaluation config.
- `[mod.rs:Graph::extend]` — atomic merge of another graph.
- `[mod.rs:Graph::validate]` — evaluates all non-operation-bound policies and returns a `ValidationResult`.
- `[mod.rs:DeclaredDimension]`, `[mod.rs:DeclaredUnit]` — graph-owned declaration records.
- `[to_ast.rs:Graph::to_ast]` — renders the graph back to an `Ast` (entities and resources only, sorted by name then namespace).
- `[entity_validation.rs:validate]` — `pub(super) fn validate(graph)` returning `Result<(), Vec<String>>` for entity-instance schema conformance.

## Invariants
- `[mod.rs:Graph::add_entity]`/`[mod.rs:Graph::add_role]`/`[mod.rs:Graph::add_resource]`/`[mod.rs:Graph::add_flow]` — insertion is keyed by `ConceptId` and rejects duplicates with an `Err`, never silently replacing.
- `[mod.rs:Graph::remove_entity]` — removal is refused while any flow references the entity as `from_id`/`to_id` or any instance references it as `entity_id`; the error message names the referencing IDs.
- `[mod.rs:Graph::assign_role_to_entity]` — both the entity and the role must already exist, and the same role is not assigned twice to one entity.
- `[mod.rs:Graph::extend]` — merge is atomic: the receiver is only replaced when the whole merge succeeds, which prevents partial state on error.
- `[mod.rs:Graph::validate]` — policies bound to an operation's `access policy_governed by` clause are skipped and counted as not evaluated, because their terms only resolve in the operation's typed contract context (documented limitation L9).
- `[mod.rs:Graph::validate]` — a policy whose evaluation errors produces an ERROR-severity `Violation` instead of aborting validation.
- `[mod.rs:GraphConfig::default]` — three-valued logic defaults to enabled; `use_three_valued_logic` is a serialized field.
- `[mod.rs:DeclaredDimension]` — declared dimensions are graph-owned rather than read from `UnitRegistry`, because the registry also holds ambient builtins and is unordered.

## Side Effects
- None observed in source. `Graph` is an in-memory aggregate: no filesystem, network, or process-global mutation in this module.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
