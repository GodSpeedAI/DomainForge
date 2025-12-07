use sea_core::parser::{parse_to_graph, PrettyPrinter};

#[test]
fn test_round_trip_basic() {
    let source = r#"Entity "Server"
Resource "CPU"
"#;

    // 1. Parse to Graph
    let graph = parse_to_graph(source).unwrap();

    // 2. Graph to AST
    let ast = graph.to_ast();

    // 3. AST to Source (PrettyPrint)
    let printer = PrettyPrinter::new();
    let printed = printer.print(&ast);

    println!("Printed:\n{}", printed);

    // 4. Parse Printed to Graph
    let graph2 = parse_to_graph(&printed).unwrap();

    // 5. Compare Graphs (basic check)
    assert_eq!(graph.entity_count(), graph2.entity_count());
    assert_eq!(graph.resource_count(), graph2.resource_count());
}
