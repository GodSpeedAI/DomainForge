//! Comment extraction and preservation utilities.
//!
//! This module handles extracting comments from source code and associating
//! them with their corresponding declarations.

use std::collections::BTreeMap;

/// A comment extracted from source code.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// The comment text (without the leading //)
    pub text: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Whether this is a trailing comment (on same line as code)
    pub is_trailing: bool,
}

/// Extracts comments from source code, preserving their positions.
///
/// Returns a map of line numbers to comments, where the line number
/// indicates where the comment appears in the source.
pub fn extract_comments(source: &str) -> BTreeMap<usize, Vec<Comment>> {
    let mut comments: BTreeMap<usize, Vec<Comment>> = BTreeMap::new();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Check for line comment
        if let Some(comment_start) = trimmed.find("//") {
            let before_comment = &line[..line.find("//").unwrap_or(0)];
            let is_trailing = !before_comment.trim().is_empty();

            let comment_text = trimmed[comment_start + 2..].trim();

            comments.entry(line_num).or_default().push(Comment {
                text: comment_text.to_string(),
                line: line_num,
                is_trailing,
            });
        }
    }

    comments
}

/// Groups comments with declarations.
///
/// Associates each leading comment block with the next declaration's line.
/// Returns a map from declaration start line to its leading comments.
pub fn associate_comments_with_lines(
    comments: &BTreeMap<usize, Vec<Comment>>,
    declaration_lines: &[usize],
) -> BTreeMap<usize, Vec<Comment>> {
    let mut associated: BTreeMap<usize, Vec<Comment>> = BTreeMap::new();

    // Sort declaration lines
    let mut decl_lines = declaration_lines.to_vec();
    decl_lines.sort();

    // For each declaration, find leading comments
    for &decl_line in &decl_lines {
        let mut leading_comments = Vec::new();

        // Look backwards from the declaration for comment lines
        let mut check_line = decl_line.saturating_sub(1);
        while check_line > 0 {
            if let Some(line_comments) = comments.get(&check_line) {
                // Only consider non-trailing comments as leading
                let non_trailing: Vec<_> = line_comments
                    .iter()
                    .filter(|c| !c.is_trailing)
                    .cloned()
                    .collect();

                if non_trailing.is_empty() {
                    break;
                }

                // Insert at beginning to maintain order
                for c in non_trailing.into_iter().rev() {
                    leading_comments.insert(0, c);
                }
                check_line -= 1;
            } else {
                break;
            }
        }

        if !leading_comments.is_empty() {
            associated.insert(decl_line, leading_comments);
        }
    }

    associated
}

/// Represents a source file with comments tracked separately.
#[derive(Debug, Clone)]
pub struct CommentedSource {
    /// The original source code
    pub source: String,
    /// Map of line numbers to comments
    pub comments: BTreeMap<usize, Vec<Comment>>,
    /// Leading comments to output before the file header
    pub file_header_comments: Vec<Comment>,
}

impl CommentedSource {
    /// Parse source code and extract comments.
    pub fn new(source: &str) -> Self {
        let comments = extract_comments(source);

        // Find the first non-comment, non-empty line for header comments
        let mut file_header_comments = Vec::new();
        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("//") {
                if let Some(line_comments) = comments.get(&line_num) {
                    file_header_comments.extend(line_comments.iter().cloned());
                }
            } else {
                // Found first non-comment line, stop
                break;
            }
        }

        Self {
            source: source.to_string(),
            comments,
            file_header_comments,
        }
    }

    /// Get leading comments for a specific line.
    pub fn leading_comments_for(&self, line: usize) -> Vec<&Comment> {
        // Look at previous lines for consecutive comments
        let mut result = Vec::new();
        let mut check_line = line.saturating_sub(1);

        while check_line > 0 {
            if let Some(comments) = self.comments.get(&check_line) {
                let non_trailing: Vec<_> = comments.iter().filter(|c| !c.is_trailing).collect();

                if non_trailing.is_empty() {
                    break;
                }

                for c in non_trailing.into_iter().rev() {
                    result.insert(0, c);
                }
                check_line -= 1;
            } else {
                break;
            }
        }

        result
    }

    /// Check if there are any comments in the source.
    pub fn has_comments(&self) -> bool {
        !self.comments.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_line_comments() {
        let source = r#"
// This is a comment
Entity "Foo"
"#;
        let comments = extract_comments(source);
        assert_eq!(comments.len(), 1);
        let line2_comments = comments.get(&2).unwrap();
        assert_eq!(line2_comments[0].text, "This is a comment");
        assert!(!line2_comments[0].is_trailing);
    }

    #[test]
    fn test_extract_trailing_comments() {
        let source = r#"Entity "Foo" // trailing comment
"#;
        let comments = extract_comments(source);
        let line1_comments = comments.get(&1).unwrap();
        assert_eq!(line1_comments[0].text, "trailing comment");
        assert!(line1_comments[0].is_trailing);
    }

    #[test]
    fn test_multiple_comments() {
        let source = r#"
// Comment 1
// Comment 2
Entity "Foo"
"#;
        let comments = extract_comments(source);
        assert_eq!(comments.len(), 2);
        assert!(comments.contains_key(&2));
        assert!(comments.contains_key(&3));
    }

    #[test]
    fn test_commented_source() {
        let source = r#"// File header comment
Entity "Foo"
"#;
        let cs = CommentedSource::new(source);
        assert_eq!(cs.file_header_comments.len(), 1);
        assert_eq!(cs.file_header_comments[0].text, "File header comment");
    }
}
