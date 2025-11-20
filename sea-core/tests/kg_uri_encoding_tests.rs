use sea_core::primitives::Entity;
use sea_core::Graph;

#[test]
fn test_kg_uri_encoding_rfc3986() {
    let mut graph = Graph::new();

    // Test characters that should be encoded to be safe in Turtle QNames or URIs
    // We expect percent encoding for these
    let special_chars = [
        ("Space", " ", "%20"),
        ("Colon", ":", "%3A"),
        ("Slash", "/", "%2F"),
        ("Hash", "#", "%23"),
        // These might fail if URI_ENCODE_SET is not comprehensive enough
        // But let's test what we expect for a robust implementation
        ("Question", "?", "%3F"),
        ("Ampersand", "&", "%26"),
        ("Equals", "=", "%3D"),
        ("Plus", "+", "%2B"),
        ("Comma", ",", "%2C"),
        ("At", "@", "%40"),
        ("Semicolon", ";", "%3B"),
    ];

    for (name, char, encoded) in special_chars {
        let entity_name = format!("Entity{}{}", name, char);
        let entity = Entity::new_with_namespace(entity_name, "default".to_string());
        graph.add_entity(entity).unwrap();

        let turtle = graph.export_rdf("turtle").unwrap();
        let expected = format!("sea:Entity{}{}", name, encoded);

        // We check if the output contains the encoded version
        // If it contains the unencoded version (e.g. sea:EntityQuestion?), it might be invalid Turtle
        if !turtle.contains(&expected) {
            println!(
                "WARNING: '{}' was not encoded as '{}'. Output: {}",
                char, encoded, turtle
            );
            // For now, we might assert failure if we want to enforce it,
            // or we can just log it if we are exploring.
            // But the task is "Add tests", implying we want to verify behavior.
            // I'll assert it.
            assert!(
                turtle.contains(&expected),
                "Failed to encode '{}' to '{}'",
                char,
                encoded
            );
        }
    }
}
