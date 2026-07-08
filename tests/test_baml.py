"""Python binding surface for the BAML projection (export_baml).

The projection is resolver-grounded, so the authority environment is passed
explicitly (the recipe's `authority_config` path is not resolved in-memory).
"""

import json
import pathlib

import domainforge

ROOT = pathlib.Path(__file__).parent.parent
FIXTURE = ROOT / "fixtures" / "baml" / "basic"
FIXED_TS = "2026-07-02T00:00:00+00:00"


def _fixture_graph():
    source = (FIXTURE / "domain" / "model.sea").read_text()
    return domainforge.Graph.parse(source)


def _recipe_json():
    return (FIXTURE / "recipes" / "baml.json").read_text()


def _authority_json():
    return (FIXTURE / "authority" / "environment.json").read_text()


def _export():
    graph = _fixture_graph()
    return json.loads(
        graph.export_baml(
            _recipe_json(),
            _authority_json(),
            "test.sea",
            None,
            FIXED_TS,
        )
    )


def test_export_baml_manifest_and_types():
    artifacts = _export()
    assert set(artifacts) == {
        "README.md",
        "baml_src/clients.baml",
        "baml_src/domain.baml",
        "baml_src/functions.baml",
        "baml_src/tests.baml",
    }
    domain = artifacts["baml_src/domain.baml"]
    assert "enum ActorRole {" in domain
    assert "class AuthorityRequest {" in domain
    assert "class AuthorityRuling {" in domain

    func = artifacts["baml_src/functions.baml"]
    assert "function DecideAuthority(request: AuthorityRequest) -> AuthorityRuling" in func
    # Client is a commented placeholder: no vendor baked in.
    assert "// client<llm> DomainForgeAuthorityClient" in artifacts["baml_src/clients.baml"]


def test_authority_case_appears_verbatim_as_a_test():
    tests = _export()["baml_src/tests.baml"]
    assert 'actor_role "CertifiedAuditor"' in tests
    assert 'operation "close_audit_finding"' in tests
    assert 'resource_type "AuditFinding"' in tests
    assert 'resolves to decision "allow"' in tests


def test_export_baml_is_deterministic():
    graph = _fixture_graph()
    a = graph.export_baml(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    b = graph.export_baml(_recipe_json(), _authority_json(), "test.sea", None, FIXED_TS)
    assert a == b
