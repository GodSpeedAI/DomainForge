#![cfg(feature = "cli")]

//! Integration tests for the DSPy projection
//! (fixtures/dspy/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/dspy/basic")
}

fn recipe() -> PathBuf {
    fixture_dir().join("recipes/dspy.json")
}

fn model() -> PathBuf {
    fixture_dir().join("domain/model.sea")
}

fn project_dspy(out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("dspy")
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
fn emits_the_dspy_program_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_dspy(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(
        files.keys().collect::<Vec<_>>(),
        vec![
            "README.md",
            "dspy.config.json",
            "metric.py",
            "optimize.py",
            "program.py",
        ]
    );
}

#[test]
fn program_defines_the_authority_decision_signature_and_module() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_dspy(tmp.path()).success();
    let program = read_tree(tmp.path())["program.py"].clone();

    // The signature (capability) and the module wrapping it.
    assert!(program.contains("class AuthorityDecision(dspy.Signature):"));
    assert!(program.contains("question: str = dspy.InputField("));
    assert!(program.contains("decision: str = dspy.OutputField("));
    assert!(program.contains("class AuthorityDecisionProgram(dspy.Module):"));
    // Predict strategy (the v1 default) lowers to dspy.Predict.
    assert!(program.contains("self.decide = dspy.Predict(AuthorityDecision)"));
    // Policy semantics are embedded in the signature docstring.
    assert!(program.contains("allow_certified_auditor_close_finding"));
    // Only `dspy` and the standard library are imported.
    assert!(program.contains("import dspy"));
    assert!(!program.contains("import torch"));
    assert!(!program.contains("import openai"));
}

#[test]
fn metric_measures_decision_label_agreement() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_dspy(tmp.path()).success();
    let metric = read_tree(tmp.path())["metric.py"].clone();
    assert!(metric.contains("def decision_match(example, prediction, trace=None):"));
    assert!(metric.contains("gold == pred"));
    // Self-contained: no imports needed.
    assert!(!metric.contains("import "));
}

#[test]
fn optimize_declares_the_regression_gate_and_does_not_run_here() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project_dspy(tmp.path()).success();
    let optimize = read_tree(tmp.path())["optimize.py"].clone();
    // Baseline-vs-optimized comparison + regression enforcement live here.
    assert!(optimize.contains("baseline_score = evaluate(baseline, devset)"));
    assert!(optimize.contains("optimized_score = evaluate(optimized, devset)"));
    assert!(optimize.contains("optimized_score < baseline_score - threshold"));
    // It builds the optimizer but only runs under `__main__` (user's compute).
    assert!(optimize.contains("optimizer.compile("));
    assert!(optimize.contains("if __name__ == \"__main__\":"));
}

// The plan's explicit teeth-check: the DSPy example refs must resolve to files
// the ai-learning projection ACTUALLY emits for the SAME fixture. Run both
// projections and assert the referenced train/dev paths are present in the
// ai-learning output — cross-projection consistency (single producer, no
// divergence between DSPy examples and the LLM dataset).
#[test]
fn example_refs_resolve_to_ai_learning_output() {
    let base = tempfile::tempdir().expect("tempdir");
    let dspy_out = base.path().join("dspy");
    let ail_out = base.path().join("ai_learning");
    project_dspy(&dspy_out).success();
    project_ai_learning(&ail_out).success();

    let config: serde_json::Value =
        serde_json::from_str(&read_tree(&dspy_out)["dspy.config.json"]).expect("valid config json");
    let dataset = &config["dataset"];
    let train_ref = dataset["train"].as_str().expect("train ref");
    let dev_ref = dataset["dev"].as_str().expect("dev ref");

    // The ai-learning output, keyed by relative path (e.g. llm_dataset/train.jsonl).
    let ail_files = read_tree(&ail_out);
    assert!(
        ail_files.contains_key(train_ref),
        "DSPy train ref {train_ref:?} is not an ai-learning artifact; \
         emitted: {:?}",
        ail_files.keys().collect::<Vec<_>>()
    );
    assert!(
        ail_files.contains_key(dev_ref),
        "DSPy dev ref {dev_ref:?} is not an ai-learning artifact"
    );

    // And the referenced files actually carry labeled authorization_qa rows the
    // DSPy metric can score against (the linkage is not merely a dangling path).
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
        project_dspy(out).success();
    });
}

#[test]
fn dspy_requires_an_authority_environment() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // No --recipe (so no authority_config) and no --authority-config: must fail.
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("dspy")
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
    project_dspy(&file).failure();
}
