//! RED/GREEN tests for the SEA application-contract grammar and partial AST.

use domainforge_core::parser::ast::{
    AstNode, EntityBody, EnumDecl, FieldConstraintDecl, FieldDecl, FieldType, FieldTypeRef,
    OperationClause, OperationDecl, RecordDecl,
};
use domainforge_core::parser::parse;
use domainforge_core::policy::Expression;

fn unwrap_export(node: &AstNode) -> &AstNode {
    match node {
        AstNode::Export(inner) => &inner.node,
        other => other,
    }
}

fn flagship_command() -> String {
    std::fs::read_to_string("../fixtures/application_generation/flagship/command-write.sea")
        .expect("flagship command fixture must be readable")
}

#[allow(dead_code)]
fn flagship_query() -> String {
    std::fs::read_to_string("../fixtures/application_generation/flagship/query-read.sea")
        .expect("flagship query fixture must be readable")
}

fn find_node(
    ast: &domainforge_core::parser::ast::Ast,
    predicate: impl Fn(&AstNode) -> bool,
) -> &AstNode {
    ast.declarations
        .iter()
        .map(|d| unwrap_export(&d.node))
        .find(|n| predicate(n))
        .expect("expected matching node")
}

#[test]
fn parses_flagship_declarations_and_keeps_partial_operations() {
    let command = flagship_command();
    let ast = parse(&command).expect("accepted command syntax must parse");

    // Entity Order has a typed body
    let order = find_node(
        &ast,
        |n| matches!(n, AstNode::Entity { name, .. } if name == "Order"),
    );
    let order_body = match order {
        AstNode::Entity { body: Some(b), .. } => b,
        _ => panic!("Order entity must carry a typed body"),
    };
    assert!(order_body
        .fields
        .iter()
        .any(|f| f.is_key && f.name == "order_id"));
    assert!(order_body.fields.iter().any(|f| f.name == "status"));

    // Operation place_order keeps all clauses in authored order. The reference
    // fixture (§11.1) carries three failure clauses; the plan's "15" was a
    // planning slip against the normative source.
    let op = find_node(&ast, |n| matches!(n, AstNode::Operation(_)));
    let op_decl = match op {
        AstNode::Operation(o) => o,
        _ => panic!("expected Operation node"),
    };
    assert_eq!(op_decl.name, "place_order");
    assert_eq!(op_decl.clauses.len(), 16);

    // Partial operation keeps authored clauses (here: only one)
    let partial = parse("operation incomplete { direction inbound }").unwrap();
    let partial_op = unwrap_export(&partial.declarations[0].node);
    match partial_op {
        AstNode::Operation(o) => assert_eq!(o.clauses.len(), 1),
        _ => panic!("expected Operation"),
    }
}

#[test]
fn parses_enum_record_field_types_constraints_and_defaults() {
    let ast = parse(&flagship_command()).unwrap();

    let status_enum = find_node(
        &ast,
        |n| matches!(n, AstNode::Enum(EnumDecl { name, .. }) if name == "OrderStatus"),
    );
    match status_enum {
        AstNode::Enum(e) => {
            assert_eq!(e.members.len(), 1);
            assert_eq!(e.members[0].name, "placed");
            assert_eq!(e.members[0].wire, "placed");
        }
        _ => unreachable!(),
    }

    let place_in = find_node(
        &ast,
        |n| matches!(n, AstNode::Record(RecordDecl { name, .. }) if name == "PlaceOrderInput"),
    );
    match place_in {
        AstNode::Record(r) => {
            assert_eq!(r.name, "PlaceOrderInput");
            // order_id: uuid
            assert!(
                matches!(r.fields[0].field_type, FieldType::Scalar(FieldTypeRef { ref symbol, alias: None }) if symbol == "uuid")
            );
            // total: quantity<USD>
            assert!(
                matches!(r.fields[2].field_type, FieldType::Quantity(ref t) if t.symbol == "USD")
            );
            // client_order_id has min_length 1, max_length 64
            let client = &r.fields[1];
            assert!(client.constraints.iter().any(|c| matches!(
                c,
                FieldConstraintDecl::MinLength(v) if *v == 1
            )));
            assert!(client.constraints.iter().any(|c| matches!(
                c,
                FieldConstraintDecl::MaxLength(v) if *v == 64
            )));
        }
        _ => unreachable!(),
    }

    // Entity Order.status default "placed"
    let order = find_node(
        &ast,
        |n| matches!(n, AstNode::Entity { name, body: Some(_), .. } if name == "Order"),
    );
    if let AstNode::Entity { body: Some(b), .. } = order {
        let status = b.fields.iter().find(|f| f.name == "status").unwrap();
        match &status.default {
            Some(Expression::Literal(serde_json::Value::String(s))) if s == "placed" => {}
            other => panic!("status default must be literal \"placed\", got {:?}", other),
        }
    }
}

#[test]
fn parses_role_reference_in_policy_expression() {
    // role<Customer> appears as an opaque expression leaf
    let src = "@namespace \"app\"\npolicy actor_is_customer per Constraint Obligation priority 1 as: actor = role<Customer>";
    let ast = parse(src).expect("role<...> must parse");
    let policy = unwrap_export(&ast.declarations[0].node);
    match policy {
        AstNode::Policy { expression, .. } => {
            let found = expression_contains_role_ref(expression);
            assert!(
                found,
                "policy expression must contain a RoleReference, got: {:?}",
                expression
            );
        }
        _ => panic!("expected Policy"),
    }
}

fn expression_contains_role_ref(expr: &Expression) -> bool {
    match expr {
        Expression::RoleReference { .. } => true,
        Expression::Binary { left, right, .. } => {
            expression_contains_role_ref(left) || expression_contains_role_ref(right)
        }
        Expression::Unary { operand, .. } => expression_contains_role_ref(operand),
        Expression::Cast { operand, .. } => expression_contains_role_ref(operand),
        Expression::Quantifier { condition, .. } => expression_contains_role_ref(condition),
        _ => false,
    }
}

#[test]
fn parses_alias_qualified_role_reference() {
    let src = "@namespace \"app\"\npolicy p per Constraint Obligation priority 1 as: actor = role<alias.Customer>";
    let ast = parse(src).expect("role<alias.Customer> must parse");
    let policy = unwrap_export(&ast.declarations[0].node);
    let expr = match policy {
        AstNode::Policy { expression, .. } => expression,
        _ => panic!("expected Policy"),
    };
    assert!(expression_contains_role_ref(expr));
}

#[test]
fn parses_not_applicable_idempotency_clause() {
    let src = "@namespace \"app\"\nrecord In { id: uuid }\nrecord Out { id: uuid }\nentity \"E\" { key id: uuid }\noperation q {\n  intent \"q\"\n  direction inbound\n  actor anonymous\n  access public\n  input In\n  output Out\n  state E\n  effect reads E\n  transaction read_only\n  failure missing for missing_state \"none\"\n  idempotency not_applicable(read_only, \"inherent read\")\n  concurrency read_snapshot\n  evidence operation_trace\n  lifecycle synchronous_request_response\n}";
    let ast = parse(src).expect("not_applicable must parse");
    let op = unwrap_export(&ast.declarations[3].node);
    if let AstNode::Operation(o) = op {
        assert!(o
            .clauses
            .iter()
            .any(|c| matches!(c, OperationClause::IdempotencyNotApplicable { .. })));
    } else {
        panic!("expected Operation");
    }
}

#[test]
fn parses_all_parsed_but_invalid_v01_direction_kinds() {
    for dir in ["outbound", "internal"] {
        let src = format!("@namespace \"app\"\noperation o {{ direction {} }}", dir);
        let ast = parse(&src).unwrap_or_else(|e| panic!("{} must parse, got: {}", dir, e));
        let op = unwrap_export(&ast.declarations[0].node);
        if let AstNode::Operation(o) = op {
            assert!(
                o.clauses
                    .iter()
                    .any(|c| matches!(c, OperationClause::Direction { .. })),
                "direction {} must be captured",
                dir
            );
        }
    }
}

#[test]
fn entity_without_body_parses_unchanged() {
    let ast = parse("@namespace \"legacy\"\nEntity \"Bare\"").unwrap();
    let e = unwrap_export(&ast.declarations[0].node);
    match e {
        AstNode::Entity {
            name, body: None, ..
        } if name == "Bare" => {}
        other => panic!("bodyless entity must keep body: None, got {:?}", other),
    }
}

// Suppress unused-function warnings until GREEN uses them
#[test]
fn _compile_helpers() {
    let _: Option<EntityBody> = None;
    let _: Option<&FieldDecl> = None;
    let _: Option<&FieldType> = None;
    let _: Option<&FieldTypeRef> = None;
    let _: Option<&FieldConstraintDecl> = None;
    let _: Option<&RecordDecl> = None;
    let _: Option<&EnumDecl> = None;
    let _: Option<&OperationDecl> = None;
    let _: Option<&OperationClause> = None;
}
