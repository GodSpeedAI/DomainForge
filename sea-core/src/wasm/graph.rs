#![allow(clippy::new_without_default)]

use crate::graph::Graph as RustGraph;
use crate::wasm::primitives::{Entity, Flow, Instance, Resource};
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Graph {
    inner: RustGraph,
}

#[wasm_bindgen]
impl Graph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RustGraph::new(),
        }
    }

    #[wasm_bindgen(js_name = parse)]
    pub fn parse(source: String) -> Result<Graph, JsValue> {
        // parse_to_graph returns a Graph; parser::parse returns an AST
        let graph = crate::parser::parse_to_graph(&source)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        Ok(Self { inner: graph })
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[wasm_bindgen(js_name = addEntity)]
    pub fn add_entity(&mut self, entity: &Entity) -> Result<(), JsValue> {
        self.inner
            .add_entity(entity.inner().clone())
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = hasEntity)]
    pub fn has_entity(&self, id: String) -> Result<bool, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self.inner.has_entity(&cid))
    }

    #[wasm_bindgen(js_name = getEntity)]
    pub fn get_entity(&self, id: String) -> Result<Option<Entity>, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self
            .inner
            .get_entity(&cid)
            .map(|e| Entity::from_inner(e.clone())))
    }

    #[wasm_bindgen(js_name = removeEntity)]
    pub fn remove_entity(&mut self, id: String) -> Result<Entity, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let entity = self
            .inner
            .remove_entity(&cid)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Entity::from_inner(entity))
    }

    #[wasm_bindgen(js_name = findEntityByName)]
    pub fn find_entity_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_entity_by_name(&name)
            .map(|id| id.to_string())
    }

    #[wasm_bindgen(js_name = entityCount)]
    pub fn entity_count(&self) -> usize {
        self.inner.entity_count()
    }

    #[wasm_bindgen(js_name = allEntities)]
    pub fn all_entities(&self) -> Result<JsValue, JsValue> {
        let entities: Vec<Entity> = self
            .inner
            .all_entities()
            .into_iter()
            .map(|e| Entity::from_inner(e.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&entities)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = addResource)]
    pub fn add_resource(&mut self, resource: &Resource) -> Result<(), JsValue> {
        self.inner
            .add_resource(resource.inner().clone())
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = hasResource)]
    pub fn has_resource(&self, id: String) -> Result<bool, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self.inner.has_resource(&cid))
    }

    #[wasm_bindgen(js_name = getResource)]
    pub fn get_resource(&self, id: String) -> Result<Option<Resource>, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self
            .inner
            .get_resource(&cid)
            .map(|r| Resource::from_inner(r.clone())))
    }

    #[wasm_bindgen(js_name = removeResource)]
    pub fn remove_resource(&mut self, id: String) -> Result<Resource, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let resource = self
            .inner
            .remove_resource(&cid)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Resource::from_inner(resource))
    }

    #[wasm_bindgen(js_name = findResourceByName)]
    pub fn find_resource_by_name(&self, name: String) -> Option<String> {
        self.inner
            .find_resource_by_name(&name)
            .map(|id| id.to_string())
    }

    #[wasm_bindgen(js_name = resourceCount)]
    pub fn resource_count(&self) -> usize {
        self.inner.resource_count()
    }

    #[wasm_bindgen(js_name = allResources)]
    pub fn all_resources(&self) -> Result<JsValue, JsValue> {
        let resources: Vec<Resource> = self
            .inner
            .all_resources()
            .into_iter()
            .map(|r| Resource::from_inner(r.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&resources)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = addFlow)]
    pub fn add_flow(&mut self, flow: &Flow) -> Result<(), JsValue> {
        self.inner
            .add_flow(flow.inner().clone())
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = hasFlow)]
    pub fn has_flow(&self, id: String) -> Result<bool, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self.inner.has_flow(&cid))
    }

    #[wasm_bindgen(js_name = getFlow)]
    pub fn get_flow(&self, id: String) -> Result<Option<Flow>, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self
            .inner
            .get_flow(&cid)
            .map(|f| Flow::from_inner(f.clone())))
    }

    #[wasm_bindgen(js_name = removeFlow)]
    pub fn remove_flow(&mut self, id: String) -> Result<Flow, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let flow = self
            .inner
            .remove_flow(&cid)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Flow::from_inner(flow))
    }

    #[wasm_bindgen(js_name = flowCount)]
    pub fn flow_count(&self) -> usize {
        self.inner.flow_count()
    }

    #[wasm_bindgen(js_name = allFlows)]
    pub fn all_flows(&self) -> Result<JsValue, JsValue> {
        let flows: Vec<Flow> = self
            .inner
            .all_flows()
            .into_iter()
            .map(|f| Flow::from_inner(f.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&flows)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = addInstance)]
    pub fn add_instance(&mut self, instance: &Instance) -> Result<(), JsValue> {
        self.inner
            .add_instance(instance.inner().clone())
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = addPolicy)]
    pub fn add_policy(&mut self, policy: &crate::policy::Policy) -> Result<(), JsValue> {
        self.inner
            .add_policy(policy.clone())
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = addAssociation)]
    pub fn add_association(
        &mut self,
        owner: String,
        owned: String,
        rel_type: String,
    ) -> Result<(), JsValue> {
        let owner_uuid = Uuid::from_str(&owner)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let owner_cid = crate::ConceptId::from(owner_uuid);
        let owned_uuid = Uuid::from_str(&owned)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let owned_cid = crate::ConceptId::from(owned_uuid);
        self.inner
            .add_association(&owner_cid, &owned_cid, &rel_type)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = hasInstance)]
    pub fn has_instance(&self, id: String) -> Result<bool, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self.inner.has_instance(&cid))
    }

    #[wasm_bindgen(js_name = getInstance)]
    pub fn get_instance(&self, id: String) -> Result<Option<Instance>, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self
            .inner
            .get_instance(&cid)
            .map(|i| Instance::from_inner(i.clone())))
    }

    #[wasm_bindgen(js_name = removeInstance)]
    pub fn remove_instance(&mut self, id: String) -> Result<Instance, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let instance = self
            .inner
            .remove_instance(&cid)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Instance::from_inner(instance))
    }

    #[wasm_bindgen(js_name = instanceCount)]
    pub fn instance_count(&self) -> usize {
        self.inner.instance_count()
    }

    #[wasm_bindgen(js_name = allInstances)]
    pub fn all_instances(&self) -> Result<JsValue, JsValue> {
        let instances: Vec<Instance> = self
            .inner
            .all_instances()
            .into_iter()
            .map(|i| Instance::from_inner(i.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&instances)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = flowsFrom)]
    pub fn flows_from(&self, entity_id: String) -> Result<JsValue, JsValue> {
        let uuid = Uuid::from_str(&entity_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let flows: Vec<Flow> = self
            .inner
            .flows_from(&cid)
            .into_iter()
            .map(|f| Flow::from_inner(f.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&flows)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = flowsTo)]
    pub fn flows_to(&self, entity_id: String) -> Result<JsValue, JsValue> {
        let uuid = Uuid::from_str(&entity_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let flows: Vec<Flow> = self
            .inner
            .flows_to(&cid)
            .into_iter()
            .map(|f| Flow::from_inner(f.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&flows)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = upstreamEntities)]
    pub fn upstream_entities(&self, entity_id: String) -> Result<JsValue, JsValue> {
        let uuid = Uuid::from_str(&entity_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let entities: Vec<Entity> = self
            .inner
            .upstream_entities(&cid)
            .into_iter()
            .map(|e| Entity::from_inner(e.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&entities)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = downstreamEntities)]
    pub fn downstream_entities(&self, entity_id: String) -> Result<JsValue, JsValue> {
        let uuid = Uuid::from_str(&entity_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let entities: Vec<Entity> = self
            .inner
            .downstream_entities(&cid)
            .into_iter()
            .map(|e| Entity::from_inner(e.clone()))
            .collect();
        serde_wasm_bindgen::to_value(&entities)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    #[wasm_bindgen(js_name = exportCalm)]
    pub fn export_calm(&self) -> Result<String, JsValue> {
        crate::calm::export(&self.inner)
            .and_then(|value| {
                serde_json::to_string_pretty(&value)
                    .map_err(|e| format!("Serialization error: {}", e))
            })
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = importCalm)]
    pub fn import_calm(calm_json: String) -> Result<Graph, JsValue> {
        let value: serde_json::Value = serde_json::from_str(&calm_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

        let graph = crate::calm::import(value)
            .map_err(|e| JsValue::from_str(&format!("Import error: {}", e)))?;

        Ok(Self { inner: graph })
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}
