//! Sandbox renderer: conservative-by-default isolation template. This repo
//! only generates the template; runtime activation is an external runner's
//! job (activation mode, like Devbox/Dagger — see
//! `docs/projection-families.md`).

use super::ir::CellIr;
use serde_json::json;

const SCHEMA: &str = "domainforge-cell-sandbox/v1";

pub fn render(ir: &CellIr) -> Vec<(String, String)> {
    let template = json!({
        "schema": SCHEMA,
        "cell_id": ir.identity.cell_id,
        "architecture": ir.platform.architecture,
        "runtime_user": ir.platform.runtime_user,
        "network_default": ir.platform.network_default,
    });

    let resources = json!({
        "schema": SCHEMA,
        "cpu": ir.resources.cpu,
        "memory_mb": ir.resources.memory_mb,
        "disk_mb": ir.resources.disk_mb,
    });

    let mounts: Vec<_> = ir
        .mounts
        .iter()
        .map(|m| json!({ "name": m.name, "source": m.source, "target": m.target, "mode": m.mode }))
        .collect();
    let mounts_doc = json!({ "schema": SCHEMA, "mounts": mounts });

    let lifecycle = json!({
        "schema": SCHEMA,
        "timeout_seconds": ir.resources.timeout_seconds,
        "snapshot_after_build": false,
    });

    vec![
        (
            "sandbox/template.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&template).unwrap()),
        ),
        (
            "sandbox/resources.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&resources).unwrap()),
        ),
        (
            "sandbox/mounts.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&mounts_doc).unwrap()),
        ),
        (
            "sandbox/lifecycle.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&lifecycle).unwrap()),
        ),
    ]
}
