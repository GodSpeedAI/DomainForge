#!/usr/bin/env python3
"""
Scan test files (sea-core/tests/*.rs) for `use` statements and detect likely unused imports.
Only flags imports where the `use` imports a single identifier and that identifier is not used elsewhere in the file.
This heuristic avoids removing multi-import groups that might be used via macros or shadowed names.
"""
import re
from pathlib import Path

repo_root = Path(__file__).resolve().parents[1]

def get_test_files():
    tests_dir = repo_root / 'sea-core' / 'tests'
    return list(tests_dir.glob('*.rs'))

USE_RE = re.compile(r'^use\s+([^;]+);')
GROUPED_RE = re.compile(r'^(?P<prefix>[^:;{]+::)\{(?P<items>[^}]+)\}$')
SIMPLE_SINGLE_IDENT_RE = re.compile(r'^(?P<path>[^:]+)::(?P<ident>[A-Za-z0-9_]+)$')

candidates = []
for f in get_test_files():
    text = f.read_text(encoding='utf8')
    lines = text.splitlines()
    for i, line in enumerate(lines, start=1):
        m = USE_RE.match(line.strip())
        if not m:
            continue
        use_expr = m.group(1).strip()
        # Detect grouped imports like `foo::bar::{A, B as C}`
        group_match = GROUPED_RE.match(use_expr)
        if group_match:
            prefix = group_match.group('prefix')
            items = [it.strip() for it in group_match.group('items').split(',')]
            for item in items:
                # Handle `A as B` case, take alias as `B` or `A` when no alias
                parts = [p.strip() for p in item.split(' as ')]
                if len(parts) == 2:
                    ident = parts[1]
                else:
                    ident = parts[0]
                # If the item has a path `kg::KnowledgeGraph`, only the last segment
                # (the type or ident) is relevant for references in the file.
                if '::' in ident:
                    ident = ident.split('::')[-1]
                # Don't try to remove common traits which are used via method calls
                if ident == 'FromStr':
                    continue
                occ = [j for j, line_text in enumerate(lines, start=1) if j != i and re.search(r'\b' + re.escape(ident) + r'\b', line_text)]
                if not occ:
                    candidates.append((str(f), i, line.strip(), ident))
            continue
        # Try to capture single import names like `crate::graph::Graph` or `sea_core::Graph` or `std::fmt::Result`
        # Remove trailing traits like "pub use"? Not present here
        if '::' not in use_expr:
            continue
        # Extract the last path component
        ident = use_expr.split('::')[-1]
        if not ident.isidentifier():
            continue
        # Search if ident appears elsewhere in file
        # Exclude the import line
        occ = [j for j, line_text in enumerate(lines, start=1) if j != i and re.search(r'\b' + re.escape(ident) + r'\b', line_text)]
        # Skip flagging trait names which are used implicitly (FromStr)
        if ident == 'FromStr':
            continue
        if not occ:
            candidates.append((str(f), i, line.strip(), ident))

if candidates:
    print('Potential unused single-ident imports detected:')
    for path, line_no, line, ident in candidates:
        print(f'{path}:{line_no}: {line}  (ident={ident})')
else:
    print('No obvious single-identifier unused imports detected in test files.')
