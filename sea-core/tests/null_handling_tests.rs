#[cfg(test)]
mod tests {
    use sea_core::policy::Expression;

    #[test]
    fn smoke_truth_table_examples() {
        // Basic compile-time smoke test: build a few expressions and ensure
        // they construct correctly. Detailed evaluator tests require the
        // evaluation context; those are added where the evaluator exists.
        let _lit_true = Expression::Literal(serde_json::json!(true));
        let _lit_null = Expression::Literal(serde_json::Value::Null);
        let _lit_num = Expression::Literal(serde_json::json!(42));
        match _lit_true {
            Expression::Literal(_) => {}
            _ => panic!("expected literal"),
        }
        match _lit_num {
            Expression::Literal(_) => {}
            _ => panic!("expected literal"),
        }
        match _lit_null {
            Expression::Literal(_) => {}
            _ => panic!("expected literal"),
        }
    }

    #[cfg(feature = "three_valued_logic")]
    #[test]
    fn aggregator_nulls() {
        use rust_decimal::Decimal;
        use sea_core::policy::three_valued::aggregators;

        let vals: Vec<Option<Decimal>> =
            vec![Some(Decimal::new(1, 0)), None, Some(Decimal::new(3, 0))];
        assert_eq!(aggregators::sum_nullable(&vals), None);
        assert_eq!(aggregators::sum_nonnull(&vals), Decimal::new(4, 0));
        assert_eq!(aggregators::count_all(&vals), 3usize);
        assert_eq!(aggregators::count_nonnull(&vals), 2usize);
    }
}
