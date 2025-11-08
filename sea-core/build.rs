extern crate napi_build;

fn main() {
    #[cfg(feature = "typescript")]
    napi_build::setup();
}
