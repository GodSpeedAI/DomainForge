//! Mise renderer: language runtimes, toolchains, package managers, and
//! developer CLIs only. Application libraries are never duplicated here —
//! they stay owned by the native manifest/lockfile (see `deps.rs`).

use super::ir::CellIr;
use serde_json::json;

pub fn render(ir: &CellIr) -> Vec<(String, String)> {
    let mut tools: Vec<(String, String)> = ir
        .runtimes
        .iter()
        .map(|r| (r.name.clone(), r.version.clone()))
        .chain(ir.tools.iter().map(|t| (t.name.clone(), t.version.clone())))
        .collect();
    tools.sort_by(|a, b| a.0.cmp(&b.0));

    let mut toml = String::from("[tools]\n");
    for (name, version) in &tools {
        toml.push_str(&format!("{name} = \"{version}\"\n"));
    }

    if !ir.dependency_sets.is_empty() {
        toml.push_str("\n[tasks.install]\nrun = [\n");
        for depset in &ir.dependency_sets {
            toml.push_str(&format!("  \"{}\",\n", escape_toml(&depset.install)));
        }
        toml.push_str("]\n");
    }

    let probes: Vec<&str> = ir
        .evidence_probes
        .iter()
        .map(|p| p.command.as_str())
        .collect();
    if !probes.is_empty() {
        toml.push_str("\n[tasks.prove-environment]\nrun = [\n");
        for probe in &probes {
            if probe.starts_with("metric:") {
                continue;
            }
            toml.push_str(&format!("  \"{}\",\n", escape_toml(probe)));
        }
        toml.push_str("]\n");
    }

    let manifest = json!({
        "schema": "domainforge-cell-mise-manifest/v1",
        "cell_id": ir.identity.cell_id,
        "profile": { "id": ir.profile_id, "version": ir.profile_version },
        "files": ["mise/mise.toml"],
    });

    vec![
        ("mise/mise.toml".to_string(), toml),
        (
            "mise/projection-manifest.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
        ),
    ]
}

fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
