"""Python binding surface for the DSPy projection (export_dspy).

The projection is resolver-grounded, so the authority environment is passed
explicitly (the recipe's `authority_config` path is not resolved in-memory).

The plan's teeth-check is a cross-projection consistency check: the DSPy example
refs must resolve to files the ai-learning projection ACTUALLY emits for the
same fixture. This test runs both binding surfaces against the same graph and
asserts path agreement.
"""

import ast
import json
import pathlib

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "dspy" / "basic"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def _recipe_json():
    return (FIXTURE / "recipes" / "dspy.json").read_text()


def _authority_json():
    return (FIXTURE / "authority" / "environment.json").read_text()


def _export_dspy():
    graph = _fixture_graph()
    return json.loads(
        graph.export_dspy(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    )


def _export_ai_learning():
    graph = _fixture_graph()
    return json.loads(
        graph.export_ai_learning(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    )


def test_export_dspy_manifest_and_program():
    artifacts = _export_dspy()
    assert set(artifacts) == {
        "README.md",
        "dspy.config.json",
        "metric.py",
        "optimize.py",
        "program.py",
    }
    program = artifacts["program.py"]
    assert "class AuthorityDecision(dspy.Signature):" in program
    assert "class AuthorityDecisionProgram(dspy.Module):" in program
    assert "self.decide = dspy.Predict(AuthorityDecision)" in program


def test_every_emitted_python_file_parses():
    artifacts = _export_dspy()
    for name, content in artifacts.items():
        if name.endswith(".py"):
            ast.parse(content)  # raises SyntaxError on malformed output


def test_config_declares_the_optimizer_and_regression_gate():
    config = json.loads(_export_dspy()["dspy.config.json"])
    assert config["optimizer"]["name"] == "BootstrapFewShot"
    assert config["optimizer"]["regression_threshold"] == 0.0
    assert config["metric"]["kind"] == "decision_label_agreement"
    assert config["signature"]["module_strategy"] == "predict"
    # Provenance stamp is carried verbatim from the ai-learning family.
    assert config["provenance"]["source_model_hash"].startswith("sha256:")


def test_example_refs_resolve_to_ai_learning_output():
    """The DSPy dataset refs must be files the ai-learning projection emits."""
    dspy_artifacts = _export_dspy()
    ail_artifacts = _export_ai_learning()

    config = json.loads(dspy_artifacts["dspy.config.json"])
    dataset = config["dataset"]

    for key in ("train", "dev"):
        ref = dataset[key]
        assert ref in ail_artifacts, (
            f"DSPy {key} ref {ref!r} is not an ai-learning artifact; "
            f"emitted: {sorted(ail_artifacts)}"
        )

    # The referenced train file carries labeled authorization_qa rows the metric
    # can score against (linkage is real, not a dangling path).
    train = ail_artifacts[dataset["train"]]
    labeled = [
        line
        for line in train.splitlines()
        if line.strip() and '"family":"authorization_qa"' in line and '"label":' in line
    ]
    assert labeled, "referenced train split has no labeled authorization_qa rows"


def test_export_dspy_is_deterministic():
    graph = _fixture_graph()
    a = graph.export_dspy(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    b = graph.export_dspy(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    assert a == b
