#![allow(clippy::useless_conversion, clippy::wrong_self_convention)]

use crate::primitives::{
    Entity as RustEntity, Flow as RustFlow, MappingContract as RustMapping, Metric as RustMetric,
    ProjectionContract as RustProjection, RelationType as RustRelation, Resource as RustResource,
    ResourceInstance as RustResourceInstance, Role as RustRole,
};
use crate::units::unit_from_string;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[pyclass]
#[derive(Clone)]
pub struct Entity {
    inner: RustEntity,
}

#[pymethods]
impl Entity {
    #[new]
    #[pyo3(signature = (name, namespace=None))]
    fn new(name: String, namespace: Option<String>) -> Self {
        let inner = match namespace {
            Some(ns) => RustEntity::new_with_namespace(name, ns),
            None => RustEntity::new_with_namespace(name, "default".to_string()),
        };
        Self { inner }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    fn set_attribute(&mut self, key: String, value: Bound<'_, PyAny>) -> PyResult<()> {
        let json_value = pythonize::depythonize(&value)?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    fn get_attribute(&self, key: String, py: Python) -> PyResult<PyObject> {
        match self.inner.get_attribute(&key) {
            Some(value) => {
                let py_value = pythonize::pythonize(py, &value)?;
                Ok(py_value.into())
            }
            None => Err(PyKeyError::new_err(format!(
                "Attribute '{}' not found",
                key
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Entity(id='{}', name='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.namespace()
        )
    }

    fn __str__(&self) -> String {
        self.inner.name().to_string()
    }
}

impl Entity {
    pub fn from_rust(inner: RustEntity) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustEntity {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Resource {
    inner: RustResource,
}

#[pymethods]
impl Resource {
    #[new]
    #[pyo3(signature = (name, unit, namespace=None))]
    fn new(name: String, unit: String, namespace: Option<String>) -> Self {
        let unit_obj = unit_from_string(unit);
        let inner = match namespace {
            Some(ns) => RustResource::new_with_namespace(name, unit_obj, ns),
            None => RustResource::new_with_namespace(name, unit_obj, "default".to_string()),
        };
        Self { inner }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn unit(&self) -> String {
        self.inner.unit().symbol().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    fn set_attribute(&mut self, key: String, value: Bound<'_, PyAny>) -> PyResult<()> {
        let json_value = pythonize::depythonize(&value)?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    fn get_attribute(&self, key: String, py: Python) -> PyResult<PyObject> {
        match self.inner.get_attribute(&key) {
            Some(value) => {
                let py_value = pythonize::pythonize(py, &value)?;
                Ok(py_value.into())
            }
            None => Err(PyKeyError::new_err(format!(
                "Attribute '{}' not found",
                key
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Resource(id='{}', name='{}', unit='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.unit().symbol(),
            self.inner.namespace()
        )
    }

    fn __str__(&self) -> String {
        format!("{} ({})", self.inner.name(), self.inner.unit().symbol())
    }
}

impl Resource {
    pub fn from_rust(inner: RustResource) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustResource {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Flow {
    inner: RustFlow,
}

#[pymethods]
impl Flow {
    #[new]
    fn new(resource_id: String, from_id: String, to_id: String, quantity: f64) -> PyResult<Self> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid resource_id UUID: {}", e)))?;
        let from_uuid = Uuid::from_str(&from_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid from_id UUID: {}", e)))?;
        let to_uuid = Uuid::from_str(&to_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid to_id UUID: {}", e)))?;
        let decimal_quantity = Decimal::from_f64(quantity)
            .ok_or_else(|| PyValueError::new_err("Invalid quantity value"))?;
        // Convert Uuid to ConceptId for Flow constructor
        let inner = RustFlow::new(
            crate::ConceptId::from(resource_uuid),
            crate::ConceptId::from(from_uuid),
            crate::ConceptId::from(to_uuid),
            decimal_quantity,
        );
        Ok(Self { inner })
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn resource_id(&self) -> String {
        self.inner.resource_id().to_string()
    }

    #[getter]
    fn from_id(&self) -> String {
        self.inner.from_id().to_string()
    }

    #[getter]
    fn to_id(&self) -> String {
        self.inner.to_id().to_string()
    }

    #[getter]
    fn quantity(&self) -> PyResult<f64> {
        self.inner.quantity().to_f64().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert Decimal quantity {} to f64 (overflow or out of range)",
                self.inner.quantity()
            ))
        })
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    fn set_attribute(&mut self, key: String, value: Bound<'_, PyAny>) -> PyResult<()> {
        let json_value = pythonize::depythonize(&value)?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    fn get_attribute(&self, key: String, py: Python) -> PyResult<PyObject> {
        match self.inner.get_attribute(&key) {
            Some(value) => {
                let py_value = pythonize::pythonize(py, &value)?;
                Ok(py_value.into())
            }
            None => Err(PyKeyError::new_err(format!(
                "Attribute '{}' not found",
                key
            ))),
        }
    }

    fn __repr__(&self) -> String {
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
}

#[pyclass]
#[derive(Clone)]
pub struct ResourceInstance {
    inner: RustResourceInstance,
}

#[pymethods]
impl ResourceInstance {
    #[new]
    #[pyo3(signature = (resource_id, entity_id, namespace=None))]
    fn new(resource_id: String, entity_id: String, namespace: Option<String>) -> PyResult<Self> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid resource_id UUID: {}", e)))?;
        let entity_uuid = Uuid::from_str(&entity_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid entity_id UUID: {}", e)))?;
        // Convert Uuid to ConceptId for ResourceInstance constructor
        let inner = match namespace {
            Some(ns) => RustResourceInstance::new_with_namespace(
                crate::ConceptId::from(resource_uuid),
                crate::ConceptId::from(entity_uuid),
                ns,
            ),
            // Use explicit default namespace to match Entity/Resource behavior
            None => RustResourceInstance::new_with_namespace(
                crate::ConceptId::from(resource_uuid),
                crate::ConceptId::from(entity_uuid),
                "default".to_string(),
            ),
        };
        Ok(Self { inner })
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn resource_id(&self) -> String {
        self.inner.resource_id().to_string()
    }

    #[getter]
    fn entity_id(&self) -> String {
        self.inner.entity_id().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    fn set_attribute(&mut self, key: String, value: Bound<'_, PyAny>) -> PyResult<()> {
        let json_value = pythonize::depythonize(&value)?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    fn get_attribute(&self, key: String, py: Python) -> PyResult<PyObject> {
        match self.inner.get_attribute(&key) {
            Some(value) => {
                let py_value = pythonize::pythonize(py, &value)?;
                Ok(py_value.into())
            }
            None => Err(PyKeyError::new_err(format!(
                "Attribute '{}' not found",
                key
            ))),
        }
    }

    fn __repr__(&self) -> String {
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
}

#[pyclass]
#[derive(Clone)]
pub struct Instance {
    inner: crate::primitives::Instance,
}

#[pymethods]
impl Instance {
    #[new]
    #[pyo3(signature = (name, entity_type, namespace=None))]
    fn new(name: String, entity_type: String, namespace: Option<String>) -> Self {
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

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn entity_type(&self) -> String {
        self.inner.entity_type().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    fn set_field(&mut self, key: String, value: Bound<'_, PyAny>) -> PyResult<()> {
        let json_value = pythonize::depythonize(&value)?;
        self.inner.set_field(key, json_value);
        Ok(())
    }

    fn get_field(&self, key: String, py: Python) -> PyResult<PyObject> {
        match self.inner.get_field(&key) {
            Some(value) => {
                let py_value = pythonize::pythonize(py, &value)?;
                Ok(py_value.into())
            }
            None => Err(PyKeyError::new_err(format!("Field '{}' not found", key))),
        }
    }

    fn __repr__(&self) -> String {
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
}

#[pyclass]
#[derive(Clone)]
pub struct Metric {
    inner: RustMetric,
}

#[pymethods]
impl Metric {
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        if self.inner.namespace == "default" {
            None
        } else {
            Some(self.inner.namespace.clone())
        }
    }

    #[getter]
    fn threshold(&self) -> Option<f64> {
        self.inner
            .threshold
            .as_ref()
            .and_then(|d| d.to_f64())
            .filter(|v| v.is_finite())
    }

    #[getter]
    fn target(&self) -> PyResult<Option<f64>> {
        match self.inner.target.as_ref() {
            Some(decimal) => {
                let as_f64 = decimal.to_f64().ok_or_else(|| {
                    PyValueError::new_err("Metric target cannot be represented as f64")
                })?;
                if as_f64.is_finite() {
                    Ok(Some(as_f64))
                } else {
                    Err(PyValueError::new_err(
                        "Metric target is not a finite f64 value",
                    ))
                }
            }
            None => Ok(None),
        }
    }

    #[getter]
    fn unit(&self) -> Option<String> {
        self.inner.unit.clone()
    }

    #[getter]
    fn severity(&self) -> Option<String> {
        self.inner.severity.as_ref().map(|s| format!("{:?}", s))
    }
}

impl Metric {
    pub fn from_rust(inner: RustMetric) -> Self {
        Self { inner }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Mapping {
    inner: RustMapping,
}

#[pymethods]
impl Mapping {
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn target_format(&self) -> String {
        format!("{}", self.inner.target_format())
    }

    fn __repr__(&self) -> String {
        format!(
            "Mapping(name='{}', target_format='{}')",
            self.inner.name(),
            self.inner.target_format()
        )
    }
}

impl Mapping {
    pub fn from_rust(inner: RustMapping) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustMapping {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Projection {
    inner: RustProjection,
}

#[pymethods]
impl Projection {
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn target_format(&self) -> String {
        format!("{}", self.inner.target_format())
    }

    fn __repr__(&self) -> String {
        format!(
            "Projection(name='{}', target_format='{}')",
            self.inner.name(),
            self.inner.target_format()
        )
    }
}

impl Projection {
    pub fn from_rust(inner: RustProjection) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustProjection {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Role {
    inner: RustRole,
}

#[pymethods]
impl Role {
    #[new]
    #[pyo3(signature = (name, namespace=None))]
    fn new(name: String, namespace: Option<String>) -> Self {
        let inner = match namespace {
            Some(ns) => RustRole::new_with_namespace(name, ns),
            None => RustRole::new_with_namespace(name, "default".to_string()),
        };
        Self { inner }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        let ns = self.inner.namespace();
        if ns == "default" {
            None
        } else {
            Some(ns.to_string())
        }
    }

    fn set_attribute(&mut self, key: String, value: Bound<'_, PyAny>) -> PyResult<()> {
        let json_value = pythonize::depythonize(&value)?;
        self.inner.set_attribute(key, json_value);
        Ok(())
    }

    fn get_attribute(&self, key: String, py: Python) -> PyResult<PyObject> {
        match self.inner.attributes().get(&key) {
            Some(value) => {
                let py_value = pythonize::pythonize(py, value)?;
                Ok(py_value.into())
            }
            None => Err(PyKeyError::new_err(format!(
                "Attribute '{}' not found",
                key
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Role(id='{}', name='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.namespace()
        )
    }
}

impl Role {
    pub fn from_rust(inner: RustRole) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustRole {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Relation {
    inner: RustRelation,
}

#[pymethods]
impl Relation {
    #[new]
    #[pyo3(signature = (name, subject_role_id, predicate, object_role_id, namespace=None, via_flow_id=None))]
    fn new(
        name: String,
        subject_role_id: String,
        predicate: String,
        object_role_id: String,
        namespace: Option<String>,
        via_flow_id: Option<String>,
    ) -> PyResult<Self> {
        let subject_id =
            crate::ConceptId::from(Uuid::from_str(&subject_role_id).map_err(|e| {
                PyValueError::new_err(format!("Invalid subject_role_id UUID: {}", e))
            })?);

        let object_id =
            crate::ConceptId::from(Uuid::from_str(&object_role_id).map_err(|e| {
                PyValueError::new_err(format!("Invalid object_role_id UUID: {}", e))
            })?);

        let via_id = match via_flow_id {
            Some(id) => Some(crate::ConceptId::from(Uuid::from_str(&id).map_err(
                |e| PyValueError::new_err(format!("Invalid via_flow_id UUID: {}", e)),
            )?)),
            None => None,
        };

        let ns = namespace.unwrap_or_else(|| "default".to_string());
        let inner = RustRelation::new(name, ns, subject_id, predicate, object_id, via_id);

        Ok(Self { inner })
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        Some(self.inner.namespace().to_string())
    }

    #[getter]
    fn subject_role_id(&self) -> String {
        self.inner.subject_role().to_string()
    }

    #[getter]
    fn predicate(&self) -> String {
        self.inner.predicate().to_string()
    }

    #[getter]
    fn object_role_id(&self) -> String {
        self.inner.object_role().to_string()
    }

    #[getter]
    fn via_flow_id(&self) -> Option<String> {
        self.inner.via_flow().map(|id| id.to_string())
    }

    fn __repr__(&self) -> String {
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
    pub fn from_rust(inner: RustRelation) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustRelation {
        self.inner
    }
}
