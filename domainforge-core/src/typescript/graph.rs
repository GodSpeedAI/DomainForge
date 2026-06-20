#![allow(clippy::new_without_default, clippy::inherent_to_string)]

use crate::graph::Graph as RustGraph;
use crate::parser;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::str::FromStr;
use uuid::Uuid;

use super::primitives::{Entity, Flow, Relation, Resource, ResourceInstance, Role};

#[napi]
pub struct Graph {
    inner: RustGraph,
}

/// Helper function to parse a UUID string into a ConceptId
fn parse_concept_id(id: &str) -> Result<crate::ConceptId> {
    let uuid =
        Uuid::from_str(id).map_err(|e| Error::from_reason(format!("Invalid UUID: {}", e)))?;
    Ok(crate::ConceptId::from(uuid))
}

#[napi]
impl Graph {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RustGraph::new(),
        }
    }

    #[napi]
    pub fn add_entity(&mut self, entity: &Entity) -> Result<()> {
        self.inner
            .add_entity(entity.inner_ref().clone())
            .map_err(Error::from_reason)
    }

    #[napi]
    pub fn add_resource(&mut self, resource: &Resource) -> Result<()> {
        self.inner
            .add_resource(resource.inner_ref().clone())
            .map_err(Error::from_reason)
    }

    #[napi]
    pub fn add_flow(&mut self, flow: &Flow) -> Result<()> {
        self.inner
            .add_flow(flow.inner_ref().clone())
            .map_err(Error::from_reason)
    }

    #[napi]
    pub fn add_instance(&mut self, instance: &ResourceInstance) -> Result<()> {
        self.inner
            .add_instance(instance.inner_ref().clone())
            .map_err(Error::from_reason)
    }

    #[napi]
    pub fn add_role(&mut self, role: &Role) -> Result<()> {
        self.inner
            .add_role(role.inner_ref().clone())
            .map_err(Error::from_reason)
    }

    #[napi]
    pub fn add_relation(&mut self, relation: &Relation) -> Result<()> {
        self.inner
            .add_relation_type(relation.inner_ref().clone())
            .map_err(Error::from_reason)
    }

    #[napi]
    pub fn entity_count(&self) -> u32 {
        self.inner.entity_count() as u32
    }

    #[napi]
    pub fn resource_count(&self) -> u32 {
        self.inner.resource_count() as u32
    }

    #[napi]
    pub fn flow_count(&self) -> u32 {
        self.inner.flow_count() as u32
    }

    #[napi]
    pub fn instance_count(&self) -> u32 {
        self.inner.instance_count() as u32
    }

    #[napi]
    pub fn pattern_count(&self) -> u32 {
        self.inner.pattern_count() as u32
    }

    #[napi]
    pub fn role_count(&self) -> u32 {
        self.inner.role_count() as u32
    }

    #[napi]
    pub fn relation_count(&self) -> u32 {
        self.inner.relation_count() as u32
    }

    #[napi]
    pub fn has_entity(&self, id: String) -> Result<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_entity(&cid))
    }

    #[napi]
    pub fn has_resource(&self, id: String) -> Result<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_resource(&cid))
    }

    #[napi]
    pub fn has_flow(&self, id: String) -> Result<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_flow(&cid))
    }

    #[napi]
    pub fn has_instance(&self, id: String) -> Result<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_instance(&cid))
    }

    #[napi]
    pub fn get_entity(&self, id: String) -> Result<Option<Entity>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_entity(&cid)
            .map(|e| Entity::from_rust(e.clone())))
    }

    #[napi]
    pub fn get_resource(&self, id: String) -> Result<Option<Resource>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_resource(&cid)
            .map(|r| Resource::from_rust(r.clone())))
    }

    #[napi]
    pub fn get_flow(&self, id: String) -> Result<Option<Flow>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_flow(&cid)
            .map(|f| Flow::from_rust(f.clone())))
    }

    #[napi]
    pub fn get_instance(&self, id: String) -> Result<Option<ResourceInstance>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_instance(&cid)
            .map(|i| ResourceInstance::from_rust(i.clone())))
    }

    #[napi]
    pub fn find_role_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_role_by_name(&name)
            .map(|uuid| uuid.to_string())
    }

    #[napi]
    pub fn find_entity_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_entity_by_name(&name)
            .map(|uuid| uuid.to_string())
    }

    #[napi]
    pub fn find_resource_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_resource_by_name(&name)
            .map(|uuid| uuid.to_string())
    }

    #[napi]
    pub fn flows_from(&self, entity_id: String) -> Result<Vec<Flow>> {
        let cid = parse_concept_id(&entity_id)?;
        Ok(self
            .inner
            .flows_from(&cid)
            .into_iter()
            .map(|f| Flow::from_rust(f.clone()))
            .collect())
    }

    #[napi]
    pub fn flows_to(&self, entity_id: String) -> Result<Vec<Flow>> {
        let cid = parse_concept_id(&entity_id)?;
        Ok(self
            .inner
            .flows_to(&cid)
            .into_iter()
            .map(|f| Flow::from_rust(f.clone()))
            .collect())
    }

    #[napi]
    pub fn all_entities(&self) -> Vec<Entity> {
        self.inner
            .all_entities()
            .into_iter()
            .map(|e| Entity::from_rust(e.clone()))
            .collect()
    }

    #[napi]
    pub fn all_resources(&self) -> Vec<Resource> {
        self.inner
            .all_resources()
            .into_iter()
            .map(|r| Resource::from_rust(r.clone()))
            .collect()
    }

    #[napi]
    pub fn all_flows(&self) -> Vec<Flow> {
        self.inner
            .all_flows()
            .into_iter()
            .map(|f| Flow::from_rust(f.clone()))
            .collect()
    }

    #[napi]
    pub fn all_instances(&self) -> Vec<ResourceInstance> {
        self.inner
            .all_instances()
            .into_iter()
            .map(|i| ResourceInstance::from_rust(i.clone()))
            .collect()
    }

    #[napi]
    pub fn all_roles(&self) -> Vec<Role> {
        self.inner
            .all_roles()
            .into_iter()
            .map(|r| Role::from_rust(r.clone()))
            .collect()
    }

    #[napi]
    pub fn all_relations(&self) -> Vec<Relation> {
        self.inner
            .all_relations()
            .into_iter()
            .map(|relation| Relation::from_rust(relation.clone()))
            .collect()
    }

    #[napi(factory)]
    pub fn parse(source: String) -> Result<Self> {
        let graph = parser::parse_to_graph(&source)
            .map_err(|e| Error::from_reason(format!("Parse error: {}", e)))?;

        Ok(Self { inner: graph })
    }

    #[napi]
    pub fn export_calm(&self) -> Result<String> {
        crate::calm::export(&self.inner)
            .and_then(|value| {
                serde_json::to_string_pretty(&value)
                    .map_err(|e| format!("Serialization error: {}", e))
            })
            .map_err(Error::from_reason)
    }

    #[napi(factory)]
    pub fn import_calm(calm_json: String) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(&calm_json)
            .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

        let graph = crate::calm::import(value)
            .map_err(|e| Error::from_reason(format!("Import error: {}", e)))?;

        Ok(Self { inner: graph })
    }

    /// Export the graph to Protobuf .proto text format.
    ///
    /// @param package - The Protobuf package name (e.g., "com.example.api")
    /// @param namespace - Optional namespace filter (undefined = all namespaces)
    /// @param projectionName - Optional name for the projection (used in comments)
    /// @param includeGovernance - Whether to include governance messages (PolicyViolation, MetricEvent)
    /// @param includeServices - Whether to generate gRPC service definitions from Flow patterns
    /// @returns The generated .proto file content as a string
    #[napi]
    pub fn export_protobuf(
        &self,
        package: String,
        namespace: Option<String>,
        projection_name: Option<String>,
        include_governance: Option<bool>,
        include_services: Option<bool>,
    ) -> String {
        let ns = namespace.as_deref().unwrap_or("");
        let proj_name = projection_name.as_deref().unwrap_or("");
        let include_gov = include_governance.unwrap_or(false);
        let include_svc = include_services.unwrap_or(false);

        let proto = crate::projection::ProtobufEngine::project_with_full_options(
            &self.inner,
            ns,
            &package,
            proj_name,
            include_gov,
            include_svc,
        );
        proto.to_proto_string()
    }

    #[napi]
    pub fn add_policy(&mut self, policy_json: String) -> Result<()> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| Error::from_reason(format!("Invalid Policy JSON: {}", e)))?;
        self.inner.add_policy(policy).map_err(Error::from_reason)
    }

    #[napi]
    pub fn add_association(
        &mut self,
        owner: String,
        owned: String,
        rel_type: String,
    ) -> Result<()> {
        let owner_cid = parse_concept_id(&owner)?;
        let owned_cid = parse_concept_id(&owned)?;
        self.inner
            .add_association(&owner_cid, &owned_cid, &rel_type)
            .map_err(Error::from_reason)
    }

    /// Evaluate a Policy JSON payload against this Graph.
    ///
    /// The `policy_json` argument must be the JSON representation of `crate::policy::Policy`
    /// (including fields like `id`, `name`, `modality`, `kind`, `version`, and `expression`).
    /// Returns `EvaluationResult` exposed to TypeScript via napi, or an error if the policy JSON
    /// is invalid or evaluation fails.
    #[napi]
    pub fn evaluate_policy(&self, policy_json: String) -> Result<super::policy::EvaluationResult> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| Error::from_reason(format!("Invalid Policy JSON: {}", e)))?;
        let result = policy
            .evaluate(&self.inner)
            .map_err(|e| Error::from_reason(format!("Policy evaluation error: {}", e)))?;
        Ok(result.into())
    }

    /// Set the evaluation mode for policy evaluation.
    /// When `useThreeValuedLogic` is true, policies will use three-valued logic (true, false, null).
    /// When false, policies will use strict boolean logic (true, false).
    #[napi]
    pub fn set_evaluation_mode(&mut self, use_three_valued_logic: bool) {
        self.inner.set_evaluation_mode(use_three_valued_logic);
    }

    /// Get the current evaluation mode.
    /// Returns true if three-valued logic is enabled, false otherwise.
    #[napi]
    pub fn use_three_valued_logic(&self) -> bool {
        self.inner.use_three_valued_logic()
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Graph(entities={}, resources={}, flows={}, instances={})",
            self.inner.entity_count(),
            self.inner.resource_count(),
            self.inner.flow_count(),
            self.inner.instance_count()
        )
    }
}
