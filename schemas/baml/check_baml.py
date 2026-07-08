#!/usr/bin/env python3
"""Structural well-formedness checker for DomainForge-generated `.baml` source.

Why this instead of `baml-cli generate`: the generated project leaves the
`client<llm>` block as a documented, commented-out placeholder so the output is
vendor-neutral (no baked-in provider or credentials). `baml-cli generate`
therefore cannot run against it as-shipped — it would fail on the intentionally
undefined client. Rather than bake a vendor into the fixture just to satisfy the
generator, we validate the emitted syntax structurally. This mirrors the
`weaver`/OTel substitution documented in docs/otel-projections.md.

Target BAML syntax revision: see BAML_TARGET_VERSION in
domainforge-core/src/projection/baml/ir.rs and docs/baml-projections.md.

Checks performed across every `.baml` file in the given directory:
  * balanced `{}` and `[]` (ignoring string/comment/raw-prompt content),
  * matched `#"` / `"#` raw-string delimiters,
  * every top-level block uses a known BAML keyword,
  * every type referenced by the function resolves to a defined class/enum or a
    builtin, and
  * every `test` references a function that exists.

Exit code 0 = well-formed; nonzero (with a message) = malformed.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

BUILTINS = {"string", "int", "float", "bool", "null"}
BLOCK_KEYWORDS = {"enum", "class", "function", "test", "client", "generator", "retry_policy"}


def strip_noise(text: str) -> str:
    """Remove raw-prompt blocks, quoted strings, and line comments so brace and
    keyword scanning only sees structural tokens."""
    # Raw prompt blocks: #" ... "# (may span lines, non-greedy).
    text = re.sub(r'#"', "\x00RAWOPEN\x00", text)
    text = re.sub(r'"#', "\x00RAWCLOSE\x00", text)
    out = []
    depth = 0
    i = 0
    while i < len(text):
        if text.startswith("\x00RAWOPEN\x00", i):
            depth += 1
            i += len("\x00RAWOPEN\x00")
            continue
        if text.startswith("\x00RAWCLOSE\x00", i):
            depth = max(0, depth - 1)
            i += len("\x00RAWCLOSE\x00")
            continue
        if depth == 0:
            out.append(text[i])
        i += 1
    text = "".join(out)
    # Ordinary double-quoted strings.
    text = re.sub(r'"(?:\\.|[^"\\])*"', '""', text)
    # Line comments.
    text = re.sub(r"//[^\n]*", "", text)
    return text


def check_delimiters(raw: str, name: str) -> list[str]:
    errors = []
    if raw.count('#"') != raw.count('"#'):
        errors.append(f"{name}: unbalanced raw-string delimiters (#\" vs \"#)")
    stripped = strip_noise(raw)
    for open_c, close_c in (("{", "}"), ("[", "]"), ("(", ")")):
        if stripped.count(open_c) != stripped.count(close_c):
            errors.append(
                f"{name}: unbalanced '{open_c}{close_c}' "
                f"({stripped.count(open_c)} vs {stripped.count(close_c)})"
            )
    return errors


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_baml.py <baml_src_dir>", file=sys.stderr)
        return 2
    root = Path(sys.argv[1])
    files = sorted(root.rglob("*.baml"))
    if not files:
        print(f"no .baml files found under {root}", file=sys.stderr)
        return 2

    errors: list[str] = []
    enums: set[str] = set()
    classes: set[str] = set()
    functions: dict[str, tuple[list[str], str]] = {}
    test_fns: list[tuple[str, list[str]]] = []

    for f in files:
        raw = f.read_text()
        errors += check_delimiters(raw, f.name)
        stripped = strip_noise(raw)

        for m in re.finditer(r"\benum\s+([A-Za-z_]\w*)", stripped):
            enums.add(m.group(1))
        for m in re.finditer(r"\bclass\s+([A-Za-z_]\w*)", stripped):
            classes.add(m.group(1))
        # function Name(param: Type, ...) -> RetType
        for m in re.finditer(
            r"\bfunction\s+([A-Za-z_]\w*)\s*\(([^)]*)\)\s*->\s*([A-Za-z_]\w*(?:\[\])?)",
            stripped,
        ):
            param_types = re.findall(r":\s*([A-Za-z_]\w*(?:\[\])?)", m.group(2))
            functions[m.group(1)] = (param_types, m.group(3))
        for m in re.finditer(r"\bfunctions\s*\[([^\]]*)\]", stripped):
            names = [n.strip() for n in m.group(1).split(",") if n.strip()]
            test_fns.append((f.name, names))

        # Every top-level (column-0) block must open with a known BAML keyword.
        for line in stripped.splitlines():
            m = re.match(r"^([A-Za-z_]\w*)(?:<[^>]*>)?\s", line)
            if m and m.group(1) not in BLOCK_KEYWORDS:
                errors.append(
                    f"{f.name}: top-level block '{m.group(1)}' is not a known BAML keyword"
                )

    def resolve(t: str) -> bool:
        base = t[:-2] if t.endswith("[]") else t
        return base in BUILTINS or base in enums or base in classes

    for fn, (params, ret) in sorted(functions.items()):
        for t in params + [ret]:
            if not resolve(t):
                errors.append(f"function {fn}: unresolved type '{t}'")

    for fname, names in test_fns:
        for n in names:
            if n not in functions:
                errors.append(f"{fname}: test references unknown function '{n}'")

    if errors:
        print("BAML structural check FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1

    print(
        f"BAML structural check OK: {len(files)} files, {len(enums)} enums, "
        f"{len(classes)} classes, {len(functions)} functions, "
        f"{sum(len(n) for _, n in test_fns)} test bindings."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
