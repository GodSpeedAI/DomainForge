use crate::units::{Dimension as RustDimension, Unit as RustUnit};
use napi_derive::napi;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

#[napi]
pub struct Dimension {
    inner: RustDimension,
}

#[napi]
impl Dimension {
    #[napi(factory)]
    pub fn parse(name: String) -> Self {
        Self {
            inner: RustDimension::parse(&name),
        }
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        format!("{}", self.inner)
    }
}

#[napi]
pub struct Unit {
    inner: RustUnit,
}

#[napi]
impl Unit {
    #[napi(constructor)]
    pub fn new(
        symbol: String,
        name: String,
        dimension: String,
        base_factor: f64,
        base_unit: String,
    ) -> Self {
        let dec =
            rust_decimal::Decimal::from_f64(base_factor).unwrap_or(rust_decimal::Decimal::ONE);
        let dim = crate::units::Dimension::parse(&dimension);
        let inner = RustUnit::new(symbol, name, dim, dec, base_unit);
        Self { inner }
    }
    #[napi(getter)]
    pub fn symbol(&self) -> String {
        self.inner.symbol().to_string()
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi(getter)]
    pub fn dimension(&self) -> String {
        format!("{}", self.inner.dimension())
    }

    #[napi(getter)]
    pub fn base_factor(&self) -> f64 {
        self.inner.base_factor().to_f64().unwrap_or(1.0)
    }

    #[napi(getter)]
    pub fn base_unit(&self) -> String {
        self.inner.base_unit().to_string()
    }
}

impl Unit {
    pub fn from_rust(inner: RustUnit) -> Self {
        Self { inner }
    }
}
