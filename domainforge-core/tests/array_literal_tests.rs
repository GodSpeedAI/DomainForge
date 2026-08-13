//! `literal` had no array form (`sea.pest`), so a declared `list<T>` or
//! `list<ref<T>>` field could never be given a concrete value by any
//! instance — the entity was silently uninstantiable for that field, with
//! no diagnostic (limitation L2). The evaluator side (`FieldType::List`
//! recursive element validation, `MinItems`/`MaxItems`, dangling-reference
//! rejection per element) was already fully implemented; only the parser
//! could not produce an array-valued `Expression::Literal`.

use domainforge_core::parser::parse_to_graph;

#[test]
fn list_of_strings_instantiates_and_validates() {
    let source = r#"
@namespace "array_literal_test"

export entity "Widget" {
    key widget_id: string (min_length 1)
    tags: list<string> (min_items 1, max_items 3)
}

instance w1 of "Widget" {
    widget_id: "W1",
    tags: ["red", "blue"]
}
"#;
    let graph = parse_to_graph(source).expect("array literal instantiates a list<string> field");
    assert_eq!(graph.entity_instance_count(), 1);
}

#[test]
fn list_below_min_items_is_rejected() {
    let source = r#"
@namespace "array_literal_min_items_test"

export entity "Widget" {
    key widget_id: string (min_length 1)
    tags: list<string> (min_items 2)
}

instance w1 of "Widget" {
    widget_id: "W1",
    tags: ["only-one"]
}
"#;
    let err = parse_to_graph(source).expect_err("one element must violate min_items 2");
    assert!(
        err.to_string().contains("min_items") || err.to_string().to_lowercase().contains("min")
    );
}

#[test]
fn list_above_max_items_is_rejected() {
    let source = r#"
@namespace "array_literal_max_items_test"

export entity "Widget" {
    key widget_id: string (min_length 1)
    tags: list<string> (max_items 1)
}

instance w1 of "Widget" {
    widget_id: "W1",
    tags: ["a", "b"]
}
"#;
    let err = parse_to_graph(source).expect_err("two elements must violate max_items 1");
    assert!(
        err.to_string().contains("max_items") || err.to_string().to_lowercase().contains("max")
    );
}

#[test]
fn list_of_typed_references_validates_each_element() {
    let source = r#"
@namespace "array_literal_ref_test"

export entity "Journey" {
    key journey_id: string (min_length 1)
}

export entity "Surface" {
    key surface_id: string (min_length 1)
    journeys: list<ref<Journey>>
}

instance j1 of "Journey" { journey_id: "CJ01" }
instance j2 of "Journey" { journey_id: "CJ02" }

instance s1 of "Surface" {
    surface_id: "S1",
    journeys: ["CJ01", "CJ02"]
}
"#;
    let graph = parse_to_graph(source)
        .expect("a list<ref<T>> field with every element a valid key instantiates");
    assert_eq!(graph.entity_instance_count(), 3);
}

#[test]
fn list_of_typed_references_rejects_a_dangling_element() {
    let source = r#"
@namespace "array_literal_dangling_ref_test"

export entity "Journey" {
    key journey_id: string (min_length 1)
}

export entity "Surface" {
    key surface_id: string (min_length 1)
    journeys: list<ref<Journey>>
}

instance j1 of "Journey" { journey_id: "CJ01" }

instance s1 of "Surface" {
    surface_id: "S1",
    journeys: ["CJ01", "CJ99"]
}
"#;
    let err = parse_to_graph(source)
        .expect_err("a list<ref<T>> element with no matching instance must be rejected");
    assert!(
        err.to_string().contains("CJ99"),
        "error must name the dangling reference: {err}"
    );
}

#[test]
fn empty_array_literal_parses() {
    let source = r#"
@namespace "array_literal_empty_test"

export entity "Widget" {
    key widget_id: string (min_length 1)
    tags: list<string> optional
}

instance w1 of "Widget" {
    widget_id: "W1",
    tags: []
}
"#;
    let graph = parse_to_graph(source).expect("an empty array literal parses and validates");
    assert_eq!(graph.entity_instance_count(), 1);
}
