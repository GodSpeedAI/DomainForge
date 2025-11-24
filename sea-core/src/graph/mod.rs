use crate::policy::{Policy, Severity, Violation};

use crate::primitives::{Entity, Flow, Instance, Relation, Resource, Role};
use crate::validation_result::ValidationResult;
use crate::ConceptId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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
    resources: IndexMap<ConceptId, Resource>,
    flows: IndexMap<ConceptId, Flow>,
    instances: IndexMap<ConceptId, Instance>,

    #[serde(default)]
    roles: IndexMap<ConceptId, Role>,
    #[serde(default)]
    relations: IndexMap<ConceptId, Relation>,
    policies: IndexMap<ConceptId, Policy>,
    #[serde(default)]
    config: GraphConfig,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
            && self.resources.is_empty()
            && self.flows.is_empty()
            && self.instances.is_empty()

            && self.roles.is_empty()
            && self.relations.is_empty()
            && self.policies.is_empty()
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

    pub fn add_instance(&mut self, instance: Instance) -> Result<(), String> {
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

    pub fn get_instance(&self, id: &ConceptId) -> Option<&Instance> {
        self.instances.get(id)
    }

    pub fn remove_instance(&mut self, id: &ConceptId) -> Result<Instance, String> {
        self.instances
            .shift_remove(id)
            .ok_or_else(|| format!("Instance with ID {} not found", id))
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

    pub fn all_entities(&self) -> Vec<&Entity> {
        self.entities.values().collect()
    }

    pub fn all_resources(&self) -> Vec<&Resource> {
        self.resources.values().collect()
    }

    pub fn all_flows(&self) -> Vec<&Flow> {
        self.flows.values().collect()
    }

    pub fn all_instances(&self) -> Vec<&Instance> {
        self.instances.values().collect()
    }



    pub fn role_count(&self) -> usize {
        self.roles.len()
    }

    pub fn add_role(&mut self, role: Role) -> Result<(), String> {
        let id = role.id().clone();
        if self.roles.contains_key(&id) {
            return Err(format!("Role with ID {} already exists", id));
        }
        if !self.entities.contains_key(role.entity_id()) {
            return Err("Entity not found".to_string());
        }
        self.roles.insert(id, role);
        Ok(())
    }

    pub fn has_role(&self, id: &ConceptId) -> bool {
        self.roles.contains_key(id)
    }

    pub fn get_role(&self, id: &ConceptId) -> Option<&Role> {
        self.roles.get(id)
    }

    pub fn remove_role(&mut self, id: &ConceptId) -> Result<Role, String> {
        // Check for references in relations
        let referencing_relations: Vec<String> = self
            .relations
            .values()
            .filter(|rel| {
                rel.subject_role_id() == Some(id) || rel.object_role_id() == Some(id)
            })
            .map(|rel| rel.id().to_string())
            .collect();

        if !referencing_relations.is_empty() {
            return Err(format!(
                "Cannot remove role {} because it is referenced by relations: {}",
                id,
                referencing_relations.join(", ")
            ));
        }

        self.roles
            .shift_remove(id)
            .ok_or_else(|| format!("Role with ID {} not found", id))
    }

    pub fn all_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    pub fn add_relation(&mut self, relation: Relation) -> Result<(), String> {
        let id = relation.id().clone();
        if self.relations.contains_key(&id) {
            return Err(format!("Relation with ID {} already exists", id));
        }
        
        if let Some(role_id) = relation.subject_role_id() {
            if !self.roles.contains_key(role_id) {
                return Err("Subject role not found".to_string());
            }
        }
        
        if let Some(role_id) = relation.object_role_id() {
            if !self.roles.contains_key(role_id) {
                return Err("Object role not found".to_string());
            }
        }
        
        if let Some(resource_id) = relation.via_resource_id() {
            if !self.resources.contains_key(resource_id) {
                return Err("Via resource not found".to_string());
            }

            // Validate that a flow exists between the entities of the subject and object roles
            if let (Some(subject_role_id), Some(object_role_id)) =
                (relation.subject_role_id(), relation.object_role_id())
            {
                // We've already checked that these roles exist above
                let subject_role = self.roles.get(subject_role_id).unwrap();
                let object_role = self.roles.get(object_role_id).unwrap();

                let from_entity = subject_role.entity_id();
                let to_entity = object_role.entity_id();

                let flow_exists = self.flows.values().any(|f| {
                    f.resource_id() == resource_id
                        && f.from_id() == from_entity
                        && f.to_id() == to_entity
                });

                if !flow_exists {
                    return Err(format!(
                        "Relation implies a flow of resource '{}' from entity '{}' to entity '{}', but no such flow exists",
                        resource_id, from_entity, to_entity
                    ));
                }
            } else {
                // If via_resource is specified, we generally expect both roles to be present to define the flow.
                // However, we can be lenient or strict here. Let's enforce it for now as it makes sense.
                return Err(
                    "Relation with 'via flow' must have both subject and object roles defined"
                        .to_string(),
                );
            }
        }
        
        self.relations.insert(id, relation);
        Ok(())
    }

    pub fn has_relation(&self, id: &ConceptId) -> bool {
        self.relations.contains_key(id)
    }

    pub fn get_relation(&self, id: &ConceptId) -> Option<&Relation> {
        self.relations.get(id)
    }

    pub fn remove_relation(&mut self, id: &ConceptId) -> Result<Relation, String> {
        self.relations
            .shift_remove(id)
            .ok_or_else(|| format!("Relation with ID {} not found", id))
    }

    pub fn all_relations(&self) -> Vec<&Relation> {
        self.relations.values().collect()
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
            resources,
            flows,
            instances,

            roles,
            relations,
            policies,
            config: _,
        } = other;

        for entity in entities.into_values() {
            self.add_entity(entity)?;
        }

        for resource in resources.into_values() {
            self.add_resource(resource)?;
        }

        for instance in instances.into_values() {
            self.add_instance(instance)?;
        }

        for flow in flows.into_values() {
            self.add_flow(flow)?;
        }

        for role in roles.into_values() {
            self.add_role(role)?;
        }

        for relation in relations.into_values() {
            self.add_relation(relation)?;
        }

        for policy in policies.into_values() {
            self.add_policy(policy)?;
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
