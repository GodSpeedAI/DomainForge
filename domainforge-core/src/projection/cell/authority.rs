//! Authority renderer: SEA-Forge-compatible mutation/credential contracts.
//! Decisions reuse the `Allow`/`Deny`/`Escalate` vocabulary from
//! `crate::authority::types::FinalDecision` so downstream authority tooling
//! reads one consistent decision language across projections.

use super::ir::CellIr;
use serde_json::json;

fn mutation_decision(mutation: &str) -> &'static str {
    match mutation {
        "deny" => "Deny",
        "allow" => "Allow",
        _ => "Escalate",
    }
}

pub fn render(ir: &CellIr) -> Vec<(String, String)> {
    let dependency_mutations: Vec<_> = ir
        .dependency_sets
        .iter()
        .map(|d| {
            json!({
                "dependency_set": d.name,
                "manifest_change": mutation_decision(&d.mutation),
                "lockfile_change": mutation_decision(&d.mutation),
            })
        })
        .collect();

    let denying_policies: Vec<&str> = ir
        .policies
        .iter()
        .filter(|p| p.modality == "Prohibition")
        .map(|p| p.name.as_str())
        .collect();

    let mutation_policy = json!({
        "schema": "domainforge-cell-authority/v1",
        "cell_id": ir.identity.cell_id,
        "dependency_mutations": dependency_mutations,
        "system_dependency_change": "Escalate",
        "mount_change": "Escalate",
        "network_change": "Escalate",
        "prohibition_policies": denying_policies,
        "unsafe_override_active": ir.unsafe_override.is_some(),
    });

    let runtime_upgrade_policy = json!({
        "schema": "domainforge-cell-authority/v1",
        "cell_id": ir.identity.cell_id,
        "runtime_upgrade": "Escalate",
        "runtimes": ir.runtimes.iter().map(|r| &r.name).collect::<Vec<_>>(),
    });

    let credential_contracts: Vec<_> = ir
        .credentials
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "provider": c.provider,
                "scope": c.scope,
                "delivery": c.delivery,
                "credential_issuance": "Escalate",
            })
        })
        .collect();
    let credential_doc = json!({
        "schema": "domainforge-cell-authority/v1",
        "cell_id": ir.identity.cell_id,
        "credentials": credential_contracts,
    });

    vec![
        (
            "authority/dependency-mutation-policy.json".to_string(),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&mutation_policy).unwrap()
            ),
        ),
        (
            "authority/runtime-upgrade-policy.json".to_string(),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&runtime_upgrade_policy).unwrap()
            ),
        ),
        (
            "authority/credential-contracts.json".to_string(),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&credential_doc).unwrap()
            ),
        ),
    ]
}
