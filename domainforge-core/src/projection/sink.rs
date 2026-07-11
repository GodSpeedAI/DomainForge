//! Shared artifact sink for every projection family.
//!
//! Where emitted artifacts go: a directory (CLI) or an in-memory map keyed by
//! relative path (language bindings, wasm-safe — no filesystem access). Every
//! renderer writes through this one type so the CLI and the binding surfaces
//! stay byte-for-byte identical.

use std::collections::BTreeMap;
use std::path::Path;

/// Write one artifact under the output root, path-traversal-safe.
pub enum ArtifactSink<'a> {
    Dir(&'a Path),
    Memory {
        prefix: String,
        map: &'a mut BTreeMap<String, String>,
    },
}

impl ArtifactSink<'_> {
    pub fn write(&mut self, rel: &str, content: &str) -> Result<(), String> {
        match self {
            ArtifactSink::Dir(root) => write_artifact(root, rel, content),
            ArtifactSink::Memory { prefix, map } => {
                map.insert(format!("{prefix}{rel}"), content.to_string());
                Ok(())
            }
        }
    }
}

pub(crate) fn write_artifact(root: &Path, rel: &str, content: &str) -> Result<(), String> {
    let abs = root.join(rel);
    crate::projection::protobuf::validate_output_path(root, &abs)?;
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    std::fs::write(&abs, content).map_err(|e| format!("failed to write {}: {e}", abs.display()))
}
