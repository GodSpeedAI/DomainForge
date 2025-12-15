#![allow(clippy::useless_conversion)]

use crate::graph::Graph as RustGraph;
use crate::parser;
use crate::ConceptId;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

use super::primitives::{Entity, Flow, Relation, Resource, ResourceInstance, Role};

#[pyclass]
#[derive(Clone)]
pub struct Graph {
    inner: RustGraph,
}

/// Helper function to parse a UUID string into a ConceptId
fn parse_concept_id(id: &str) -> PyResult<ConceptId> {
    let uuid =
        Uuid::from_str(id).map_err(|e| PyValueError::new_err(format!("Invalid UUID: {}", e)))?;
    Ok(ConceptId::from(uuid))
}

#[pymethods]
impl Graph {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustGraph::new(),
        }
    }

    fn add_entity(&mut self, entity: &Entity) -> PyResult<()> {
        self.inner
            .add_entity(entity.clone().into_inner())
            .map_err(|e| PyValueError::new_err(format!("Add entity error: {}", e)))
    }

    fn add_resource(&mut self, resource: &Resource) -> PyResult<()> {
        self.inner
            .add_resource(resource.clone().into_inner())
            .map_err(|e| PyValueError::new_err(format!("Add resource error: {}", e)))
    }

    fn add_flow(&mut self, flow: &Flow) -> PyResult<()> {
        self.inner
            .add_flow(flow.clone().into_inner())
            .map_err(|e| PyValueError::new_err(format!("Add flow error: {}", e)))
    }

    fn add_instance(&mut self, instance: &ResourceInstance) -> PyResult<()> {
        self.inner
            .add_instance(instance.clone().into_inner())
            .map_err(|e| PyValueError::new_err(format!("Add instance error: {}", e)))
    }

    fn add_role(&mut self, role: &Role) -> PyResult<()> {
        self.inner
            .add_role(role.clone().into_inner())
            .map_err(|e| PyValueError::new_err(format!("Add role error: {}", e)))
    }

    fn add_relation(&mut self, relation: &Relation) -> PyResult<()> {
        // The Rust Graph API currently exposes `add_relation_type`; keep the same call for parity.
        self.inner
            .add_relation_type(relation.clone().into_inner())
            .map_err(|e| PyValueError::new_err(format!("Add relation error: {}", e)))
    }

    fn entity_count(&self) -> usize {
        self.inner.entity_count()
    }

    fn instance_count(&self) -> usize {
        self.inner.instance_count()
    }

    fn resource_count(&self) -> usize {
        self.inner.resource_count()
    }

    fn flow_count(&self) -> usize {
        self.inner.flow_count()
    }

    fn pattern_count(&self) -> usize {
        self.inner.pattern_count()
    }

    fn role_count(&self) -> usize {
        self.inner.role_count()
    }

    fn relation_count(&self) -> usize {
        self.inner.relation_count()
    }

    fn has_entity(&self, id: String) -> PyResult<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_entity(&cid))
    }

    fn has_resource(&self, id: String) -> PyResult<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_resource(&cid))
    }

    fn has_flow(&self, id: String) -> PyResult<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_flow(&cid))
    }

    fn has_instance(&self, id: String) -> PyResult<bool> {
        let cid = parse_concept_id(&id)?;
        Ok(self.inner.has_instance(&cid))
    }

    fn get_entity(&self, id: String) -> PyResult<Option<Entity>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_entity(&cid)
            .map(|e| Entity::from_rust(e.clone())))
    }

    fn get_resource(&self, id: String) -> PyResult<Option<Resource>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_resource(&cid)
            .map(|r| Resource::from_rust(r.clone())))
    }

    fn get_flow(&self, id: String) -> PyResult<Option<Flow>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_flow(&cid)
            .map(|f| Flow::from_rust(f.clone())))
    }

    fn get_instance(&self, id: String) -> PyResult<Option<ResourceInstance>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_instance(&cid)
            .map(|i| ResourceInstance::from_rust(i.clone())))
    }

    fn find_role_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_role_by_name(&name)
            .map(|uuid| uuid.to_string())
    }

    fn find_entity_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_entity_by_name(&name)
            .map(|uuid| uuid.to_string())
    }

    fn find_resource_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_resource_by_name(&name)
            .map(|uuid| uuid.to_string())
    }

    fn flows_from(&self, entity_id: String) -> PyResult<Vec<Flow>> {
        let cid = parse_concept_id(&entity_id)?;
        Ok(self
            .inner
            .flows_from(&cid)
            .into_iter()
            .map(|f| Flow::from_rust(f.clone()))
            .collect())
    }

    fn flows_to(&self, entity_id: String) -> PyResult<Vec<Flow>> {
        let cid = parse_concept_id(&entity_id)?;
        Ok(self
            .inner
            .flows_to(&cid)
            .into_iter()
            .map(|f| Flow::from_rust(f.clone()))
            .collect())
    }

    fn all_entities(&self) -> Vec<Entity> {
        self.inner
            .all_entities()
            .into_iter()
            .map(|e| Entity::from_rust(e.clone()))
            .collect()
    }

    fn all_resources(&self) -> Vec<Resource> {
        self.inner
            .all_resources()
            .into_iter()
            .map(|r| Resource::from_rust(r.clone()))
            .collect()
    }

    fn all_flows(&self) -> Vec<Flow> {
        self.inner
            .all_flows()
            .into_iter()
            .map(|f| Flow::from_rust(f.clone()))
            .collect()
    }

    fn all_instances(&self) -> Vec<ResourceInstance> {
        self.inner
            .all_instances()
            .into_iter()
            .map(|i| ResourceInstance::from_rust(i.clone()))
            .collect()
    }

    fn all_roles(&self) -> Vec<Role> {
        self.inner
            .all_roles()
            .into_iter()
            .map(|role| Role::from_rust(role.clone()))
            .collect()
    }

    fn all_relations(&self) -> Vec<Relation> {
        self.inner
            .all_relations()
            .into_iter()
            .map(|relation| Relation::from_rust(relation.clone()))
            .collect()
    }

    #[staticmethod]
    fn parse(source: String) -> PyResult<Self> {
        let graph = parser::parse_to_graph(&source)
            .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;

        Ok(Self { inner: graph })
    }

    fn export_calm(&self) -> PyResult<String> {
        crate::calm::export(&self.inner)
            .and_then(|value| {
                serde_json::to_string_pretty(&value)
                    .map_err(|e| format!("Serialization error: {}", e))
            })
            .map_err(PyValueError::new_err)
    }

    #[staticmethod]
    fn import_calm(calm_json: String) -> PyResult<Self> {
        let value: serde_json::Value = serde_json::from_str(&calm_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

        let graph = crate::calm::import(value)
            .map_err(|e| PyValueError::new_err(format!("Import error: {}", e)))?;

        Ok(Self { inner: graph })
    }

    /// Export the graph to Protobuf .proto text format.
    ///
    /// Args:
    ///     package: The Protobuf package name (e.g., "com.example.api")
    ///     namespace: Optional namespace filter (empty string = all namespaces)
    ///     projection_name: Optional name for the projection (used in comments)
    ///     include_governance: Whether to include governance messages (PolicyViolation, MetricEvent)
    ///     include_services: Whether to generate gRPC service definitions from Flow patterns
    ///
    /// Returns:
    ///     The generated .proto file content as a string
    #[pyo3(signature = (package, namespace = "", projection_name = "", include_governance = false, include_services = false))]
    fn export_protobuf(
        &self,
        package: String,
        namespace: &str,
        projection_name: &str,
        include_governance: bool,
        include_services: bool,
    ) -> String {
        let proto = crate::projection::ProtobufEngine::project_with_full_options(
            &self.inner,
            namespace,
            &package,
            projection_name,
            include_governance,
            include_services,
        );
        proto.to_proto_string()
    }

    fn add_policy(&mut self, policy_json: String) -> PyResult<()> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid Policy JSON: {}", e)))?;
        self.inner
            .add_policy(policy)
            .map_err(|e| PyValueError::new_err(format!("Add policy error: {}", e)))
    }

    fn add_association(&mut self, owner: String, owned: String, rel_type: String) -> PyResult<()> {
        let owner_id = parse_concept_id(&owner)?;
        let owned_id = parse_concept_id(&owned)?;
        self.inner
            .add_association(&owner_id, &owned_id, &rel_type)
            .map_err(|e| PyValueError::new_err(format!("Add association error: {}", e)))
    }

    fn evaluate_policy(&self, policy_json: String) -> PyResult<super::policy::EvaluationResult> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid Policy JSON: {}", e)))?;
        let result = policy
            .evaluate(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Policy evaluation error: {}", e)))?;
        Ok(result.into())
    }

    /// Set the evaluation mode for policy evaluation.
    /// When `use_three_valued_logic` is True, policies will use three-valued logic (True, False, NULL).
    /// When False, policies will use strict boolean logic (True, False).
    fn set_evaluation_mode(&mut self, use_three_valued_logic: bool) {
        self.inner.set_evaluation_mode(use_three_valued_logic);
    }

    /// Get the current evaluation mode.
    /// Returns True if three-valued logic is enabled, False otherwise.
    fn use_three_valued_logic(&self) -> bool {
        self.inner.use_three_valued_logic()
    }

    fn __repr__(&self) -> String {
        format!(
            "Graph(entities={}, resources={}, flows={}, instances={})",
            self.inner.entity_count(),
            self.inner.resource_count(),
            self.inner.flow_count(),
            self.inner.instance_count()
        )
    }
}
