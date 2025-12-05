//! Fuzzy string matching for "did you mean?" suggestions.
//!
//! Implements the Levenshtein distance algorithm (Wagner-Fischer) to provide
//! helpful suggestions when users make typos in entity names, resource names,
//! or other identifiers.
/// Calculate the Levenshtein distance between two strings
///
/// Uses the Wagner-Fischer dynamic programming algorithm with O(mn) time complexity
/// and O(min(m,n)) space complexity.
///
/// # Examples
///
/// ```
/// use sea_core::error::fuzzy::levenshtein_distance;
///
/// assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
/// assert_eq!(levenshtein_distance("Saturday", "Sunday"), 3);
/// assert_eq!(levenshtein_distance("", "abc"), 3);
/// ```
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use the shorter string for the columns to minimize space
    let (short, long, short_len) = if a_len < b_len {
        (a, b, a_len)
    } else {
        (b, a, b_len)
    };

    // We only need two rows for the dynamic programming table
    let mut prev_row: Vec<usize> = (0..=short_len).collect();
    let mut curr_row: Vec<usize> = vec![0; short_len + 1];

    let short_chars: Vec<char> = short.chars().collect();

    for (i, long_char) in long.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, &short_char) in short_chars.iter().enumerate() {
            let cost = if long_char == short_char { 0 } else { 1 };

            curr_row[j + 1] = std::cmp::min(
                std::cmp::min(
                    curr_row[j] + 1,     // insertion
                    prev_row[j + 1] + 1, // deletion
                ),
                prev_row[j] + cost, // substitution
            );
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[short_len]
}

/// Find similar strings within a given edit distance threshold
///
/// Returns all candidates that are within `threshold` edits of the target string,
/// sorted by distance (closest first).
///
/// # Arguments
///
/// * `target` - The string to match against
/// * `candidates` - Potential matches to consider
/// * `threshold` - Maximum edit distance to include (default recommendation: 2)
///
/// # Examples
///
/// ```
/// use sea_core::error::fuzzy::suggest_similar;
///
/// let candidates = vec![
///     "Warehouse".to_string(),
///     "Factory".to_string(),
///     "Supplier".to_string(),
/// ];
///
/// let suggestions = suggest_similar("Warehous", &candidates, 2);
/// assert_eq!(suggestions, vec!["Warehouse"]);
/// ```
pub fn suggest_similar(target: &str, candidates: &[String], threshold: usize) -> Vec<String> {
    let mut matches: Vec<(String, usize)> = candidates
        .iter()
        .filter_map(|candidate| {
            let distance = levenshtein_distance(target, candidate);
            if distance <= threshold {
                Some((candidate.clone(), distance))
            } else {
                None
            }
        })
        .collect();

    // Sort by distance (closest first), then alphabetically for ties
    matches.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    matches.into_iter().map(|(s, _)| s).collect()
}

/// Find the single best match for a target string
///
/// Returns the closest match if it's within the threshold, otherwise None.
/// If multiple candidates have the same distance, returns the first alphabetically.
///
/// # Arguments
///
/// * `target` - The string to match against
/// * `candidates` - Potential matches to consider
/// * `threshold` - Maximum edit distance to accept (default recommendation: 2)
///
/// # Examples
///
/// ```
/// use sea_core::error::fuzzy::find_best_match;
///
/// let candidates = vec![
///     "Warehouse".to_string(),
///     "Factory".to_string(),
/// ];
///
/// assert_eq!(find_best_match("Warehous", &candidates, 2), Some("Warehouse".to_string()));
/// assert_eq!(find_best_match("XYZ", &candidates, 2), None);
/// ```
pub fn find_best_match(target: &str, candidates: &[String], threshold: usize) -> Option<String> {
    suggest_similar(target, candidates, threshold)
        .into_iter()
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_distance_empty() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_levenshtein_distance_single_char() {
        assert_eq!(levenshtein_distance("a", "b"), 1);
        assert_eq!(levenshtein_distance("a", "a"), 0);
    }

    #[test]
    fn test_levenshtein_distance_classic_examples() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("Saturday", "Sunday"), 3);
        assert_eq!(levenshtein_distance("rosettacode", "raisethysword"), 8);
    }

    #[test]
    fn test_levenshtein_distance_case_sensitive() {
        assert_eq!(levenshtein_distance("Hello", "hello"), 1);
        assert_eq!(levenshtein_distance("HELLO", "hello"), 5);
    }

    #[test]
    fn test_suggest_similar_exact_match() {
        let candidates = vec!["Warehouse".to_string(), "Factory".to_string()];
        let suggestions = suggest_similar("Warehouse", &candidates, 2);
        assert_eq!(suggestions, vec!["Warehouse"]);
    }

    #[test]
    fn test_suggest_similar_typo() {
        let candidates = vec![
            "Warehouse".to_string(),
            "Factory".to_string(),
            "Supplier".to_string(),
        ];
        let suggestions = suggest_similar("Warehous", &candidates, 2);
        assert_eq!(suggestions, vec!["Warehouse"]);
    }

    #[test]
    fn test_suggest_similar_multiple_matches() {
        let candidates = vec![
            "Warehouse".to_string(),
            "Warehouses".to_string(),
            "Factory".to_string(),
        ];
        let suggestions = suggest_similar("Warehous", &candidates, 2);
        // Both Warehouse and Warehouses are within threshold
        assert!(suggestions.contains(&"Warehouse".to_string()));
        assert!(suggestions.contains(&"Warehouses".to_string()));
    }

    #[test]
    fn test_suggest_similar_no_matches() {
        let candidates = vec!["Warehouse".to_string(), "Factory".to_string()];
        let suggestions = suggest_similar("XYZ", &candidates, 2);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_suggest_similar_sorted_by_distance() {
        let candidates = vec![
            "Warehouse".to_string(),
            "Warehouses".to_string(),
            "Ware".to_string(),
        ];
        let suggestions = suggest_similar("War", &candidates, 5);
        // "Ware" (dist 1), "Warehouse" (dist 6), "Warehouses" (dist 7).
        // With threshold 5, only "Ware" is expected.
        assert_eq!(suggestions, vec!["Ware"]);
    }

    #[test]
    fn test_find_best_match() {
        let candidates = vec![
            "Warehouse".to_string(),
            "Factory".to_string(),
            "Supplier".to_string(),
        ];
        assert_eq!(
            find_best_match("Warehous", &candidates, 2),
            Some("Warehouse".to_string())
        );
        assert_eq!(find_best_match("XYZ", &candidates, 2), None);
    }

    #[test]
    fn test_find_best_match_tie_alphabetical() {
        let candidates = vec!["Apple".to_string(), "Apples".to_string()];
        // Both are distance 1 from "Appl", should return alphabetically first
        let result = find_best_match("Appl", &candidates, 2);
        assert_eq!(result, Some("Apple".to_string()));
    }

    #[test]
    fn test_unicode_support() {
        assert_eq!(levenshtein_distance("café", "cafe"), 1);
        assert_eq!(levenshtein_distance("北京", "北京"), 0);
        assert_eq!(levenshtein_distance("北京", "上海"), 2);
    }
}
