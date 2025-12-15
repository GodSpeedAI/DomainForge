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
            .add_entity_instance(instance.inner().clone())
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = addPolicy)]
    pub fn add_policy(&mut self, policy: &JsValue) -> Result<(), JsValue> {
        // Deserialize policy from JsValue using serde_wasm_bindgen. Policy derives
        // Serialize/Deserialize so this will convert from the JS representation.
        let policy: crate::policy::Policy = serde_wasm_bindgen::from_value(policy.clone())
            .map_err(|e| JsValue::from_str(&format!("Invalid Policy: {}", e)))?;
        self.inner
            .add_policy(policy)
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
        Ok(self.inner.has_entity_instance_by_id(&cid))
    }

    #[wasm_bindgen(js_name = getInstance)]
    pub fn get_instance(&self, id: String) -> Result<Option<Instance>, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        Ok(self
            .inner
            .get_entity_instance_by_id(&cid)
            .map(|i| Instance::from_inner(i.clone())))
    }

    #[wasm_bindgen(js_name = removeInstance)]
    pub fn remove_instance(&mut self, id: String) -> Result<Instance, JsValue> {
        let uuid =
            Uuid::from_str(&id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        let cid = crate::ConceptId::from(uuid);
        let instance = self
            .inner
            .remove_entity_instance_by_id(&cid)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Instance::from_inner(instance))
    }

    #[wasm_bindgen(js_name = instanceCount)]
    pub fn instance_count(&self) -> usize {
        self.inner.entity_instance_count()
    }

    pub fn pattern_count(&self) -> usize {
        self.inner.pattern_count()
    }

    #[wasm_bindgen(js_name = allInstances)]
    pub fn all_instances(&self) -> Result<JsValue, JsValue> {
        let instances: Vec<Instance> = self
            .inner
            .all_entity_instances()
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

    /// Export the graph to Protobuf .proto text format.
    ///
    /// @param package - The Protobuf package name (e.g., "com.example.api")
    /// @param namespace - Optional namespace filter (undefined/null = all namespaces)
    /// @param projectionName - Optional name for the projection (used in comments)
    /// @param includeGovernance - Whether to include governance messages
    /// @param includeServices - Whether to generate gRPC service definitions from Flow patterns
    /// @returns The generated .proto file content as a string
    #[wasm_bindgen(js_name = exportProtobuf)]
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

    #[wasm_bindgen(js_name = evaluatePolicy)]
    pub fn evaluate_policy(
        &self,
        policy_json: String,
    ) -> Result<crate::wasm::policy::EvaluationResult, JsValue> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid Policy JSON: {}", e)))?;

        let result = policy
            .evaluate(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Policy evaluation error: {}", e)))?;

        Ok(result.into())
    }

    /// Set the evaluation mode for policy evaluation.
    /// When `useThreeValuedLogic` is true, policies will use three-valued logic (true, false, null).
    /// When false, policies will use strict boolean logic (true, false).
    #[wasm_bindgen(js_name = setEvaluationMode)]
    pub fn set_evaluation_mode(&mut self, use_three_valued_logic: bool) {
        self.inner.set_evaluation_mode(use_three_valued_logic);
    }

    /// Get the current evaluation mode.
    /// Returns true if three-valued logic is enabled, false otherwise.
    #[wasm_bindgen(js_name = useThreeValuedLogic)]
    pub fn use_three_valued_logic(&self) -> bool {
        self.inner.use_three_valued_logic()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}
