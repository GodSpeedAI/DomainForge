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

    // 5. Compare Graphs (structural check via serialization)
    let graph_json =
        serde_json::to_value(&graph).expect("Failed to serialize graph for comparison");
    let graph2_json =
        serde_json::to_value(&graph2).expect("Failed to serialize parsed graph for comparison");
    assert_eq!(graph_json, graph2_json);

    // 6. Ensure serialized forms stay stable across another round-trip
    let printed_again = printer.print(&graph2.to_ast());
    assert_eq!(printed, printed_again);
}
