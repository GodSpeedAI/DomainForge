#![allow(clippy::new_without_default, clippy::inherent_to_string)]

use crate::graph::Graph as RustGraph;
use crate::parser;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::str::FromStr;
use uuid::Uuid;

use super::primitives::{Entity, Flow, Instance, Resource};

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
    pub fn add_instance(&mut self, instance: &Instance) -> Result<()> {
        self.inner
            .add_instance(instance.inner_ref().clone())
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
    pub fn get_instance(&self, id: String) -> Result<Option<Instance>> {
        let cid = parse_concept_id(&id)?;
        Ok(self
            .inner
            .get_instance(&cid)
            .map(|i| Instance::from_rust(i.clone())))
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
    pub fn all_instances(&self) -> Vec<Instance> {
        self.inner
            .all_instances()
            .into_iter()
            .map(|i| Instance::from_rust(i.clone()))
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

    #[napi]
    pub fn add_policy(&mut self, policy: &crate::policy::Policy) -> Result<()> {
        self.inner
            .add_policy(policy.clone())
            .map_err(Error::from_reason)
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
