use crate::primitives::{
    Entity as RustEntity, Flow as RustFlow, MappingContract as RustMapping, Metric as RustMetric,
    ProjectionContract as RustProjection, RelationType as RustRelation, Resource as RustResource,
    ResourceInstance as RustResourceInstance, Role as RustRole,
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
            None => RustEntity::new_with_namespace(name, "default".to_string()),
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
            None => RustResource::new_with_namespace(name, unit_obj, "default".to_string()),
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
pub struct ResourceInstance {
    inner: RustResourceInstance,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl ResourceInstance {
    #[napi(constructor)]
    pub fn new(resource_id: String, entity_id: String, namespace: Option<String>) -> Result<Self> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| Error::from_reason(format!("Invalid resource_id UUID: {}", e)))?;
        let entity_uuid = Uuid::from_str(&entity_id)
            .map_err(|e| Error::from_reason(format!("Invalid entity_id UUID: {}", e)))?;

        let inner = match namespace {
            Some(ns) => RustResourceInstance::new_with_namespace(
                crate::ConceptId::from(resource_uuid),
                crate::ConceptId::from(entity_uuid),
                ns,
            ),
            None => RustResourceInstance::new(
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
            "ResourceInstance(id='{}', resource_id='{}', entity_id='{}', namespace={:?})",
            self.inner.id(),
            self.inner.resource_id(),
            self.inner.entity_id(),
            self.inner.namespace()
        )
    }
}

impl ResourceInstance {
    pub fn from_rust(inner: RustResourceInstance) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustResourceInstance {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustResourceInstance {
        &self.inner
    }
}

#[napi]
pub struct Instance {
    inner: crate::primitives::Instance,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Instance {
    #[napi(constructor)]
    pub fn new(name: String, entity_type: String, namespace: Option<String>) -> Self {
        let inner = match namespace {
            Some(ns) => crate::primitives::Instance::new_with_namespace(name, entity_type, ns),
            None => crate::primitives::Instance::new_with_namespace(
                name,
                entity_type,
                "default".to_string(),
            ),
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
    pub fn entity_type(&self) -> String {
        self.inner.entity_type().to_string()
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
    pub fn set_field(&mut self, key: String, value_json: String) -> Result<()> {
        let json_value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        self.inner.set_field(key, json_value);
        Ok(())
    }

    #[napi]
    pub fn get_field(&self, key: String) -> Result<Option<String>> {
        match self.inner.get_field(&key) {
            Some(value) => serde_json::to_string(value).map(Some).map_err(|e| {
                Error::from_reason(format!("Failed to serialize field '{}': {}", key, e))
            }),
            None => Ok(None),
        }
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Instance(id='{}', name='{}', entity_type='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.entity_type(),
            self.inner.namespace()
        )
    }
}

impl Instance {
    pub fn from_rust(inner: crate::primitives::Instance) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> crate::primitives::Instance {
        self.inner
    }

    pub fn inner_ref(&self) -> &crate::primitives::Instance {
        &self.inner
    }
}

#[napi]
pub struct Metric {
    inner: RustMetric,
}

#[napi]
impl Metric {
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[napi(getter)]
    pub fn namespace(&self) -> Option<String> {
        if self.inner.namespace == "default" {
            None
        } else {
            Some(self.inner.namespace.clone())
        }
    }

    #[napi(getter)]
    pub fn threshold(&self) -> Option<f64> {
        self.inner
            .threshold
            .as_ref()
            .and_then(|d| d.to_f64())
            .filter(|v| v.is_finite())
    }

    #[napi(getter)]
    pub fn target(&self) -> Option<f64> {
        self.inner
            .target
            .as_ref()
            .and_then(|d| d.to_f64())
            .filter(|v| v.is_finite())
    }

    #[napi(getter)]
    pub fn unit(&self) -> Option<String> {
        self.inner.unit.clone()
    }

    #[napi(getter)]
    pub fn severity(&self) -> Option<String> {
        self.inner.severity.as_ref().map(|s| format!("{:?}", s))
    }
}

impl Metric {
    pub fn from_rust(inner: RustMetric) -> Self {
        Self { inner }
    }
}

#[napi]
pub struct Mapping {
    inner: RustMapping,
}

#[napi]
impl Mapping {
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi(getter)]
    pub fn target_format(&self) -> String {
        format!("{}", self.inner.target_format())
    }
}

impl Mapping {
    pub fn from_rust(inner: RustMapping) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustMapping {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustMapping {
        &self.inner
    }
}

#[napi]
pub struct Projection {
    inner: RustProjection,
}

#[napi]
impl Projection {
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi(getter)]
    pub fn target_format(&self) -> String {
        format!("{}", self.inner.target_format())
    }
}

impl Projection {
    pub fn from_rust(inner: RustProjection) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustProjection {
        self.inner
    }

    pub fn inner_ref(&self) -> &RustProjection {
        &self.inner
    }
}

#[napi]
pub struct Role {
    inner: RustRole,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Role {
    #[napi(constructor)]
    pub fn new(name: String, namespace: Option<String>) -> Self {
        let inner = match namespace {
            Some(ns) => RustRole::new_with_namespace(name, ns),
            None => RustRole::new_with_namespace(name, "default".to_string()),
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
            .attributes()
            .get(&key)
            .and_then(|v| serde_json::to_string(v).ok())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!(
            "Role(id='{}', name='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.namespace()
        )
    }
}

impl Role {
    pub(crate) fn from_rust(inner: RustRole) -> Self {
        Self { inner }
    }

    pub(crate) fn inner_ref(&self) -> &RustRole {
        &self.inner
    }

    // `into_inner` previously existed here but is not used in the TypeScript
    // bindings. Removing it to avoid `dead_code` lints from clippy.
}

#[napi]
pub struct Relation {
    inner: RustRelation,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Relation {
    #[napi(constructor)]
    pub fn new(
        name: String,
        subject_role_id: String,
        predicate: String,
        object_role_id: String,
        namespace: Option<String>,
        via_flow_id: Option<String>,
    ) -> Result<Self> {
        let subject_id = crate::ConceptId::from(
            Uuid::from_str(&subject_role_id)
                .map_err(|e| Error::from_reason(format!("Invalid subject_role_id UUID: {}", e)))?,
        );

        let object_id = crate::ConceptId::from(
            Uuid::from_str(&object_role_id)
                .map_err(|e| Error::from_reason(format!("Invalid object_role_id UUID: {}", e)))?,
        );

        let via_id = match via_flow_id {
            Some(id) => Some(crate::ConceptId::from(Uuid::from_str(&id).map_err(
                |e| Error::from_reason(format!("Invalid via_flow_id UUID: {}", e)),
            )?)),
            None => None,
        };

        let ns = namespace.unwrap_or_else(|| "default".to_string());
        let inner = RustRelation::new(name, ns, subject_id, predicate, object_id, via_id);

        Ok(Self { inner })
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
    pub fn namespace(&self) -> String {
        self.inner.namespace().to_string()
    }

    #[napi(getter)]
    pub fn subject_role_id(&self) -> String {
        self.inner.subject_role().to_string()
    }

    #[napi(getter)]
    pub fn predicate(&self) -> String {
        self.inner.predicate().to_string()
    }

    #[napi(getter)]
    pub fn object_role_id(&self) -> String {
        self.inner.object_role().to_string()
    }

    #[napi(getter)]
    pub fn via_flow_id(&self) -> Option<String> {
        self.inner.via_flow().map(|id| id.to_string())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        let via_flow = self
            .inner
            .via_flow()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "None".to_string());
        format!(
            "Relation(id='{}', name='{}', namespace='{}', predicate='{}', subject='{}', object='{}', via_flow_id='{}')",
            self.inner.id(),
            self.inner.name(),
            self.inner.namespace(),
            self.inner.predicate(),
            self.inner.subject_role(),
            self.inner.object_role(),
            via_flow
        )
    }
}

impl Relation {
    pub(crate) fn from_rust(inner: RustRelation) -> Self {
        Self { inner }
    }

    pub(crate) fn inner_ref(&self) -> &RustRelation {
        &self.inner
    }

    // `into_inner` previously existed here but is not used in the TypeScript
    // bindings. Removing it to avoid `dead_code` lints from clippy.
}
