use crate::units::{Dimension as RustDimension, Unit as RustUnit};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

#[pyclass]
#[derive(Clone)]
pub struct Dimension {
    inner: RustDimension,
}

#[pymethods]
impl Dimension {
    #[staticmethod]
    pub fn parse(name: &str) -> Self {
        Self {
            inner: RustDimension::parse(name),
        }
    }

    pub fn __repr__(&self) -> String {
        format!("Dimension('{}')", self.inner)
    }

    pub fn __str__(&self) -> String {
        format!("{}", self.inner)
    }
}

impl Dimension {
    pub fn from_rust(inner: RustDimension) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustDimension {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Unit {
    inner: RustUnit,
}

#[pymethods]
impl Unit {
    #[new]
    pub fn new(
        symbol: String,
        name: String,
        dimension: String,
        base_factor: f64,
        base_unit: String,
    ) -> PyResult<Self> {
        let dec = Decimal::from_f64(base_factor)
            .ok_or_else(|| PyValueError::new_err("Invalid base_factor"))?;
        let dim = RustDimension::parse(&dimension);
        let inner = RustUnit::new(symbol, name, dim, dec, base_unit);
        Ok(Self { inner })
    }
    #[getter]
    pub fn symbol(&self) -> String {
        self.inner.symbol().to_string()
    }

    #[getter]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[getter]
    pub fn base_unit(&self) -> String {
        self.inner.base_unit().to_string()
    }

    #[getter]
    pub fn base_factor(&self) -> f64 {
        self.inner.base_factor().to_f64().unwrap_or(1.0)
    }

    pub fn __repr__(&self) -> String {
        format!("Unit('{}')", self.inner.symbol())
    }
}

impl Unit {
    pub fn from_rust(inner: RustUnit) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> RustUnit {
        self.inner
    }
}
