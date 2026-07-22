//! Devbox renderer: OS packages and native libraries only. Runtimes and
//! tools never appear here — they are Mise's job (see `mise.rs`).

use super::ir::CellIr;
use serde_json::{json, Map, Value};

/// Cross-ecosystem `SystemDependency` names that don't match their nixpkgs
/// attribute name (verified against real `devbox generate`; `cacert` is the
/// nixpkgs name for what SEA authors write as `ca-certificates`). Consulted
/// only when no `[devbox.packages]` override already picked a realization.
fn nixpkgs_alias(name: &str) -> &str {
    match name {
        "ca-certificates" => "cacert",
        other => other,
    }
}

pub fn render(ir: &CellIr) -> Vec<(String, String)> {
    let mut packages: Vec<String> = ir
        .system_dependencies
        .iter()
        .map(|d| {
            d.realized_package.clone().unwrap_or_else(|| {
                let pkg = nixpkgs_alias(&d.name);
                match &d.version {
                    Some(v) => format!("{pkg}@{v}"),
                    None => format!("{pkg}@latest"),
                }
            })
        })
        .collect();
    for service in &ir.services {
        if let Some(v) = &service.version {
            packages.push(format!("{}@{}", service.name, v));
        }
    }
    packages.sort();
    packages.dedup();

    let mut env: Map<String, Value> = Map::new();
    env.insert("DOMAINFORGE_CELL".to_string(), json!(ir.identity.cell_id));

    let doc = json!({
        "packages": packages,
        "env": env,
    });
    let manifest = json!({
        "schema": "domainforge-cell-devbox-manifest/v1",
        "cell_id": ir.identity.cell_id,
        "profile": { "id": ir.profile_id, "version": ir.profile_version },
        "files": ["devbox/devbox.json"],
    });

    vec![
        (
            "devbox/devbox.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&doc).unwrap()),
        ),
        (
            "devbox/projection-manifest.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
        ),
    ]
}
