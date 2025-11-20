use crate::registry::{NamespaceBinding as RustBinding, NamespaceRegistry as RustRegistry};
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct NamespaceBinding {
    pub inner: RustBinding,
}

#[pymethods]
impl NamespaceBinding {
    #[getter]
    pub fn path(&self) -> String {
        self.inner.path.display().to_string()
    }

    #[getter]
    pub fn namespace(&self) -> String {
        self.inner.namespace.clone()
    }
}

impl NamespaceBinding {
    pub fn from_rust(inner: RustBinding) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustBinding {
        self.inner
    }
}

#[pyclass]
pub struct NamespaceRegistry {
    inner: RustRegistry,
}

#[pymethods]
impl NamespaceRegistry {
    #[staticmethod]
    pub fn from_file(path: String) -> PyResult<Self> {
        let reg = RustRegistry::from_file(std::path::Path::new(&path))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;
        Ok(Self { inner: reg })
    }

    #[staticmethod]
    pub fn discover(path: String) -> PyResult<Option<Self>> {
        let res = RustRegistry::discover(std::path::Path::new(&path))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;
        Ok(res.map(|r| NamespaceRegistry { inner: r }))
    }

    pub fn resolve_files(
        &self,
        fail_on_ambiguity: Option<bool>,
    ) -> PyResult<Vec<NamespaceBinding>> {
        let fail = fail_on_ambiguity.unwrap_or(false);
        let bindings = self
            .inner
            .resolve_files_with_options(fail)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;
        Ok(bindings
            .into_iter()
            .map(NamespaceBinding::from_rust)
            .collect())
    }

    pub fn namespace_for(&self, path: String, fail_on_ambiguity: Option<bool>) -> PyResult<String> {
        let fail = fail_on_ambiguity.unwrap_or(false);
        let res = self
            .inner
            .namespace_for_with_options(std::path::Path::new(&path), fail)
            .map(|s| s.to_string())
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", err)))?;
        Ok(res)
    }

    #[getter]
    pub fn root(&self) -> String {
        self.inner.root().display().to_string()
    }

    #[getter]
    pub fn default_namespace(&self) -> String {
        self.inner.default_namespace().to_string()
    }
}

impl NamespaceRegistry {
    pub fn from_rust(inner: RustRegistry) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustRegistry {
        self.inner
    }
}
