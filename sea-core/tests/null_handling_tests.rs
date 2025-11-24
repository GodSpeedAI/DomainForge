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
}
