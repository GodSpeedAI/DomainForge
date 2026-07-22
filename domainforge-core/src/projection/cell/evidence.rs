//! Evidence renderer: environment-proof probes and the shape of an
//! environment-proof event. No settlement contract in v1 — there is no
//! substrate for it in this repository (see known limitations in
//! `docs/cell-environment-projections.md`).

use super::ir::CellIr;
use serde_json::json;

pub fn render(ir: &CellIr) -> Vec<(String, String)> {
    let probes: Vec<_> = ir
        .evidence_probes
        .iter()
        .map(|p| json!({ "id": p.id, "command": p.command, "source": p.source }))
        .collect();
    let proof_contract = json!({
        "schema": "domainforge-cell-evidence/v1",
        "cell_id": ir.identity.cell_id,
        "profile": { "id": ir.profile_id, "version": ir.profile_version },
        "probes": probes,
    });

    let event_schema = json!({
        "schema": "domainforge-cell-evidence/v1",
        "type": "object",
        "properties": {
            "cell_id": { "type": "string" },
            "probe_id": { "type": "string" },
            "command": { "type": "string" },
            "exit_code": { "type": "integer" },
            "observed_version": { "type": ["string", "null"] },
            "timestamp": { "type": "string", "format": "date-time" },
        },
        "required": ["cell_id", "probe_id", "command", "exit_code", "timestamp"],
    });

    vec![
        (
            "evidence/environment-proof-contract.json".to_string(),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&proof_contract).unwrap()
            ),
        ),
        (
            "evidence/event-schema.json".to_string(),
            format!("{}\n", serde_json::to_string_pretty(&event_schema).unwrap()),
        ),
    ]
}
