#![cfg(feature = "cli")]

//! Integration tests for the AI Learning Projections
//! (fixtures/ai_learning/manufacturing_quality is the single proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::FIXED_TS;
use predicates::prelude::*;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/ai_learning/manufacturing_quality")
        .join(rel)
}

fn project(format: &str, out: &Path, extra: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg(format)
        .arg("--created-at")
        .arg(FIXED_TS)
        .args(extra)
        .arg(fixture("domain/model.sea"))
        .arg(out);
    cmd.assert()
}

fn project_with_recipe(format: &str, out: &Path) -> assert_cmd::assert::Assert {
    let recipe = fixture("recipes/ai_learning.json");
    project(format, out, &["--recipe", recipe.to_str().unwrap()])
}

fn read_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn read_jsonl(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .lines()
        .map(|l| serde_json::from_str(l).expect("jsonl row parses"))
        .collect()
}

fn all_llm_rows(dir: &Path) -> Vec<(String, Value)> {
    ["train", "validation", "test"]
        .iter()
        .flat_map(|split| {
            read_jsonl(&dir.join(format!("{split}.jsonl")))
                .into_iter()
                .map(move |row| (split.to_string(), row))
        })
        .collect()
}

/// Test 1: same fixture + recipe + fixed created_at => byte-identical output.
#[test]
fn determinism_byte_identical_across_runs() {
    let out1 = tempfile::tempdir().unwrap();
    let out2 = tempfile::tempdir().unwrap();
    project_with_recipe("ai-learning", out1.path()).success();
    project_with_recipe("ai-learning", out2.path()).success();

    let mut files: Vec<PathBuf> = walk(out1.path());
    files.sort();
    assert!(
        files.len() > 100,
        "expected many artifacts, got {}",
        files.len()
    );
    for file in files {
        let rel = file.strip_prefix(out1.path()).unwrap();
        let other = out2.path().join(rel);
        let a = std::fs::read(&file).unwrap();
        let b = std::fs::read(&other)
            .unwrap_or_else(|e| panic!("missing in second run: {}: {e}", rel.display()));
        assert_eq!(a, b, "artifact differs between runs: {}", rel.display());
    }
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// Test 2: split assignments are recorded, disjoint, and complete.
#[test]
fn splits_are_disjoint_and_recorded() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-llm", out.path()).success();
    let rows = all_llm_rows(out.path());
    assert!(!rows.is_empty());
    let mut ids = BTreeSet::new();
    for (split, row) in &rows {
        let meta = &row["metadata"];
        assert_eq!(meta["split"].as_str().unwrap(), split);
        let id = meta["record_id"].as_str().unwrap().to_string();
        assert!(ids.insert(id), "record id appears in more than one split");
    }
    let manifest = read_json(&out.path().join("dataset_manifest.json"));
    assert_eq!(
        manifest["record_count"].as_u64().unwrap() as usize,
        rows.len()
    );
}

/// Test 3: deny rows exist for Trainee/close/AuditFinding and the validator's
/// re-resolution found zero disagreements.
#[test]
fn deny_rows_match_resolver() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-llm", out.path()).success();
    let rows = all_llm_rows(out.path());
    let deny_row = rows.iter().find(|(_, row)| {
        row["metadata"]["family"] == "authorization_qa"
            && row["metadata"]["label"] == "deny"
            && row["messages"][1]["content"]
                .as_str()
                .unwrap()
                .contains("Trainee")
            && row["messages"][1]["content"]
                .as_str()
                .unwrap()
                .contains("close audit finding on AuditFinding")
    });
    assert!(deny_row.is_some(), "expected a Trainee deny row");
    let (_, row) = deny_row.unwrap();
    assert!(row["messages"][2]["content"]
        .as_str()
        .unwrap()
        .contains("deny_trainee_close_finding"));

    let report = read_json(&out.path().join("validation_report.json"));
    assert_eq!(report["resolver_disagreement_count"], 0);
    assert_eq!(report["status"], "passed");
}

/// Test 4: unknown-authority prompts answer "insufficient information",
/// never allow/deny.
#[test]
fn unknown_rows_never_guess() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-llm", out.path()).success();
    let rows = all_llm_rows(out.path());
    let unknown_rows: Vec<_> = rows
        .iter()
        .filter(|(_, row)| {
            row["metadata"]["family"] == "authorization_qa" && row["metadata"]["label"] == "unknown"
        })
        .collect();
    assert!(
        !unknown_rows.is_empty(),
        "fixture must produce unknown cases"
    );
    for (_, row) in unknown_rows {
        let answer = row["messages"][2]["content"].as_str().unwrap();
        assert!(
            answer.contains("does not contain enough authority information"),
            "unknown answer must state insufficiency: {answer}"
        );
        assert!(!answer.starts_with("Yes"), "unknown must not allow");
        assert!(!answer.starts_with("No."), "unknown must not deny");
    }
}

/// Test 5: graph is closed over its nodes; no negative sample corresponds to
/// a resolver Allow.
#[test]
fn graph_edges_closed_and_negatives_grounded() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-graph-ml", out.path()).success();
    let graph = read_json(&out.path().join("graph.json"));
    let node_ids: BTreeSet<&str> = graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["node_id"].as_str().unwrap())
        .collect();
    assert!(node_ids.len() >= 10);
    for edge in graph["edges"].as_array().unwrap() {
        assert!(node_ids.contains(edge["source"].as_str().unwrap()));
        assert!(node_ids.contains(edge["target"].as_str().unwrap()));
    }

    let samples = read_json(&out.path().join("negative_samples.json"));
    let positives: BTreeSet<String> = samples["positives"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["sample_id"].as_str().unwrap().to_string())
        .collect();
    assert!(!positives.is_empty(), "fixture must yield allow samples");
    let negatives = samples["negatives"].as_array().unwrap();
    assert!(
        !negatives.is_empty(),
        "fixture must yield deny/escalate samples"
    );
    for negative in negatives {
        assert!(
            !positives.contains(negative["sample_id"].as_str().unwrap()),
            "negative sample duplicates an allow tuple"
        );
        assert_ne!(negative["reason"], "allowed_by_policy");
    }
    // Open-world default: unknown tuples are not negatives.
    assert!(samples["policy"]["unknown_excluded_by_default"]
        .as_bool()
        .unwrap());
    for negative in negatives {
        assert_ne!(negative["reason"], "unknown_authority");
    }
}

/// Test 6: AuthorityBench — every case has an answer key, keys match resolver
/// decisions, and escalate + unknown cases are present.
#[test]
fn authoritybench_cases_and_keys() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("cep-eval", out.path()).success();
    let cases_dir = out.path().join("cases");
    let mut decisions: BTreeSet<String> = BTreeSet::new();
    let mut checked = 0;
    for case_path in walk(&cases_dir) {
        let case = read_json(&case_path);
        if case["family_ref"] != "AuthorityBench" {
            continue;
        }
        let key_path = out.path().join(case["answer_key_ref"].as_str().unwrap());
        let key = read_json(&key_path);
        assert_eq!(key["case_ref"], case["case_id"]);
        assert_eq!(key["answer_key_type"], "programmatic");
        let decision = key["expected_values"]["decision"].as_str().unwrap();
        decisions.insert(decision.to_string());

        let input = &case["input_representations"];
        match (
            input["actor_role"].as_str().unwrap(),
            input["operation"].as_str().unwrap(),
            input["resource_type"].as_str().unwrap(),
        ) {
            ("CertifiedAuditor", "close_audit_finding", "AuditFinding") => {
                assert_eq!(decision, "allow");
                checked += 1;
            }
            ("Trainee", "close_audit_finding", "AuditFinding") => {
                assert_eq!(decision, "deny");
                checked += 1;
            }
            ("Trainee", "close_audit_finding", "HighRiskSafetyFinding") => {
                assert_eq!(decision, "escalate");
                checked += 1;
            }
            _ => {}
        }
    }
    assert_eq!(checked, 3, "expected the three sentinel tuples");
    for expected in ["allow", "deny", "escalate", "unknown"] {
        assert!(decisions.contains(expected), "missing decision {expected}");
    }
    let report = read_json(&out.path().join("reports/validation_report.json"));
    assert_eq!(report["resolver_disagreement_count"], 0);
    assert_eq!(report["status"], "passed");
}

/// Test 7: each ConformanceBench invalid case actually exhibits exactly the
/// failure its answer key names.
#[test]
fn conformancebench_invalid_cases_fail_for_their_reason() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("cep-eval", out.path()).success();
    let mut invalid_seen = 0;
    for case_path in walk(&out.path().join("cases")) {
        let case = read_json(&case_path);
        if case["family_ref"] != "ConformanceBench" {
            continue;
        }
        let key = read_json(&out.path().join(case["answer_key_ref"].as_str().unwrap()));
        let status = key["expected_values"]["conformance_status"]
            .as_str()
            .unwrap();
        let artifact = &case["input_representations"]["artifact"];
        if status == "conformant" {
            assert!(artifact["envelope"]["envelope_id"].is_string());
            continue;
        }
        invalid_seen += 1;
        let failures = key["expected_values"]["failures"].as_array().unwrap();
        assert_eq!(failures.len(), 1, "one corrupted property per invalid case");
        let failure = failures[0].as_str().unwrap();
        match failure {
            "missing_envelope_identity" => assert!(artifact["envelope"]["envelope_id"].is_null()),
            "missing_cep_version" => assert!(artifact["envelope"]["cep_version"].is_null()),
            "missing_scope" => assert!(artifact["envelope"]["scope"].is_null()),
            "missing_boundary_record" => {
                assert!(artifact["envelope"]["boundary_record"].is_null())
            }
            "missing_provenance" => assert!(artifact["provenance_refs"].is_null()),
            "missing_completeness_status" => {
                assert!(artifact["envelope"]["completeness_status"].is_null())
            }
            "missing_primary_question" => assert!(artifact["primary_question_ref"].is_null()),
            "missing_answer_shape" => assert!(artifact["expected_answer_shape"].is_null()),
            "missing_evidence_requirements" => {
                assert!(artifact["evidence_requirements"].is_null())
            }
            "decision_outside_allowed_set" => {
                let d = artifact["inline_answer_preview"]["decision"]
                    .as_str()
                    .unwrap();
                assert!(!["allow", "deny", "escalate", "unknown", "reject"].contains(&d));
            }
            "generated_case_marked_reviewed" => {
                assert_eq!(artifact["review_status"], "reviewed")
            }
            "raw_prompt_text_as_envelope" => assert!(artifact.is_string()),
            other => panic!("unexpected failure kind {other}"),
        }
    }
    assert_eq!(
        invalid_seen, 12,
        "one invalid case per §53-derived mutation"
    );
}

/// Test 8: everything defaults to review_status unreviewed; entity_lookup
/// rows are the stated not_required exception.
#[test]
fn review_status_defaults() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-llm", out.path()).success();
    for (_, row) in all_llm_rows(out.path()) {
        let family = row["metadata"]["family"].as_str().unwrap();
        let status = row["metadata"]["provenance"]["review_status"]
            .as_str()
            .unwrap();
        if family == "entity_lookup" {
            assert_eq!(status, "not_required");
        } else {
            assert_eq!(status, "unreviewed");
        }
    }

    let cep = tempfile::tempdir().unwrap();
    project_with_recipe("cep-eval", cep.path()).success();
    for case_path in walk(&cep.path().join("cases")) {
        assert_eq!(read_json(&case_path)["review_status"], "unreviewed");
    }
}

/// Test 9: strict recipe validation — unknown keys, bad ratios, unknown
/// family are hard errors.
#[test]
fn recipe_rejection() {
    use domainforge_core::projection::ai_learning::recipe::Recipe;
    assert!(Recipe::from_json(r#"{"name":"x","bogus_key":1}"#)
        .unwrap_err()
        .contains("invalid recipe"));
    assert!(Recipe::from_json(
        r#"{"name":"x","splits":{"train":0.5,"validation":0.1,"test":0.1}}"#
    )
    .unwrap_err()
    .contains("sum to 1.0"));
    assert!(Recipe::from_json(
        r#"{"name":"x","projections":{"llm_dataset":{"families":["made_up"]}}}"#
    )
    .unwrap_err()
    .contains("unknown llm family"));
}

/// Test 10: all four formats succeed with the expected layout; ai-learning
/// without --recipe errors clearly.
#[test]
fn cli_formats_and_layout() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-learning", out.path()).success();
    for rel in [
        "llm_dataset/train.jsonl",
        "llm_dataset/dataset_manifest.json",
        "graph_dataset/graph.json",
        "graph_dataset/nodes.csv",
        "graph_dataset/edges.csv",
        "graph_dataset/labels.json",
        "graph_dataset/masks.json",
        "graph_dataset/negative_samples.json",
        "cep_eval/dataset.json",
        "cep_eval/reports/validation_report.json",
    ] {
        assert!(out.path().join(rel).exists(), "missing {rel}");
    }

    for format in ["ai-llm", "ai-graph-ml", "cep-eval"] {
        let single = tempfile::tempdir().unwrap();
        let recipe = fixture("recipes/ai_learning.json");
        project(
            format,
            single.path(),
            &["--recipe", recipe.to_str().unwrap()],
        )
        .success();
        assert!(
            single.path().join("generation_report.json").exists()
                || single
                    .path()
                    .join("reports/generation_report.json")
                    .exists()
        );
    }

    let no_recipe = tempfile::tempdir().unwrap();
    project("ai-learning", no_recipe.path(), &[])
        .failure()
        .stderr(predicate::str::contains("requires --recipe"));
}

/// Test 11: validation/coverage/generation reports exist, parse, and
/// known_gaps is computed (deferred families always appear for cep-eval).
#[test]
fn reports_exist_and_known_gaps_computed() {
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-learning", out.path()).success();
    for rel in [
        "llm_dataset/validation_report.json",
        "llm_dataset/coverage_report.json",
        "llm_dataset/generation_report.json",
        "graph_dataset/validation_report.json",
        "graph_dataset/coverage_report.json",
        "graph_dataset/generation_report.json",
        "cep_eval/reports/validation_report.json",
        "cep_eval/reports/coverage_report.json",
        "cep_eval/reports/generation_report.json",
    ] {
        let report = read_json(&out.path().join(rel));
        assert!(report.is_object(), "{rel} must parse to an object");
    }
    let coverage = read_json(&out.path().join("cep_eval/reports/coverage_report.json"));
    let gaps = coverage["known_gaps"].as_array().unwrap();
    assert!(
        gaps.iter()
            .any(|g| g.as_str().unwrap().contains("RealityGapBench")),
        "deferred families must be a computed known gap"
    );
    // Provenance is embedded in every generated LLM record.
    let llm = tempfile::tempdir().unwrap();
    project_with_recipe("ai-llm", llm.path()).success();
    for (_, row) in all_llm_rows(llm.path()) {
        let p = &row["metadata"]["provenance"];
        assert!(p["source_model_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
        assert!(!p["source_refs"].as_array().unwrap().is_empty());
        assert_eq!(p["seed"], 0);
    }
}

/// v1 binding surface: project_ai_learning_in_memory (what the Python/TS/wasm
/// `export_ai_learning` methods call) produces the same artifacts as the CLI,
/// byte-for-byte, with no filesystem access.
#[test]
fn in_memory_api_matches_cli_output() {
    use domainforge_core::parser::{parse_to_graph_with_options, ParseOptions};
    use domainforge_core::projection::ai_learning::project_ai_learning_in_memory;

    let model_path = fixture("domain/model.sea");
    let source = std::fs::read_to_string(&model_path).unwrap();
    let graph = parse_to_graph_with_options(&source, &ParseOptions::default()).unwrap();
    let recipe_json = std::fs::read_to_string(fixture("recipes/ai_learning.json")).unwrap();
    let authority_json = std::fs::read_to_string(fixture("authority/environment.json")).unwrap();

    let model_ref = model_path.display().to_string();
    let artifacts = project_ai_learning_in_memory(
        &graph,
        Some(&recipe_json),
        Some(&authority_json),
        &model_ref,
        None,
        Some(FIXED_TS.to_string()),
    )
    .expect("in-memory projection succeeds");

    // Same run twice => identical map (determinism through the binding path).
    let again = project_ai_learning_in_memory(
        &graph,
        Some(&recipe_json),
        Some(&authority_json),
        &model_ref,
        None,
        Some(FIXED_TS.to_string()),
    )
    .unwrap();
    assert_eq!(artifacts, again);

    // Same layout and content as the CLI's --format ai-learning.
    let out = tempfile::tempdir().unwrap();
    project_with_recipe("ai-learning", out.path()).success();
    let cli_files = walk(out.path());
    assert_eq!(cli_files.len(), artifacts.len());
    for file in cli_files {
        let rel = file
            .strip_prefix(out.path())
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        let disk = std::fs::read_to_string(&file).unwrap();
        let mem = artifacts
            .get(&rel)
            .unwrap_or_else(|| panic!("missing artifact {rel} in in-memory output"));
        // recipe_ref/model_ref provenance strings legitimately differ between
        // the two entry points; compare after normalizing them away.
        // Forward-slash first so the recipe-path match is robust to Windows
        // path-separator mixing (the in-memory path uses "<inline>"; the CLI
        // embeds the literal recipe path, which mixes `\` and `/` on Windows).
        let recipe = fixture("recipes/ai_learning.json")
            .to_str()
            .unwrap()
            .replace('\\', "/");
        let normalize = |s: &str| {
            s.replace('\\', "/")
                .replace("<inline>", "@")
                .replace(&recipe, "@")
        };
        let (nd, nm) = (normalize(&disk), normalize(mem));
        if nd != nm {
            eprintln!("=== DEBUG artifact differs: {rel} ===");
            eprintln!("--- disk ---\n{nd}");
            eprintln!("--- mem ---\n{nm}");
        }
        assert_eq!(nd, nm, "artifact differs: {rel}");
    }

    // No unknown-authority guessing sneaks through the binding path either.
    let train = artifacts.get("llm_dataset/train.jsonl").unwrap();
    assert!(train.contains("does not contain enough authority information"));
}
