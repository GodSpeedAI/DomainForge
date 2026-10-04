#!/usr/bin/env python3
"""Independent, stdlib-only reference for the `world_ref` contract.

Shares no code with the Rust core. It recomputes DomainModelIdentity's
canonical digest (sha256 over canonical JSON: sorted keys, no whitespace,
UTF-8) and the `world:<name>@sha256:<hex>` grammar, and generates/verifies
golden-vectors.json. Usage:

    python3 world_ref_reference.py --write   # regenerate golden-vectors.json
    python3 world_ref_reference.py --check   # verify the committed vectors
"""
import hashlib
import json
import re
import sys
from pathlib import Path

VECTORS = Path(__file__).with_name("golden-vectors.json")
SCHEME = "v2-full-preimage"
NAME_RE = re.compile(r"^[a-z][a-z0-9]*([._-][a-z0-9]+)*$")
HEX_RE = re.compile(r"^[0-9a-f]{64}$")
IDENTITY_FIELDS = [
    "identity_scheme_version", "producer", "producer_version",
    "language_schema_version", "compiler_interpretation_version",
    "canonicalization_version", "source_set_hash", "content_hash",
    "semantic_closure_hash", "semantic_pack_set_hash", "registry_content_hash",
]


def canonical_json(value) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def canonical_digest(identity: dict) -> str:
    assert sorted(identity) == sorted(IDENTITY_FIELDS), "identity must carry exactly the 11 fields"
    return "sha256:" + hashlib.sha256(canonical_json(identity).encode("utf-8")).hexdigest()


def valid_name(name: str) -> bool:
    return 1 <= len(name) <= 64 and NAME_RE.match(name) is not None


def parse_world_ref(text: str):
    """Return (name, digest) or None if text is not a canonical world_ref."""
    if not text.startswith("world:") or "@" not in text:
        return None
    name, digest = text[len("world:"):].split("@", 1)
    if not valid_name(name) or not digest.startswith("sha256:"):
        return None
    return (name, digest) if HEX_RE.match(digest[len("sha256:"):]) else None


def parse_alias(text: str):
    if not text.startswith("world:") or "@" in text:
        return None
    name = text[len("world:"):]
    return name if valid_name(name) else None


def h(c: str) -> str:
    return "sha256:" + c * 64


def identity(**over) -> dict:
    base = {
        "identity_scheme_version": SCHEME,
        "producer": "domainforge-core",
        "producer_version": "0.18.2",
        "language_schema_version": "domainforge-ast/v3",
        "compiler_interpretation_version": "domainforge-interpretation/v1",
        "canonicalization_version": "domainforge-canonicalization/v1",
        "source_set_hash": h("1"),
        "content_hash": h("2"),
        "semantic_closure_hash": h("3"),
        "semantic_pack_set_hash": None,
        "registry_content_hash": None,
    }
    base.update(over)
    return base


def build() -> dict:
    cases = [
        ("minimal", "godspeed-corporate", identity()),
        ("with-packs-and-registry", "godspeed-corporate",
         identity(semantic_pack_set_hash=h("4"), registry_content_hash=h("5"))),
        ("semantic-change", "godspeed-corporate", identity(semantic_closure_hash=h("9"))),
        ("source-change-same-closure", "godspeed-corporate", identity(source_set_hash=h("8"))),
        ("compiler-upgrade-same-closure", "godspeed-corporate", identity(producer_version="0.19.0")),
        ("dotted-name", "acme.supply_chain-v2", identity()),
    ]
    valid = []
    for label, name, ident in cases:
        digest = canonical_digest(ident)
        valid.append({"case": label, "name": name, "identity": ident,
                      "canonical_digest": digest, "world_ref": f"world:{name}@{digest}"})
    hexa = "a" * 64
    return {
        "format": "world:<name>@sha256:<64 lowercase hex>",
        "identity_scheme_version": SCHEME,
        "valid": valid,
        "valid_aliases": ["world:corporate", "world:godspeed-corporate", "world:a1.b_c-d"],
        "invalid_refs": [
            "", "world:", "world:corp", f"corp@sha256:{hexa}", f"world:Corp@sha256:{hexa}",
            f"world:corp@sha1:{hexa}", f"world:corp@sha256:{'a' * 63}",
            f"world:corp@sha256:{'A' * 64}", f"world:corp@sha256:{'g' * 64}",
            f" world:corp@sha256:{hexa}", f"world:corp@sha256:{hexa} ",
            f"world:corp@@sha256:{hexa}", f"world:-corp@sha256:{hexa}",
            f"world:corp--x@sha256:{hexa}", f"world:{'a' * 65}@sha256:{hexa}",
        ],
        "invalid_aliases": ["", "world:", "corporate", "world:Corporate", f"world:corp@sha256:{hexa}"],
    }


def check(vectors: dict) -> list:
    errors = []
    for v in vectors["valid"]:
        if canonical_digest(v["identity"]) != v["canonical_digest"]:
            errors.append(f"{v['case']}: digest mismatch")
        if parse_world_ref(v["world_ref"]) != (v["name"], v["canonical_digest"]):
            errors.append(f"{v['case']}: world_ref does not parse back")
    errors += [f"accepted invalid ref {r!r}" for r in vectors["invalid_refs"] if parse_world_ref(r)]
    errors += [f"rejected valid alias {a!r}" for a in vectors["valid_aliases"] if parse_alias(a) is None]
    errors += [f"accepted invalid alias {a!r}" for a in vectors["invalid_aliases"] if parse_alias(a)]
    # Digest is a function of the identity alone: equal identities share a
    # digest (the name is not part of it); different identities never do.
    seen = {}
    for v in vectors["valid"]:
        key = canonical_json(v["identity"])
        if seen.setdefault(v["canonical_digest"], key) != key:
            errors.append(f"{v['case']}: different identities collided on a digest")
    if len({canonical_json(v["identity"]) for v in vectors["valid"]}) != len(seen):
        errors.append("an identity produced more than one digest")
    return errors


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else "--check"
    if mode == "--write":
        VECTORS.write_text(json.dumps(build(), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"wrote {VECTORS}")
    errs = check(json.loads(VECTORS.read_text(encoding="utf-8")))
    print("OK" if not errs else "\n".join(errs))
    sys.exit(1 if errs else 0)
