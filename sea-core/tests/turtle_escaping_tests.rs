#[cfg(test)]
mod turtle_escaping_tests {
    use sea_core::kg::KnowledgeGraph;

    #[test]
    fn test_escape_backslash() {
        let input = r#"Name\With\Backslash"#;
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, r#"Name\\With\\Backslash"#);
    }

    #[test]
    fn test_escape_double_quote() {
        let input = r#"Name"With"Quotes"#;
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, r#"Name\"With\"Quotes"#);
    }

    #[test]
    fn test_escape_newline() {
        let input = "Name\nWith\nNewlines";
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, r#"Name\nWith\nNewlines"#);
    }

    #[test]
    fn test_escape_carriage_return() {
        let input = "Name\rWith\rReturns";
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, r#"Name\rWith\rReturns"#);
    }

    #[test]
    fn test_escape_tab() {
        let input = "Name\tWith\tTabs";
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, r#"Name\tWith\tTabs"#);
    }

    #[test]
    fn test_escape_combined_characters() {
        let input = "Complex\\\"Name\n\r\t";
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, r#"Complex\\\"Name\n\r\t"#);
    }

    #[test]
    fn test_no_escape_needed() {
        let input = "SimpleNameWithoutSpecialChars";
        let escaped = KnowledgeGraph::escape_turtle_literal(input);
        assert_eq!(escaped, "SimpleNameWithoutSpecialChars");
    }
}
