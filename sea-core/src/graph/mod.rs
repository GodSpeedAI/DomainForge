use crate::patterns::Pattern;
use crate::policy::{Policy, Severity, Violation};
use crate::primitives::{
    ConceptChange, Entity, Flow, Instance, MappingContract, Metric, ProjectionContract,
    RelationType, Resource, ResourceInstance, Role,
};
use crate::validation_result::ValidationResult;
use crate::ConceptId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod to_ast;

/// Configuration for graph evaluation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Enable three-valued logic (True, False, NULL) for policy evaluation.
    /// When false, uses strict boolean logic (True, False).
    pub use_three_valued_logic: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            // Default to three-valued logic for backward compatibility with the feature flag
            use_three_valued_logic: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Graph {
    entities: IndexMap<ConceptId, Entity>,
    roles: IndexMap<ConceptId, Role>,
    resources: IndexMap<ConceptId, Resource>,
    flows: IndexMap<ConceptId, Flow>,
    relations: IndexMap<ConceptId, RelationType>,
    instances: IndexMap<ConceptId, ResourceInstance>,
    /// Entity instances keyed by ConceptId for consistency with other graph collections.
    entity_instances: IndexMap<ConceptId, Instance>,
    policies: IndexMap<ConceptId, Policy>,
    #[serde(default)]
    patterns: IndexMap<ConceptId, Pattern>,
    #[serde(default)]
    concept_changes: IndexMap<ConceptId, ConceptChange>,
    #[serde(default)]
    metrics: IndexMap<ConceptId, Metric>,
    #[serde(default)]
    mappings: IndexMap<ConceptId, MappingContract>,
    #[serde(default)]
    projections: IndexMap<ConceptId, ProjectionContract>,
    #[serde(default)]
    entity_roles: IndexMap<ConceptId, Vec<ConceptId>>,
    #[serde(default)]
    config: GraphConfig,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
            && self.roles.is_empty()
            && self.resources.is_empty()
            && self.flows.is_empty()
            && self.relations.is_empty()
            && self.instances.is_empty()
            && self.entity_instances.is_empty()
            && self.policies.is_empty()
            && self.patterns.is_empty()
            && self.concept_changes.is_empty()
            && self.concept_changes.is_empty()
            && self.metrics.is_empty()
            && self.mappings.is_empty()
            && self.projections.is_empty()
    }

    /// Set the evaluation mode for policy evaluation.
    /// When `use_three_valued_logic` is true, policies will use three-valued logic (True, False, NULL).
    /// When false, policies will use strict boolean logic (True, False).
    pub fn set_evaluation_mode(&mut self, use_three_valued_logic: bool) {
        self.config.use_three_valued_logic = use_three_valued_logic;
    }

    /// Get the current evaluation mode.
    pub fn use_three_valued_logic(&self) -> bool {
        self.config.use_three_valued_logic
    }

    pub fn config(&self) -> &GraphConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut GraphConfig {
        &mut self.config
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn add_entity(&mut self, entity: Entity) -> Result<(), String> {
        let id = entity.id().clone();
        if self.entities.contains_key(&id) {
            return Err(format!("Entity with ID {} already exists", id));
        }
        self.entities.insert(id, entity);
        Ok(())
    }

    pub fn has_entity(&self, id: &ConceptId) -> bool {
        self.entities.contains_key(id)
    }

    pub fn get_entity(&self, id: &ConceptId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Returns a mutable reference to an Entity identified by `id`.
    /// This method is necessary for operations that need to update attributes
    /// on existing entities (such as association relationships).
    pub fn get_entity_mut(&mut self, id: &ConceptId) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }

    pub fn remove_entity(&mut self, id: &ConceptId) -> Result<Entity, String> {
        // Check for references in flows
        let referencing_flows: Vec<String> = self
            .flows
            .values()
            .filter(|flow| flow.from_id() == id || flow.to_id() == id)
            .map(|flow| flow.id().to_string())
            .collect();

        // Check for references in instances
        let referencing_instances: Vec<String> = self
            .instances
            .values()
            .filter(|instance| instance.entity_id() == id)
            .map(|instance| instance.id().to_string())
            .collect();

        if !referencing_flows.is_empty() || !referencing_instances.is_empty() {
            let mut error_msg = format!("Cannot remove entity {} because it is referenced", id);
            if !referencing_flows.is_empty() {
                error_msg.push_str(&format!(" by flows: {}", referencing_flows.join(", ")));
            }
            if !referencing_instances.is_empty() {
                error_msg.push_str(&format!(
                    " by instances: {}",
                    referencing_instances.join(", ")
                ));
            }
            return Err(error_msg);
        }

        self.entities
            .shift_remove(id)
            .ok_or_else(|| format!("Entity with ID {} not found", id))
    }

    pub fn role_count(&self) -> usize {
        self.roles.len()
    }

    pub fn add_role(&mut self, role: Role) -> Result<(), String> {
        let id = role.id().clone();
        if self.roles.contains_key(&id) {
            return Err(format!("Role with ID {} already exists", id));
        }
        self.roles.insert(id, role);
        Ok(())
    }

    pub fn get_role(&self, id: &ConceptId) -> Option<&Role> {
        self.roles.get(id)
    }

    pub fn has_role(&self, id: &ConceptId) -> bool {
        self.roles.contains_key(id)
    }

    pub fn assign_role_to_entity(
        &mut self,
        entity_id: ConceptId,
        role_id: ConceptId,
    ) -> Result<(), String> {
        if !self.entities.contains_key(&entity_id) {
            return Err(format!("Entity with ID {} not found", entity_id));
        }
        if !self.roles.contains_key(&role_id) {
            return Err(format!("Role with ID {} not found", role_id));
        }

        let roles = self.entity_roles.entry(entity_id).or_default();
        if !roles.contains(&role_id) {
            roles.push(role_id);
        }
        Ok(())
    }

    pub fn roles_for_entity(&self, entity_id: &ConceptId) -> Option<&Vec<ConceptId>> {
        self.entity_roles.get(entity_id)
    }

    pub fn role_names_for_entity(&self, entity_id: &ConceptId) -> Vec<String> {
        self.entity_roles
            .get(entity_id)
            .into_iter()
            .flat_map(|roles| roles.iter())
            .filter_map(|role_id| self.roles.get(role_id))
            .map(|role| role.name().to_string())
            .collect()
    }

    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    pub fn add_resource(&mut self, resource: Resource) -> Result<(), String> {
        let id = resource.id().clone();
        if self.resources.contains_key(&id) {
            return Err(format!("Resource with ID {} already exists", id));
        }
        self.resources.insert(id, resource);
        Ok(())
    }

    pub fn has_resource(&self, id: &ConceptId) -> bool {
        self.resources.contains_key(id)
    }

    pub fn get_resource(&self, id: &ConceptId) -> Option<&Resource> {
        self.resources.get(id)
    }

    pub fn remove_resource(&mut self, id: &ConceptId) -> Result<Resource, String> {
        // Check for references in flows
        let referencing_flows: Vec<String> = self
            .flows
            .values()
            .filter(|flow| flow.resource_id() == id)
            .map(|flow| flow.id().to_string())
            .collect();

        // Check for references in instances
        let referencing_instances: Vec<String> = self
            .instances
            .values()
            .filter(|instance| instance.resource_id() == id)
            .map(|instance| instance.id().to_string())
            .collect();

        if !referencing_flows.is_empty() || !referencing_instances.is_empty() {
            let mut error_msg = format!("Cannot remove resource {} because it is referenced", id);
            if !referencing_flows.is_empty() {
                error_msg.push_str(&format!(" by flows: {}", referencing_flows.join(", ")));
            }
            if !referencing_instances.is_empty() {
                error_msg.push_str(&format!(
                    " by instances: {}",
                    referencing_instances.join(", ")
                ));
            }
            return Err(error_msg);
        }

        self.resources
            .shift_remove(id)
            .ok_or_else(|| format!("Resource with ID {} not found", id))
    }

    pub fn flow_count(&self) -> usize {
        self.flows.len()
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    pub fn add_relation_type(&mut self, relation: RelationType) -> Result<(), String> {
        let id = relation.id().clone();
        if self.relations.contains_key(&id) {
            return Err(format!("Relation with ID {} already exists", id));
        }
        self.relations.insert(id, relation);
        Ok(())
    }

    pub fn all_relations(&self) -> Vec<&RelationType> {
        self.relations.values().collect()
    }

    pub fn add_flow(&mut self, flow: Flow) -> Result<(), String> {
        let id = flow.id().clone();
        if self.flows.contains_key(&id) {
            return Err(format!("Flow with ID {} already exists", id));
        }
        if !self.entities.contains_key(flow.from_id()) {
            return Err("Source entity not found".to_string());
        }
        if !self.entities.contains_key(flow.to_id()) {
            return Err("Target entity not found".to_string());
        }
        if !self.resources.contains_key(flow.resource_id()) {
            return Err("Resource not found".to_string());
        }
        self.flows.insert(id, flow);
        Ok(())
    }

    pub fn add_pattern(&mut self, pattern: Pattern) -> Result<(), String> {
        let id = pattern.id().clone();
        if self.patterns.contains_key(&id) {
            return Err(format!("Pattern with ID {} already exists", id));
        }

        if self.patterns.values().any(|existing| {
            existing.name() == pattern.name() && existing.namespace() == pattern.namespace()
        }) {
            return Err(format!(
                "Pattern '{}' already declared in namespace '{}'",
                pattern.name(),
                pattern.namespace()
            ));
        }

        self.patterns.insert(id, pattern);
        Ok(())
    }

    pub fn add_concept_change(&mut self, change: ConceptChange) -> Result<(), String> {
        let id = change.id().clone();
        if self.concept_changes.contains_key(&id) {
            return Err(format!("ConceptChange with ID {} already exists", id));
        }
        self.concept_changes.insert(id, change);
        Ok(())
    }

    pub fn get_concept_change(&self, id: &ConceptId) -> Option<&ConceptChange> {
        self.concept_changes.get(id)
    }

    pub fn all_concept_changes(&self) -> Vec<&ConceptChange> {
        self.concept_changes.values().collect()
    }

    pub fn has_flow(&self, id: &ConceptId) -> bool {
        self.flows.contains_key(id)
    }

    pub fn get_flow(&self, id: &ConceptId) -> Option<&Flow> {
        self.flows.get(id)
    }

    pub fn remove_flow(&mut self, id: &ConceptId) -> Result<Flow, String> {
        self.flows
            .shift_remove(id)
            .ok_or_else(|| format!("Flow with ID {} not found", id))
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn add_instance(&mut self, instance: ResourceInstance) -> Result<(), String> {
        let id = instance.id().clone();
        if self.instances.contains_key(&id) {
            return Err(format!("Instance with ID {} already exists", id));
        }
        if !self.entities.contains_key(instance.entity_id()) {
            return Err("Entity not found".to_string());
        }
        if !self.resources.contains_key(instance.resource_id()) {
            return Err("Resource not found".to_string());
        }
        self.instances.insert(id, instance);
        Ok(())
    }

    /// Adds an association relationship from `owner` -> `owned` using a named `rel_type`.
    /// Associations are represented as a JSON attribute `associations` on the source entity to
    /// maintain a simple, serializable representation without introducing a new primitive.
    pub fn add_association(
        &mut self,
        owner: &ConceptId,
        owned: &ConceptId,
        rel_type: &str,
    ) -> Result<(), String> {
        if !self.entities.contains_key(owner) {
            return Err("Source entity not found".to_string());
        }
        if !self.entities.contains_key(owned) {
            return Err("Destination entity not found".to_string());
        }

        let owned_str = owned.to_string();
        let rel_type_str = rel_type.to_string();

        // Mutably borrow the entity and insert/update the `associations` attribute.
        if let Some(entity) = self.entities.get_mut(owner) {
            use serde_json::{json, Value};
            let mut associations: Value = entity
                .get_attribute("associations")
                .cloned()
                .unwrap_or_else(|| json!([]));

            if let Value::Array(arr) = &mut associations {
                arr.push(json!({"type": rel_type_str, "target": owned_str}));
            } else {
                // Replace non-array associations with an array structure.
                associations = json!([ {"type": rel_type_str, "target": owned_str} ]);
            }

            entity.set_attribute("associations", associations);
            Ok(())
        } else {
            Err("Failed to retrieve entity for association".to_string())
        }
    }

    pub fn has_instance(&self, id: &ConceptId) -> bool {
        self.instances.contains_key(id)
    }

    pub fn get_instance(&self, id: &ConceptId) -> Option<&ResourceInstance> {
        self.instances.get(id)
    }

    pub fn remove_instance(&mut self, id: &ConceptId) -> Result<ResourceInstance, String> {
        self.instances
            .shift_remove(id)
            .ok_or_else(|| format!("Instance with ID {} not found", id))
    }

    // Entity Instance methods
    pub fn entity_instance_count(&self) -> usize {
        self.entity_instances.len()
    }

    pub fn add_entity_instance(&mut self, instance: Instance) -> Result<(), String> {
        let id = instance.id().clone();
        if self.entity_instances.contains_key(&id) {
            return Err(format!(
                "Entity instance '{}' already exists",
                instance.name()
            ));
        }
        if self.find_entity_instance_by_name(instance.name()).is_some() {
            return Err(format!(
                "Entity instance '{}' already exists",
                instance.name()
            ));
        }

        let namespace = instance.namespace();
        let entity_type = instance.entity_type();

        self.find_entity_by_name_and_namespace(entity_type, namespace)
            .ok_or_else(|| {
                format!(
                    "Entity '{}' not found in namespace '{}'",
                    entity_type, namespace
                )
            })?;

        self.entity_instances.insert(id, instance);
        Ok(())
    }

    pub fn get_entity_instance(&self, name: &str) -> Option<&Instance> {
        self.find_entity_instance_by_name(name)
            .and_then(|id| self.entity_instances.get(&id))
    }

    pub fn get_entity_instance_mut(&mut self, name: &str) -> Option<&mut Instance> {
        let id = self.find_entity_instance_by_name(name)?;
        self.entity_instances.get_mut(&id)
    }

    pub fn all_entity_instances(&self) -> Vec<&Instance> {
        self.entity_instances.values().collect()
    }

    pub fn remove_entity_instance(&mut self, name: &str) -> Result<Instance, String> {
        let id = self
            .find_entity_instance_by_name(name)
            .ok_or_else(|| format!("Entity instance '{}' not found", name))?;

        self.entity_instances
            .shift_remove(&id)
            .ok_or_else(|| format!("Entity instance '{}' not found", name))
    }

    pub fn has_entity_instance_by_id(&self, id: &ConceptId) -> bool {
        self.entity_instances.contains_key(id)
    }

    pub fn get_entity_instance_by_id(&self, id: &ConceptId) -> Option<&Instance> {
        self.entity_instances.get(id)
    }

    pub fn remove_entity_instance_by_id(&mut self, id: &ConceptId) -> Result<Instance, String> {
        self.entity_instances
            .shift_remove(id)
            .ok_or_else(|| format!("Entity instance with id '{}' not found", id))
    }

    pub fn flows_from(&self, entity_id: &ConceptId) -> Vec<&Flow> {
        self.flows
            .values()
            .filter(|flow| flow.from_id() == entity_id)
            .collect()
    }

    pub fn flows_to(&self, entity_id: &ConceptId) -> Vec<&Flow> {
        self.flows
            .values()
            .filter(|flow| flow.to_id() == entity_id)
            .collect()
    }

    pub fn upstream_entities(&self, entity_id: &ConceptId) -> Vec<&Entity> {
        self.flows_to(entity_id)
            .iter()
            .filter_map(|flow| self.get_entity(flow.from_id()))
            .collect()
    }

    pub fn downstream_entities(&self, entity_id: &ConceptId) -> Vec<&Entity> {
        self.flows_from(entity_id)
            .iter()
            .filter_map(|flow| self.get_entity(flow.to_id()))
            .collect()
    }

    pub fn find_entity_by_name_and_namespace(
        &self,
        name: &str,
        namespace: &str,
    ) -> Option<ConceptId> {
        self.entities
            .iter()
            .find(|(_, entity)| entity.name() == name && entity.namespace() == namespace)
            .map(|(id, _)| id.clone())
    }

    pub fn find_entity_by_name(&self, name: &str) -> Option<ConceptId> {
        self.entities
            .iter()
            .find(|(_, entity)| entity.name() == name)
            .map(|(id, _)| id.clone())
    }

    pub fn find_resource_by_name(&self, name: &str) -> Option<ConceptId> {
        self.resources
            .iter()
            .find(|(_, resource)| resource.name() == name)
            .map(|(id, _)| id.clone())
    }

    pub fn find_role_by_name(&self, name: &str) -> Option<ConceptId> {
        self.roles
            .iter()
            .find(|(_, role)| role.name() == name)
            .map(|(id, _)| id.clone())
    }

    pub fn find_entity_instance_by_name(&self, name: &str) -> Option<ConceptId> {
        self.entity_instances
            .iter()
            .find(|(_, instance)| instance.name() == name)
            .map(|(id, _)| id.clone())
    }

    pub fn find_entity_instance_by_name_and_namespace(
        &self,
        name: &str,
        namespace: &str,
    ) -> Option<ConceptId> {
        self.entity_instances
            .iter()
            .find(|(_, instance)| instance.name() == name && instance.namespace() == namespace)
            .map(|(id, _)| id.clone())
    }

    pub fn find_pattern(&self, name: &str, namespace: Option<&str>) -> Option<&Pattern> {
        if let Some(ns) = namespace {
            if let Some((_, pattern)) = self
                .patterns
                .iter()
                .find(|(_, pattern)| pattern.name() == name && pattern.namespace() == ns)
            {
                return Some(pattern);
            }
        }

        self.patterns
            .iter()
            .find(|(_, pattern)| pattern.name() == name)
            .map(|(_, pattern)| pattern)
    }

    pub fn all_entities(&self) -> Vec<&Entity> {
        self.entities.values().collect()
    }

    pub fn all_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    pub fn all_resources(&self) -> Vec<&Resource> {
        self.resources.values().collect()
    }

    pub fn all_flows(&self) -> Vec<&Flow> {
        self.flows.values().collect()
    }

    pub fn all_instances(&self) -> Vec<&ResourceInstance> {
        self.instances.values().collect()
    }

    pub fn all_patterns(&self) -> Vec<&Pattern> {
        self.patterns.values().collect()
    }

    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }

    pub fn add_policy(&mut self, policy: Policy) -> Result<(), String> {
        let id = policy.id.clone();
        if self.policies.contains_key(&id) {
            return Err(format!("Policy with ID {} already exists", id));
        }
        self.policies.insert(id, policy);
        Ok(())
    }

    pub fn has_policy(&self, id: &ConceptId) -> bool {
        self.policies.contains_key(id)
    }

    pub fn get_policy(&self, id: &ConceptId) -> Option<&Policy> {
        self.policies.get(id)
    }

    pub fn remove_policy(&mut self, id: &ConceptId) -> Result<Policy, String> {
        self.policies
            .shift_remove(id)
            .ok_or_else(|| format!("Policy with ID {} not found", id))
    }

    pub fn all_policies(&self) -> Vec<&Policy> {
        self.policies.values().collect()
    }

    pub fn metric_count(&self) -> usize {
        self.metrics.len()
    }

    pub fn add_metric(&mut self, metric: Metric) -> Result<(), String> {
        let id = metric.id().clone();
        if self.metrics.contains_key(&id) {
            return Err(format!("Metric with ID {} already exists", id));
        }
        self.metrics.insert(id, metric);
        Ok(())
    }

    pub fn has_metric(&self, id: &ConceptId) -> bool {
        self.metrics.contains_key(id)
    }

    pub fn get_metric(&self, id: &ConceptId) -> Option<&Metric> {
        self.metrics.get(id)
    }

    pub fn all_metrics(&self) -> Vec<&Metric> {
        self.metrics.values().collect()
    }

    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    pub fn add_mapping(&mut self, mapping: MappingContract) -> Result<(), String> {
        let id = mapping.id().clone();
        if self.mappings.contains_key(&id) {
            return Err(format!("Mapping with ID {} already exists", id));
        }
        self.mappings.insert(id, mapping);
        Ok(())
    }

    pub fn has_mapping(&self, id: &ConceptId) -> bool {
        self.mappings.contains_key(id)
    }

    pub fn get_mapping(&self, id: &ConceptId) -> Option<&MappingContract> {
        self.mappings.get(id)
    }

    pub fn all_mappings(&self) -> Vec<&MappingContract> {
        self.mappings.values().collect()
    }

    pub fn projection_count(&self) -> usize {
        self.projections.len()
    }

    pub fn add_projection(&mut self, projection: ProjectionContract) -> Result<(), String> {
        let id = projection.id().clone();
        if self.projections.contains_key(&id) {
            return Err(format!("Projection with ID {} already exists", id));
        }
        self.projections.insert(id, projection);
        Ok(())
    }

    pub fn has_projection(&self, id: &ConceptId) -> bool {
        self.projections.contains_key(id)
    }

    pub fn get_projection(&self, id: &ConceptId) -> Option<&ProjectionContract> {
        self.projections.get(id)
    }

    pub fn all_projections(&self) -> Vec<&ProjectionContract> {
        self.projections.values().collect()
    }

    /// Extend this graph with all nodes and policies from another graph.
    /// The operation is atomic: the existing graph is only modified if the entire
    /// merge succeeds, which prevents partial state when errors occur.
    pub fn extend(&mut self, other: Graph) -> Result<(), String> {
        let mut merged = self.clone();
        merged.extend_from_graph(other)?;
        *self = merged;
        Ok(())
    }

    fn extend_from_graph(&mut self, other: Graph) -> Result<(), String> {
        let Graph {
            entities,
            roles,
            resources,
            flows,
            relations,
            instances,
            entity_instances,
            policies,
            patterns,
            concept_changes,
            metrics,
            mappings,
            projections,
            entity_roles,
            config: _,
        } = other;

        for entity in entities.into_values() {
            self.add_entity(entity)?;
        }

        for role in roles.into_values() {
            self.add_role(role)?;
        }

        for resource in resources.into_values() {
            self.add_resource(resource)?;
        }

        for instance in instances.into_values() {
            self.add_instance(instance)?;
        }

        for entity_instance in entity_instances.into_values() {
            self.add_entity_instance(entity_instance)?;
        }

        for flow in flows.into_values() {
            self.add_flow(flow)?;
        }

        for relation in relations.into_values() {
            self.add_relation_type(relation)?;
        }

        for pattern in patterns.into_values() {
            self.add_pattern(pattern)?;
        }

        for (entity_id, roles_for_entity) in entity_roles.into_iter() {
            for role_id in roles_for_entity {
                self.assign_role_to_entity(entity_id.clone(), role_id)?;
            }
        }

        for policy in policies.into_values() {
            self.add_policy(policy)?;
        }

        for change in concept_changes.into_values() {
            self.add_concept_change(change)?;
        }

        for metric in metrics.into_values() {
            self.add_metric(metric)?;
        }

        for mapping in mappings.into_values() {
            self.add_mapping(mapping)?;
        }

        for projection in projections.into_values() {
            self.add_projection(projection)?;
        }

        Ok(())
    }

    /// Validate the graph by evaluating all policies against it and
    /// collecting any violations produced. Returns a `ValidationResult`.
    pub fn validate(&self) -> ValidationResult {
        let mut all_violations: Vec<Violation> = Vec::new();
        let use_three_valued_logic = self.config.use_three_valued_logic;

        for policy in self.policies.values() {
            match policy.evaluate_with_mode(self, use_three_valued_logic) {
                Ok(eval) => {
                    all_violations.extend(eval.violations);
                }
                Err(err) => {
                    // If a policy evaluation fails, produce an ERROR severity
                    // violation indicating evaluation failure so the user can
                    // see the issue.
                    let v = Violation::new(
                        &policy.name,
                        format!("Policy evaluation failed: {}", err),
                        Severity::Error,
                    );
                    all_violations.push(v);
                }
            }
        }

        ValidationResult::new(self.policies.len(), all_violations)
    }
}
