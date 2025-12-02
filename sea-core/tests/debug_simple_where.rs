use sea_core::parser::parse_source;

#[test]
fn test_simple_where() {
    let source = r#"
    Policy test as:
        forall f in flows: f.quantity > 150
    "#;
    
    let result = parse_source(source);
    match result {
        Ok(ast) => {
            println!("Parsed successfully!");
        }
        Err(e) => {
            panic!("Parse error: {:?}", e);
        }
    }
}
