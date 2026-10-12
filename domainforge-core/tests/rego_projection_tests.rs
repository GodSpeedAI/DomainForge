//! Rego projection: determinism, typed refusals, and differential fidelity
//! against the DomainForge evaluator.
//!
//! Tests that execute OPA are `#[ignore]`d so a machine without `opa` reports
//! them as ignored rather than passed. `scripts/verify/projection-targets/rego.sh`
//! and CI run them with `--include-ignored`.

use domainforge_core::parse_to_graph;
use domainforge_core::projection::rego::ir::RegoPolicyIr;
use domainforge_core::projection::rego::{self, reference, RegoProjectionError};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const CREATED_AT: &str = "2026-07-11T00:00:00Z";

/// Every construct the projection claims as `isomorphic`, across modalities so
/// the aggregate outcome is exercised too.
const CORPUS: &str = r#"
@namespace "diff"

Policy cap_below per Constraint Obligation priority 5 as:
    forall e in entities: (e.capacity < 100)

Policy cap_at_least per Constraint Prohibition priority 3 as:
    forall e in entities: (e.capacity >= 5)

Policy has_a per Constraint Permission priority 1 as:
    exists e in entities: (e.name = "A")

Policy one_a per Constraint Obligation priority 2 as:
    exists_unique e in entities: (e.name = "A")

Policy none_x per Constraint Obligation priority 4 as:
    forall e in entities: (not (e.kind = "x"))

Policy unique_not_x per Constraint Permission priority 1 as:
    exists_unique e in entities: (not (e.kind = "x"))

Policy mixed_and per Constraint Obligation priority 7 as:
    (forall e in entities: (e.capacity <= 100)) and (exists e in entities: (e.kind != "x"))

Policy mixed_or per Constraint Obligation priority 6 as:
    (exists e in entities: (e.kind = "x")) or (not (exists f in flows: (f.quantity > 10)))

Policy literal_fold per Constraint Obligation priority 1 as:
    (exists_unique e in entities: (e.name = "A")) and (count(flows) >= 1)

Policy count_filtered per Constraint Obligation priority 8 as:
    count(e in entities where e.kind = "x": e) > 1

Policy count_ordering_filter per Constraint Obligation priority 9 as:
    count(f in flows where f.quantity > 5: f.quantity) = 1

Policy names_match per Constraint Permission priority 1 as:
    forall e in entities: (e.name startswith "A" or e.name endswith "e" or e.name contains "x")

Policy nested per Constraint Obligation priority 2 as:
    forall e in entities: (exists f in flows: (f.owner = e.name))

Policy field_to_field per Constraint Obligation priority 2 as:
    forall f in flows: (f.quantity != f.limit and not (f.owner = "nobody"))

Policy constant per Constraint Prohibition priority 1 as: 1 = 1
"#;

fn project(
    source: &str,
) -> Result<std::collections::BTreeMap<String, String>, RegoProjectionError> {
    let graph = parse_to_graph(source).expect("corpus parses");
    rego::project_rego_in_memory(&graph, source, "model.sea", Some(CREATED_AT.to_string()))
}

fn write_tree(files: &std::collections::BTreeMap<String, String>) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "domainforge-rego-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    for (rel, content) in files {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    root
}

fn opa(args: &[&str], dir: &Path) -> (bool, String) {
    let out = Command::new("opa")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("`opa` must be on PATH for the ignored OPA tests");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// Decisions OPA computes for `inputs`, in order, from one process.
fn opa_decisions(dir: &Path, inputs: &[Value]) -> Vec<Value> {
    std::fs::write(
        dir.join("cases.json"),
        serde_json::to_string(&json!({ "cases": inputs })).unwrap(),
    )
    .unwrap();
    let (ok, text) = opa(
        &[
            "eval",
            "--format",
            "json",
            "--bundle",
            "bundle",
            "--input",
            "cases.json",
            "[d | some c in input.cases; d := data.domainforge.policy.decision with input as c]",
        ],
        dir,
    );
    assert!(ok, "opa eval failed: {text}");
    let parsed: Value = serde_json::from_str(&text).expect("opa prints JSON");
    parsed["result"][0]["expressions"][0]["value"]
        .as_array()
        .expect("one decision per case")
        .clone()
}

/// Small deterministic generator; no external randomness.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound
    }
}

fn value_pool() -> Vec<Option<Value>> {
    vec![
        None, // field absent
        Some(Value::Null),
        Some(json!(0)),
        Some(json!(5)),
        Some(json!(5.0)),
        Some(json!(7.5)),
        Some(json!(100)),
        Some(json!(1000)),
        Some(json!("A")),
        Some(json!("B")),
        Some(json!("x")),
        Some(json!("Axe")),
        Some(json!("5")),
        Some(json!("nobody")),
        Some(json!(true)),
        Some(json!(false)),
        Some(json!([1, 2])),
        Some(json!({"k": "v"})),
    ]
}

fn random_input(ir: &RegoPolicyIr, rng: &mut Lcg) -> Value {
    let pool = value_pool();
    let mut facts = Map::new();
    for collection in &ir.collections {
        let items: Vec<Value> = (0..rng.next(4))
            .map(|_| {
                let mut item = Map::new();
                for (owner, field) in &ir.fields {
                    if owner != collection {
                        continue;
                    }
                    if let Some(value) = &pool[rng.next(pool.len())] {
                        item.insert(field.clone(), value.clone());
                    }
                }
                Value::Object(item)
            })
            .collect();
        facts.insert(collection.clone(), Value::Array(items));
    }
    json!({ "facts": facts })
}

#[test]
fn projection_is_deterministic_and_self_describing() {
    let first = project(CORPUS).unwrap();
    let second = project(CORPUS).unwrap();
    assert_eq!(
        first, second,
        "same model and created_at must be byte-identical"
    );

    for required in [
        rego::MANIFEST_FILE,
        rego::DATA_FILE,
        rego::LIB_FILE,
        rego::DECISION_FILE,
        rego::POLICIES_FILE,
        rego::INPUT_SCHEMA_FILE,
        rego::DECISION_SCHEMA_FILE,
        rego::TEST_FILE,
        rego::FIDELITY_FILE,
        rego::PROVENANCE_FILE,
        "fixtures/model.json",
        "fixtures/empty.json",
        "fixtures/incomplete.json",
        "fixtures/malformed.json",
    ] {
        assert!(first.contains_key(required), "missing artifact {required}");
    }

    let hash = rego::model_hash(CORPUS).unwrap();
    let manifest: Value = serde_json::from_str(&first[rego::MANIFEST_FILE]).unwrap();
    assert_eq!(manifest["revision"], json!(hash));
    let provenance: Value = serde_json::from_str(&first[rego::PROVENANCE_FILE]).unwrap();
    assert_eq!(provenance["model"]["hash"], json!(hash));
    assert_eq!(provenance["created_at"], json!(CREATED_AT));

    let fidelity: Value = serde_json::from_str(&first[rego::FIDELITY_FILE]).unwrap();
    assert_eq!(
        fidelity["decision_contract"]["classification"],
        json!("adapter")
    );
    assert_eq!(fidelity["policies"].as_array().unwrap().len(), 15);
    assert!(fidelity["policies"]
        .as_array()
        .unwrap()
        .iter()
        .all(|p| p["classification"] == json!("isomorphic")));
}

#[test]
fn whitespace_only_edits_keep_the_model_identity() {
    let reformatted = CORPUS.replace("\n\n", "\n\n\n");
    assert_eq!(
        rego::model_hash(CORPUS).unwrap(),
        rego::model_hash(&reformatted).unwrap()
    );
}

#[test]
fn unsupported_constructs_are_typed_refusals() {
    let cases = [
        ("Policy p as: total <= 10000", "free_reference"),
        (
            "Policy p as: forall f in flows: (f.quantity + 1 > 2)",
            "arithmetic",
        ),
        (
            "Policy p as: forall e in entities: (e.name matches \"pattern\")",
            "matches",
        ),
        ("Policy p as: sum(flows.quantity) > 10", "aggregation"),
        (
            "Policy p as: forall e in entities: (e.active)",
            "bare_reference_as_condition",
        ),
        (
            "Policy p as: forall f in flows: (f.at before \"2026-01-01T00:00:00Z\")",
            "temporal",
        ),
        (
            "Policy p as: forall e in entities: (count(e in flows where e.quantity > 1: e) > 0)",
            "shadowed_comprehension_variable",
        ),
    ];
    for (policy, construct) in cases {
        let source = format!("@namespace \"refuse\"\n{policy}\n");
        match project(&source) {
            Err(RegoProjectionError::Unrepresentable {
                policy,
                construct: actual,
                ..
            }) => {
                assert_eq!(policy, "refuse::p");
                assert_eq!(actual, construct, "for `{source}`");
            }
            other => panic!("expected REGO001 {construct} for `{source}`, got {other:?}"),
        }
    }
    let error = project("@namespace \"refuse\"\nPolicy p as: total <= 10000\n").unwrap_err();
    assert!(error.to_string().starts_with("REGO001 unrepresentable:"));
}

/// The reference path (collections rebound to a facts document) must agree
/// with evaluating the untouched policy against the graph itself.
#[test]
fn reference_evaluation_matches_the_evaluator_on_the_graph() {
    let source = r#"
@namespace "plant"
Entity "Assembler" in plant
Entity "Axe" in plant
Resource "Camera" units in plant
Flow "Camera" from "Assembler" to "Axe" quantity 12

Policy named_a as: exists_unique e in entities: (e.name = "Assembler")
Policy heavy as: forall f in flows: (f.quantity > 100)
Policy unknown_field as: forall e in entities: (e.capacity < 100)
Policy not_missing as: exists e in entities: (not (e.capacity = 1))
Policy counted as: count(f in flows where f.quantity >= 12: f) = 1
"#;
    let graph = parse_to_graph(source).unwrap();
    let ir = RegoPolicyIr::build(&graph).unwrap();
    let input = reference::facts_from_graph(&graph, &ir).unwrap();
    let decision = reference::decide(&ir, &json!({}), &input);

    for policy in graph.all_policies() {
        let direct = match policy.evaluate_with_mode(&graph, true) {
            Ok(r) => match r.is_satisfied_tristate {
                Some(true) => "true",
                Some(false) => "false",
                None => "unknown",
            },
            Err(_) => "error",
        };
        let key = format!("{}::{}", policy.namespace, policy.name);
        assert_eq!(decision["results"][&key], json!(direct), "policy {key}");
    }
    assert_eq!(decision["results"]["plant::named_a"], json!("true"));
    assert_eq!(decision["results"]["plant::heavy"], json!("false"));
    assert_eq!(
        decision["results"]["plant::unknown_field"],
        json!("unknown")
    );
    assert_eq!(decision["outcome"], json!("deny"));
}

#[test]
fn decision_contract_separates_deny_escalate_and_allow() {
    let source = r#"
@namespace "gate"
Policy must per Constraint Obligation priority 9 as: forall e in entities: (e.capacity < 100)
Policy may per Constraint Permission priority 1 as: exists e in entities: (e.name = "A")
"#;
    let graph = parse_to_graph(source).unwrap();
    let ir = RegoPolicyIr::build(&graph).unwrap();
    let decide = |input: Value| reference::decide(&ir, &json!({}), &input);

    let allow = decide(json!({"facts": {"entities": [{"name": "A", "capacity": 1}]}}));
    assert_eq!(allow["outcome"], json!("allow"));
    assert_eq!(allow["reasons"], json!([]));

    // A violated Permission is reported but does not block (severity info).
    let info = decide(json!({"facts": {"entities": [{"name": "B", "capacity": 1}]}}));
    assert_eq!(info["outcome"], json!("allow"));
    assert_eq!(info["reasons"][0]["code"], json!("violated"));
    assert_eq!(info["reasons"][0]["blocking"], json!(false));

    // A missing fact is Unknown, which escalates; it is never allow.
    let escalate = decide(json!({"facts": {"entities": [{"name": "A"}]}}));
    assert_eq!(escalate["results"]["gate::must"], json!("unknown"));
    assert_eq!(escalate["outcome"], json!("escalate"));

    let deny = decide(json!({"facts": {"entities": [{"name": "A", "capacity": 500}]}}));
    assert_eq!(deny["outcome"], json!("deny"));

    // A type mismatch is an evaluation error, which denies.
    let mismatch = decide(json!({"facts": {"entities": [{"name": "A", "capacity": "big"}]}}));
    assert_eq!(mismatch["results"]["gate::must"], json!("error"));
    assert_eq!(mismatch["reasons"][0]["code"], json!("evaluation_error"));
    assert_eq!(mismatch["outcome"], json!("deny"));

    for malformed in [
        json!({}),
        json!({"facts": []}),
        json!({"facts": {"entities": "all"}}),
    ] {
        let decision = decide(malformed);
        assert_eq!(decision["outcome"], json!("deny"));
        assert_eq!(decision["reasons"][0]["code"], json!("malformed_input"));
        assert!(!decision["input_errors"].as_array().unwrap().is_empty());
    }
}

#[test]
fn repair_agent_fixture_projects() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/cell_env/repair-agent/model.sea");
    let source = std::fs::read_to_string(path).unwrap();
    let files = project(&source).unwrap();
    let fixture: Value = serde_json::from_str(&files["fixtures/model.json"]).unwrap();
    assert_eq!(fixture["expected"]["outcome"], json!("allow"));
    assert_eq!(
        fixture["expected"]["results"]["godspeed.cells.repair::dependency_files_immutable"],
        json!("true")
    );
}

#[test]
#[ignore = "requires opa on PATH"]
fn opa_accepts_the_bundle_and_its_generated_tests() {
    let dir = write_tree(&project(CORPUS).unwrap());

    let (ok, text) = opa(
        &["check", "--strict", "--v1-compatible", "bundle", "tests"],
        &dir,
    );
    assert!(ok, "opa check --strict failed: {text}");

    // The test module embeds fixture JSON verbatim, so only the bundle is held
    // to `opa fmt`.
    let (ok, text) = opa(
        &["fmt", "--v1-compatible", "--fail", "--list", "bundle"],
        &dir,
    );
    assert!(ok, "generated Rego is not opa-fmt clean: {text}");

    let (ok, text) = opa(&["test", "--v1-compatible", "bundle", "tests"], &dir);
    assert!(ok && text.contains("PASS: 4/4"), "opa test failed: {text}");

    let (ok, text) = opa(
        &["build", "--bundle", "bundle", "--output", "bundle.tar.gz"],
        &dir,
    );
    assert!(ok, "opa build failed: {text}");
    std::fs::remove_dir_all(dir).ok();
}

/// A model whose policies can all hold at once, so generated inputs reach
/// allow and escalate as well as deny.
const GATE: &str = r#"
@namespace "gate"
Policy must per Constraint Obligation priority 9 as: forall e in entities: (e.capacity < 100)
Policy may per Constraint Permission priority 1 as: exists e in entities: (e.name = "A")
Policy never per Constraint Prohibition priority 4 as: not (exists e in entities: (e.kind = "x"))
"#;

/// Compare OPA with the evaluator on generated and hand-written inputs.
/// Returns how often each outcome and each per-policy result occurred, so
/// callers can require that the comparison actually reached every branch.
fn differential(
    source: &str,
    seed: u64,
    extra: Vec<Value>,
) -> (
    std::collections::BTreeMap<String, usize>,
    std::collections::BTreeSet<String>,
) {
    let graph = parse_to_graph(source).unwrap();
    let ir = RegoPolicyIr::build(&graph).unwrap();
    let files = project(source).unwrap();
    let identity: Value =
        serde_json::from_str::<Value>(&files[rego::PROVENANCE_FILE]).unwrap()["model"].clone();
    let dir = write_tree(&files);

    let mut rng = Lcg(seed);
    let mut inputs: Vec<Value> = (0..600).map(|_| random_input(&ir, &mut rng)).collect();
    inputs.extend(extra);

    let actual = opa_decisions(&dir, &inputs);
    assert_eq!(actual.len(), inputs.len());

    let mut outcomes = std::collections::BTreeMap::new();
    let mut results = std::collections::BTreeSet::new();
    for (input, actual) in inputs.iter().zip(&actual) {
        let expected = reference::decide(&ir, &identity, input);
        assert_eq!(
            actual, &expected,
            "OPA and the evaluator disagree on input {input}"
        );
        *outcomes
            .entry(expected["outcome"].as_str().unwrap().to_string())
            .or_insert(0usize) += 1;
        for result in expected["results"].as_object().unwrap().values() {
            results.insert(result.as_str().unwrap().to_string());
        }
    }
    std::fs::remove_dir_all(dir).ok();
    (outcomes, results)
}

#[test]
#[ignore = "requires opa on PATH"]
fn opa_agrees_with_the_evaluator_on_every_projected_construct() {
    let malformed = vec![
        json!({}),
        json!({"facts": null}),
        json!({"facts": {"entities": [], "flows": "none"}}),
        json!({"facts": {"entities": [1], "flows": []}}),
        json!({"facts": {"entities": [], "flows": []}}),
    ];
    let (outcomes, results) = differential(CORPUS, 0x5EA_F0E6E, malformed);
    assert!(outcomes.contains_key("deny"));
    for result in ["true", "false", "unknown", "error"] {
        assert!(
            results.contains(result),
            "no generated input produced a {result} result"
        );
    }
}

#[test]
#[ignore = "requires opa on PATH"]
fn opa_agrees_with_the_evaluator_on_every_outcome() {
    let (outcomes, _) = differential(GATE, 0xD0_5EED, vec![json!({"facts": []})]);
    for outcome in ["allow", "deny", "escalate"] {
        assert!(
            outcomes.get(outcome).copied().unwrap_or(0) > 0,
            "no generated input produced {outcome}: {outcomes:?}"
        );
    }
}
