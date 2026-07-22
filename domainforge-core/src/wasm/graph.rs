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

    /// Parse SEA DSL source and return AST as JSON string.
    ///
    /// This returns the Abstract Syntax Tree (AST) representation of the source,
    /// which preserves the exact structure and line numbers from the source file.
    /// Use this for tools that need to work with the raw parsed structure.
    ///
    /// For a semantic graph representation, use `Graph.parse()` instead.
    ///
    /// @param source - SEA DSL source code string
    /// @returns JSON string conforming to ast-v3.schema.json
    #[wasm_bindgen(js_name = parseToAstJson)]
    pub fn parse_to_ast_json(source: String) -> Result<String, JsValue> {
        let internal_ast = crate::parser::parse(&source)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        let schema_ast: crate::parser::ast_schema::Ast = internal_ast.into();
        serde_json::to_string_pretty(&schema_ast)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Resolve an in-memory source map into canonical Application Contract
    /// document JSON (ADR-013 Milestone 0).
    #[wasm_bindgen(js_name = resolveApplicationContractJson)]
    pub fn resolve_application_contract_json(
        entry_logical_path: String,
        sources_json: String,
    ) -> Result<String, JsValue> {
        crate::application::resolve_application_contract_json(&entry_logical_path, &sources_json)
            .map_err(|diags| {
                JsValue::from_str(&serde_json::to_string(&diags).unwrap_or_else(|e| e.to_string()))
            })
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

    /// Compile the graph into AI Learning Projection artifacts.
    ///
    /// @param recipeJson - Optional JSON recipe (defaults enable all projections)
    /// @param authorityConfigJson - Optional AuthorityEnvironmentConfig JSON; required for resolver-grounded families
    /// @param modelRef - Optional provenance label for the source model
    /// @param seed - Optional split/sampling seed (overrides the recipe seed)
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths (the `--format ai-learning` layout) to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportAiLearning)]
    pub fn export_ai_learning(
        &self,
        recipe_json: Option<String>,
        authority_config_json: Option<String>,
        model_ref: Option<String>,
        seed: Option<u32>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::ai_learning::project_ai_learning_in_memory(
            &self.inner,
            recipe_json.as_deref(),
            authority_config_json.as_deref(),
            model_ref.as_deref().unwrap_or("<in-memory>"),
            seed.map(u64::from),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit a Lean 4 formal verification package (the `--format lean` layout).
    ///
    /// @param modelRef - Optional provenance label for the source model
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportLean)]
    pub fn export_lean(
        &self,
        model_ref: Option<String>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::lean::project_lean_in_memory(
            &self.inner,
            model_ref.as_deref().unwrap_or("<in-memory>"),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit an RDF/OWL dataset (the `--format rdf` layout).
    ///
    /// @param modelRef - Optional provenance label for the source model
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @param baseIri - Optional base IRI the `sea:` prefix expands to
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportRdfProjection)]
    pub fn export_rdf_projection(
        &self,
        model_ref: Option<String>,
        created_at: Option<String>,
        base_iri: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::rdf::project_rdf_in_memory(
            &self.inner,
            model_ref.as_deref().unwrap_or("<in-memory>"),
            created_at,
            base_iri,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit a BPMN 2.0 process (the `--format bpmn` layout: model.bpmn).
    ///
    /// @param modelRef - Optional provenance label for the source model
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportBpmn)]
    pub fn export_bpmn(
        &self,
        model_ref: Option<String>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::bpmn::project_bpmn_in_memory(
            &self.inner,
            model_ref.as_deref().unwrap_or("<in-memory>"),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit a CMMN 1.1 case (the `--format cmmn` layout: model.cmmn).
    ///
    /// @param modelRef - Optional provenance label for the source model
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportCmmn)]
    pub fn export_cmmn(
        &self,
        model_ref: Option<String>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::cmmn::project_cmmn_in_memory(
            &self.inner,
            model_ref.as_deref().unwrap_or("<in-memory>"),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit an ArchiMate 3.0 Model Exchange File (the `--format archimate`
    /// layout: model.xml).
    ///
    /// @param modelRef - Optional provenance label for the source model
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportArchimate)]
    pub fn export_archimate(
        &self,
        model_ref: Option<String>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::archimate::project_archimate_in_memory(
            &self.inner,
            model_ref.as_deref().unwrap_or("<in-memory>"),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit an OpenTelemetry SemConv projection (the `--format otel-semconv`
    /// layout: registry/telemetry.yaml + constants/attributes.{rs,py,ts}).
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportOtelSemconv)]
    pub fn export_otel_semconv(
        &self,
        model_ref: Option<String>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::otel::project_otel_semconv_in_memory(
            &self.inner,
            model_ref.as_deref().unwrap_or("<in-memory>"),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit a BAML capability (the `--format baml` layout: baml_src/*.baml +
    /// README.md). Requires an authority environment (passed as JSON): the
    /// function and its tests are resolver-grounded.
    ///
    /// @param recipeJson - Optional JSON recipe (its `baml` section configures naming)
    /// @param authorityConfigJson - AuthorityEnvironmentConfig JSON (required)
    /// @param modelRef - Optional provenance label for the source model
    /// @param seed - Optional seed override
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportBaml)]
    pub fn export_baml(
        &self,
        recipe_json: Option<String>,
        authority_config_json: Option<String>,
        model_ref: Option<String>,
        seed: Option<u32>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::baml::project_baml_in_memory(
            &self.inner,
            recipe_json.as_deref(),
            authority_config_json.as_deref(),
            model_ref.as_deref().unwrap_or("<in-memory>"),
            seed.map(u64::from),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit a DSPy optimization program (the `--format dspy` layout:
    /// program.py, metric.py, optimize.py, dspy.config.json, README.md).
    /// Requires an authority environment (passed as JSON): the signature and
    /// its examples are resolver-grounded. Train/dev examples are referenced
    /// from the ai-learning LLM dataset, never copied.
    ///
    /// @param recipeJson - Optional JSON recipe (its `dspy` section configures naming/optimizer)
    /// @param authorityConfigJson - AuthorityEnvironmentConfig JSON (required)
    /// @param modelRef - Optional provenance label for the source model
    /// @param seed - Optional seed override
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportDspy)]
    pub fn export_dspy(
        &self,
        recipe_json: Option<String>,
        authority_config_json: Option<String>,
        model_ref: Option<String>,
        seed: Option<u32>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::dspy::project_dspy_in_memory(
            &self.inner,
            recipe_json.as_deref(),
            authority_config_json.as_deref(),
            model_ref.as_deref().unwrap_or("<in-memory>"),
            seed.map(u64::from),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Emit a ZenML learning pipeline (the `--format zenml` layout:
    /// pipeline.py, steps.py, run.py, requirements.txt, zenml.config.json,
    /// README.md). Requires an authority environment (passed as JSON): the
    /// pipeline and its labeled examples are resolver-grounded. Train/dev
    /// examples are referenced from the ai-learning LLM dataset, never copied.
    ///
    /// @param recipeJson - Optional JSON recipe (its `zenml` section configures pipeline/model/gate)
    /// @param authorityConfigJson - AuthorityEnvironmentConfig JSON (required)
    /// @param modelRef - Optional provenance label for the source model
    /// @param seed - Optional seed override
    /// @param createdAt - Optional fixed RFC3339 timestamp for reproducible output
    /// @returns JSON object mapping relative artifact paths to file contents
    #[cfg(feature = "wasm-projections")]
    #[wasm_bindgen(js_name = exportZenml)]
    pub fn export_zenml(
        &self,
        recipe_json: Option<String>,
        authority_config_json: Option<String>,
        model_ref: Option<String>,
        seed: Option<u32>,
        created_at: Option<String>,
    ) -> Result<String, JsValue> {
        let artifacts = crate::projection::zenml::project_zenml_in_memory(
            &self.inner,
            recipe_json.as_deref(),
            authority_config_json.as_deref(),
            model_ref.as_deref().unwrap_or("<in-memory>"),
            seed.map(u64::from),
            created_at,
        )
        .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&artifacts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
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
    #[cfg(feature = "wasm-projections")]
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
