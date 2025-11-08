use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyKeyError};
use crate::primitives::{
    Entity as RustEntity,
    Resource as RustResource,
    Flow as RustFlow,
    Instance as RustInstance,
};
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;

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
            None => RustEntity::new(name),
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
        self.inner.namespace().map(|s| s.to_string())
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
            None => Err(PyKeyError::new_err(format!("Attribute '{}' not found", key))),
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
        let inner = match namespace {
            Some(ns) => RustResource::new_with_namespace(name, unit, ns),
            None => RustResource::new(name, unit),
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
        self.inner.unit().to_string()
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        self.inner.namespace().map(|s| s.to_string())
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
            None => Err(PyKeyError::new_err(format!("Attribute '{}' not found", key))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Resource(id='{}', name='{}', unit='{}', namespace={:?})",
            self.inner.id(),
            self.inner.name(),
            self.inner.unit(),
            self.inner.namespace()
        )
    }

    fn __str__(&self) -> String {
        format!("{} ({})", self.inner.name(), self.inner.unit())
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

        let inner = RustFlow::new(resource_uuid, from_uuid, to_uuid, decimal_quantity);
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
    fn quantity(&self) -> f64 {
        self.inner.quantity().to_string().parse().expect("Failed to parse Decimal quantity to f64 in Python binding")
    }

    #[getter]
    fn namespace(&self) -> Option<String> {
        self.inner.namespace().map(|s| s.to_string())
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
            None => Err(PyKeyError::new_err(format!("Attribute '{}' not found", key))),
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
pub struct Instance {
    inner: RustInstance,
}

#[pymethods]
impl Instance {
    #[new]
    #[pyo3(signature = (resource_id, entity_id, namespace=None))]
    fn new(resource_id: String, entity_id: String, namespace: Option<String>) -> PyResult<Self> {
        let resource_uuid = Uuid::from_str(&resource_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid resource_id UUID: {}", e)))?;
        let entity_uuid = Uuid::from_str(&entity_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid entity_id UUID: {}", e)))?;

        let inner = match namespace {
            Some(ns) => RustInstance::new_with_namespace(resource_uuid, entity_uuid, ns),
            None => RustInstance::new(resource_uuid, entity_uuid),
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
        self.inner.namespace().map(|s| s.to_string())
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
            None => Err(PyKeyError::new_err(format!("Attribute '{}' not found", key))),
        }
    }

    fn __repr__(&self) -> String {
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
}
