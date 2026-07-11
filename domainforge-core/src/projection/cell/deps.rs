//! Dependency-set renderer: references native manifests/lockfiles, never
//! reproduces individual package entries (those are owned by the native
//! manifest and lockfile, not `.sea`).
//!
//! Lockfile handling is hardened: when a real `source_dir` is supplied the
//! declared `@lockfile` is resolved canonically and rejected if it is
//! absolute, contains `..`, escapes `source_dir` via symlink, isn't a regular
//! file, is missing, or exceeds the size ceiling (`CELL022`). In-memory
//! callers pass `source_dir = None` and lockfile hashing is skipped
//! (`lockfile_sha256` is emitted as null), matching the binding surface which
//! has no filesystem.

use super::ir::CellIr;
use serde_json::json;
use std::io::Read;
use std::path::{Component, Path};

/// Upper bound on a single lockfile read (16 MiB). Real lockfiles are far
/// smaller; anything larger is almost certainly a misconfigured `@lockfile`.
const MAX_LOCKFILE_BYTES: u64 = 16 * 1024 * 1024;

pub fn render(ir: &CellIr, source_dir: Option<&Path>) -> Result<Vec<(String, String)>, String> {
    let mut sets = serde_json::Map::new();
    for depset in &ir.dependency_sets {
        let lockfile_sha256 = match source_dir {
            None => None,
            Some(dir) => Some(hash_lockfile(dir, &depset.lockfile, &depset.name)?),
        };
        sets.insert(
            depset.name.clone(),
            json!({
                "ecosystem": depset.ecosystem,
                "manifest": depset.manifest,
                "lockfile": depset.lockfile,
                "lockfile_sha256": lockfile_sha256,
                "install": depset.install,
                "mutation": depset.mutation,
            }),
        );
    }
    let doc = json!({
        "schema": "domainforge-cell-dependency-sets/v1",
        "cell_id": ir.identity.cell_id,
        "dependency_sets": sets,
    });
    Ok(vec![(
        "dependencies/dependency-sets.json".to_string(),
        format!("{}\n", serde_json::to_string_pretty(&doc).unwrap()),
    )])
}

/// Resolve and hash a declared `@lockfile` under `source_dir` with
/// path-traversal / symlink-escape / size protection. All failure modes map
/// to `CELL022` so an invalid lockfile is surfaced rather than silently
/// hashed as null.
fn hash_lockfile(source_dir: &Path, declared: &str, depset_name: &str) -> Result<String, String> {
    let declared_path = Path::new(declared);
    if declared_path.is_absolute() {
        return Err(format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' must be a relative path (absolute paths are rejected)"
        ));
    }
    // Reject `..` and any non-normal components before touching the
    // filesystem. `CurDir` (`.`) is harmless and allowed.
    for comp in declared_path.components() {
        match comp {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' must not contain '..' or root components (path traversal rejected)"
                ));
            }
        }
    }

    let resolved = source_dir.join(declared_path);

    // Canonicalize both before the containment check so symlink escapes are
    // caught: if the lockfile resolves outside the canonical source dir, the
    // `starts_with` check fails.
    let canon_dir = source_dir.canonicalize().map_err(|e| {
        format!(
            "CELL022 DependencySet '{depset_name}' source dir '{}' is not readable: {e}",
            source_dir.display()
        )
    })?;
    let canon_target = resolved.canonicalize().map_err(|e| {
        format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' could not be resolved under '{}': {e}",
            source_dir.display()
        )
    })?;
    if !canon_target.starts_with(&canon_dir) {
        return Err(format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' resolves outside its source directory (symlink/traversal escape rejected)"
        ));
    }
    if !canon_target.is_file() {
        return Err(format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' is not a regular file"
        ));
    }

    // Bounded streaming read: cap at MAX_LOCKFILE_BYTES + 1 so an oversized
    // lockfile is rejected instead of being pulled fully into memory.
    let file = std::fs::File::open(&canon_target).map_err(|e| {
        format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' could not be opened: {e}"
        )
    })?;
    let mut reader = file.take(MAX_LOCKFILE_BYTES + 1);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| {
        format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' could not be read: {e}"
        )
    })?;
    if bytes.len() as u64 > MAX_LOCKFILE_BYTES {
        return Err(format!(
            "CELL022 DependencySet '{depset_name}' @lockfile '{declared}' exceeds the {MAX_LOCKFILE_BYTES} byte ceiling"
        ));
    }
    Ok(format!("sha256:{}", super::lock::sha256_hex(&bytes)))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::io::Write;

    fn minimal_ir(lockfile: &str) -> CellIr {
        use crate::projection::cell::ir::*;
        CellIr {
            identity: CellIdentity {
                namespace: "ns".into(),
                name: "C".into(),
                model_version: "0.0.0".into(),
                owner: None,
                cell_id: "cell:ns:C".into(),
            },
            profile_id: "p".into(),
            profile_version: "1".into(),
            platform: PlatformSpec {
                architecture: "x86_64".into(),
                runtime_user: "agent".into(),
                network_default: "deny".into(),
            },
            resources: ResourceSpec {
                cpu: 1,
                memory_mb: 1,
                disk_mb: 1,
                timeout_seconds: 1,
            },
            system_dependencies: vec![],
            runtimes: vec![],
            tools: vec![],
            dependency_sets: vec![DependencySetIr {
                id: "cell:depset:ns:app".into(),
                name: "app".into(),
                ecosystem: "rust".into(),
                manifest: "Cargo.toml".into(),
                lockfile: lockfile.into(),
                install: "cargo fetch --locked".into(),
                mutation: "escalate".into(),
                origin: Origin::Sea,
            }],
            services: vec![],
            mounts: vec![],
            endpoints: vec![],
            network_flows: vec![],
            credentials: vec![],
            policies: vec![],
            evidence_probes: vec![],
            unsafe_override: None,
        }
    }

    #[test]
    fn none_source_dir_skips_hashing() {
        let ir = minimal_ir("Cargo.lock");
        let out = render(&ir, None).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&out[0].1).unwrap();
        assert!(doc["dependency_sets"]["app"]["lockfile_sha256"].is_null());
    }

    #[test]
    fn valid_lockfile_is_hashed() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join("Cargo.lock")).unwrap();
        writeln!(f, "# lock").unwrap();
        let ir = minimal_ir("Cargo.lock");
        let out = render(&ir, Some(dir.path())).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&out[0].1).unwrap();
        let sha = doc["dependency_sets"]["app"]["lockfile_sha256"]
            .as_str()
            .unwrap();
        assert!(sha.starts_with("sha256:"));
    }

    #[test]
    fn absolute_lockfile_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ir = minimal_ir("/etc/passwd");
        let err = render(&ir, Some(dir.path())).unwrap_err();
        assert!(err.starts_with("CELL022"), "{err}");
        assert!(err.contains("relative"), "{err}");
    }

    #[test]
    fn traversal_lockfile_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ir = minimal_ir("../escape.lock");
        let err = render(&ir, Some(dir.path())).unwrap_err();
        assert!(err.starts_with("CELL022"), "{err}");
        assert!(err.contains("traversal"), "{err}");
    }

    #[test]
    fn missing_lockfile_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ir = minimal_ir("nope.lock");
        let err = render(&ir, Some(dir.path())).unwrap_err();
        assert!(err.starts_with("CELL022"), "{err}");
    }
}
