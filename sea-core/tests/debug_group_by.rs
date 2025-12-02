use sea_core::parser::parse_source;

#[test]
fn test_parse_group_by() {
    let source = r#"
    Policy test as:
        group_by(f in flows: f.to_entity) {
            sum(f.quantity) >= 200
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
