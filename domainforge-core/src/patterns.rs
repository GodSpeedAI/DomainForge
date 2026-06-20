use crate::ConceptId;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    id: ConceptId,
    name: String,
    namespace: String,
    regex: String,
    #[serde(skip)]
    #[serde(default = "Pattern::default_compiled")]
    compiled: OnceCell<Regex>,
}

impl Pattern {
    pub fn new(
        name: impl Into<String>,
        namespace: impl Into<String>,
        regex: impl Into<String>,
    ) -> Result<Self, String> {
        let name = name.into();
        let namespace = namespace.into();
        let regex_string = regex.into();

        // Validate regex eagerly so parse-time errors are surfaced
        Regex::new(&regex_string)
            .map_err(|e| format!("Invalid regex for pattern '{}': {}", name, e))?;

        Ok(Self {
            id: ConceptId::from_concept(&namespace, &name),
            name,
            namespace,
            regex: regex_string,
            compiled: OnceCell::new(),
        })
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn regex(&self) -> &str {
        &self.regex
    }

    pub fn is_match(&self, candidate: &str) -> Result<bool, String> {
        let compiled = self
            .compiled
            .get_or_try_init(|| Regex::new(&self.regex).map_err(|e| e.to_string()))?
            .clone();

        Ok(compiled.is_match(candidate))
    }

    fn default_compiled() -> OnceCell<Regex> {
        OnceCell::new()
    }
}
