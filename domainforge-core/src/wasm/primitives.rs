use crate::primitives::{
    entity::Entity as RustEntity, flow::Flow as RustFlow, instance::Instance as RustInstance,
    resource::Resource as RustResource,
};
use crate::units::unit_from_string;
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

macro_rules! impl_wasm_common {
    ($name:ident, $inner:ident) => {
        #[wasm_bindgen]
        impl $name {
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
                    .and_then(|v| serde_wasm_bindgen::to_value(v).inspect_err(|_e| {}).ok())
                    .unwrap_or(JsValue::NULL)
            }

            #[wasm_bindgen(js_name = toJSON)]
            pub fn to_json(&self) -> Result<JsValue, JsValue> {
                serde_wasm_bindgen::to_value(&self.inner)
                    .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
            }
        }

        impl $name {
            pub fn inner(&self) -> &$inner {
                &self.inner
            }

            pub fn from_inner(inner: $inner) -> Self {
                Self { inner }
            }

            pub fn into_inner(self) -> $inner {
                self.inner
            }
        }
    };
}

#[wasm_bindgen]
#[derive(Serialize)]
pub struct Entity {
    inner: RustEntity,
}

#[wasm_bindgen]
impl Entity {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, namespace: Option<String>) -> Self {
        let entity = match namespace {
            Some(ns) => RustEntity::new_with_namespace(name, ns),
            None => RustEntity::new_with_namespace(name, "default".to_string()),
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
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }
}

impl_wasm_common!(Entity, RustEntity);

#[wasm_bindgen]
#[derive(Serialize)]
pub struct Resource {
    inner: RustResource,
}

#[wasm_bindgen]
impl Resource {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, unit: String, namespace: Option<String>) -> Self {
        let unit_obj = unit_from_string(unit);
        let resource = match namespace {
            Some(ns) => RustResource::new_with_namespace(name, unit_obj, ns),
            None => RustResource::new_with_namespace(name, unit_obj, "default".to_string()),
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
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }
}

impl_wasm_common!(Resource, RustResource);

#[wasm_bindgen]
#[derive(Serialize)]
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
        namespace: Option<String>,
    ) -> Result<Flow, JsValue> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid resource_id UUID: {}", e)))?;
        let from_uuid = Uuid::from_str(&from_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid from_id UUID: {}", e)))?;
        let to_uuid = Uuid::from_str(&to_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid to_id UUID: {}", e)))?;
        let quantity_decimal = Decimal::from_str(&quantity)
            .map_err(|e| JsValue::from_str(&format!("Invalid quantity: {}", e)))?;

        let ns = match namespace {
            Some(n) => n,
            None => "default".to_string(),
        };

        let flow = RustFlow::new_with_namespace(
            crate::ConceptId::from(resource_uuid),
            crate::ConceptId::from(from_uuid),
            crate::ConceptId::from(to_uuid),
            quantity_decimal,
            ns,
        );

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
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }
}

impl_wasm_common!(Flow, RustFlow);

#[wasm_bindgen]
#[derive(Serialize)]
pub struct Instance {
    inner: RustInstance,
}

#[wasm_bindgen]
impl Instance {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, entity_type: String, namespace: Option<String>) -> Instance {
        let ns = namespace.unwrap_or_else(|| "default".to_string());
        let instance = RustInstance::new_with_namespace(name, entity_type, ns);
        Self { inner: instance }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[wasm_bindgen(getter, js_name = entityType)]
    pub fn entity_type(&self) -> String {
        self.inner.entity_type().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    #[wasm_bindgen(js_name = setField)]
    pub fn set_field(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert value: {}", e)))?;
        self.inner.set_field(key, json_value);
        Ok(())
    }

    #[wasm_bindgen(js_name = getField)]
    pub fn get_field(&self, key: String) -> JsValue {
        self.inner
            .get_field(&key)
            .and_then(|v| serde_wasm_bindgen::to_value(v).inspect_err(|_e| {}).ok())
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
