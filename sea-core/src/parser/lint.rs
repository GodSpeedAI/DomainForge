use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LintError {
    #[error("Keyword collision: '{name}' is a reserved keyword. Hint: {hint}")]
    KeywordCollision { name: String, hint: String },
}

pub struct Linter {
    keywords: HashSet<String>,
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter {
    pub fn new() -> Self {
        Self {
            keywords: ["Entity", "Resource", "Flow", "Policy", "Unit", "Dimension"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn check_identifier(&self, name: &str, quoted: bool) -> Result<(), LintError> {
        if !quoted && self.keywords.contains(name) {
            return Err(LintError::KeywordCollision {
                name: name.to_string(),
                hint: format!("Use quoted identifier: \"{}\"", name),
            });
        }
        Ok(())
    }
}
