use globset::{Glob, GlobSet, GlobSetBuilder};
use globwalk::{GlobWalkerBuilder, WalkError};
use serde::Deserialize;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_FILE_NAME: &str = ".sea-registry.toml";

#[derive(Debug, Clone)]
pub struct NamespaceBinding {
    pub namespace: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct NamespaceRegistry {
    root: PathBuf,
    default_namespace: String,
    entries: Vec<CompiledRule>,
}

#[derive(Debug, Clone)]
struct CompiledRule {
    namespace: String,
    matcher: GlobSet,
    patterns: Vec<String>,
    // Length of literal prefix of the pattern before any wildcard or special glob char
    literal_prefix_len: usize,
}

#[derive(Debug, Deserialize)]
struct RawRegistry {
    version: u8,
    #[serde(default)]
    default_namespace: Option<String>,
    #[serde(default)]
    namespaces: Vec<RawNamespace>,
}

#[derive(Debug, Deserialize)]
struct RawNamespace {
    namespace: String,
    patterns: Vec<String>,
}

#[derive(Debug)]
pub enum RegistryError {
    Io(std::io::Error),
    ParseToml(toml::de::Error),
    InvalidVersion(u8),
    MissingNamespaces,
    MissingPatterns {
        namespace: String,
    },
    InvalidPattern {
        namespace: String,
        pattern: String,
        source: globset::Error,
    },
    InvalidGlob {
        pattern: String,
        message: String,
    },
    Walk(WalkError),
    Conflict {
        path: PathBuf,
        existing: String,
        requested: String,
    },
    Ambiguous {
        path: PathBuf,
        namespaces: Vec<String>,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Io(err) => write!(f, "IO error: {}", err),
            RegistryError::ParseToml(err) => write!(f, "Failed to parse registry: {}", err),
            RegistryError::InvalidVersion(version) => {
                write!(f, "Unsupported registry version {}", version)
            }
            RegistryError::MissingNamespaces => {
                write!(f, "Registry must declare at least one namespace")
            }
            RegistryError::MissingPatterns { namespace } => {
                write!(
                    f,
                    "Namespace '{}' must provide at least one pattern",
                    namespace
                )
            }
            RegistryError::InvalidPattern {
                namespace,
                pattern,
                source,
            } => write!(
                f,
                "Invalid pattern '{}' for namespace '{}': {}",
                pattern, namespace, source
            ),
            RegistryError::InvalidGlob { pattern, message } => {
                write!(f, "Failed to build glob '{}': {}", pattern, message)
            }
            RegistryError::Walk(err) => write!(f, "Failed to walk globbed files: {}", err),
            RegistryError::Ambiguous { path, namespaces } => write!(
                f,
                "File '{}' matched multiple namespaces: {}",
                path.display(),
                namespaces.join(", "),
            ),
            RegistryError::Conflict {
                path,
                existing,
                requested,
            } => write!(
                f,
                "File '{}' matched multiple namespaces: '{}' and '{}'",
                path.display(),
                existing,
                requested
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<std::io::Error> for RegistryError {
    fn from(value: std::io::Error) -> Self {
        RegistryError::Io(value)
    }
}

impl From<toml::de::Error> for RegistryError {
    fn from(value: toml::de::Error) -> Self {
        RegistryError::ParseToml(value)
    }
}

impl From<WalkError> for RegistryError {
    fn from(value: WalkError) -> Self {
        RegistryError::Walk(value)
    }
}

impl NamespaceRegistry {
    pub fn new_empty(root: PathBuf) -> Result<Self, RegistryError> {
        let canonical_root = root.canonicalize()?;
        Ok(Self {
            root: canonical_root,
            default_namespace: "default".to_string(),
            entries: Vec::new(),
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let registry_path = path.as_ref();
        let contents = fs::read_to_string(registry_path)?;
        let raw: RawRegistry = toml::from_str(&contents)?;

        if raw.version != 1 {
            return Err(RegistryError::InvalidVersion(raw.version));
        }

        if raw.namespaces.is_empty() {
            return Err(RegistryError::MissingNamespaces);
        }

        let root = registry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()?;

        let default_namespace = raw
            .default_namespace
            .unwrap_or_else(|| "default".to_string());

        let mut entries = Vec::with_capacity(raw.namespaces.len());
        for ns in raw.namespaces {
            if ns.patterns.is_empty() {
                return Err(RegistryError::MissingPatterns {
                    namespace: ns.namespace,
                });
            }

            let mut builder = GlobSetBuilder::new();
            for pattern in &ns.patterns {
                let glob = Glob::new(pattern).map_err(|source| RegistryError::InvalidPattern {
                    namespace: ns.namespace.clone(),
                    pattern: pattern.clone(),
                    source,
                })?;
                builder.add(glob);
            }
            let matcher = builder
                .build()
                .map_err(|source| RegistryError::InvalidPattern {
                    namespace: ns.namespace.clone(),
                    pattern: ns.patterns.join(","),
                    source,
                })?;

            // compute the longest literal prefix across patterns for this namespace rule
            let mut literal_prefix_len = 0usize;
            for p in &ns.patterns {
                let mut len = 0usize;
                for ch in p.chars() {
                    // stop at glob meta characters
                    if ch == '*'
                        || ch == '?'
                        || ch == '['
                        || ch == '{'
                        || ch == ']'
                        || ch == '}'
                        || ch == '\\'
                    {
                        break;
                    }
                    len += ch.len_utf8();
                }
                if len > literal_prefix_len {
                    literal_prefix_len = len;
                }
            }

            entries.push(CompiledRule {
                namespace: ns.namespace,
                matcher,
                patterns: ns.patterns,
                literal_prefix_len,
            });
        }

        Ok(Self {
            root,
            default_namespace,
            entries,
        })
    }

    pub fn discover(start: impl AsRef<Path>) -> Result<Option<Self>, RegistryError> {
        let mut current = start.as_ref();
        let path_buf;
        if current.is_file() {
            path_buf = current
                .parent()
                .map(|p| {
                    if p.as_os_str().is_empty() {
                        PathBuf::from(".")
                    } else {
                        p.to_path_buf()
                    }
                })
                .unwrap_or_else(|| PathBuf::from("."));
            current = &path_buf;
        }

        let mut dir = current.canonicalize()?;
        loop {
            let candidate = dir.join(REGISTRY_FILE_NAME);
            if candidate.is_file() {
                return Ok(Some(Self::from_file(candidate)?));
            }

            if !dir.pop() {
                break;
            }
        }

        Ok(None)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    pub fn namespace_for(&self, path: impl AsRef<Path>) -> Option<&str> {
        self.namespace_for_with_options(path, false).ok()
    }

    pub fn namespace_for_with_options(
        &self,
        path: impl AsRef<Path>,
        fail_on_ambiguity: bool,
    ) -> Result<&str, RegistryError> {
        let mut absolute = path.as_ref().to_path_buf();
        if !absolute.is_absolute() {
            absolute = self.root.join(absolute);
        }
        let absolute = absolute.canonicalize().ok().unwrap_or(absolute);
        let relative = match absolute.strip_prefix(&self.root) {
            Ok(rel) => rel,
            Err(_) => return Ok(self.default_namespace.as_str()),
        };
        let normalized = normalize_path(relative);

        // Collect all matches and pick the best candidate(s) using longest literal
        // prefix precedence. If more than one candidate exist with equal prefix
        // length and 'fail_on_ambiguity' is true, return an error.
        let mut candidates: Vec<(&CompiledRule, usize)> = vec![];
        let mut best_len: usize = 0;
        for entry in &self.entries {
            if entry.matcher.is_match(normalized.as_str()) {
                let len = entry.literal_prefix_len;
                if len > best_len {
                    candidates.clear();
                    candidates.push((entry, len));
                    best_len = len;
                } else if len == best_len {
                    candidates.push((entry, len));
                }
            }
        }

        if candidates.is_empty() {
            return Ok(self.default_namespace.as_str());
        }

        if candidates.len() == 1 {
            return Ok(candidates[0].0.namespace.as_str());
        }

        // multiple candidates with equal prefix length
        if fail_on_ambiguity {
            let mut names: Vec<String> = candidates
                .into_iter()
                .map(|(e, _)| e.namespace.clone())
                .collect();
            names.sort();
            return Err(RegistryError::Ambiguous {
                path: path.as_ref().to_path_buf(),
                namespaces: names,
            });
        }

        // alphabetical fallback determination
        let mut chosen_entry: &CompiledRule = candidates[0].0;
        for (entry, _) in candidates.into_iter().skip(1) {
            if entry.namespace < chosen_entry.namespace {
                chosen_entry = entry;
            }
        }

        Ok(chosen_entry.namespace.as_str())
    }

    pub fn resolve_files(&self) -> Result<Vec<NamespaceBinding>, RegistryError> {
        self.resolve_files_with_options(false)
    }

    pub fn resolve_files_with_options(
        &self,
        fail_on_ambiguity: bool,
    ) -> Result<Vec<NamespaceBinding>, RegistryError> {
        // Map a file to (namespace, literal_prefix_len) and pick the best namespace for a file
        let mut matches: HashMap<PathBuf, (String, usize)> = HashMap::new();

        for entry in &self.entries {
            for pattern in &entry.patterns {
                let walker = GlobWalkerBuilder::from_patterns(&self.root, &[pattern.as_str()])
                    .follow_links(true)
                    .file_type(globwalk::FileType::FILE)
                    .build()
                    .map_err(|err| RegistryError::InvalidGlob {
                        pattern: pattern.clone(),
                        message: err.to_string(),
                    })?;

                for dir_entry in walker.into_iter() {
                    let dir_entry = dir_entry?;
                    let path = dir_entry.into_path();
                    let current_len = entry.literal_prefix_len;
                    match matches.entry(path.clone()) {
                        Entry::Vacant(v) => {
                            v.insert((entry.namespace.clone(), current_len));
                        }
                        Entry::Occupied(mut occupied) => {
                            let (ref existing_ns, existing_len) = occupied.get().clone();
                            if existing_ns == &entry.namespace {
                                // same namespace - nothing to do
                                continue;
                            }
                            if current_len > existing_len {
                                occupied.insert((entry.namespace.clone(), current_len));
                            } else if current_len == existing_len {
                                if fail_on_ambiguity {
                                    // collect ambiguous namespaces
                                    let mut conflicted = vec![existing_ns.clone()];
                                    conflicted.push(entry.namespace.clone());
                                    conflicted.sort();
                                    return Err(RegistryError::Ambiguous {
                                        path: path.clone(),
                                        namespaces: conflicted,
                                    });
                                }
                                // deterministic tie-breaker: choose alphabetically smallest namespace
                                if entry.namespace < *existing_ns {
                                    occupied.insert((entry.namespace.clone(), current_len));
                                }
                            }
                            // if current_len < existing_len -> keep existing namespace
                        }
                    }
                }
            }
        }

        let mut bindings: Vec<NamespaceBinding> = matches
            .into_iter()
            .map(|(path, (namespace, _len))| NamespaceBinding { path, namespace })
            .collect();
        bindings.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(bindings)
    }
}

fn normalize_path(path: &Path) -> String {
    let repr = path.to_string_lossy().replace('\\', "/");
    repr.trim_start_matches("./").to_string()
}

#[cfg(test)]
mod tests;
