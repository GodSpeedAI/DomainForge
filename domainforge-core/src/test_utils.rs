#[cfg(test)]
mod proptest_integration {
    use crate::uuid_module;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn uuid_generation_never_panics(_seed in 0u64..1_000_000) {
            let _ = uuid_module::generate_uuid_v7();
        }
    }
}
