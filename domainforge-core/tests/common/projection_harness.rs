//! Cross-family projection test harness: the fixed timestamp that makes output
//! byte-deterministic, a directory-tree reader, and the two-run determinism
//! assertion every family reuses. Kept in one place so a new `<family>` test
//! copies a call, not a helper.

// Each test binary uses a different subset of these helpers; suppress the
// per-binary unused warnings that the `mod common;` pattern otherwise raises.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

/// Fixed `--created-at` value: identical inputs must yield identical bytes, so
/// every projection test pins the clock to this instant.
pub const FIXED_TS: &str = "2026-07-02T00:00:00+00:00";

/// Read a directory tree into a relative-path → contents map, with paths
/// normalized to `/` so two runs compare structurally regardless of platform.
pub fn read_tree(root: &Path) -> BTreeMap<String, String> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).expect("rel").to_string_lossy();
                out.insert(
                    rel.replace('\\', "/"),
                    std::fs::read_to_string(&path).expect("read file"),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// Project into two fresh tempdirs via `project` and assert the output trees
/// are byte-identical — the determinism gate every family must pass.
pub fn assert_byte_deterministic(project: impl Fn(&Path)) {
    let a = tempfile::tempdir().expect("tempdir");
    let b = tempfile::tempdir().expect("tempdir");
    project(a.path());
    project(b.path());
    assert_eq!(
        read_tree(a.path()),
        read_tree(b.path()),
        "projection output is not byte-deterministic across runs"
    );
}
