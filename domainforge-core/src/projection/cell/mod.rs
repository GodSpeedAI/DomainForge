//! Cell environment projection: `.sea` -> Cell IR -> {Devbox, Mise,
//! dependency-set, sandbox/network, authority, evidence, `cell.lock`}.
//!
//! Renderers only translate `ir::CellIr`; all semantics are decided once in
//! `ir::CellIr::build` (see that module's doc comment for the documented
//! deviation from the plan's Graph-centric architecture: this projection
//! reads the parsed `Ast` directly rather than `Graph`, because none of its
//! ten new declaration kinds are consumed by any other projection).

pub mod authority;
pub mod deps;
pub mod devbox;
pub mod evidence;
pub mod ir;
pub mod lock;
pub mod mise;
pub mod network;
pub mod overrides;
pub mod profiles;
pub mod sandbox;

use crate::formatter::{format as format_source, FormatConfig};
use crate::parser::ast::Ast;
use crate::projection::sink::ArtifactSink;
use ir::CellIr;
use overrides::CellOverrides;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

/// Which component directories `--only` selects. The IR and `cell.lock`
/// always cover every category regardless of this filter — `--only` is a
/// fast-iteration UX convenience, not a semantic subset.
#[derive(Debug, Clone, Copy)]
pub struct CellComponents {
    pub devbox: bool,
    pub mise: bool,
    pub dependencies: bool,
    pub sandbox: bool,
    pub authority: bool,
    pub evidence: bool,
}

impl Default for CellComponents {
    fn default() -> Self {
        CellComponents {
            devbox: true,
            mise: true,
            dependencies: true,
            sandbox: true,
            authority: true,
            evidence: true,
        }
    }
}

impl CellComponents {
    /// Parse a comma-separated `--only` value. Unknown component names fail
    /// closed with the list of accepted names.
    pub fn parse_only(spec: &str) -> Result<Self, String> {
        let mut c = CellComponents {
            devbox: false,
            mise: false,
            dependencies: false,
            sandbox: false,
            authority: false,
            evidence: false,
        };
        for part in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match part {
                "devbox" => c.devbox = true,
                "mise" => c.mise = true,
                "dependencies" => c.dependencies = true,
                "sandbox" => c.sandbox = true,
                "authority" => c.authority = true,
                "evidence" => c.evidence = true,
                other => {
                    return Err(format!(
                        "unknown --only component '{other}'; expected one of: devbox, mise, dependencies, sandbox, authority, evidence"
                    ))
                }
            }
        }
        Ok(c)
    }
}

/// Render the full cell-environment projection into `sink`. `source_dir` is
/// the directory containing the `.sea` file, used to resolve
/// `DependencySet` lockfile paths for hashing. Pass `None` for in-memory /
/// binding callers (lockfile hashing is skipped and `lockfile_sha256` is
/// emitted as null); pass `Some(dir)` from filesystem-backed CLI callers.
#[allow(clippy::too_many_arguments)]
pub fn emit(
    ast: &Ast,
    source: &str,
    model_ref: &str,
    source_dir: Option<&Path>,
    created_at: Option<String>,
    overrides: Option<&CellOverrides>,
    overrides_source: Option<&str>,
    components: &CellComponents,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ir = CellIr::build(ast, model_ref, &created_at, overrides)?;

    let canonical_source = format_source(source, FormatConfig::default())
        .map_err(|e| format!("failed to canonicalize source for cell.lock hashing: {e}"))?;

    let mut files: BTreeMap<String, String> = BTreeMap::new();
    if components.devbox {
        files.extend(devbox::render(&ir));
    }
    if components.mise {
        files.extend(mise::render(&ir));
    }
    if components.dependencies && !ir.dependency_sets.is_empty() {
        files.extend(deps::render(&ir, source_dir)?);
    }
    if components.sandbox {
        files.extend(sandbox::render(&ir));
        files.extend(network::render(&ir));
    }
    if components.authority {
        files.extend(authority::render(&ir));
    }
    if components.evidence {
        files.extend(evidence::render(&ir));
    }

    files.insert(
        "semantic/cell-ir.json".to_string(),
        format!("{}\n", serde_json::to_string_pretty(&ir).unwrap()),
    );
    files.insert(
        "semantic/canonical-model.json".to_string(),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "model_ref": model_ref,
                "namespace": ir.identity.namespace,
                "canonical_source": canonical_source,
            }))
            .unwrap()
        ),
    );

    // Computed from `files` before insertion, so every other emitted file is
    // captured by construction — there is no code path that can produce a
    // file the lock doesn't hash (the plan's CELL030 concern).
    let lock_doc = lock::build(
        &ir,
        &created_at,
        &canonical_source,
        overrides_source,
        &files,
    );
    files.insert("cell.lock".to_string(), lock_doc);
    files.insert("README.md".to_string(), build_readme(&ir, &files));

    let mut written = Vec::new();
    for (path, content) in &files {
        sink.write(path, content)?;
        written.push(path.clone());
    }
    Ok(written)
}

/// Binding surface: string in, path->content map out, no filesystem (mirrors
/// `devbox::project_devbox_in_memory`).
pub fn project_cell_in_memory(
    source: &str,
    model_ref: &str,
    created_at: Option<String>,
    overrides_source: Option<&str>,
    only: Option<&str>,
) -> Result<BTreeMap<String, String>, String> {
    let ast = crate::parser::parse(source).map_err(|e| format!("parse failed: {e}"))?;
    let overrides = overrides_source.map(overrides::parse).transpose()?;
    let components = match only {
        Some(spec) => CellComponents::parse_only(spec)?,
        None => CellComponents::default(),
    };
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(
        &ast,
        source,
        model_ref,
        None,
        created_at,
        overrides.as_ref(),
        overrides_source,
        &components,
        &mut sink,
    )?;
    Ok(map)
}

fn build_readme(ir: &CellIr, files: &BTreeMap<String, String>) -> String {
    let mut file_list: Vec<&String> = files.keys().collect();
    file_list.sort();
    let mut body = format!(
        "# {} — cell environment\n\n\
         Projected from a DomainForge `.sea` Cell declaration using profile \
         `{}` (v{}).\n\n\
         ## Generated files\n\n",
        ir.identity.name, ir.profile_id, ir.profile_version
    );
    for f in file_list {
        body.push_str(&format!("- `{f}`\n"));
    }
    body.push_str(
        "\n## Next steps\n\n\
         ```bash\n\
         devbox shell\n\
         mise install\n\
         mise run prove-environment\n\
         ```\n",
    );
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED_TS: &str = "2026-07-11T00:00:00+00:00";

    const SOURCE: &str = r#"
@namespace "godspeed.cells.repair"
@version "1.0.0"
@owner "godspeed-platform"

Cell "RepairAgent"
    @architecture "x86_64"
    @network_default "deny"
    @runtime_user "agent"
    @profile "python-agent-v1"

SystemDependency "git" version "2.45"
SystemDependency "pkg-config"

Runtime "python" version "3.13"

Tool "uv" version "0.9"

DependencySet "python-application"
    @ecosystem "python"
    @manifest "pyproject.toml"
    @lockfile "uv.lock"
    @install "uv sync --frozen"
    @mutation "escalate"

Mount "workspace"
    @source "."
    @target "/workspace"
    @mode "read-write"

Endpoint "package-mirror"
    @host "pypi.godspeed.internal"
    @port "443"
    @protocol "https"

NetworkFlow "resolve-python-packages" from "RepairAgent" to "package-mirror"
    @operation "read"

Credential "package-mirror-token"
    @provider "broker"
    @scope "package-mirror:read"
    @delivery "ephemeral"

Policy dependency_files_immutable per Constraint Prohibition priority 5
    @rationale "Agents may not modify dependency manifests or lockfiles without escalation."
    as: true

Metric "environment-proof" as: count(entities) >= 0
"#;

    fn project(source: &str) -> BTreeMap<String, String> {
        project_cell_in_memory(
            source,
            "repair-agent/model.sea",
            Some(FIXED_TS.to_string()),
            None,
            None,
        )
        .expect("projection succeeds")
    }

    #[test]
    fn emits_the_documented_tree() {
        let files = project(SOURCE);
        for expected in [
            "devbox/devbox.json",
            "devbox/projection-manifest.json",
            "mise/mise.toml",
            "mise/projection-manifest.json",
            "dependencies/dependency-sets.json",
            "sandbox/template.json",
            "sandbox/resources.json",
            "sandbox/mounts.json",
            "sandbox/network.json",
            "sandbox/lifecycle.json",
            "authority/dependency-mutation-policy.json",
            "authority/runtime-upgrade-policy.json",
            "authority/credential-contracts.json",
            "evidence/environment-proof-contract.json",
            "evidence/event-schema.json",
            "semantic/cell-ir.json",
            "semantic/canonical-model.json",
            "cell.lock",
            "README.md",
        ] {
            assert!(files.contains_key(expected), "missing {expected}");
        }
    }

    #[test]
    fn devbox_has_only_os_dependencies() {
        let files = project(SOURCE);
        let doc: serde_json::Value = extract_json(&files["devbox/devbox.json"]);
        let packages = doc["packages"].as_array().unwrap();
        let names: Vec<&str> = packages.iter().map(|p| p.as_str().unwrap()).collect();
        assert!(names.iter().any(|p| p.starts_with("git")));
        assert!(names.iter().any(|p| p.starts_with("pkg-config")));
        assert!(!names.iter().any(|p| p.starts_with("python")));
        assert!(!names.iter().any(|p| p.starts_with("uv")));
    }

    #[test]
    fn mise_has_only_runtime_and_tools() {
        let files = project(SOURCE);
        let toml = &files["mise/mise.toml"];
        assert!(toml.contains("python = \"3.13\""));
        assert!(toml.contains("uv = \"0.9\""));
        assert!(!toml.contains("git ="));
    }

    #[test]
    fn dependency_set_references_native_files() {
        let files = project(SOURCE);
        let doc: serde_json::Value = extract_json(&files["dependencies/dependency-sets.json"]);
        let set = &doc["dependency_sets"]["python-application"];
        assert_eq!(set["manifest"], "pyproject.toml");
        assert_eq!(set["lockfile"], "uv.lock");
    }

    #[test]
    fn sandbox_is_deny_by_default() {
        let files = project(SOURCE);
        let doc: serde_json::Value = extract_json(&files["sandbox/template.json"]);
        assert_eq!(doc["network_default"], "deny");
        assert_ne!(doc["runtime_user"], "root");
    }

    #[test]
    fn only_declared_endpoint_appears_in_egress() {
        let files = project(SOURCE);
        let doc: serde_json::Value = extract_json(&files["sandbox/network.json"]);
        let endpoints = doc["endpoints"].as_array().unwrap();
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0]["name"], "package-mirror");
    }

    #[test]
    fn no_credential_value_in_any_output() {
        let files = project(SOURCE);
        for (path, content) in &files {
            assert!(
                !content.contains("ephemeral-secret-value"),
                "unexpected secret literal in {path}"
            );
        }
        let doc: serde_json::Value = extract_json(&files["authority/credential-contracts.json"]);
        let cred = &doc["credentials"][0];
        assert_eq!(cred["scope"], "package-mirror:read");
        assert!(cred.get("value").is_none());
    }

    #[test]
    fn repeated_generation_is_byte_identical() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn missing_cell_declaration_fails_closed() {
        let err = project_cell_in_memory(
            "@namespace \"x\"\nEntity \"Foo\"\n",
            "x.sea",
            Some(FIXED_TS.to_string()),
            None,
            None,
        )
        .unwrap_err();
        assert!(err.starts_with("CELL002"), "{err}");
    }

    #[test]
    fn unknown_profile_fails_closed() {
        let src = "Cell \"X\"\n    @profile \"nonexistent-v1\"\n";
        let err = project_cell_in_memory(src, "x.sea", Some(FIXED_TS.to_string()), None, None)
            .unwrap_err();
        assert!(err.starts_with("CELL001"), "{err}");
    }

    #[test]
    fn undeclared_network_flow_endpoint_fails_closed() {
        let src = r#"
Cell "X"
    @profile "python-agent-v1"
NetworkFlow "bad" from "X" to "nowhere"
"#;
        let err = project_cell_in_memory(src, "x.sea", Some(FIXED_TS.to_string()), None, None)
            .unwrap_err();
        assert!(err.starts_with("CELL020"), "{err}");
    }

    #[test]
    fn only_filters_component_directories() {
        let files = project_cell_in_memory(
            SOURCE,
            "repair-agent/model.sea",
            Some(FIXED_TS.to_string()),
            None,
            Some("devbox"),
        )
        .unwrap();
        assert!(files.contains_key("devbox/devbox.json"));
        assert!(!files.contains_key("mise/mise.toml"));
        // The IR/lock always cover every category regardless of --only.
        assert!(files.contains_key("cell.lock"));
    }

    #[test]
    fn duplicate_system_dependency_declaration_fails_closed() {
        // Two SEA SystemDependency declarations of the same name must hit
        // CELL005 even though upsert would otherwise let the second
        // silently overwrite the first.
        let src = r#"
Cell "X"
    @profile "python-agent-v1"
SystemDependency "git" version "2.45"
SystemDependency "git" version "2.46"
"#;
        let err = project_cell_in_memory(src, "x.sea", Some(FIXED_TS.to_string()), None, None)
            .unwrap_err();
        assert!(err.starts_with("CELL005"), "{err}");
        assert!(err.contains("git"), "{err}");
    }

    #[test]
    fn wildcard_host_override_without_unsafe_authorization_fails() {
        // A network endpoint override that introduces a wildcard host must
        // re-run CELL021 validation and require an authorized unsafe
        // override (CELL015/016), not bypass it.
        let src = r#"
Cell "X"
    @profile "python-agent-v1"
Endpoint "egress" @host "registry.example.com"
"#;
        let overrides = r#"
schema = "domainforge-cell-overrides/v1"

[network.endpoints.egress]
host = "*"
"#;
        let err = project_cell_in_memory(
            src,
            "x.sea",
            Some(FIXED_TS.to_string()),
            Some(overrides),
            None,
        )
        .unwrap_err();
        assert!(err.starts_with("CELL021"), "{err}");
        assert!(err.contains("wildcard"), "{err}");
    }

    #[test]
    fn install_command_override_updates_evidence_probe() {
        // The depset's evidence probe must reflect the *effective* install
        // command after an override, not the originally declared one.
        let overrides = r#"
schema = "domainforge-cell-overrides/v1"

[dependency_sets.python-application]
install_command = "uv sync --frozen --override-probe"
"#;
        let files = project_cell_in_memory(
            SOURCE,
            "repair-agent/model.sea",
            Some(FIXED_TS.to_string()),
            Some(overrides),
            None,
        )
        .unwrap();
        let ir: serde_json::Value = extract_json(&files["semantic/cell-ir.json"]);
        let probes = ir["evidence_probes"].as_array().unwrap();
        let install_probe = probes
            .iter()
            .find(|p| p["source"].as_str() == Some("dependency-set"))
            .unwrap();
        assert_eq!(
            install_probe["command"], "uv sync --frozen --override-probe",
            "evidence probe must mirror the overridden install command"
        );
    }

    fn extract_json(body: &str) -> serde_json::Value {
        serde_json::from_str(body).expect("valid JSON")
    }
}
