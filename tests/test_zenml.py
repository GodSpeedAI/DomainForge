"""Python binding surface for the ZenML projection (export_zenml).

The projection is resolver-grounded, so the authority environment is passed
explicitly (the recipe's `authority_config` path is not resolved in-memory).

The plan's teeth-check is a cross-projection consistency check: the ZenML
dataset refs must resolve to files the ai-learning projection ACTUALLY emits for
the same fixture. This test runs both binding surfaces against the same graph
and asserts path agreement.
"""

import ast
import json
import pathlib

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "zenml" / "basic"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def _recipe_json():
    return (FIXTURE / "recipes" / "zenml.json").read_text()


def _authority_json():
    return (FIXTURE / "authority" / "environment.json").read_text()


def _export_zenml():
    graph = _fixture_graph()
    return json.loads(
        graph.export_zenml(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    )


def _export_ai_learning():
    graph = _fixture_graph()
    return json.loads(
        graph.export_ai_learning(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    )


def test_export_zenml_manifest_and_pipeline():
    artifacts = _export_zenml()
    assert set(artifacts) == {
        "README.md",
        "pipeline.py",
        "requirements.txt",
        "run.py",
        "steps.py",
        "zenml.config.json",
    }
    pipeline = artifacts["pipeline.py"]
    assert "def authority_learning_pipeline():" in pipeline
    assert "from zenml import Model, pipeline" in pipeline
    steps = artifacts["steps.py"]
    for fn in ("load_dataset", "train_model", "evaluate_model", "register_model"):
        assert f"def {fn}(" in steps


def test_every_emitted_python_file_parses():
    artifacts = _export_zenml()
    for name, content in artifacts.items():
        if name.endswith(".py"):
            ast.parse(content)  # raises SyntaxError on malformed output


def test_config_declares_the_promotion_gate_and_model_version():
    config = json.loads(_export_zenml()["zenml.config.json"])
    assert config["metric"]["kind"] == "decision_label_agreement"
    assert config["promotion_gate"]["threshold"] == 0.0
    assert config["model"]["version_keyed_to"] == "source_model_hash"
    # The model version is keyed to the model hash (v-<hash>).
    assert config["model"]["version"].startswith("v-")
    assert len(config["dag_edges"]) == 5
    # Provenance stamp is carried verbatim from the ai-learning family.
    assert config["provenance"]["source_model_hash"].startswith("sha256:")


def test_dataset_refs_resolve_to_ai_learning_output():
    """The ZenML dataset refs must be files the ai-learning projection emits."""
    zenml_artifacts = _export_zenml()
    ail_artifacts = _export_ai_learning()

    config = json.loads(zenml_artifacts["zenml.config.json"])
    dataset = config["dataset"]

    for key in ("train", "dev"):
        ref = dataset[key]
        assert ref in ail_artifacts, (
            f"ZenML {key} ref {ref!r} is not an ai-learning artifact; "
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


def test_export_zenml_is_deterministic():
    graph = _fixture_graph()
    a = graph.export_zenml(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    b = graph.export_zenml(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    assert a == b
