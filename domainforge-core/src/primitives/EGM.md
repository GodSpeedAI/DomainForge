# primitives — Core Domain Primitives

## Overview
The declared model elements of the DSL: `Entity`, `Resource`, `Flow`, `Instance`,
`ResourceInstance`, `Role`, `RelationType`, `Pattern`, `Metric`, `ConceptChange`,
`MappingContract`, `ProjectionContract`, and `Quantity`. Each carries a
deterministic `ConceptId`, a namespace, and a free-form attribute bag.

## Interface
- `[entity.rs:Entity]` — `+new/new_with_namespace/with_version/with_replaces/with_changes/version/replaces/changes/from_legacy_uuid/id/name/namespace/set_attribute/get_attribute/attributes`.
- `[resource.rs:Resource]` — `+new/new_with_namespace/from_legacy_uuid/id/name/unit/unit_symbol/namespace/set_attribute/get_attribute/attributes`.
- `[flow.rs:Flow]` — `+new/new_with_namespace/id/resource_id/from_id/to_id/quantity/namespace/set_attribute/get_attribute/attributes`.
- `[instance.rs:Instance]` — `+new/new_with_namespace/id/name/entity_type/namespace/set_field/get_field/fields/fields_mut`.
- `[resource_instance.rs:ResourceInstance]` — `+new/new_with_namespace/id/resource_id/entity_id/namespace/set_attribute/get_attribute/attributes`.
- `[role.rs:Role]` — `+new_with_namespace/id/name/namespace/attributes/set_attribute`.
- `[relation.rs:RelationType]` — `+new/id/name/namespace/subject_role/predicate/object_role/via_flow`.
- `[metric.rs:Metric]` — `+new/id/with_refresh_interval/with_unit/with_threshold/with_severity/with_target/with_window/validate`; `[metric.rs:Severity]` is a metric-scoped severity enum distinct from `policy::Severity`.
- `[concept_change.rs:ConceptChange]` — `+new/new_with_namespace/id/name/namespace/from_version/to_version/migration_policy/is_breaking_change`.
- `[mapping_contract.rs:MappingContract]` — `+new/id/name/namespace/target_format/rules`.
- `[projection_contract.rs:ProjectionContract]` — `+new/id/name/namespace/target_format/overrides`.
- `[quantity.rs:Quantity]` — `+new(value, unit, dimension) -> Result<Self, String>`; `[quantity.rs:QuantityFormatter]` (`+new(locale)/format`).
- `[mod.rs]` — re-exports all of the above (`Metric, Severity`, `Quantity`, `RelationType`, `Resource`, `ResourceInstance`, `Role`, ...).

## Invariants
- `[entity.rs:Entity]`/`[resource.rs:Resource]`/`[flow.rs:Flow]`/`[concept_change.rs:ConceptChange]` — every primitive has both `new` and `new_with_namespace`; the plain constructor must not silently invent a namespace that the namespaced constructor would have specified.
- `[metric.rs:Metric::validate]` — a metric is not considered well-formed until `validate()` returns `Ok`; callers must not assume construction implies validity.
- `[quantity.rs:Quantity::new]` — constructing a `Quantity` from a value, unit, and dimension is fallible, so unit/dimension mismatch surfaces as `Err` rather than a silently coerced quantity.
- `[resource.rs:Resource::unit_symbol]` — the resource's unit symbol is derived from its `Unit`, so the two cannot diverge.

## Side Effects
- None observed in source. All primitives are in-memory values with getters/setters over their own fields.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
