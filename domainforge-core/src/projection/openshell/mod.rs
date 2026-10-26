//! OpenShell projection: the Cell IR becomes an NVIDIA OpenShell sandbox
//! policy plus the parameters OpenShell keeps outside the policy.
//!
//! This is an execution-policy view of the same `CellIr` the `cell` target
//! renders; there is no second reading of the `.sea` source. Targets the
//! OpenShell policy schema `version: 1` as shipped in [`OPENSHELL_VERSION`].
//!
//! What OpenShell enforces, and when (its own split, not ours):
//! - `filesystem_policy`, `landlock`, `process`: fixed at sandbox startup.
//! - `network_policies`: may be updated on a running sandbox.
//! - CPU, memory and disk: not in the policy at all. The MicroVM driver takes
//!   them as the VM shape in the gateway's `[openshell.drivers.vm]` table;
//!   container drivers take CPU and memory at creation and have no disk limit.
//! - Execution timeout: per `exec` call.
//!
//! Anything the Cell requires that OpenShell cannot enforce is a typed
//! [`OpenShellProjectionError::Unrepresentable`] or, where the executor can
//! discharge it, an entry under `obligations` in `sandbox.json`. Nothing is
//! omitted, and no rule is emitted in audit mode or with a wildcard.
//!
//! Output (deterministic; only `openshell.lock` carries `created_at`):
//!
//! ```text
//! policy.yaml          OpenShell policy; also the boundary the effective
//!                      policy is checked against
//! sandbox.json         VM shape, exec timeout, workspace, probes, obligations
//! mapping-report.json  source Cell element for every enforcement rule
//! openshell.lock       binds the artifacts to the Cell and model identity
//! ```

use crate::formatter::{format as format_source, FormatConfig};
use crate::parser::ast::Ast;
use crate::projection::cell::ir::CellIr;
use crate::projection::cell::lock::sha256_hex;
use crate::projection::cell::overrides::CellOverrides;
use crate::projection::sink::ArtifactSink;
use crate::projection::yaml::to_yaml;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// OpenShell release whose policy schema and CLI this projection targets.
pub const OPENSHELL_VERSION: &str = "0.1.3";
pub const PROJECTION_VERSION: &str = "domainforge-openshell/v1";
pub const SANDBOX_SCHEMA: &str = "domainforge.openshell.sandbox/v1";
pub const MAPPING_SCHEMA: &str = "domainforge.openshell.mapping/v1";
pub const LOCK_SCHEMA: &str = "domainforge.openshell.lock/v1";

pub const POLICY_FILE: &str = "policy.yaml";
pub const SANDBOX_FILE: &str = "sandbox.json";
pub const MAPPING_FILE: &str = "mapping-report.json";
pub const LOCK_FILE: &str = "openshell.lock";

/// OpenShell's working directory, the only tree the sandbox user can create
/// paths under. Cell mount targets are relocated beneath it.
const WORKDIR: &str = "/sandbox";

/// Paths a process needs to start at all. Read-only, and not derived from the
/// Cell: recorded as `baseline` in the mapping report.
///
/// OpenShell's supervisor adds its own baseline to every proxy-mode policy
/// (`enrich_proto_baseline_paths`, OpenShell v0.1.3
/// `crates/openshell-supervisor/src/lib.rs`): read-only `/usr`, `/lib`, `/etc`,
/// `/app`, `/var/log`, `/proc`, `/dev/urandom`, each one that exists in the
/// guest. The gateway's effective policy therefore includes `/app` and
/// `/var/log`, and this list must be a superset of that set so the projected
/// Cell boundary contains the enforced policy. Re-check on every OpenShell
/// bump; the superset test in `tests/openshell_projection_tests.rs` pins it.
const BASELINE_READ_ONLY: [&str; 11] = [
    "/app",
    "/bin",
    "/dev/urandom",
    "/etc",
    "/lib",
    "/lib64",
    "/proc",
    "/sandbox",
    "/sbin",
    "/usr",
    "/var/log",
];

/// Scratch space inside the sandbox; never host state.
const BASELINE_READ_WRITE: [&str; 2] = ["/dev/null", "/tmp"];

/// Where a declared tool or runtime may be installed in the image. Exact
/// paths, because OpenShell binds network rules to binary identity and the
/// prover cannot reason about globs.
const EXECUTABLE_DIRS: [&str; 2] = ["/usr/bin", "/usr/local/bin"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenShellProjectionError {
    /// The Cell requires something OpenShell cannot enforce.
    Unrepresentable {
        element: String,
        requirement: String,
        reason: String,
    },
    /// The Cell model itself is invalid (a `CELL0xx` error).
    Cell(String),
    /// An artifact could not be rendered or written.
    Write(String),
}

impl std::fmt::Display for OpenShellProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unrepresentable {
                element,
                requirement,
                reason,
            } => write!(
                f,
                "OSH001 unrepresentable: {element} requires {requirement}: {reason}"
            ),
            Self::Cell(message) => write!(f, "OSH002 cell: {message}"),
            Self::Write(message) => write!(f, "OSH003 write: {message}"),
        }
    }
}

impl std::error::Error for OpenShellProjectionError {}

fn unrepresentable<T>(
    element: &str,
    requirement: &str,
    reason: &str,
) -> Result<T, OpenShellProjectionError> {
    Err(OpenShellProjectionError::Unrepresentable {
        element: element.to_string(),
        requirement: requirement.to_string(),
        reason: reason.to_string(),
    })
}

/// One line of the mapping report.
struct Mapping {
    target: String,
    source: String,
    class: &'static str,
    phase: &'static str,
    note: String,
}

impl Mapping {
    fn json(&self) -> Value {
        json!({
            "target": self.target,
            "source": self.source,
            "class": self.class,
            "phase": self.phase,
            "note": self.note,
        })
    }
}

struct Projection {
    policy: Value,
    sandbox: Value,
    mappings: Vec<Mapping>,
}

fn rule_key(name: &str) -> String {
    let key: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    // `_provider_` is reserved for rules OpenShell composes from providers.
    format!("cell_{}", key.trim_matches('_'))
}

fn relocate(element: &str, target: &str) -> Result<String, OpenShellProjectionError> {
    let valid = target.starts_with('/')
        && target != "/"
        && !target.split('/').any(|part| part == ".." || part == ".");
    if !valid {
        return unrepresentable(
            element,
            &format!("mount target '{target}'"),
            "the target must be an absolute path without '.' or '..' components, and not '/'",
        );
    }
    Ok(format!("{WORKDIR}{}", target.trim_end_matches('/')))
}

fn project(ir: &CellIr) -> Result<Projection, OpenShellProjectionError> {
    let cell = format!("Cell \"{}\"", ir.identity.name);
    let mut mappings = Vec::new();

    if ir.platform.network_default != "deny" {
        return unrepresentable(
            &cell,
            &format!("network_default '{}'", ir.platform.network_default),
            "OpenShell denies all egress that no rule allows; a permissive default cannot be expressed",
        );
    }
    if let Some(service) = ir.services.first() {
        return unrepresentable(
            &format!("Service \"{}\"", service.name),
            "a companion service",
            "an OpenShell sandbox runs one workload; companion services are not supported",
        );
    }

    // Process identity. OpenShell accepts only its own unprivileged account
    // name or a non-zero numeric id.
    let user = ir.platform.runtime_user.as_str();
    let run_as = if user == "root" || user == "0" {
        return unrepresentable(
            &cell,
            "runtime_user root",
            "OpenShell rejects root as the sandbox identity",
        );
    } else if user.parse::<u32>().is_ok() {
        mappings.push(Mapping {
            target: "policy.yaml#/process".into(),
            source: ir.identity.cell_id.clone(),
            class: "enforced",
            phase: "startup",
            note: format!("numeric runtime_user {user}"),
        });
        user.to_string()
    } else {
        mappings.push(Mapping {
            target: "policy.yaml#/process".into(),
            source: ir.identity.cell_id.clone(),
            class: "adapter",
            phase: "startup",
            note: format!(
                "runtime_user '{user}' runs as OpenShell's unprivileged 'sandbox' account; the non-root property is enforced, the account name is not preserved"
            ),
        });
        "sandbox".to_string()
    };

    // Filesystem.
    let mut read_write: BTreeSet<String> =
        BASELINE_READ_WRITE.iter().map(|p| p.to_string()).collect();
    let mut workspaces = Vec::new();
    for mount in &ir.mounts {
        let element = format!("Mount \"{}\"", mount.name);
        if mount.mode != "read-write" {
            return unrepresentable(
                &element,
                &format!("mode '{}'", mount.mode),
                "OpenShell has no host mounts on its public interface; content reaches a sandbox only by upload into a writable path",
            );
        }
        let path = relocate(&element, &mount.target)?;
        read_write.insert(path.clone());
        workspaces.push(json!({
            "name": mount.name,
            "source": mount.source,
            "declared_target": mount.target,
            "sandbox_path": path,
            "mode": mount.mode,
        }));
        mappings.push(Mapping {
            target: format!("policy.yaml#/filesystem_policy/read_write/{path}"),
            source: mount.id.clone(),
            class: "adapter",
            phase: "startup",
            note: format!(
                "target '{}' is relocated under {WORKDIR}; the host source is uploaded, never mounted",
                mount.target
            ),
        });
    }
    for path in BASELINE_READ_ONLY {
        mappings.push(Mapping {
            target: format!("policy.yaml#/filesystem_policy/read_only/{path}"),
            source: format!("profile:{}@{}", ir.profile_id, ir.profile_version),
            class: "baseline",
            phase: "startup",
            note: "read-only system path needed to start a process".into(),
        });
    }
    for path in BASELINE_READ_WRITE {
        mappings.push(Mapping {
            target: format!("policy.yaml#/filesystem_policy/read_write/{path}"),
            source: format!("profile:{}@{}", ir.profile_id, ir.profile_version),
            class: "baseline",
            phase: "startup",
            note: "scratch space inside the sandbox".into(),
        });
    }
    mappings.push(Mapping {
        target: "policy.yaml#/landlock".into(),
        source: ir.identity.cell_id.clone(),
        class: "enforced",
        phase: "startup",
        note: "hard_requirement: the sandbox must not start if Landlock is unavailable".into(),
    });

    // Network. Rules exist only for declared flows, bound to declared
    // executables.
    let mut binaries: BTreeSet<String> = BTreeSet::new();
    for name in ir
        .tools
        .iter()
        .map(|t| &t.name)
        .chain(ir.runtimes.iter().map(|r| &r.name))
    {
        for dir in EXECUTABLE_DIRS {
            binaries.insert(format!("{dir}/{name}"));
        }
    }
    let mut network = Map::new();
    for flow in &ir.network_flows {
        let element = format!("NetworkFlow \"{}\"", flow.name);
        if flow.from_ref != ir.identity.name {
            return unrepresentable(
                &element,
                &format!("source '{}'", flow.from_ref),
                "an OpenShell policy governs one sandbox; only flows from the Cell itself can be projected",
            );
        }
        if flow.credential.is_some() {
            return unrepresentable(
                &element,
                "a credential",
                "credential-bound flows need an OpenShell provider profile whose composed rules have been verified; none is projected yet",
            );
        }
        let Some(endpoint) = ir.endpoints.iter().find(|e| e.name == flow.to_ref) else {
            return Err(OpenShellProjectionError::Cell(format!(
                "{element} targets undeclared endpoint '{}'",
                flow.to_ref
            )));
        };
        if endpoint.host.contains('*') {
            return unrepresentable(
                &element,
                "a wildcard host",
                "wildcard destinations are never projected",
            );
        }
        if binaries.is_empty() {
            return unrepresentable(
                &element,
                "an executable identity",
                "OpenShell allows egress only to listed binaries and the Cell declares no Tool or Runtime",
            );
        }

        let mut rule = Map::new();
        rule.insert("host".into(), json!(endpoint.host));
        let default_port = match endpoint.protocol.as_str() {
            "https" => Some(443),
            "http" => Some(80),
            _ => None,
        };
        let Some(port) = endpoint.port.or(default_port) else {
            return unrepresentable(
                &element,
                "a port",
                "the endpoint declares no port and its protocol has no default",
            );
        };
        rule.insert("port".into(), json!(port));
        match (endpoint.protocol.as_str(), flow.operation.as_str()) {
            ("https" | "http", "read") => {
                rule.insert("protocol".into(), json!("rest"));
                rule.insert("access".into(), json!("read-only"));
            }
            ("https" | "http", "write" | "read-write") => {
                rule.insert("protocol".into(), json!("rest"));
                rule.insert("access".into(), json!("read-write"));
            }
            ("tcp", "connect") => {
                rule.insert("protocol".into(), json!("tcp"));
            }
            ("tcp", operation) => {
                return unrepresentable(
                    &element,
                    &format!("operation '{operation}' over tcp"),
                    "a raw TCP rule cannot restrict what is sent; declare the operation as 'connect' or use http(s)",
                );
            }
            ("https" | "http", operation) => {
                return unrepresentable(
                    &element,
                    &format!("operation '{operation}'"),
                    "only 'read' (GET/HEAD/OPTIONS) and 'write' (adds POST/PUT/PATCH) map to OpenShell access presets",
                );
            }
            (protocol, _) => {
                return unrepresentable(
                    &element,
                    &format!("protocol '{protocol}'"),
                    "only http, https and tcp endpoints are projected",
                );
            }
        }
        if rule.contains_key("access") {
            rule.insert("enforcement".into(), json!("enforce"));
        }

        let key = rule_key(&flow.name);
        if network.contains_key(&key) {
            return Err(OpenShellProjectionError::Cell(format!(
                "{element} collides with another flow on rule key '{key}'"
            )));
        }
        mappings.push(Mapping {
            target: format!("policy.yaml#/network_policies/{key}"),
            source: flow.id.clone(),
            class: "enforced",
            phase: "live",
            note: format!(
                "{} {}:{} for operation '{}', restricted to the Cell's declared executables",
                endpoint.protocol, endpoint.host, port, flow.operation
            ),
        });
        network.insert(
            key,
            json!({
                "name": flow.name,
                "endpoints": [Value::Object(rule)],
                "binaries": binaries.iter().map(|path| json!({ "path": path })).collect::<Vec<_>>(),
            }),
        );
    }

    let policy = json!({
        "version": 1,
        "filesystem_policy": {
            "include_workdir": false,
            "read_only": BASELINE_READ_ONLY,
            "read_write": read_write,
        },
        "landlock": { "compatibility": "hard_requirement" },
        "process": { "run_as_user": run_as, "run_as_group": run_as },
        "network_policies": network,
    });

    // Resources: the VM shape. `vcpus` is a u8 in the gateway schema.
    let Ok(vcpus) = u8::try_from(ir.resources.cpu) else {
        return unrepresentable(
            &cell,
            &format!("cpu {}", ir.resources.cpu),
            "the MicroVM driver accepts at most 255 vCPUs",
        );
    };
    for (field, note) in [
        ("vcpus", "cpu"),
        ("mem_mib", "memory_mb"),
        ("overlay_disk_mib", "disk_mb"),
    ] {
        mappings.push(Mapping {
            target: format!("sandbox.json#/vm_shape/{field}"),
            source: ir.identity.cell_id.clone(),
            class: "enforced",
            phase: "gateway",
            note: format!(
                "{note}: set in the gateway's [openshell.drivers.vm] table; the MicroVM driver has no per-sandbox limits"
            ),
        });
    }
    mappings.push(Mapping {
        target: "sandbox.json#/exec_timeout_seconds".into(),
        source: ir.identity.cell_id.clone(),
        class: "enforced",
        phase: "exec",
        note: "timeout_seconds: passed to every exec; OpenShell reports exit 124 on expiry".into(),
    });

    // What OpenShell cannot enforce and the executor must.
    let mut obligations = vec![json!({
        "kind": "architecture",
        "value": ir.platform.architecture,
        "discharge": "probe `uname -m` in the sandbox and refuse on mismatch",
    })];
    for dep in &ir.dependency_sets {
        obligations.push(json!({
            "kind": "protected_paths",
            "source": dep.id,
            "paths": [dep.manifest, dep.lockfile],
            "mutation": dep.mutation,
            "discharge": "apply the mutation decision to any changeset touching these paths before promotion",
        }));
    }
    for policy_ref in &ir.policies {
        obligations.push(json!({
            "kind": "policy",
            "source": policy_ref.id,
            "name": policy_ref.name,
            "modality": policy_ref.modality,
            "discharge": "evaluate through the authority using the model's Rego projection",
        }));
    }
    let image_requirements: Vec<Value> = ir
        .system_dependencies
        .iter()
        .map(|d| json!({ "kind": "system", "name": d.name, "version": d.version }))
        .chain(
            ir.runtimes
                .iter()
                .map(|r| json!({ "kind": "runtime", "name": r.name, "version": r.version })),
        )
        .chain(
            ir.tools
                .iter()
                .map(|t| json!({ "kind": "tool", "name": t.name, "version": t.version })),
        )
        .collect();
    obligations.push(json!({
        "kind": "image_contents",
        "requirements": image_requirements,
        "discharge": "run the evidence probes in the sandbox and refuse on failure; OpenShell does not verify image contents",
    }));

    let credentials: Vec<Value> = ir
        .credentials
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "provider": c.provider,
                "scope": c.scope,
                "delivery": c.delivery,
                // No flow is bound to a credential (bound flows are refused
                // above), so nothing is delivered into the sandbox.
                "delivered": false,
            })
        })
        .collect();

    let sandbox = json!({
        "schema": SANDBOX_SCHEMA,
        "cell_id": ir.identity.cell_id,
        "openshell_version": OPENSHELL_VERSION,
        "compute_driver": "vm",
        "vm_shape": {
            "vcpus": vcpus,
            "mem_mib": ir.resources.memory_mb,
            "overlay_disk_mib": ir.resources.disk_mb,
        },
        "container_limits": {
            "cpu": ir.resources.cpu.to_string(),
            "memory": format!("{}Mi", ir.resources.memory_mb),
            "disk": Value::Null,
        },
        "exec_timeout_seconds": ir.resources.timeout_seconds,
        "workdir": WORKDIR,
        "workspaces": workspaces,
        "dependency_sets": ir.dependency_sets.iter().map(|d| json!({
            "name": d.name,
            "ecosystem": d.ecosystem,
            "manifest": d.manifest,
            "lockfile": d.lockfile,
            "install": d.install,
            "mutation": d.mutation,
        })).collect::<Vec<_>>(),
        "evidence_probes": ir.evidence_probes.iter().map(|p| json!({
            "id": p.id,
            "command": p.command,
            "source": p.source,
        })).collect::<Vec<_>>(),
        "credentials": credentials,
        "obligations": obligations,
    });

    Ok(Projection {
        policy,
        sandbox,
        mappings,
    })
}

fn pretty(value: &Value) -> String {
    let mut out = serde_json::to_string_pretty(value).expect("JSON values serialize");
    out.push('\n');
    out
}

/// Emit the OpenShell projection into `sink`; returns the emitted relative paths.
pub fn emit(
    ast: &Ast,
    source: &str,
    model_ref: &str,
    created_at: Option<String>,
    overrides: Option<&CellOverrides>,
    overrides_source: Option<&str>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, OpenShellProjectionError> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ir = CellIr::build(ast, model_ref, &created_at, overrides)
        .map_err(OpenShellProjectionError::Cell)?;
    let projection = project(&ir)?;

    let canonical = format_source(source, FormatConfig::default()).map_err(|e| {
        OpenShellProjectionError::Cell(format!("failed to canonicalize source: {e}"))
    })?;
    let model_hash = format!("sha256:{}", sha256_hex(canonical.as_bytes()));

    let policy_yaml = format!(
        "# Generated by `domainforge project --format openshell`. Do not edit.\n# Cell: {} ({})\n{}",
        ir.identity.name,
        ir.identity.cell_id,
        to_yaml(&projection.policy)
            .map_err(|e| OpenShellProjectionError::Write(e.to_string()))?
    );

    let mut files: BTreeMap<String, String> = BTreeMap::new();
    files.insert(POLICY_FILE.to_string(), policy_yaml);
    files.insert(SANDBOX_FILE.to_string(), pretty(&projection.sandbox));
    files.insert(
        MAPPING_FILE.to_string(),
        pretty(&json!({
            "schema": MAPPING_SCHEMA,
            "projection": PROJECTION_VERSION,
            "openshell_version": OPENSHELL_VERSION,
            "cell_id": ir.identity.cell_id,
            "classes": {
                "enforced": "enforced by OpenShell as declared",
                "adapter": "enforced by OpenShell after an explicit, recorded adaptation",
                "baseline": "not declared by the Cell; required for any process to run",
            },
            "phases": {
                "startup": "fixed when the sandbox starts",
                "live": "may be updated on a running sandbox",
                "gateway": "set in the gateway's driver configuration",
                "exec": "applied per command",
            },
            "rules": projection.mappings.iter().map(Mapping::json).collect::<Vec<_>>(),
            "obligations": projection.sandbox["obligations"],
        })),
    );

    let hashes: Map<String, Value> = files
        .iter()
        .map(|(path, content)| {
            (
                path.clone(),
                json!(format!("sha256:{}", sha256_hex(content.as_bytes()))),
            )
        })
        .collect();
    files.insert(
        LOCK_FILE.to_string(),
        pretty(&json!({
            "schema": LOCK_SCHEMA,
            "projection": PROJECTION_VERSION,
            "openshell_version": OPENSHELL_VERSION,
            "cell_id": ir.identity.cell_id,
            "created_at": created_at,
            "model": { "version": ir.identity.model_version, "hash": model_hash },
            "profile": { "id": ir.profile_id, "version": ir.profile_version },
            "overrides": {
                "present": overrides_source.is_some(),
                "hash": overrides_source.map(|s| format!("sha256:{}", sha256_hex(s.as_bytes()))),
            },
            "policy_digest": hashes[POLICY_FILE],
            "files": hashes,
        })),
    );

    for (path, content) in &files {
        sink.write(path, content)
            .map_err(OpenShellProjectionError::Write)?;
    }
    Ok(files.into_keys().collect())
}

/// Binding surface: source in, path→content map out, no filesystem.
pub fn project_openshell_in_memory(
    source: &str,
    model_ref: &str,
    created_at: Option<String>,
    overrides_source: Option<&str>,
) -> Result<BTreeMap<String, String>, OpenShellProjectionError> {
    let ast = crate::parser::parse(source)
        .map_err(|e| OpenShellProjectionError::Cell(format!("parse failed: {e}")))?;
    let overrides = overrides_source
        .map(crate::projection::cell::overrides::parse)
        .transpose()
        .map_err(OpenShellProjectionError::Cell)?;
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(
        &ast,
        source,
        model_ref,
        created_at,
        overrides.as_ref(),
        overrides_source,
        &mut sink,
    )?;
    Ok(map)
}
