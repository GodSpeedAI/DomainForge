use sea_core::parser::{parse, parse_to_graph};

#[test]
fn test_parse_profile_annotation() {
    let source = r#"
@profile "cloud"
Entity "Server"
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.metadata.profile, Some("cloud".to_string()));
}

#[test]
fn test_parse_unknown_profile() {
    let source = r#"
@profile "unknown_profile"
Entity "Server"
"#;
    let result = parse_to_graph(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("Unknown profile: 'unknown_profile'"));
}

#[test]
fn test_parse_valid_profile() {
    let source = r#"
@profile "data"
Entity "Table"
"#;
    let result = parse_to_graph(source);
    assert!(result.is_ok());
}
