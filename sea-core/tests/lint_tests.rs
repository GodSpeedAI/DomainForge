use sea_core::parser::lint::Linter;

#[test]
fn test_lint_keyword_collision() {
    let linter = Linter::new();
    assert!(linter.check_identifier("Entity", false).is_err());
    assert!(linter.check_identifier("Resource", false).is_err());
    assert!(linter.check_identifier("Flow", false).is_err());
    assert!(linter.check_identifier("Policy", false).is_err());
    assert!(linter.check_identifier("Unit", false).is_err());
    assert!(linter.check_identifier("Dimension", false).is_err());
}

#[test]
fn test_lint_quoted_identifier() {
    let linter = Linter::new();
    assert!(linter.check_identifier("Entity", true).is_ok());
    assert!(linter.check_identifier("MyEntity", false).is_ok());
}
