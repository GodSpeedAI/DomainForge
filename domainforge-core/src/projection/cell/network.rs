//! Network renderer: "no declared route, no packet." Allow-rules are
//! derived only from declared `NetworkFlow`s; reachability (this file) stays
//! distinct from authorization (`authority.rs`).

use super::ir::CellIr;
use serde_json::json;

pub fn render(ir: &CellIr) -> Vec<(String, String)> {
    let endpoints: Vec<_> = ir
        .endpoints
        .iter()
        .map(|e| json!({ "name": e.name, "host": e.host, "port": e.port, "protocol": e.protocol }))
        .collect();
    let flows: Vec<_> = ir
        .network_flows
        .iter()
        .map(|f| {
            json!({
                "name": f.name,
                "from": f.from_ref,
                "to": f.to_ref,
                "operation": f.operation,
                "credential": f.credential,
            })
        })
        .collect();

    let doc = json!({
        "schema": "domainforge-cell-network/v1",
        "cell_id": ir.identity.cell_id,
        "default": ir.platform.network_default,
        "endpoints": endpoints,
        "allowed_flows": flows,
    });

    vec![(
        "sandbox/network.json".to_string(),
        format!("{}\n", serde_json::to_string_pretty(&doc).unwrap()),
    )]
}
