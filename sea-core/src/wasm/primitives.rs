use crate::primitives::{
    entity::Entity as RustEntity, flow::Flow as RustFlow, instance::Instance as RustInstance,
    resource::Resource as RustResource,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct Entity {
    inner: RustEntity,
}

#[wasm_bindgen]
impl Entity {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, namespace: Option<String>) -> Self {
        let entity = match namespace {
            Some(ns) => RustEntity::new_with_namespace(name, ns),
            None => RustEntity::new(name),
        };
        Self { inner: entity }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn namespace(&self) -> Option<String> {
        self.inner.namespace().map(|s| s.to_string())
    }

    #[wasm_bindgen(js_name = setAttribute)]
    pub fn set_attribute(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert value: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[wasm_bindgen(js_name = getAttribute)]
    pub fn get_attribute(&self, key: String) -> JsValue {
        self.inner
            .get_attribute(&key)
            .and_then(|v| {
                serde_wasm_bindgen::to_value(v)
                    .map_err(|e| {
                        console::error_1(&JsValue::from_str(&format!(
                            "Failed to serialize attribute '{}' to JS value: {}",
                            key, e
                        )));
                        e
                    })
                    .ok()
            })
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

impl Entity {
    pub fn inner(&self) -> &RustEntity {
        &self.inner
    }

    pub fn from_inner(inner: RustEntity) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustEntity {
        self.inner
    }
}

#[wasm_bindgen]
pub struct Resource {
    inner: RustResource,
}

#[wasm_bindgen]
impl Resource {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, unit: String, namespace: Option<String>) -> Self {
        let resource = match namespace {
            Some(ns) => RustResource::new_with_namespace(name, unit, ns),
            None => RustResource::new(name, unit),
        };
        Self { inner: resource }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn unit(&self) -> String {
        self.inner.unit().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn namespace(&self) -> Option<String> {
        self.inner.namespace().map(|s| s.to_string())
    }

    #[wasm_bindgen(js_name = setAttribute)]
    pub fn set_attribute(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert value: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[wasm_bindgen(js_name = getAttribute)]
    pub fn get_attribute(&self, key: String) -> JsValue {
        self.inner
            .get_attribute(&key)
            .and_then(|v| {
                serde_wasm_bindgen::to_value(v)
                    .map_err(|e| {
                        console::error_1(&JsValue::from_str(&format!(
                            "Failed to serialize attribute '{}' to JS value: {}",
                            key, e
                        )));
                        e
                    })
                    .ok()
            })
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

impl Resource {
    pub fn inner(&self) -> &RustResource {
        &self.inner
    }

    pub fn from_inner(inner: RustResource) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustResource {
        self.inner
    }
}

#[wasm_bindgen]
pub struct Flow {
    inner: RustFlow,
}

#[wasm_bindgen]
impl Flow {
    #[wasm_bindgen(constructor)]
    pub fn new(
        resource_id: String,
        from_id: String,
        to_id: String,
        quantity: String,
    ) -> Result<Flow, JsValue> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid resource_id UUID: {}", e)))?;
        let from_uuid = Uuid::from_str(&from_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid from_id UUID: {}", e)))?;
        let to_uuid = Uuid::from_str(&to_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid to_id UUID: {}", e)))?;
        let quantity_decimal = Decimal::from_str(&quantity)
            .map_err(|e| JsValue::from_str(&format!("Invalid quantity: {}", e)))?;

        let flow = RustFlow::new(resource_uuid, from_uuid, to_uuid, quantity_decimal);

        Ok(Self { inner: flow })
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[wasm_bindgen(getter, js_name = resourceId)]
    pub fn resource_id(&self) -> String {
        self.inner.resource_id().to_string()
    }

    #[wasm_bindgen(getter, js_name = fromId)]
    pub fn from_id(&self) -> String {
        self.inner.from_id().to_string()
    }

    #[wasm_bindgen(getter, js_name = toId)]
    pub fn to_id(&self) -> String {
        self.inner.to_id().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn quantity(&self) -> String {
        self.inner.quantity().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn namespace(&self) -> Option<String> {
        self.inner.namespace().map(|s| s.to_string())
    }

    #[wasm_bindgen(js_name = setAttribute)]
    pub fn set_attribute(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert value: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[wasm_bindgen(js_name = getAttribute)]
    pub fn get_attribute(&self, key: String) -> JsValue {
        self.inner
            .get_attribute(&key)
            .and_then(|v| {
                serde_wasm_bindgen::to_value(v)
                    .map_err(|e| {
                        console::error_1(&JsValue::from_str(&format!(
                            "Failed to serialize attribute '{}' to JS value: {}",
                            key, e
                        )));
                        e
                    })
                    .ok()
            })
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

impl Flow {
    pub fn inner(&self) -> &RustFlow {
        &self.inner
    }

    pub fn from_inner(inner: RustFlow) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustFlow {
        self.inner
    }
}

#[wasm_bindgen]
pub struct Instance {
    inner: RustInstance,
}

#[wasm_bindgen]
impl Instance {
    #[wasm_bindgen(constructor)]
    pub fn new(
        resource_id: String,
        entity_id: String,
        namespace: Option<String>,
    ) -> Result<Instance, JsValue> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid resource_id UUID: {}", e)))?;
        let entity_uuid = Uuid::from_str(&entity_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid entity_id UUID: {}", e)))?;

        let instance = match namespace {
            Some(ns) => RustInstance::new_with_namespace(resource_uuid, entity_uuid, ns),
            None => RustInstance::new(resource_uuid, entity_uuid),
        };

        Ok(Self { inner: instance })
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[wasm_bindgen(getter, js_name = entityId)]
    pub fn entity_id(&self) -> String {
        self.inner.entity_id().to_string()
    }

    #[wasm_bindgen(getter, js_name = resourceId)]
    pub fn resource_id(&self) -> String {
        self.inner.resource_id().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn namespace(&self) -> Option<String> {
        self.inner.namespace().map(|s| s.to_string())
    }

    #[wasm_bindgen(js_name = setAttribute)]
    pub fn set_attribute(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert value: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[wasm_bindgen(js_name = getAttribute)]
    pub fn get_attribute(&self, key: String) -> JsValue {
        self.inner
            .get_attribute(&key)
            .and_then(|v| {
                serde_wasm_bindgen::to_value(v)
                    .map_err(|e| {
                        console::error_1(&JsValue::from_str(&format!(
                            "Failed to serialize attribute '{}' to JS value: {}",
                            key, e
                        )));
                        e
                    })
                    .ok()
            })
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

impl Instance {
    pub fn inner(&self) -> &RustInstance {
        &self.inner
    }

    pub fn from_inner(inner: RustInstance) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustInstance {
        self.inner
    }
}
