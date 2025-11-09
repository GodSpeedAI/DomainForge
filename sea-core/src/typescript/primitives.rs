use crate::primitives::{
    Entity as RustEntity, Flow as RustFlow, Instance as RustInstance, Resource as RustResource,
};

use crate::units::unit_from_string;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[napi]
pub struct Entity {
    inner: RustEntity,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Entity {
    #[napi(constructor)]
    pub fn new(name: String, namespace: Option<String>) -> Self {
        let inner = match namespace {
            Some(ns) => RustEntity::new_with_namespace(name, ns),
            None => RustEntity::new(name),
        };
        Self { inner }
    }

    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi(getter)]
    pub fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    #[napi]
    pub fn set_attribute(&mut self, key: String, value_json: String) -> Result<()> {
        let json_value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[napi]
    pub fn get_attribute(&self, key: String) -> Option<String> {
        self.inner
            .get_attribute(&key)
            .and_then(|v| serde_json::to_string(v).ok())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Entity(id='{}', name='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.namespace()
        )
    }
}

impl Entity {
    pub fn from_rust(inner: RustEntity) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustEntity {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustEntity {
        &self.inner
    }
}

#[napi]
pub struct Resource {
    inner: RustResource,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Resource {
    #[napi(constructor)]
    pub fn new(name: String, unit: String, namespace: Option<String>) -> Self {
        let unit_obj = unit_from_string(unit);
        let inner = match namespace {
            Some(ns) => RustResource::new_with_namespace(name, unit_obj, ns),
            None => RustResource::new(name, unit_obj),
        };
        Self { inner }
    }

    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi(getter)]
    pub fn unit(&self) -> String {
        self.inner.unit().to_string()
    }

    #[napi(getter)]
    pub fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    #[napi]
    pub fn set_attribute(&mut self, key: String, value_json: String) -> Result<()> {
        let json_value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[napi]
    pub fn get_attribute(&self, key: String) -> Option<String> {
        self.inner
            .get_attribute(&key)
            .and_then(|v| serde_json::to_string(v).ok())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Resource(id='{}', name='{}', unit='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.unit(),
            self.inner.namespace()
        )
    }
}

impl Resource {
    pub fn from_rust(inner: RustResource) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustResource {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustResource {
        &self.inner
    }
}

#[napi]
pub struct Flow {
    inner: RustFlow,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Flow {
    #[napi(constructor)]
    pub fn new(resource_id: String, from_id: String, to_id: String, quantity: f64) -> Result<Self> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| Error::from_reason(format!("Invalid resource_id UUID: {}", e)))?;
        let from_uuid = Uuid::from_str(&from_id)
            .map_err(|e| Error::from_reason(format!("Invalid from_id UUID: {}", e)))?;
        let to_uuid = Uuid::from_str(&to_id)
            .map_err(|e| Error::from_reason(format!("Invalid to_id UUID: {}", e)))?;
        let decimal_quantity = Decimal::from_f64(quantity)
            .ok_or_else(|| Error::from_reason("Invalid quantity value"))?;

        let inner = RustFlow::new(
            crate::ConceptId::from(resource_uuid),
            crate::ConceptId::from(from_uuid),
            crate::ConceptId::from(to_uuid),
            decimal_quantity,
        );
        Ok(Self { inner })
    }

    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[napi(getter)]
    pub fn resource_id(&self) -> String {
        self.inner.resource_id().to_string()
    }

    #[napi(getter)]
    pub fn from_id(&self) -> String {
        self.inner.from_id().to_string()
    }

    #[napi(getter)]
    pub fn to_id(&self) -> String {
        self.inner.to_id().to_string()
    }

    #[napi(getter)]
    pub fn quantity(&self) -> f64 {
        match self.inner.quantity().to_f64() {
            Some(value) => {
                if value.is_finite() {
                    value
                } else if value.is_infinite() && value.is_sign_positive() {
                    f64::INFINITY
                } else if value.is_infinite() && value.is_sign_negative() {
                    f64::NEG_INFINITY
                } else {
                    // NaN case
                    0.0
                }
            }
            // Conversion failure - return 0.0 as fallback
            None => 0.0,
        }
    }

    #[napi(getter)]
    pub fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    #[napi]
    pub fn set_attribute(&mut self, key: String, value_json: String) -> Result<()> {
        let json_value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[napi]
    pub fn get_attribute(&self, key: String) -> Option<String> {
        self.inner
            .get_attribute(&key)
            .and_then(|v| serde_json::to_string(v).ok())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Flow(id='{}', resource_id='{}', from_id='{}', to_id='{}', quantity={})",
            self.inner.id(),
            self.inner.resource_id(),
            self.inner.from_id(),
            self.inner.to_id(),
            self.inner.quantity()
        )
    }
}

impl Flow {
    pub fn from_rust(inner: RustFlow) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustFlow {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustFlow {
        &self.inner
    }
}

#[napi]
pub struct Instance {
    inner: RustInstance,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Instance {
    #[napi(constructor)]
    pub fn new(resource_id: String, entity_id: String, namespace: Option<String>) -> Result<Self> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| Error::from_reason(format!("Invalid resource_id UUID: {}", e)))?;
        let entity_uuid = Uuid::from_str(&entity_id)
            .map_err(|e| Error::from_reason(format!("Invalid entity_id UUID: {}", e)))?;

        let inner = match namespace {
            Some(ns) => RustInstance::new_with_namespace(
                crate::ConceptId::from(resource_uuid),
                crate::ConceptId::from(entity_uuid),
                ns,
            ),
            None => RustInstance::new(
                crate::ConceptId::from(resource_uuid),
                crate::ConceptId::from(entity_uuid),
            ),
        };
        Ok(Self { inner })
    }

    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[napi(getter)]
    pub fn resource_id(&self) -> String {
        self.inner.resource_id().to_string()
    }

    #[napi(getter)]
    pub fn entity_id(&self) -> String {
        self.inner.entity_id().to_string()
    }

    #[napi(getter)]
    pub fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    #[napi]
    pub fn set_attribute(&mut self, key: String, value_json: String) -> Result<()> {
        let json_value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    #[napi]
    pub fn get_attribute(&self, key: String) -> Option<String> {
        self.inner
            .get_attribute(&key)
            .and_then(|v| serde_json::to_string(v).ok())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Instance(id='{}', resource_id='{}', entity_id='{}', namespace={:?})",
            self.inner.id(),
            self.inner.resource_id(),
            self.inner.entity_id(),
            self.inner.namespace()
        )
    }
}

impl Instance {
    pub fn from_rust(inner: RustInstance) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustInstance {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustInstance {
        &self.inner
    }
}
