"""Python binding surface for AI Learning Projections (export_ai_learning)."""

import json
import pathlib

import domainforge

FIXTURE = pathlib.Path(__file__).parent.parent / "fixtures" / "ai_learning" / "manufacturing_quality"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def test_export_ai_learning_full_recipe():
    graph = _fixture_graph()
    recipe_json = (FIXTURE / "recipes" / "ai_learning.json").read_text()
    authority_json = (FIXTURE / "authority" / "environment.json").read_text()

    artifacts = json.loads(
        graph.export_ai_learning(
            recipe_json=recipe_json,
            authority_config_json=authority_json,
            created_at=FIXED_TS,
        )
    )

    # Same layout as `domainforge project --format ai-learning`.
    for path in [
        "llm_dataset/train.jsonl",
        "llm_dataset/validation_report.json",
        "graph_dataset/graph.json",
        "graph_dataset/negative_samples.json",
        "cep_eval/dataset.json",
        "cep_eval/reports/coverage_report.json",
    ]:
        assert path in artifacts, f"missing artifact {path}"

    # Labels are resolver-grounded; unknown tuples never guess.
    train = artifacts["llm_dataset/train.jsonl"]
    assert "does not contain enough authority information" in train

    report = json.loads(artifacts["llm_dataset/validation_report.json"])
    assert report["status"] == "passed"
    assert report["resolver_disagreement_count"] == 0


def test_export_ai_learning_deterministic():
    graph = _fixture_graph()
    recipe_json = (FIXTURE / "recipes" / "ai_learning.json").read_text()
    authority_json = (FIXTURE / "authority" / "environment.json").read_text()
    kwargs = dict(
        recipe_json=recipe_json,
        authority_config_json=authority_json,
        created_at=FIXED_TS,
    )
    assert graph.export_ai_learning(**kwargs) == graph.export_ai_learning(**kwargs)


def test_export_ai_learning_without_authority():
    """Families that need no resolver work without an authority config."""
    graph = _fixture_graph()
    recipe = {
        "name": "no_authority",
        "projections": {
            "llm_dataset": {"enabled": True, "families": ["entity_lookup"]},
            "graph_ml_dataset": {
                "enabled": True,
                "tasks": ["node_classification", "link_prediction"],
            },
            "cep_eval_dataset": {"enabled": False},
        },
    }
    artifacts = json.loads(
        graph.export_ai_learning(recipe_json=json.dumps(recipe), created_at=FIXED_TS)
    )
    assert "llm_dataset/train.jsonl" in artifacts
    assert "graph_dataset/graph.json" in artifacts
    assert not any(p.startswith("cep_eval/") for p in artifacts)


def test_export_ai_learning_requires_authority_for_authz():
    """Resolver-grounded families must fail loudly without an authority config."""
    graph = _fixture_graph()
    try:
        graph.export_ai_learning(created_at=FIXED_TS)
    except ValueError as e:
        assert "authority" in str(e)
    else:
        raise AssertionError("expected ValueError without authority config")
