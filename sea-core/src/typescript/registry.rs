use crate::registry::{NamespaceBinding as RustBinding, NamespaceRegistry as RustRegistry};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::Path;

#[napi]
pub struct NamespaceBinding {
    inner: RustBinding,
}

#[napi]
impl NamespaceBinding {
    #[napi(constructor)]
    pub fn new(path: String, namespace: String) -> Self {
        Self {
            inner: RustBinding {
                path: Path::new(&path).to_path_buf(),
                namespace,
            },
        }
    }

    #[napi(getter)]
    pub fn path(&self) -> String {
        self.inner.path.display().to_string()
    }

    #[napi(getter)]
    pub fn namespace(&self) -> String {
        self.inner.namespace.clone()
    }
}

#[napi]
pub struct NamespaceRegistry {
    inner: RustRegistry,
}

#[napi]
impl NamespaceRegistry {
    #[napi(factory)]
    pub fn from_file(path: String) -> Result<Self> {
        let reg = RustRegistry::from_file(Path::new(&path))
            .map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(Self { inner: reg })
    }

    #[napi]
    pub fn discover(path: String) -> Result<Option<Self>> {
        let res = RustRegistry::discover(Path::new(&path))
            .map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(res.map(|r| NamespaceRegistry { inner: r }))
    }

    #[napi]
    pub fn resolve_files(&self) -> Result<Vec<NamespaceBinding>> {
        let bindings = self
            .inner
            .resolve_files()
            .map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(bindings
            .into_iter()
            .map(|b| NamespaceBinding { inner: b })
            .collect())
    }

    #[napi]
    pub fn namespace_for(&self, path: String) -> Result<String> {
        let res = self.inner.namespace_for(Path::new(&path));
        Ok(res
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.inner.default_namespace().to_string()))
    }

    #[napi(getter)]
    pub fn root(&self) -> String {
        self.inner.root().display().to_string()
    }

    #[napi(getter)]
    pub fn default_namespace(&self) -> String {
        self.inner.default_namespace().to_string()
    }
}
