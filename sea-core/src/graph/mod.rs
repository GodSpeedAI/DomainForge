use crate::policy::{Policy, Violation, Severity};
use crate::validation_result::ValidationResult;
use crate::primitives::{Entity, Flow, Instance, Resource};
use crate::ConceptId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Graph {
    entities: IndexMap<ConceptId, Entity>,
    resources: IndexMap<ConceptId, Resource>,
    flows: IndexMap<ConceptId, Flow>,
    instances: IndexMap<ConceptId, Instance>,
    policies: IndexMap<ConceptId, Policy>,
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
            && self.policies.is_empty()
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

    /// Validate the graph by evaluating all policies against it and
    /// collecting any violations produced. Returns a `ValidationResult`.
    pub fn validate(&self) -> ValidationResult {
        let mut all_violations: Vec<Violation> = Vec::new();
        for policy in self.policies.values() {
            match policy.evaluate(self) {
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
