pub fn unescape_string(s: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    if chars.next() == Some('{') {
                        let mut hex_digits = String::new();
                        loop {
                            match chars.peek() {
                                Some(&'}') => {
                                    chars.next();
                                    break;
                                }
                                Some(&c) if c.is_ascii_hexdigit() => {
                                    hex_digits.push(c);
                                    chars.next();
                                }
                                Some(&c) => {
                                    return Err(format!(
                                        "Invalid character in unicode escape: {}",
                                        c
                                    ));
                                }
                                None => {
                                    return Err("Unterminated unicode escape sequence".to_string());
                                }
                            }
                        }

                        if hex_digits.is_empty() || hex_digits.len() > 6 {
                            return Err(format!(
                                "Invalid unicode escape length: {}",
                                hex_digits.len()
                            ));
                        }

                        let code_point = u32::from_str_radix(&hex_digits, 16)
                            .map_err(|e| format!("Invalid hex in unicode escape: {}", e))?;

                        let unicode_char = char::from_u32(code_point).ok_or_else(|| {
                            format!("Invalid unicode code point: U+{:X}", code_point)
                        })?;

                        result.push(unicode_char);
                    } else {
                        return Err("Expected '{' after \\u".to_string());
                    }
                }
                Some(c) => {
                    return Err(format!("Unknown escape sequence: \\{}", c));
                }
                None => {
                    return Err("Unexpected end of string after backslash".to_string());
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_basic() {
        assert_eq!(unescape_string("hello").unwrap(), "hello");
    }

    #[test]
    fn test_unescape_backslash() {
        assert_eq!(unescape_string("a\\\\b").unwrap(), "a\\b");
    }

    #[test]
    fn test_unescape_quote() {
        assert_eq!(
            unescape_string("say \\\"hello\\\"").unwrap(),
            "say \"hello\""
        );
    }

    #[test]
    fn test_unescape_newline() {
        assert_eq!(unescape_string("line1\\nline2").unwrap(), "line1\nline2");
    }

    #[test]
    fn test_unescape_tab() {
        assert_eq!(unescape_string("col1\\tcol2").unwrap(), "col1\tcol2");
    }

    #[test]
    fn test_unescape_carriage_return() {
        assert_eq!(unescape_string("text\\rmore").unwrap(), "text\rmore");
    }

    #[test]
    fn test_unescape_unicode() {
        assert_eq!(unescape_string("\\u{1F600}").unwrap(), "😀");
        assert_eq!(unescape_string("\\u{4E2D}").unwrap(), "中");
        assert_eq!(unescape_string("\\u{41}").unwrap(), "A");
    }

    #[test]
    fn test_unescape_mixed() {
        assert_eq!(
            unescape_string("Hello\\nWorld\\t\\u{1F44B}").unwrap(),
            "Hello\nWorld\t👋"
        );
    }

    #[test]
    fn test_unescape_invalid_escape() {
        assert!(unescape_string("invalid\\x").is_err());
    }

    #[test]
    fn test_unescape_unterminated_unicode() {
        assert!(unescape_string("\\u{123").is_err());
    }

    #[test]
    fn test_unescape_invalid_unicode_codepoint() {
        assert!(unescape_string("\\u{110000}").is_err()); // Beyond valid Unicode range
    }
}
