#![cfg(feature = "cli")]

//! Integration tests for the ZenML projection
//! (fixtures/zenml/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/zenml/basic")
}

fn recipe() -> PathBuf {
    fixture_dir().join("recipes/zenml.json")
}

fn model() -> PathBuf {
    fixture_dir().join("domain/model.sea")
}

fn project_zenml(out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("zenml")
        .arg("--recipe")
        .arg(recipe())
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model())
        .arg(out);
    cmd.assert()
}

fn project_ai_learning(out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("ai-learning")
        .arg("--recipe")
        .arg(recipe())
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model())
        .arg(out);
    cmd.assert()
}

#[test]
fn emits_the_zenml_pipeline_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_zenml(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(
        files.keys().collect::<Vec<_>>(),
        vec![
            "README.md",
            "pipeline.py",
            "requirements.txt",
            "run.py",
            "steps.py",
            "zenml.config.json",
        ]
    );
}

#[test]
fn pipeline_wires_the_load_train_eval_register_dag() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_zenml(tmp.path()).success();
    let files = read_tree(tmp.path());
    let pipeline = files["pipeline.py"].clone();

    // The @pipeline function and the four steps wired into the DAG.
    assert!(pipeline.contains("@pipeline(model=MODEL, enable_cache=False)"));
    assert!(pipeline.contains("def authority_learning_pipeline():"));
    assert!(pipeline.contains("dataset = load_dataset()"));
    assert!(pipeline.contains("model = train_model(dataset)"));
    assert!(pipeline.contains("metrics = evaluate_model(dataset, model)"));
    assert!(pipeline.contains("return register_model(model, metrics)"));
    // Model-version grouping keyed to the model hash (v-<hash>).
    assert!(pipeline.contains("version=\"v-"));
    // Only `zenml` and the standard library are imported.
    assert!(pipeline.contains("from zenml import Model, pipeline"));
    assert!(!pipeline.contains("import torch"));

    // steps.py declares the four @step functions.
    let steps = files["steps.py"].clone();
    assert!(steps.contains("def load_dataset()"));
    assert!(steps.contains("def train_model(dataset: dict)"));
    assert!(steps.contains("def evaluate_model(dataset: dict, model: dict)"));
    assert!(steps.contains("def register_model(model: dict, metrics: dict)"));
    // Policy semantics reach the package (via the pipeline docstring).
    assert!(pipeline.contains("allow_certified_auditor_close_finding"));
}

#[test]
fn register_step_declares_the_promotion_gate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_zenml(tmp.path()).success();
    let steps = read_tree(tmp.path())["steps.py"].clone();
    assert!(steps.contains("promoted = score >= PROMOTION_THRESHOLD"));
    assert!(steps.contains("METRIC_NAME = \"decision_agreement\""));

    // run.py has a --dry-run that constructs without executing.
    let run = read_tree(tmp.path())["run.py"].clone();
    assert!(run.contains("--dry-run"));
    assert!(run.contains("print(\"Constructed ZenML pipeline:\", pipeline_def.name)"));
    assert!(run.contains("if __name__ == \"__main__\":"));
}

#[test]
fn config_records_the_dag_edges_and_promotion_gate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_zenml(tmp.path()).success();
    let config: serde_json::Value =
        serde_json::from_str(&read_tree(tmp.path())["zenml.config.json"]).expect("valid config");
    assert_eq!(config["metric"]["kind"], "decision_label_agreement");
    assert_eq!(config["promotion_gate"]["threshold"], 0.0);
    assert_eq!(config["model"]["version_keyed_to"], "source_model_hash");
    // Provenance stamp is carried verbatim from the ai-learning family.
    assert!(config["provenance"]["source_model_hash"]
        .as_str()
        .expect("hash")
        .starts_with("sha256:"));
    // The DAG has the five expected edges (load->train, load->eval, train->eval,
    // train->register, eval->register).
    let edges = config["dag_edges"].as_array().expect("edges");
    assert_eq!(edges.len(), 5);
}

// The plan's explicit teeth-check: the ZenML dataset refs must resolve to files
// the ai-learning projection ACTUALLY emits for the SAME fixture. Run both
// projections and assert the referenced train/dev paths are present in the
// ai-learning output — cross-projection consistency (single producer, no
// divergence between the pipeline's inputs and the LLM dataset).
#[test]
fn dataset_refs_resolve_to_ai_learning_output() {
    let base = tempfile::tempdir().expect("tempdir");
    let zenml_out = base.path().join("zenml");
    let ail_out = base.path().join("ai_learning");
    project_zenml(&zenml_out).success();
    project_ai_learning(&ail_out).success();

    let config: serde_json::Value =
        serde_json::from_str(&read_tree(&zenml_out)["zenml.config.json"])
            .expect("valid config json");
    let dataset = &config["dataset"];
    let train_ref = dataset["train"].as_str().expect("train ref");
    let dev_ref = dataset["dev"].as_str().expect("dev ref");

    // The ai-learning output, keyed by relative path (e.g. llm_dataset/train.jsonl).
    let ail_files = read_tree(&ail_out);
    assert!(
        ail_files.contains_key(train_ref),
        "ZenML train ref {train_ref:?} is not an ai-learning artifact; emitted: {:?}",
        ail_files.keys().collect::<Vec<_>>()
    );
    assert!(
        ail_files.contains_key(dev_ref),
        "ZenML dev ref {dev_ref:?} is not an ai-learning artifact"
    );

    // And the referenced files actually carry labeled authorization_qa rows the
    // pipeline's metric can score against (the linkage is not a dangling path).
    let train = &ail_files[train_ref];
    assert!(
        train
            .lines()
            .filter(|l| !l.trim().is_empty())
            .any(|l| l.contains("\"family\":\"authorization_qa\"") && l.contains("\"label\":")),
        "referenced train split has no labeled authorization_qa rows"
    );
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project_zenml(out).success();
    });
}

#[test]
fn zenml_requires_an_authority_environment() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // No --recipe (so no authority_config) and no --authority-config: must fail.
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("zenml")
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
    project_zenml(&file).failure();
}
