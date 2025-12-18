use sea_core::parser::{parse, AstNode};

#[test]
fn parses_imports_and_exports() {
    let source = r#"
        @namespace "acme.finance"
        import { Payment } from "acme.shared"
        import * as utils from "acme.utils"

        export Entity "Invoice"
        Entity "Internal" // not exported explicitly but still parsed
    "#;

    let ast = parse(source).expect("parser should succeed");
    assert_eq!(ast.metadata.imports.len(), 2);

    let exported = ast
        .declarations
        .iter()
        .filter(|node| matches!(node.node, AstNode::Export(_)))
        .count();
    assert_eq!(exported, 1, "only one declaration is explicitly exported");
}
