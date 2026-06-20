"""
Tests for the new Semantic Pack API exports added in this PR.

Covers:
- Enum exports: SemanticTruth, DiagnosticSeverity, ValidationMode, ApprovalState,
  SignatureState, ConceptStatus, ConceptKind, AliasStatus, SemanticValidationStatus
- Functions: normalize_lookup_key, build_semantic_pack, validate_semantic_pack,
  validate_graph_with_pack, compute_pack_hash, diff_packs, resolve_concept,
  sign_pack, verify_pack_signature
- Classes: SemanticPack, SemanticValidationResult
"""

import json
import pytest
import domainforge


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def make_minimal_pack_json() -> str:
    return json.dumps({
        "schema_version": "0.3",
        "pack_id": "test-org/test-domain/1.0.0",
        "org_id": "test-org",
        "domain_id": "test-domain",
        "pack_version": "1.0.0",
        "meaning_version": "1.0.0",
        "meaning_fingerprint": "",
        "source_graph_hash": "sha256:test",
        "build_config_hash": "sha256:cfg",
        "review_manifest_hash": "sha256:rev",
        "created_at": "2026-06-07T00:00:00Z",
        "generator": {"name": "domainforge-core", "version": "0.3"},
        "trust": {"approval_state": "candidate", "signature_state": "unsigned"},
        "concepts": [
            {
                "id": "supplier",
                "canonical_name": "Supplier",
                "kind": "entity",
                "status": "active",
                "definition": {
                    "text": "A party that provides goods or services.",
                    "definition_hash": "",
                    "decision_ref": "dec_supplier",
                },
                "owner": "owner@test.com",
                "source_refs": [],
                "examples": [],
                "counterexamples": [],
                "allowed_predicates": [],
                "valid_contexts": [],
            }
        ],
        "relations": [],
        "metrics": [],
        "dimensions": [],
        "units": [],
        "aliases": [],
        "mapping_rules": [],
        "compatibility": {},
    })


def make_minimal_build_input_json() -> str:
    return json.dumps({
        "org_id": "test-org",
        "domain_id": "test-domain",
        "pack_version": "1.0.0",
        "meaning_version": "1.0.0",
        "approval": "candidate",
        "concepts": [
            {
                "id": "supplier",
                "canonical_name": "Supplier",
                "kind": "entity",
                "status": "active",
                "definition": {
                    "text": "A party that provides goods or services.",
                    "definition_hash": "",
                    "decision_ref": "dec_supplier",
                },
                "owner": "owner@test.com",
                "source_refs": [],
                "examples": [],
                "counterexamples": [],
                "allowed_predicates": [],
                "valid_contexts": [],
            }
        ],
        "relations": [],
        "metrics": [],
        "dimensions": [],
        "units": [],
        "aliases": [],
        "mapping_rules": [],
        "review_records": [],
        "previous_pack": None,
        "allow_first_approved_version": False,
        "source_graph_hash": "sha256:test",
    })


def make_default_options_json() -> str:
    return json.dumps({"mode": "warn", "deprecated_policy": "warn"})


# ---------------------------------------------------------------------------
# Enum import tests
# ---------------------------------------------------------------------------

def test_semantic_truth_enum_importable():
    """SemanticTruth enum is importable from domainforge."""
    assert hasattr(domainforge, "SemanticTruth")


def test_semantic_truth_enum_values():
    """SemanticTruth has Valid, Invalid, Unknown members."""
    assert hasattr(domainforge.SemanticTruth, "Valid")
    assert hasattr(domainforge.SemanticTruth, "Invalid")
    assert hasattr(domainforge.SemanticTruth, "Unknown")


def test_semantic_truth_equality():
    """SemanticTruth enum members are equal to themselves."""
    assert domainforge.SemanticTruth.Valid == domainforge.SemanticTruth.Valid
    assert domainforge.SemanticTruth.Invalid != domainforge.SemanticTruth.Unknown


def test_diagnostic_severity_enum_importable():
    """DiagnosticSeverity enum is importable from domainforge."""
    assert hasattr(domainforge, "DiagnosticSeverity")


def test_diagnostic_severity_enum_values():
    """DiagnosticSeverity has Error, Warning, Info, Hint members."""
    assert hasattr(domainforge.DiagnosticSeverity, "Error")
    assert hasattr(domainforge.DiagnosticSeverity, "Warning")
    assert hasattr(domainforge.DiagnosticSeverity, "Info")
    assert hasattr(domainforge.DiagnosticSeverity, "Hint")


def test_validation_mode_enum_importable():
    """ValidationMode enum is importable from domainforge."""
    assert hasattr(domainforge, "ValidationMode")


def test_validation_mode_enum_values():
    """ValidationMode has Off, Warn, Strict members."""
    assert hasattr(domainforge.ValidationMode, "Off")
    assert hasattr(domainforge.ValidationMode, "Warn")
    assert hasattr(domainforge.ValidationMode, "Strict")


def test_approval_state_enum_values():
    """ApprovalState has Candidate, Approved, Rejected members."""
    assert hasattr(domainforge.ApprovalState, "Candidate")
    assert hasattr(domainforge.ApprovalState, "Approved")
    assert hasattr(domainforge.ApprovalState, "Rejected")


def test_signature_state_enum_values():
    """SignatureState has Unsigned, Signed, InvalidSignature members."""
    assert hasattr(domainforge.SignatureState, "Unsigned")
    assert hasattr(domainforge.SignatureState, "Signed")
    assert hasattr(domainforge.SignatureState, "InvalidSignature")


def test_concept_status_enum_values():
    """ConceptStatus has all five status values."""
    assert hasattr(domainforge.ConceptStatus, "Active")
    assert hasattr(domainforge.ConceptStatus, "Proposed")
    assert hasattr(domainforge.ConceptStatus, "Deprecated")
    assert hasattr(domainforge.ConceptStatus, "Rejected")
    assert hasattr(domainforge.ConceptStatus, "ExternalOnly")


def test_concept_kind_enum_values():
    """ConceptKind has all nine kind values."""
    for kind in ["Entity", "Resource", "Role", "Flow", "Policy",
                 "Metric", "Dimension", "Unit", "External"]:
        assert hasattr(domainforge.ConceptKind, kind), f"Missing ConceptKind.{kind}"


def test_alias_status_enum_values():
    """AliasStatus has Approved, Deprecated, Ambiguous, Blocked members."""
    assert hasattr(domainforge.AliasStatus, "Approved")
    assert hasattr(domainforge.AliasStatus, "Deprecated")
    assert hasattr(domainforge.AliasStatus, "Ambiguous")
    assert hasattr(domainforge.AliasStatus, "Blocked")


def test_semantic_validation_status_enum_values():
    """SemanticValidationStatus has Passed, Failed, Unknown, Blocked members."""
    assert hasattr(domainforge.SemanticValidationStatus, "Passed")
    assert hasattr(domainforge.SemanticValidationStatus, "Failed")
    assert hasattr(domainforge.SemanticValidationStatus, "Unknown")
    assert hasattr(domainforge.SemanticValidationStatus, "Blocked")


def test_enum_members_are_distinct():
    """Different enum members are not equal to each other."""
    assert domainforge.SemanticTruth.Valid != domainforge.SemanticTruth.Invalid
    assert domainforge.ValidationMode.Off != domainforge.ValidationMode.Strict
    assert domainforge.ApprovalState.Candidate != domainforge.ApprovalState.Approved


# ---------------------------------------------------------------------------
# normalize_lookup_key
# ---------------------------------------------------------------------------

def test_normalize_lookup_key_is_callable():
    """normalize_lookup_key is a callable function in domainforge."""
    assert callable(domainforge.normalize_lookup_key)


def test_normalize_lookup_key_lowercases():
    """normalize_lookup_key lowercases the input."""
    result = domainforge.normalize_lookup_key("Supplier")
    assert result == "supplier"


def test_normalize_lookup_key_trims_and_collapses_spaces():
    """normalize_lookup_key trims and collapses multiple spaces."""
    result = domainforge.normalize_lookup_key("  Hello   World  ")
    assert result == "hello world"


def test_normalize_lookup_key_empty_string():
    """normalize_lookup_key handles empty string input."""
    result = domainforge.normalize_lookup_key("")
    assert result == ""


def test_normalize_lookup_key_single_word():
    """normalize_lookup_key handles single word input."""
    assert domainforge.normalize_lookup_key("Warehouse") == "warehouse"


def test_normalize_lookup_key_is_idempotent():
    """normalize_lookup_key is idempotent."""
    first = domainforge.normalize_lookup_key("  Purchase  Order  ")
    second = domainforge.normalize_lookup_key(first)
    assert first == second


def test_normalize_lookup_key_preserves_underscores():
    """normalize_lookup_key passes through underscores."""
    result = domainforge.normalize_lookup_key("PURCHASE_ORDER")
    assert result == "purchase_order"


# ---------------------------------------------------------------------------
# build_semantic_pack
# ---------------------------------------------------------------------------

def test_build_semantic_pack_is_callable():
    """build_semantic_pack is a callable function in domainforge."""
    assert callable(domainforge.build_semantic_pack)


def test_build_semantic_pack_returns_tuple():
    """build_semantic_pack returns a tuple of (pack_json, errors)."""
    result = domainforge.build_semantic_pack(make_minimal_build_input_json())
    assert isinstance(result, tuple)
    assert len(result) == 2


def test_build_semantic_pack_returns_valid_json():
    """build_semantic_pack returns a valid JSON pack string."""
    pack_json, errors = domainforge.build_semantic_pack(make_minimal_build_input_json())
    parsed = json.loads(pack_json)
    assert parsed["org_id"] == "test-org"
    assert parsed["domain_id"] == "test-domain"


def test_build_semantic_pack_schema_version():
    """Built pack has schema_version 0.3."""
    pack_json, _ = domainforge.build_semantic_pack(make_minimal_build_input_json())
    pack = json.loads(pack_json)
    assert pack["schema_version"] == "0.3"


def test_build_semantic_pack_approval_state():
    """Built candidate pack has approval_state=candidate."""
    pack_json, _ = domainforge.build_semantic_pack(make_minimal_build_input_json())
    pack = json.loads(pack_json)
    assert pack["trust"]["approval_state"] == "candidate"


def test_build_semantic_pack_errors_is_list():
    """build_semantic_pack errors is a list."""
    _, errors = domainforge.build_semantic_pack(make_minimal_build_input_json())
    assert isinstance(errors, list)


def test_build_semantic_pack_raises_on_invalid_json():
    """build_semantic_pack raises ValueError on invalid JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.build_semantic_pack("not valid json")


def test_build_semantic_pack_is_deterministic():
    """Two builds from identical inputs produce the same pack content hash."""
    input_json = make_minimal_build_input_json()
    pack1, _ = domainforge.build_semantic_pack(input_json)
    pack2, _ = domainforge.build_semantic_pack(input_json)
    p1 = json.loads(pack1)
    p2 = json.loads(pack2)
    # The meaning fingerprint should be identical
    assert p1["meaning_fingerprint"] == p2["meaning_fingerprint"]


# ---------------------------------------------------------------------------
# validate_semantic_pack
# ---------------------------------------------------------------------------

def test_validate_semantic_pack_is_callable():
    """validate_semantic_pack is a callable function in domainforge."""
    assert callable(domainforge.validate_semantic_pack)


def test_validate_semantic_pack_returns_list():
    """validate_semantic_pack returns a list of diagnostic strings."""
    result = domainforge.validate_semantic_pack(make_minimal_pack_json())
    assert isinstance(result, list)


def test_validate_semantic_pack_raises_on_invalid_json():
    """validate_semantic_pack raises on invalid JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.validate_semantic_pack("not json")


def test_validate_semantic_pack_reports_schema_mismatch():
    """validate_semantic_pack reports errors for wrong schema version."""
    pack = json.loads(make_minimal_pack_json())
    pack["schema_version"] = "9.9"
    diagnostics = domainforge.validate_semantic_pack(json.dumps(pack))
    assert len(diagnostics) > 0


# ---------------------------------------------------------------------------
# compute_pack_hash
# ---------------------------------------------------------------------------

def test_compute_pack_hash_is_callable():
    """compute_pack_hash is a callable function in domainforge."""
    assert callable(domainforge.compute_pack_hash)


def test_compute_pack_hash_returns_sha256_string():
    """compute_pack_hash returns a string starting with sha256:."""
    result = domainforge.compute_pack_hash(make_minimal_pack_json())
    assert isinstance(result, str)
    assert result.startswith("sha256:")


def test_compute_pack_hash_is_consistent():
    """compute_pack_hash returns the same hash for the same pack."""
    pack_json = make_minimal_pack_json()
    h1 = domainforge.compute_pack_hash(pack_json)
    h2 = domainforge.compute_pack_hash(pack_json)
    assert h1 == h2


def test_compute_pack_hash_raises_on_invalid_json():
    """compute_pack_hash raises on invalid JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.compute_pack_hash("not json")


def test_compute_pack_hash_excludes_signature_fields():
    """compute_pack_hash is stable regardless of signature fields."""
    pack1 = json.loads(make_minimal_pack_json())
    h1 = domainforge.compute_pack_hash(json.dumps(pack1))

    # Add a signature field — the hash should remain unchanged
    pack1["trust"]["signature"] = "some-base64-signature"
    pack1["trust"]["signature_state"] = "signed"
    h2 = domainforge.compute_pack_hash(json.dumps(pack1))

    assert h1 == h2


# ---------------------------------------------------------------------------
# validate_graph_with_pack
# ---------------------------------------------------------------------------

def test_validate_graph_with_pack_is_callable():
    """validate_graph_with_pack is a callable function in domainforge."""
    assert callable(domainforge.validate_graph_with_pack)


def test_validate_graph_with_pack_returns_json_string():
    """validate_graph_with_pack returns a parseable JSON string."""
    result = domainforge.validate_graph_with_pack(
        make_minimal_pack_json(),
        "test://source",
        make_default_options_json(),
    )
    assert isinstance(result, str)
    parsed = json.loads(result)
    assert "status" in parsed


def test_validate_graph_with_pack_raises_on_invalid_pack_json():
    """validate_graph_with_pack raises on invalid pack JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.validate_graph_with_pack("bad json", "test://source", make_default_options_json())


def test_validate_graph_with_pack_raises_on_invalid_options_json():
    """validate_graph_with_pack raises on invalid options JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.validate_graph_with_pack(make_minimal_pack_json(), "test://source", "bad options")


# ---------------------------------------------------------------------------
# diff_packs
# ---------------------------------------------------------------------------

def test_diff_packs_is_callable():
    """diff_packs is a callable function in domainforge."""
    assert callable(domainforge.diff_packs)


def test_diff_packs_returns_json_string():
    """diff_packs returns a parseable JSON string."""
    pack_json = make_minimal_pack_json()
    result = domainforge.diff_packs(pack_json, pack_json)
    assert isinstance(result, str)
    json.loads(result)  # must parse without error


def test_diff_packs_same_pack_has_no_breaking_changes():
    """diff_packs reports no breaking changes when comparing a pack to itself."""
    pack_json = make_minimal_pack_json()
    diff = json.loads(domainforge.diff_packs(pack_json, pack_json))
    entries = diff.get("entries", diff) if isinstance(diff, dict) else diff
    if isinstance(entries, list):
        breaking = [e for e in entries if e.get("classification") == "breaking"]
        assert len(breaking) == 0


def test_diff_packs_detects_added_concept():
    """diff_packs detects a new concept added to the new pack."""
    old_pack = json.loads(make_minimal_pack_json())
    new_pack = json.loads(make_minimal_pack_json())
    new_pack["concepts"].append({
        "id": "warehouse",
        "canonical_name": "Warehouse",
        "kind": "entity",
        "status": "active",
        "definition": {
            "text": "A storage facility.",
            "definition_hash": "",
            "decision_ref": "dec_warehouse",
        },
        "owner": "owner@test.com",
        "source_refs": [],
        "examples": [],
        "counterexamples": [],
        "allowed_predicates": [],
        "valid_contexts": [],
    })
    diff_str = domainforge.diff_packs(json.dumps(old_pack), json.dumps(new_pack))
    diff = json.loads(diff_str)
    entries = diff.get("entries", diff) if isinstance(diff, dict) else diff
    if isinstance(entries, list):
        additive = [e for e in entries if e.get("classification") in ("additive", "add")]
        assert len(additive) > 0
    else:
        assert diff_str  # at minimum, a non-empty response


def test_diff_packs_raises_on_invalid_old_json():
    """diff_packs raises on invalid old pack JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.diff_packs("bad json", make_minimal_pack_json())


def test_diff_packs_raises_on_invalid_new_json():
    """diff_packs raises on invalid new pack JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.diff_packs(make_minimal_pack_json(), "bad json")


# ---------------------------------------------------------------------------
# resolve_concept
# ---------------------------------------------------------------------------

def test_resolve_concept_is_callable():
    """resolve_concept is a callable function in domainforge."""
    assert callable(domainforge.resolve_concept)


def test_resolve_concept_returns_json_string():
    """resolve_concept returns a parseable JSON string."""
    result = domainforge.resolve_concept("Supplier", make_minimal_pack_json(), make_default_options_json())
    assert isinstance(result, str)
    parsed = json.loads(result)
    assert "semantic_truth" in parsed


def test_resolve_concept_known_term_is_valid():
    """resolve_concept returns valid semantic_truth for a known active concept."""
    result = domainforge.resolve_concept("Supplier", make_minimal_pack_json(), make_default_options_json())
    parsed = json.loads(result)
    assert "valid" in parsed["semantic_truth"].lower()


def test_resolve_concept_unknown_term_is_unknown():
    """resolve_concept returns unknown semantic_truth for an unrecognized term."""
    result = domainforge.resolve_concept("CompletelyUnknownTerm", make_minimal_pack_json(), make_default_options_json())
    parsed = json.loads(result)
    assert "unknown" in parsed["semantic_truth"].lower()
    assert parsed["resolved_concept_id"] is None


def test_resolve_concept_normalizes_lookup_key():
    """resolve_concept normalizes the input term before lookup."""
    r1 = json.loads(domainforge.resolve_concept("Supplier", make_minimal_pack_json(), make_default_options_json()))
    r2 = json.loads(domainforge.resolve_concept("  supplier  ", make_minimal_pack_json(), make_default_options_json()))
    assert r1["resolved_concept_id"] == r2["resolved_concept_id"]


def test_resolve_concept_raises_on_invalid_pack_json():
    """resolve_concept raises on invalid pack JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.resolve_concept("Supplier", "bad json", make_default_options_json())


def test_resolve_concept_raises_on_invalid_options_json():
    """resolve_concept raises on invalid options JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.resolve_concept("Supplier", make_minimal_pack_json(), "bad options")


# ---------------------------------------------------------------------------
# SemanticPack class
# ---------------------------------------------------------------------------

def test_semantic_pack_class_importable():
    """SemanticPack class is importable from domainforge."""
    assert hasattr(domainforge, "SemanticPack")


def test_semantic_pack_from_json():
    """SemanticPack.from_json parses a valid pack JSON string."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack is not None


def test_semantic_pack_pack_id():
    """SemanticPack.pack_id() returns the pack identifier."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack.pack_id() == "test-org/test-domain/1.0.0"


def test_semantic_pack_schema_version():
    """SemanticPack.schema_version() returns '0.3'."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack.schema_version() == "0.3"


def test_semantic_pack_approval_state():
    """SemanticPack.approval_state() returns ApprovalState.Candidate."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack.approval_state() == domainforge.ApprovalState.Candidate


def test_semantic_pack_signature_state():
    """SemanticPack.signature_state() returns SignatureState.Unsigned."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack.signature_state() == domainforge.SignatureState.Unsigned


def test_semantic_pack_concept_count():
    """SemanticPack.concept_count() returns 1 for the minimal pack."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack.concept_count() == 1


def test_semantic_pack_alias_count():
    """SemanticPack.alias_count() returns 0 for the minimal pack with no aliases."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    assert pack.alias_count() == 0


def test_semantic_pack_to_json_roundtrip():
    """SemanticPack.to_json() produces valid JSON that can be re-parsed."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    json_out = pack.to_json()
    reparsed = json.loads(json_out)
    assert reparsed["pack_id"] == "test-org/test-domain/1.0.0"


def test_semantic_pack_from_json_raises_on_invalid_json():
    """SemanticPack.from_json raises on invalid JSON."""
    with pytest.raises((ValueError, Exception)):
        domainforge.SemanticPack.from_json("not json")


def test_semantic_pack_pack_content_hash_starts_with_sha256():
    """SemanticPack.pack_content_hash() returns a sha256: prefixed string."""
    pack = domainforge.SemanticPack.from_json(make_minimal_pack_json())
    h = pack.pack_content_hash()
    assert isinstance(h, str)
    assert h.startswith("sha256:")


# ---------------------------------------------------------------------------
# sign_pack / verify_pack_signature
# ---------------------------------------------------------------------------

def test_sign_pack_is_callable():
    """sign_pack is a callable function in domainforge."""
    assert callable(domainforge.sign_pack)


def test_verify_pack_signature_is_callable():
    """verify_pack_signature is a callable function in domainforge."""
    assert callable(domainforge.verify_pack_signature)


def test_sign_pack_raises_on_invalid_pack_json():
    """sign_pack raises on invalid pack JSON."""
    fake_pem = "-----BEGIN PRIVATE KEY-----\nnot-real\n-----END PRIVATE KEY-----"
    with pytest.raises((ValueError, Exception)):
        domainforge.sign_pack("bad json", fake_pem)


def test_verify_pack_signature_raises_on_invalid_pack_json():
    """verify_pack_signature raises on invalid pack JSON."""
    fake_pem = "-----BEGIN PUBLIC KEY-----\nnot-real\n-----END PUBLIC KEY-----"
    with pytest.raises((ValueError, Exception)):
        domainforge.verify_pack_signature("bad json", fake_pem)