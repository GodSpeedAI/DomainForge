use crate::units::{Dimension as RustDimension, Unit as RustUnit};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Dimension {
    inner: RustDimension,
}

#[wasm_bindgen]
impl Dimension {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self {
        Self {
            inner: RustDimension::parse(&name),
        }
    }
    #[allow(clippy::inherent_to_string)]
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{}", self.inner)
    }
}

#[wasm_bindgen]
pub struct Unit {
    inner: RustUnit,
}

#[wasm_bindgen]
impl Unit {
    #[wasm_bindgen(constructor)]
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
    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> String {
        self.inner.symbol().to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn base_factor(&self) -> f64 {
        self.inner.base_factor().to_f64().unwrap_or(1.0)
    }
    #[wasm_bindgen(getter)]
    pub fn base_unit(&self) -> String {
        self.inner.base_unit().to_string()
    }
}
