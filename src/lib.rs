pub mod util;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use noirc_abi::input_parser::InputValue;
    use acvm::FieldElement;

    use crate::util::noir_fn;

    #[test]
    fn test_differential_vibe_check() {
        let x = FieldElement::from(2i128);
        let y = FieldElement::from(3i128);

        let input_map = BTreeMap::from([
            ("x".to_string(), InputValue::Field(x)),
            ("y".to_string(), InputValue::Field(y)),
        ]);

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .stack_size(4 * 1024 * 1024)
            .build()
            .unwrap();

        let res = thread_pool.install(|| {
            noir_fn("differential_vibe_check", input_map)
        });

        let res = match res {
            Ok(res) => res,
            Err(e) => panic!("Error: {}", e),
        };

        assert!(res.is_some());

        let res = res.unwrap();

        match res {
            InputValue::Field(f) => assert_eq!(f, x * y),
            _ => panic!("Expected FieldElement"),
        }
    }
}
