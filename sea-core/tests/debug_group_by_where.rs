use sea_core::parser::parse_source;

#[test]
fn test_parse_group_by_with_where() {
    let source = r#"
    Policy test as:
        group_by(f in flows where f.quantity > 150: f.to_entity) {
            count(f) == 1
        }
    "#;
    
    let result = parse_source(source);
    match result {
        Ok(ast) => {
            println!("Parsed successfully!");
            println!("AST: {:#?}", ast);
        }
        Err(e) => {
            panic!("Parse error: {:?}", e);
        }
    }
}
