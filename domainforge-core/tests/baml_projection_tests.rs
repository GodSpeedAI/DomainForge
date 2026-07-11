#![cfg(feature = "cli")]

//! Integration tests for the BAML projection
//! (fixtures/baml/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/baml/basic")
}

fn recipe() -> PathBuf {
    fixture_dir().join("recipes/baml.json")
}

fn model() -> PathBuf {
    fixture_dir().join("domain/model.sea")
}

fn project_to(out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("baml")
        .arg("--recipe")
        .arg(recipe())
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model())
        .arg(out);
    cmd.assert()
}

#[test]
fn emits_the_baml_package_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_to(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(
        files.keys().collect::<Vec<_>>(),
        vec![
            "README.md",
            "baml_src/clients.baml",
            "baml_src/domain.baml",
            "baml_src/functions.baml",
            "baml_src/tests.baml",
        ]
    );
}

#[test]
fn domain_declares_enums_and_the_request_ruling_classes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_to(tmp.path()).success();
    let domain = read_tree(tmp.path())["baml_src/domain.baml"].clone();

    // Closed domains from roles / resources / operations.
    assert!(domain.contains("enum ActorRole {"));
    assert!(domain.contains("CertifiedAuditor"));
    assert!(domain.contains("enum ResourceType {"));
    assert!(domain.contains("AuditFinding"));
    assert!(domain.contains("enum Operation {"));
    assert!(domain.contains("close_audit_finding"));
    // Fixed output vocabulary.
    assert!(domain.contains("enum AuthorityDecision {"));
    // The request / ruling schemas.
    assert!(domain.contains("class AuthorityRequest {"));
    assert!(domain.contains("actor_role ActorRole"));
    assert!(domain.contains("class AuthorityRuling {"));
    assert!(domain.contains("policy_refs string[]"));
}

#[test]
fn function_embeds_policy_semantics_and_a_placeholder_client() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_to(tmp.path()).success();
    let files = read_tree(tmp.path());
    let func = &files["baml_src/functions.baml"];
    assert!(func.contains("function DecideAuthority(request: AuthorityRequest) -> AuthorityRuling"));
    assert!(func.contains("client DomainForgeAuthorityClient"));
    // Policy semantics are embedded in the prompt.
    assert!(func.contains("allow_certified_auditor_close_finding"));
    assert!(func.contains("{{ ctx.output_format }}"));

    // The client the function references is left as a commented placeholder:
    // no vendor is baked into the generated code.
    let clients = &files["baml_src/clients.baml"];
    assert!(clients.contains("// client<llm> DomainForgeAuthorityClient"));
    assert!(
        !clients.contains("\nclient<llm>"),
        "client must stay commented out"
    );
}

// The plan's core "done when": a fixture authority case appears verbatim as a
// BAML test. Assert the exact resolver decision (CertifiedAuditor may close an
// AuditFinding → allow) is present as a `test` block, byte-for-byte.
#[test]
fn a_fixture_authority_case_appears_verbatim_as_a_baml_test() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_to(tmp.path()).success();
    let tests = read_tree(tmp.path())["baml_src/tests.baml"].clone();

    let expected = "test t_allow_CertifiedAuditor_close_audit_finding_AuditFinding_4de0051a {\n  \
                    functions [DecideAuthority]\n  \
                    args {\n    \
                    request {\n      \
                    actor_role \"CertifiedAuditor\"\n      \
                    operation \"close_audit_finding\"\n      \
                    resource_type \"AuditFinding\"\n    \
                    }\n  \
                    }\n\
                    }";
    assert!(
        tests.contains(expected),
        "verbatim authority-case test block missing; got:\n{tests}"
    );
    // The resolver's decision label travels with the case.
    assert!(tests.contains("resolves to decision \"allow\""));
    // A deny case and an escalate case are present too (resolver-grounded).
    assert!(tests.contains("resolves to decision \"deny\""));
    assert!(tests.contains("resolves to decision \"escalate\""));
}

// A permission policy scoped to a role/action/resource tuple, as an authority
// pack entry.
fn permission_policy(id: &str, role: &str, action: &str, resource: &str) -> String {
    format!(
        r#"{{
          "applies_to": {{ "action": "{action}", "actor.role": "{role}", "resource.type": "{resource}" }},
          "description": "test policy {id}",
          "modality": "Permission",
          "policy_id": "{id}",
          "priority": 50,
          "requires_fact": [],
          "semantics_version": "0.4",
          "when": null
        }}"#
    )
}

// A full AuthorityEnvironmentConfig with the given policy objects.
fn authority_env(policies: &[String]) -> String {
    format!(
        r#"{{
  "resolver_semantics_version": "0.4",
  "specificity_profile": {{ "id": "default", "dimensions": ["actor","role","action","resource","scope","condition"], "scoring_rules": {{}}, "hash": "default_v1" }},
  "unknown_handling": {{ "permission": {{ "default": "escalate" }}, "prohibition": {{ "default": "deny" }}, "obligation": {{ "default": "escalate" }}, "override_": {{ "default": "not_applicable" }} }},
  "fact_sources": [],
  "fact_transforms": [],
  "authority_packs": [
    {{
      "created_at": "2026-07-02T00:00:00Z",
      "hash": "testpackhash0001",
      "id": "test-authority",
      "owner": "test@example.com",
      "policies": [{}],
      "required_specificity_profile": "default",
      "semantics_version": "0.4",
      "version": "1.0.0"
    }}
  ],
  "strict_mode": true,
  "compatibility_lowering_version": "bounded_compatibility_v1",
  "resolver_version": "0.1.0"
}}"#,
        policies.join(",\n")
    )
}

fn project_with_env(env_json: &str, out: &Path) {
    let dir = tempfile::tempdir().expect("tempdir");
    let env_path = dir.path().join("env.json");
    std::fs::write(&env_path, env_json).expect("write env");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("baml")
        .arg("--authority-config")
        .arg(&env_path)
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model())
        .arg(out);
    cmd.assert().success();
    // Keep the tempdir alive until projection completes.
    drop(dir);
}

// Teeth-check on the resolver linkage: the CertifiedAuditor-close-finding test
// exists only while the granting policy exists. A second, unrelated permission
// keeps the operation vocabulary non-empty in both runs.
#[test]
fn removing_a_policy_removes_its_baml_test() {
    let base = tempfile::tempdir().expect("tempdir");
    let other = permission_policy(
        "allow_trainee_prepare_evidence",
        "Trainee",
        "prepare_audit_evidence",
        "AuditEvidence",
    );
    let grant = permission_policy(
        "allow_certified_auditor_close_finding",
        "CertifiedAuditor",
        "close_audit_finding",
        "AuditFinding",
    );

    let with_out = base.path().join("with");
    project_with_env(&authority_env(&[grant, other.clone()]), &with_out);
    let with_tests = read_tree(&with_out)["baml_src/tests.baml"].clone();
    assert!(
        with_tests.contains("t_allow_CertifiedAuditor_close_audit_finding_AuditFinding"),
        "the granting policy must produce the allow test"
    );

    let without_out = base.path().join("without");
    project_with_env(&authority_env(&[other]), &without_out);
    let without_tests = read_tree(&without_out)["baml_src/tests.baml"].clone();
    assert!(
        !without_tests.contains("t_allow_CertifiedAuditor_close_audit_finding_AuditFinding"),
        "removing the granting policy must remove the allow test (teeth-check)"
    );
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project_to(out).success();
    });
}

#[test]
fn baml_requires_an_authority_environment() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // No --recipe (so no authority_config) and no --authority-config: must fail.
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("baml")
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model())
        .arg(tmp.path().join("out"));
    cmd.assert().failure();
}

#[test]
fn output_must_be_a_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("not_a_dir");
    std::fs::write(&file, "occupied").expect("write");
    project_to(&file).failure();
}
