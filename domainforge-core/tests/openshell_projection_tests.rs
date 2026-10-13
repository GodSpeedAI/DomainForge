//! OpenShell projection: determinism, binding to the Cell identity, typed
//! refusals, and (when `openshell-prover` is installed) boundary checks with
//! the real prover.
//!
//! Prover tests are `#[ignore]`d so a machine without the binary reports them
//! as ignored rather than passed. `scripts/verify/projection-targets/openshell.sh`
//! runs them with `--include-ignored`.

use domainforge_core::projection::openshell::{
    self, OpenShellProjectionError, LOCK_FILE, MAPPING_FILE, POLICY_FILE, SANDBOX_FILE,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const CREATED_AT: &str = "2026-07-11T00:00:00Z";

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/cell_env")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn repair_agent() -> BTreeMap<String, String> {
    openshell::project_openshell_in_memory(
        &fixture("repair-agent/model.sea"),
        "model.sea",
        Some(CREATED_AT.to_string()),
        None,
    )
    .expect("the RepairAgent cell projects")
}

fn refusal(source: &str) -> (String, String) {
    match openshell::project_openshell_in_memory(source, "model.sea", Some(CREATED_AT.into()), None)
    {
        Err(OpenShellProjectionError::Unrepresentable {
            element,
            requirement,
            ..
        }) => (element, requirement),
        other => panic!("expected OSH001, got {other:?}"),
    }
}

/// The RepairAgent model with one declaration replaced.
fn repair_agent_with(from: &str, to: &str) -> String {
    let source = fixture("repair-agent/model.sea");
    assert!(source.contains(from), "fixture no longer contains `{from}`");
    source.replace(from, to)
}

#[test]
fn projection_is_deterministic() {
    assert_eq!(repair_agent(), repair_agent());
}

#[test]
fn repair_agent_policy_is_default_deny_and_enforcing() {
    let files = repair_agent();
    let policy: Value = serde_saphyr::from_str(&files[POLICY_FILE]).unwrap();

    assert_eq!(policy["version"], json!(1));
    assert_eq!(
        policy["landlock"]["compatibility"],
        json!("hard_requirement")
    );
    assert_eq!(policy["process"]["run_as_user"], json!("sandbox"));
    assert_eq!(policy["filesystem_policy"]["include_workdir"], json!(false));
    assert_eq!(
        policy["filesystem_policy"]["read_write"],
        json!(["/dev/null", "/sandbox/workspace", "/tmp"])
    );

    // Exactly one rule: the one declared flow. Nothing else may leave.
    let rules = policy["network_policies"].as_object().unwrap();
    assert_eq!(
        rules.keys().collect::<Vec<_>>(),
        ["cell_resolve_python_packages"]
    );
    let endpoint = &rules["cell_resolve_python_packages"]["endpoints"][0];
    assert_eq!(endpoint["host"], json!("pypi.godspeed.internal"));
    assert_eq!(endpoint["port"], json!(443));
    assert_eq!(endpoint["protocol"], json!("rest"));
    // `read` is GET/HEAD/OPTIONS only: no state-changing request is allowed.
    assert_eq!(endpoint["access"], json!("read-only"));
    assert_eq!(endpoint["enforcement"], json!("enforce"));

    let text = &files[POLICY_FILE];
    assert!(!text.contains('*'), "no wildcard may be emitted");
    assert!(!text.contains("audit"), "no audit-mode rule may be emitted");

    // Egress is bound to the Cell's declared executables, by exact path.
    let binaries: Vec<&str> = rules["cell_resolve_python_packages"]["binaries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["path"].as_str().unwrap())
        .collect();
    assert!(binaries.contains(&"/usr/bin/uv") && binaries.contains(&"/usr/local/bin/python"));
}

/// YAML 1.1 readers (PyYAML, Go yaml.v2, older OpenShell tooling) type these
/// plain words as booleans or null. The emitter must quote them when they are
/// strings in the source model.
const YAML_1_1_SPECIAL_WORDS: [&str; 10] = [
    "y", "yes", "n", "no", "on", "off", "true", "false", "null", "~",
];

#[test]
fn policy_yaml_strings_survive_yaml_1_1_readers() {
    let text = &repair_agent()[POLICY_FILE];
    let policy: Value = serde_saphyr::from_str(text).expect("policy.yaml parses as YAML");

    // `version` is an integer in OpenShell's schema; every other scalar below is a string.
    assert_eq!(policy["version"], json!(1));
    assert!(policy["version"].is_u64(), "version must be an integer");
    assert!(policy["process"]["run_as_user"].is_string());
    assert!(policy["landlock"]["compatibility"].is_string());
    let rule = &policy["network_policies"]["cell_resolve_python_packages"];
    let endpoint = &rule["endpoints"][0];
    assert!(endpoint["host"].is_string());
    assert!(endpoint["protocol"].is_string());
    assert!(endpoint["access"].is_string());
    assert!(endpoint["enforcement"].is_string());
    assert_eq!(endpoint["port"], json!(443), "port stays an integer");
    for path in policy["filesystem_policy"]["read_write"]
        .as_array()
        .expect("read_write is a list")
    {
        assert!(path.is_string(), "{path} must be a string");
    }
    for binary in rule["binaries"].as_array().expect("binaries is a list") {
        assert!(binary["path"].is_string(), "{binary} path must be a string");
    }

    // Text-level check: a plain (unquoted) key or value that a YAML 1.1 reader would
    // type as bool or null must be a real boolean in the model. `true`/`false` are
    // allowed only under a key whose parsed value is a boolean; any other special
    // word (`yes`, `on`, `null`, ...) must be quoted.
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let line = line.trim_start().trim_start_matches("- ");
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        let is_special =
            |s: &str| YAML_1_1_SPECIAL_WORDS.contains(&s.to_ascii_lowercase().as_str());
        assert!(!is_special(key), "unquoted YAML 1.1 word as key: `{line}`");
        if !value.is_empty() && !value.starts_with(['"', '\'']) && is_special(value) {
            let intended_bool = matches!(value, "true" | "false") && bool_under_key(&policy, key);
            assert!(
                intended_bool,
                "unquoted YAML 1.1 word where a string was intended: `{line}`"
            );
        }
    }
}

/// True if some object in `value` has `key` mapped to a JSON boolean.
fn bool_under_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(map) => map
            .iter()
            .any(|(k, v)| (k == key && v.is_boolean()) || bool_under_key(v, key)),
        Value::Array(items) => items.iter().any(|v| bool_under_key(v, key)),
        _ => false,
    }
}

#[test]
fn resources_become_the_vm_shape_and_exec_timeout() {
    let sandbox: Value = serde_json::from_str(&repair_agent()[SANDBOX_FILE]).unwrap();
    assert_eq!(sandbox["compute_driver"], json!("vm"));
    assert_eq!(
        sandbox["vm_shape"],
        json!({ "vcpus": 4, "mem_mib": 8192, "overlay_disk_mib": 20480 })
    );
    assert_eq!(sandbox["exec_timeout_seconds"], json!(1800));
    assert_eq!(
        sandbox["workspaces"][0]["sandbox_path"],
        json!("/sandbox/workspace")
    );
    assert_eq!(
        sandbox["workspaces"][0]["declared_target"],
        json!("/workspace")
    );

    // The dependency manifests the Cell protects are handed to the executor.
    let protected: Vec<&Value> = sandbox["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["kind"] == json!("protected_paths"))
        .collect();
    assert_eq!(protected.len(), 1);
    assert_eq!(protected[0]["paths"], json!(["pyproject.toml", "uv.lock"]));
    assert_eq!(protected[0]["mutation"], json!("escalate"));
}

#[test]
fn credentials_never_carry_secret_material() {
    let files = repair_agent();
    let sandbox: Value = serde_json::from_str(&files[SANDBOX_FILE]).unwrap();
    let credential = &sandbox["credentials"][0];
    assert_eq!(credential["name"], json!("package-mirror-token"));
    assert_eq!(credential["delivered"], json!(false));
    let keys: Vec<&String> = credential.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["delivered", "delivery", "name", "provider", "scope"]);
    assert!(!files[POLICY_FILE].contains("package-mirror-token"));
}

#[test]
fn every_rule_names_its_source_and_the_lock_binds_the_cell() {
    let files = repair_agent();
    let mapping: Value = serde_json::from_str(&files[MAPPING_FILE]).unwrap();
    let rules = mapping["rules"].as_array().unwrap();
    assert!(rules
        .iter()
        .all(|r| !r["source"].as_str().unwrap().is_empty()));
    assert!(rules.iter().any(|r| r["target"]
        == json!("policy.yaml#/network_policies/cell_resolve_python_packages")
        && r["class"] == json!("enforced")
        && r["phase"] == json!("live")));
    assert!(rules.iter().any(
        |r| r["target"] == json!("sandbox.json#/vm_shape/overlay_disk_mib")
            && r["phase"] == json!("gateway")
    ));

    let lock: Value = serde_json::from_str(&files[LOCK_FILE]).unwrap();
    assert_eq!(lock["openshell_version"], json!("0.1.3"));
    assert_eq!(lock["cell_id"], mapping["cell_id"]);
    assert_eq!(lock["policy_digest"], lock["files"][POLICY_FILE]);

    // Same model identity as the cell projection's lock.
    let cell = domainforge_core::projection::cell::project_cell_in_memory(
        &fixture("repair-agent/model.sea"),
        "model.sea",
        Some(CREATED_AT.to_string()),
        None,
        None,
    )
    .unwrap();
    let cell_lock: Value = serde_json::from_str(&cell["cell.lock"]).unwrap();
    assert_eq!(lock["model"]["hash"], cell_lock["model"]["hash"]);
    assert_eq!(lock["cell_id"], cell_lock["cell_id"]);
}

#[test]
fn overrides_tighten_the_vm_shape() {
    let overrides = fixture("repair-agent/domainforge.cell.toml");
    let files = openshell::project_openshell_in_memory(
        &fixture("repair-agent/model.sea"),
        "model.sea",
        Some(CREATED_AT.to_string()),
        Some(&overrides),
    )
    .unwrap();
    let sandbox: Value = serde_json::from_str(&files[SANDBOX_FILE]).unwrap();
    assert_eq!(
        sandbox["vm_shape"],
        json!({ "vcpus": 2, "mem_mib": 4096, "overlay_disk_mib": 10240 })
    );
    assert_eq!(sandbox["exec_timeout_seconds"], json!(900));
}

#[test]
fn unsupported_semantics_are_typed_refusals() {
    let (element, requirement) = refusal(&repair_agent_with(
        "@network_default \"deny\"",
        "@network_default \"allow\"",
    ));
    assert_eq!(
        (element.as_str(), requirement.as_str()),
        ("Cell \"RepairAgent\"", "network_default 'allow'")
    );

    let (_, requirement) = refusal(&repair_agent_with(
        "@runtime_user \"agent\"",
        "@runtime_user \"root\"",
    ));
    assert_eq!(requirement, "runtime_user root");

    let (element, requirement) = refusal(&repair_agent_with(
        "@mode \"read-write\"",
        "@mode \"read-only\"",
    ));
    assert_eq!(
        (element.as_str(), requirement.as_str()),
        ("Mount \"workspace\"", "mode 'read-only'")
    );

    let (_, requirement) = refusal(&repair_agent_with(
        "@target \"/workspace\"",
        "@target \"/work/../etc\"",
    ));
    assert_eq!(requirement, "mount target '/work/../etc'");

    let (element, requirement) = refusal(&repair_agent_with(
        "@operation \"read\"",
        "@operation \"delete\"",
    ));
    assert_eq!(
        (element.as_str(), requirement.as_str()),
        (
            "NetworkFlow \"resolve-python-packages\"",
            "operation 'delete'"
        )
    );

    let (_, requirement) = refusal(&repair_agent_with(
        "@protocol \"https\"",
        "@protocol \"ssh\"",
    ));
    assert_eq!(requirement, "protocol 'ssh'");

    let (_, requirement) = refusal(&repair_agent_with("@cpu \"4\"", "@cpu \"512\""));
    assert_eq!(requirement, "cpu 512");

    let error = openshell::project_openshell_in_memory(
        &repair_agent_with("@network_default \"deny\"", "@network_default \"allow\""),
        "model.sea",
        Some(CREATED_AT.into()),
        None,
    )
    .unwrap_err();
    assert!(error.to_string().starts_with("OSH001 unrepresentable:"));
}

#[test]
fn invalid_cells_keep_their_cell_error_codes() {
    let error = openshell::project_openshell_in_memory(
        &fixture("invalid/04_wildcard_endpoint.sea"),
        "model.sea",
        Some(CREATED_AT.into()),
        None,
    )
    .unwrap_err();
    assert!(
        matches!(error, OpenShellProjectionError::Cell(_)),
        "{error:?}"
    );
    assert!(
        error.to_string().starts_with("OSH002 cell: CELL"),
        "{error}"
    );
}

fn write_tree(files: &BTreeMap<String, String>) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "domainforge-openshell-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    for (rel, content) in files {
        std::fs::write(root.join(rel), content).unwrap();
    }
    root
}

/// Run the real prover; returns its JSON envelope.
fn prove(dir: &Path, candidate: &str) -> Value {
    let out = Command::new("openshell-prover")
        .args([
            "check",
            candidate,
            "--boundary",
            POLICY_FILE,
            "--output",
            "json",
        ])
        .current_dir(dir)
        .output()
        .expect("`openshell-prover` must be on PATH for the ignored prover tests");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "prover printed no JSON: {e}: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[test]
#[ignore = "requires openshell-prover 0.1.3 on PATH"]
fn real_prover_accepts_the_projected_policy_with_full_coverage() {
    let dir = write_tree(&repair_agent());
    let envelope = prove(&dir, POLICY_FILE);
    assert_eq!(
        envelope["prover_version"],
        json!(openshell::OPENSHELL_VERSION)
    );
    assert_eq!(envelope["result"], json!("within_boundary"), "{envelope}");
    let mut domains: Vec<&str> = envelope["coverage"]["domains"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d.as_str().unwrap())
        .collect();
    domains.sort_unstable();
    assert_eq!(
        domains,
        [
            "filesystem",
            "landlock",
            "network_l4",
            "network_rest",
            "process"
        ]
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
#[ignore = "requires openshell-prover 0.1.3 on PATH"]
fn real_prover_rejects_weakened_or_expanded_policies() {
    let files = repair_agent();
    let dir = write_tree(&files);
    let policy = &files[POLICY_FILE];
    let cases = [
        // An undeclared destination, as a provider-composed rule would add.
        (
            "pypi.godspeed.internal",
            "exfil.example.com",
            "exceeds_boundary",
        ),
        // A state-changing HTTP method on the declared destination.
        (
            "access: read-only",
            "access: read-write",
            "exceeds_boundary",
        ),
        ("hard_requirement", "best_effort", "exceeds_boundary"),
        // Audit mode cannot be shown to enforce anything: not a pass.
        ("enforcement: enforce", "enforcement: audit", "unsupported"),
    ];
    for (from, to, expected) in cases {
        assert!(policy.contains(from), "policy no longer contains `{from}`");
        std::fs::write(dir.join("candidate.yaml"), policy.replace(from, to)).unwrap();
        let envelope = prove(&dir, "candidate.yaml");
        assert_eq!(
            envelope["result"],
            json!(expected),
            "{from} -> {to}: {envelope}"
        );
        assert_ne!(envelope["exit_code"], json!(0));
    }

    // A policy the schema rejects is an error, never a pass.
    std::fs::write(
        dir.join("candidate.yaml"),
        format!("{policy}\nunknown_key: 1\n"),
    )
    .unwrap();
    assert_eq!(prove(&dir, "candidate.yaml")["result"], json!("error"));
    std::fs::remove_dir_all(dir).ok();
}
