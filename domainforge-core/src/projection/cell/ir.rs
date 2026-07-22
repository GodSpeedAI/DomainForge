//! Cell IR: the one place cell-environment semantics are decided.
//!
//! `CellIr::build` merges (profile defaults -> explicit SEA declarations ->
//! safe overrides) exactly once; every renderer in this module only
//! translates the resulting `CellIr` into target-native syntax (see
//! `mod.rs`). Deviation from `.agents/plans/execution-plane.md` §0: this IR
//! is built directly from the parsed `Ast`, not from `Graph` — the ten new
//! declaration kinds have no cross-projection reuse (no other projection
//! reads them), so routing them through Graph's per-kind `IndexMap`
//! machinery would add significant plumbing for no benefit. `Policy` and
//! `Metric` are read from the same `Ast` for the same reason: it avoids
//! requiring two parses of one file for one projection.

use super::overrides::{self, CellOverrides, UnsafeOverrides};
use super::profiles::{self, CellProfileData};
use crate::parser::ast::{Ast, AstNode};
use crate::projection::ids::{element_id, slug};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Profile,
    Sea,
    Override,
}

#[derive(Debug, Clone, Serialize)]
pub struct CellIdentity {
    pub namespace: String,
    pub name: String,
    pub model_version: String,
    pub owner: Option<String>,
    pub cell_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformSpec {
    pub architecture: String,
    pub runtime_user: String,
    pub network_default: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceSpec {
    pub cpu: u32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemDependencyIr {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    /// Set by `[devbox.packages]` overrides: the exact package string to
    /// emit instead of `name[@version]`.
    pub realized_package: Option<String>,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeIr {
    pub id: String,
    pub name: String,
    pub version: String,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolIr {
    pub id: String,
    pub name: String,
    pub version: String,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencySetIr {
    pub id: String,
    pub name: String,
    pub ecosystem: String,
    pub manifest: String,
    pub lockfile: String,
    pub install: String,
    /// `escalate` | `deny` | `allow`; default `escalate`.
    pub mutation: String,
    pub origin: Origin,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceIr {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub activation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MountIr {
    pub id: String,
    pub name: String,
    pub source: String,
    pub target: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointIr {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: Option<u32>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkFlowIr {
    pub id: String,
    pub name: String,
    pub from_ref: String,
    pub to_ref: String,
    pub operation: String,
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialIr {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub scope: String,
    pub delivery: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyRefIr {
    pub id: String,
    pub name: String,
    pub modality: String,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceProbeIr {
    pub id: String,
    pub command: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnsafeOverrideRecord {
    pub ticket: String,
    pub authority: String,
    pub rationale: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CellIr {
    pub identity: CellIdentity,
    pub profile_id: String,
    pub profile_version: String,
    pub platform: PlatformSpec,
    pub resources: ResourceSpec,
    pub system_dependencies: Vec<SystemDependencyIr>,
    pub runtimes: Vec<RuntimeIr>,
    pub tools: Vec<ToolIr>,
    pub dependency_sets: Vec<DependencySetIr>,
    pub services: Vec<ServiceIr>,
    pub mounts: Vec<MountIr>,
    pub endpoints: Vec<EndpointIr>,
    pub network_flows: Vec<NetworkFlowIr>,
    pub credentials: Vec<CredentialIr>,
    pub policies: Vec<PolicyRefIr>,
    pub evidence_probes: Vec<EvidenceProbeIr>,
    pub unsafe_override: Option<UnsafeOverrideRecord>,
}

fn ann_str<'a>(annotations: &'a HashMap<String, JsonValue>, key: &str) -> Option<&'a str> {
    annotations.get(key).and_then(JsonValue::as_str)
}

fn ann_u32(annotations: &HashMap<String, JsonValue>, key: &str) -> Result<Option<u32>, String> {
    match ann_str(annotations, key) {
        None => Ok(None),
        Some(raw) => raw.parse::<u32>().map(Some).map_err(|_| {
            format!("CELL004 Cell annotation @{key} must be a non-negative integer, got '{raw}'")
        }),
    }
}

/// Flatten `Export` wrappers so the extraction pass below sees the same
/// bare `AstNode` regardless of whether the declaration was exported.
fn unwrap_export(node: &AstNode) -> &AstNode {
    match node {
        AstNode::Export(inner) => unwrap_export(&inner.node),
        other => other,
    }
}

impl CellIr {
    pub fn build(
        ast: &Ast,
        model_ref: &str,
        created_at: &str,
        overrides: Option<&CellOverrides>,
    ) -> Result<CellIr, String> {
        let nodes: Vec<&AstNode> = ast
            .declarations
            .iter()
            .map(|d| unwrap_export(&d.node))
            .collect();

        let cells: Vec<(&str, &HashMap<String, JsonValue>)> = nodes
            .iter()
            .filter_map(|n| match n {
                AstNode::Cell { name, annotations } => Some((name.as_str(), annotations)),
                _ => None,
            })
            .collect();

        let (cell_name, cell_ann) = match cells.as_slice() {
            [] => return Err("CELL002 no Cell declaration found; exactly one is required".to_string()),
            [one] => *one,
            _ => {
                return Err(format!(
                    "CELL003 {} Cell declarations found; the cell-environment projection supports exactly one Cell per file",
                    cells.len()
                ))
            }
        };

        let profile_id = ann_str(cell_ann, "profile")
            .ok_or_else(|| "CELL001 Cell is missing required @profile annotation".to_string())?;
        let profile: CellProfileData = profiles::lookup(profile_id).ok_or_else(|| {
            format!(
                "CELL001 unknown profile '{profile_id}'; known profiles: {}",
                profiles::KNOWN_PROFILES.join(", ")
            )
        })?;

        let namespace = ast
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let model_version = ast
            .metadata
            .version
            .clone()
            .unwrap_or_else(|| "0.0.0".to_string());
        let owner = ast.metadata.owner.clone();
        let cell_id = element_id("cell", &[&namespace, cell_name]);

        let platform = PlatformSpec {
            architecture: ann_str(cell_ann, "architecture")
                .unwrap_or("x86_64")
                .to_string(),
            runtime_user: ann_str(cell_ann, "runtime_user")
                .unwrap_or("agent")
                .to_string(),
            network_default: ann_str(cell_ann, "network_default")
                .unwrap_or("deny")
                .to_string(),
        };
        let resources = ResourceSpec {
            cpu: ann_u32(cell_ann, "cpu")?.unwrap_or(2),
            memory_mb: ann_u32(cell_ann, "memory_mb")?.unwrap_or(4096),
            disk_mb: ann_u32(cell_ann, "disk_mb")?.unwrap_or(10240),
            timeout_seconds: ann_u32(cell_ann, "timeout_seconds")?.unwrap_or(1800),
        };

        // --- Seed from profile (Origin::Profile) ---
        let mut system_dependencies: Vec<SystemDependencyIr> = profile
            .system_dependencies
            .iter()
            .map(|d| SystemDependencyIr {
                id: element_id("cell:sysdep", &[&namespace, d.name]),
                name: d.name.to_string(),
                version: d.version.map(str::to_string),
                realized_package: None,
                origin: Origin::Profile,
            })
            .collect();
        let mut runtimes: Vec<RuntimeIr> = profile
            .runtimes
            .iter()
            .map(|t| RuntimeIr {
                id: element_id("cell:runtime", &[&namespace, t.name]),
                name: t.name.to_string(),
                version: t.version.to_string(),
                origin: Origin::Profile,
            })
            .collect();
        let mut tools: Vec<ToolIr> = profile
            .tools
            .iter()
            .map(|t| ToolIr {
                id: element_id("cell:tool", &[&namespace, t.name]),
                name: t.name.to_string(),
                version: t.version.to_string(),
                origin: Origin::Profile,
            })
            .collect();

        // --- Overlay explicit SEA declarations (Origin::Sea); same
        // (category, name) replaces the profile default. Duplicate SEA
        // declarations of the same name within one category are rejected via
        // the SEA-name check below (CELL005); the upserts themselves only
        // decide profile-vs-SEA precedence for a unique name. ---
        let mut sea_sysdep_names: Vec<&str> = Vec::new();
        let mut sea_runtime_names: Vec<&str> = Vec::new();
        let mut sea_tool_names: Vec<&str> = Vec::new();
        for n in &nodes {
            match n {
                AstNode::SystemDependency { name, version, .. } => {
                    sea_sysdep_names.push(name.as_str());
                    upsert_sysdep(&mut system_dependencies, &namespace, name, version.clone());
                }
                AstNode::Runtime { name, version, .. } => {
                    sea_runtime_names.push(name.as_str());
                    upsert_tool(
                        &mut runtimes,
                        &namespace,
                        "cell:runtime",
                        name,
                        version,
                        |id, name, version| RuntimeIr {
                            id,
                            name,
                            version,
                            origin: Origin::Sea,
                        },
                    );
                }
                AstNode::Tool { name, version, .. } => {
                    sea_tool_names.push(name.as_str());
                    upsert_tool(
                        &mut tools,
                        &namespace,
                        "cell:tool",
                        name,
                        version,
                        |id, name, version| ToolIr {
                            id,
                            name,
                            version,
                            origin: Origin::Sea,
                        },
                    );
                }
                _ => {}
            }
        }
        // Catch duplicate SEA declarations (upsert would otherwise silently
        // let the second occurrence overwrite the first).
        check_no_duplicates("SystemDependency", sea_sysdep_names.into_iter())?;
        check_no_duplicates("Runtime", sea_runtime_names.into_iter())?;
        check_no_duplicates("Tool", sea_tool_names.into_iter())?;
        check_no_duplicates(
            "SystemDependency",
            system_dependencies.iter().map(|d| d.name.as_str()),
        )?;
        check_no_duplicates("Runtime", runtimes.iter().map(|r| r.name.as_str()))?;
        check_no_duplicates("Tool", tools.iter().map(|t| t.name.as_str()))?;

        // --- DependencySet ---
        let mut dependency_sets = Vec::new();
        for n in &nodes {
            if let AstNode::DependencySet { name, annotations } = n {
                let ecosystem = ann_str(annotations, "ecosystem").ok_or_else(|| {
                    format!("CELL005 DependencySet '{name}' is missing @ecosystem")
                })?;
                let contract = profile
                    .dependency_contract
                    .filter(|c| c.ecosystem == ecosystem);
                let manifest = ann_str(annotations, "manifest")
                    .map(str::to_string)
                    .or_else(|| contract.map(|c| c.manifest.to_string()))
                    .ok_or_else(|| {
                        format!("CELL005 DependencySet '{name}' is missing @manifest")
                    })?;
                let lockfile = ann_str(annotations, "lockfile")
                    .map(str::to_string)
                    .or_else(|| contract.map(|c| c.lockfile.to_string()))
                    .ok_or_else(|| {
                        format!("CELL005 DependencySet '{name}' is missing @lockfile")
                    })?;
                let install = ann_str(annotations, "install")
                    .map(str::to_string)
                    .or_else(|| contract.map(|c| c.install.to_string()))
                    .ok_or_else(|| format!("CELL005 DependencySet '{name}' is missing @install"))?;
                let mutation = ann_str(annotations, "mutation")
                    .unwrap_or("escalate")
                    .to_string();
                dependency_sets.push(DependencySetIr {
                    id: element_id("cell:depset", &[&namespace, name]),
                    name: name.clone(),
                    ecosystem: ecosystem.to_string(),
                    manifest,
                    lockfile,
                    install,
                    mutation,
                    origin: Origin::Sea,
                });
            }
        }
        check_no_duplicates(
            "DependencySet",
            dependency_sets.iter().map(|d| d.name.as_str()),
        )?;

        // --- Service / Mount / Endpoint / NetworkFlow / Credential ---
        let mut services = Vec::new();
        let mut mounts = Vec::new();
        let mut endpoints = Vec::new();
        let mut network_flows = Vec::new();
        let mut credentials = Vec::new();

        for n in &nodes {
            match n {
                AstNode::Service {
                    name,
                    version,
                    annotations,
                } => {
                    services.push(ServiceIr {
                        id: element_id("cell:service", &[&namespace, name]),
                        name: name.clone(),
                        version: version.clone(),
                        activation: ann_str(annotations, "activation")
                            .unwrap_or("required")
                            .to_string(),
                    });
                }
                AstNode::Mount { name, annotations } => {
                    mounts.push(MountIr {
                        id: element_id("cell:mount", &[&namespace, name]),
                        name: name.clone(),
                        source: ann_str(annotations, "source")
                            .ok_or_else(|| format!("CELL005 Mount '{name}' is missing @source"))?
                            .to_string(),
                        target: ann_str(annotations, "target")
                            .ok_or_else(|| format!("CELL005 Mount '{name}' is missing @target"))?
                            .to_string(),
                        mode: ann_str(annotations, "mode")
                            .unwrap_or("read-only")
                            .to_string(),
                    });
                }
                AstNode::Endpoint { name, annotations } => {
                    let port = ann_u32(annotations, "port")?;
                    endpoints.push(EndpointIr {
                        id: element_id("cell:endpoint", &[&namespace, name]),
                        name: name.clone(),
                        host: ann_str(annotations, "host")
                            .ok_or_else(|| format!("CELL005 Endpoint '{name}' is missing @host"))?
                            .to_string(),
                        port,
                        protocol: ann_str(annotations, "protocol")
                            .unwrap_or("https")
                            .to_string(),
                    });
                }
                AstNode::NetworkFlow {
                    name,
                    from_ref,
                    to_ref,
                    annotations,
                } => {
                    network_flows.push(NetworkFlowIr {
                        id: element_id("cell:netflow", &[&namespace, name]),
                        name: name.clone(),
                        from_ref: from_ref.clone(),
                        to_ref: to_ref.clone(),
                        operation: ann_str(annotations, "operation")
                            .unwrap_or("read")
                            .to_string(),
                        credential: ann_str(annotations, "credential").map(str::to_string),
                    });
                }
                AstNode::Credential { name, annotations } => {
                    credentials.push(CredentialIr {
                        id: element_id("cell:credential", &[&namespace, name]),
                        name: name.clone(),
                        provider: ann_str(annotations, "provider")
                            .unwrap_or("broker")
                            .to_string(),
                        scope: ann_str(annotations, "scope")
                            .ok_or_else(|| {
                                format!("CELL005 Credential '{name}' is missing @scope")
                            })?
                            .to_string(),
                        delivery: ann_str(annotations, "delivery")
                            .unwrap_or("ephemeral")
                            .to_string(),
                    });
                }
                _ => {}
            }
        }
        check_no_duplicates("Service", services.iter().map(|s| s.name.as_str()))?;
        check_no_duplicates("Mount", mounts.iter().map(|m| m.name.as_str()))?;
        check_no_duplicates("Endpoint", endpoints.iter().map(|e| e.name.as_str()))?;
        check_no_duplicates("NetworkFlow", network_flows.iter().map(|f| f.name.as_str()))?;
        check_no_duplicates("Credential", credentials.iter().map(|c| c.name.as_str()))?;

        // NetworkFlow reference validation: `from` must be the Cell or a
        // Service; `to` must be an Endpoint or a Service.
        for flow in &network_flows {
            let from_ok =
                flow.from_ref == cell_name || services.iter().any(|s| s.name == flow.from_ref);
            if !from_ok {
                return Err(format!(
                    "CELL020 NetworkFlow '{}' references unknown 'from' object '{}' (must be the Cell or a declared Service)",
                    flow.name, flow.from_ref
                ));
            }
            let to_ok = endpoints.iter().any(|e| e.name == flow.to_ref)
                || services.iter().any(|s| s.name == flow.to_ref);
            if !to_ok {
                return Err(format!(
                    "CELL020 NetworkFlow '{}' references unknown 'to' object '{}' (must be a declared Endpoint or Service)",
                    flow.name, flow.to_ref
                ));
            }
        }
        for endpoint in &endpoints {
            if endpoint.host == "*" || endpoint.host == "0.0.0.0" {
                return Err(format!(
                    "CELL021 Endpoint '{}' declares wildcard host '{}'; wildcard egress requires an authorized unsafe override",
                    endpoint.name, endpoint.host
                ));
            }
        }

        // --- Policies / Metrics (reused existing SEA categories) ---
        let mut policies = Vec::new();
        let mut evidence_probes = Vec::new();
        for n in &nodes {
            match n {
                AstNode::Policy { name, metadata, .. } => {
                    policies.push(PolicyRefIr {
                        id: element_id("cell:policy", &[&namespace, name]),
                        name: name.clone(),
                        modality: metadata
                            .modality
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "Obligation".to_string()),
                        rationale: metadata.rationale.clone(),
                    });
                }
                AstNode::Metric { name, .. } => {
                    evidence_probes.push(EvidenceProbeIr {
                        id: element_id("cell:evidence", &[&namespace, name]),
                        command: format!("metric:{name}"),
                        source: "metric".to_string(),
                    });
                }
                _ => {}
            }
        }
        for tool in runtimes
            .iter()
            .map(|r| r.name.as_str())
            .chain(tools.iter().map(|t| t.name.as_str()))
        {
            evidence_probes.push(EvidenceProbeIr {
                id: element_id("cell:evidence", &[&namespace, tool, "version"]),
                command: format!("{tool} --version"),
                source: "tool-probe".to_string(),
            });
        }
        for depset in &dependency_sets {
            evidence_probes.push(EvidenceProbeIr {
                id: element_id("cell:evidence", &[&namespace, &depset.name, "install"]),
                command: depset.install.clone(),
                source: "dependency-set".to_string(),
            });
        }

        let mut ir = CellIr {
            identity: CellIdentity {
                namespace,
                name: cell_name.to_string(),
                model_version,
                owner,
                cell_id,
            },
            profile_id: profile.id.to_string(),
            profile_version: profile.version.to_string(),
            platform,
            resources,
            system_dependencies,
            runtimes,
            tools,
            dependency_sets,
            services,
            mounts,
            endpoints,
            network_flows,
            credentials,
            policies,
            evidence_probes,
            unsafe_override: None,
        };

        if let Some(overrides) = overrides {
            apply_overrides(&mut ir, overrides, created_at, model_ref)?;
        }

        sort_ir(&mut ir);
        Ok(ir)
    }
}

fn upsert_sysdep(
    list: &mut Vec<SystemDependencyIr>,
    namespace: &str,
    name: &str,
    version: Option<String>,
) {
    if let Some(existing) = list.iter_mut().find(|d| d.name == name) {
        existing.version = version;
        existing.origin = Origin::Sea;
    } else {
        list.push(SystemDependencyIr {
            id: element_id("cell:sysdep", &[namespace, name]),
            name: name.to_string(),
            version,
            realized_package: None,
            origin: Origin::Sea,
        });
    }
}

fn upsert_tool<T>(
    list: &mut Vec<T>,
    namespace: &str,
    family: &str,
    name: &str,
    version: &str,
    make: impl Fn(String, String, String) -> T,
) where
    T: HasNameVersion,
{
    if let Some(existing) = list.iter_mut().find(|t| t.name() == name) {
        existing.set_version(version.to_string());
        existing.set_origin_sea();
    } else {
        list.push(make(
            element_id(family, &[namespace, name]),
            name.to_string(),
            version.to_string(),
        ));
    }
}

trait HasNameVersion {
    fn name(&self) -> &str;
    fn set_version(&mut self, v: String);
    fn set_origin_sea(&mut self);
}

impl HasNameVersion for RuntimeIr {
    fn name(&self) -> &str {
        &self.name
    }
    fn set_version(&mut self, v: String) {
        self.version = v;
    }
    fn set_origin_sea(&mut self) {
        self.origin = Origin::Sea;
    }
}

impl HasNameVersion for ToolIr {
    fn name(&self) -> &str {
        &self.name
    }
    fn set_version(&mut self, v: String) {
        self.version = v;
    }
    fn set_origin_sea(&mut self) {
        self.origin = Origin::Sea;
    }
}

fn check_no_duplicates<'a>(
    category: &str,
    names: impl Iterator<Item = &'a str>,
) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for name in names {
        if !seen.insert(name) {
            return Err(format!("CELL005 duplicate {category} declaration '{name}'"));
        }
    }
    Ok(())
}

fn sort_ir(ir: &mut CellIr) {
    ir.system_dependencies.sort_by(|a, b| a.name.cmp(&b.name));
    ir.runtimes.sort_by(|a, b| a.name.cmp(&b.name));
    ir.tools.sort_by(|a, b| a.name.cmp(&b.name));
    ir.dependency_sets.sort_by(|a, b| a.name.cmp(&b.name));
    ir.services.sort_by(|a, b| a.name.cmp(&b.name));
    ir.mounts.sort_by(|a, b| a.name.cmp(&b.name));
    ir.endpoints.sort_by(|a, b| a.name.cmp(&b.name));
    ir.network_flows.sort_by(|a, b| a.name.cmp(&b.name));
    ir.credentials.sort_by(|a, b| a.name.cmp(&b.name));
    ir.policies.sort_by(|a, b| a.name.cmp(&b.name));
    ir.evidence_probes.sort_by(|a, b| a.id.cmp(&b.id));
    let _ = slug; // reserved for renderer use; keep import warning-free here
}

fn apply_overrides(
    ir: &mut CellIr,
    overrides: &CellOverrides,
    created_at: &str,
    model_ref: &str,
) -> Result<(), String> {
    let _ = model_ref;

    if let Some(devbox) = &overrides.devbox {
        for (name, package) in &devbox.packages {
            let dep = ir
                .system_dependencies
                .iter_mut()
                .find(|d| &d.name == name)
                .ok_or_else(|| {
                    format!(
                        "CELL013 [devbox.packages] overrides undeclared SystemDependency '{name}'"
                    )
                })?;
            dep.realized_package = Some(package.clone());
        }
    }

    if let Some(mise) = &overrides.mise {
        for (name, version) in &mise.tools {
            if let Some(runtime) = ir.runtimes.iter_mut().find(|r| &r.name == name) {
                if overrides::version_key(version) < overrides::version_key(&runtime.version) {
                    return Err(format!(
                        "CELL011 [mise.tools].{name} = '{version}' weakens the declared version '{}'",
                        runtime.version
                    ));
                }
                runtime.version = version.clone();
                runtime.origin = Origin::Override;
            } else if let Some(tool) = ir.tools.iter_mut().find(|t| &t.name == name) {
                if overrides::version_key(version) < overrides::version_key(&tool.version) {
                    return Err(format!(
                        "CELL011 [mise.tools].{name} = '{version}' weakens the declared version '{}'",
                        tool.version
                    ));
                }
                tool.version = version.clone();
                tool.origin = Origin::Override;
            } else {
                return Err(format!(
                    "CELL013 [mise.tools] overrides undeclared runtime/tool '{name}'"
                ));
            }
        }
    }

    for (name, dep_override) in &overrides.dependency_sets {
        let depset = ir
            .dependency_sets
            .iter_mut()
            .find(|d| &d.name == name)
            .ok_or_else(|| {
                format!("CELL013 [dependency_sets.{name}] overrides an undeclared DependencySet")
            })?;
        if let Some(cmd) = &dep_override.install_command {
            depset.install = cmd.clone();
            depset.origin = Origin::Override;
            // Keep the depset's evidence probe in sync so emitted evidence
            // verifies the *effective* install command. The probe id matches
            // the one assigned during build (see the depset probe loop above).
            let probe_id = element_id(
                "cell:evidence",
                &[&ir.identity.namespace, &depset.name, "install"],
            );
            if let Some(probe) = ir.evidence_probes.iter_mut().find(|p| p.id == probe_id) {
                probe.command = cmd.clone();
            }
        }
    }

    if let Some(res) = &overrides.resources {
        macro_rules! check_reduce {
            ($field:ident, $label:literal) => {
                if let Some(v) = res.$field {
                    if v > ir.resources.$field {
                        return Err(format!(
                            "CELL012 [resources].{} = {} exceeds the declared ceiling {}",
                            $label, v, ir.resources.$field
                        ));
                    }
                    ir.resources.$field = v;
                }
            };
        }
        check_reduce!(cpu, "cpu");
        check_reduce!(memory_mb, "memory_mb");
        check_reduce!(disk_mb, "disk_mb");
        check_reduce!(timeout_seconds, "timeout_seconds");
    }

    if let Some(network) = &overrides.network {
        for (name, endpoint_override) in &network.endpoints {
            let endpoint = ir
                .endpoints
                .iter_mut()
                .find(|e| &e.name == name)
                .ok_or_else(|| {
                    format!("CELL013 [network.endpoints.{name}] overrides an undeclared Endpoint")
                })?;
            if let Some(host) = &endpoint_override.host {
                endpoint.host = host.clone();
                // Reapply CELL021: a wildcard egress host requires an
                // authorized unsafe override. Validate via the existing
                // CELL015/016 path rather than bypassing it.
                if host == "*" || host == "0.0.0.0" {
                    match &overrides.unsafe_overrides {
                        Some(uo) if uo.enabled => {
                            overrides::validate_unsafe_overrides(uo, created_at)?;
                        }
                        _ => {
                            return Err(format!(
                                "CELL021 Endpoint '{}' override sets wildcard host '{}'; \
                                 wildcard egress requires an authorized [unsafe_overrides] \
                                 (enabled, ticket, authority, rationale, expires_at)",
                                endpoint.name, endpoint.host
                            ));
                        }
                    }
                }
            }
            if let Some(port) = endpoint_override.port {
                endpoint.port = Some(port);
            }
        }
    }

    if let Some(evidence) = &overrides.evidence {
        for (i, probe) in evidence.extra_probes.iter().enumerate() {
            ir.evidence_probes.push(EvidenceProbeIr {
                id: element_id(
                    "cell:evidence",
                    &[&ir.identity.namespace, "override", &i.to_string()],
                ),
                command: probe.clone(),
                source: "override".to_string(),
            });
        }
    }

    if let Some(unsafe_overrides) = &overrides.unsafe_overrides {
        validate_and_record_unsafe(ir, unsafe_overrides, created_at)?;
    }

    Ok(())
}

fn validate_and_record_unsafe(
    ir: &mut CellIr,
    unsafe_overrides: &UnsafeOverrides,
    created_at: &str,
) -> Result<(), String> {
    overrides::validate_unsafe_overrides(unsafe_overrides, created_at)?;
    if unsafe_overrides.enabled {
        ir.unsafe_override = Some(UnsafeOverrideRecord {
            ticket: unsafe_overrides.ticket.clone(),
            authority: unsafe_overrides.authority.clone(),
            rationale: unsafe_overrides.rationale.clone(),
            expires_at: unsafe_overrides.expires_at.clone(),
        });
    }
    Ok(())
}
