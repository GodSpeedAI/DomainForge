use domainforge_core::parser::{parse, unescape_string};

#[test]
fn test_unicode_entity_names() {
    let source = r#"
        Entity "Müller GmbH" in germany
        Entity "北京公司" in china
        Entity "Société Générale" in france
    "#;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse Unicode entity names: {:?}",
        ast.err()
    );
}

#[test]
fn test_unicode_resource_names() {
    let source = r#"
        Resource "Produits" kg in france
        Resource "製品" units in japan
        Resource "Продукция" liters in russia
    "#;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse Unicode resource names: {:?}",
        ast.err()
    );
}

#[test]
fn test_unicode_identifiers_in_policies() {
    let source = r#"
        Entity "Company" in business
        Policy règle_française as: true
    "#;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse Unicode identifiers: {:?}",
        ast.err()
    );
}

#[test]
fn test_escape_sequence_backslash() {
    let result = unescape_string(r"path\\to\\file");
    assert_eq!(result.unwrap(), r"path\to\file");
}

#[test]
fn test_escape_sequence_quote() {
    let result = unescape_string(r#"Company with \"Quotes\""#);
    assert_eq!(result.unwrap(), r#"Company with "Quotes""#);
}

#[test]
fn test_escape_sequence_newline() {
    let result = unescape_string(r"Line1\nLine2");
    assert_eq!(result.unwrap(), "Line1\nLine2");
}

#[test]
fn test_escape_sequence_tab() {
    let result = unescape_string(r"Column1\tColumn2");
    assert_eq!(result.unwrap(), "Column1\tColumn2");
}

#[test]
fn test_escape_sequence_carriage_return() {
    let result = unescape_string(r"Text\rMore");
    assert_eq!(result.unwrap(), "Text\rMore");
}

#[test]
fn test_escape_sequence_unicode() {
    let result = unescape_string(r"\u{1F600}");
    assert_eq!(result.unwrap(), "😀");

    let result = unescape_string(r"\u{4E2D}");
    assert_eq!(result.unwrap(), "中");

    let result = unescape_string(r"\u{41}");
    assert_eq!(result.unwrap(), "A");
}

#[test]
fn test_escape_sequences_in_entity() {
    let source = r#"
        Entity "Company with \"Quotes\"" in test
    "#;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse escape sequences: {:?}",
        ast.err()
    );
}

#[test]
fn test_multiline_string_literal() {
    let source = r####"
        Policy long_description as: """
        This is a multi-line
        policy description
        that spans multiple lines
        """ contains "multi-line"
    "####;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse multi-line strings: {:?}",
        ast.err()
    );
}

#[test]
fn test_multiline_string_in_resource() {
    let source = r####"
        Resource """Complex
        Resource
        Name""" kg
    "####;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse multi-line resource names: {:?}",
        ast.err()
    );
}

#[test]
fn test_mixed_unicode_and_escapes() {
    let result = unescape_string(r"日本\n製品\t\u{1F1EF}\u{1F1F5}");
    assert_eq!(result.unwrap(), "日本\n製品\t🇯🇵");
}

#[test]
fn test_complex_unicode_entities() {
    let source = r#"
        Entity "Москва" in russia
        Entity "القاهرة" in egypt
        Entity "서울" in korea
        Entity "Αθήνα" in greece
    "#;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse complex Unicode names: {:?}",
        ast.err()
    );
}

#[test]
fn test_unicode_emoji_in_names() {
    let source = r#"
        Entity "Company 🏢" in tech
        Resource "Product 📦" units in warehouse
    "#;

    let ast = parse(source);
    assert!(ast.is_ok(), "Should parse emoji in names: {:?}", ast.err());
}

#[test]
fn test_all_escape_sequences_combined() {
    let result = unescape_string(r#"Line 1\nLine 2\tTabbed\r\"Quoted\"\\\u{2764}"#);
    assert_eq!(result.unwrap(), "Line 1\nLine 2\tTabbed\r\"Quoted\"\\❤");
}

#[test]
fn test_multiline_preserves_content() {
    let source = r####"
        Policy description as: """
First line
    Indented line
Last line
        """ = """
First line
    Indented line
Last line
        """
    "####;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse and preserve multiline content: {:?}",
        ast.err()
    );
}

#[test]
fn test_empty_multiline_string() {
    let source = r####"
        Policy empty as: """"""
    "####;

    let ast = parse(source);
    assert!(
        ast.is_ok(),
        "Should parse empty multiline string: {:?}",
        ast.err()
    );
}

#[test]
fn test_unicode_in_flow() {
    let source = r#"
        Entity "发送方" in china
        Entity "接收方" in china
        Resource "资源" kg in china
        Flow "资源" from "发送方" to "接收方" quantity 100
    "#;

    let ast = parse(source);
    assert!(ast.is_ok(), "Should parse Unicode in flow: {:?}", ast.err());
}

#[test]
fn test_rtl_text_support() {
    let source = r#"
        Entity "الشركة" in middle_east
        Resource "المنتج" units
    "#;

    let ast = parse(source);
    assert!(ast.is_ok(), "Should parse RTL text: {:?}", ast.err());
}

#[test]
fn test_mixed_scripts() {
    let source = r#"
        Entity "АБВ-ABC-αβγ-123" in test
    "#;

    let ast = parse(source);
    assert!(ast.is_ok(), "Should parse mixed scripts: {:?}", ast.err());
}
