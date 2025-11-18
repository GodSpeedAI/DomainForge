#!/usr/bin/env python3
"""
Locate a compiled Rust binary under the workspace `target/` directories.

Supports both workspace-level targets (e.g., `target/x86_64-unknown-linux-gnu/release/sea`)
and crate-local targets (e.g., `sea-core/target/release/sea`). The script can match
by exact name, prefix, or substring and falls back to alternate match modes when
requested. Designed for both CI (finding the built CLI) and editor automation
(finding hashed test binaries for codelldb).
"""

from __future__ import annotations

import argparse
import os
import sys
from dataclasses import dataclass
from typing import Iterable, List, Optional, Sequence, Tuple

MATCH_CHOICES = ("exact", "prefix", "contains")
DEFAULT_SEARCH_ROOTS = ["target", os.path.join("sea-core", "target")]


@dataclass(frozen=True)
class SearchOptions:
    names: Sequence[str]
    match_mode: str
    require_executable: bool


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Find a compiled Rust binary path.")
    parser.add_argument(
        "--name", required=True, help="Base name of the binary (stem without hash)."
    )
    parser.add_argument(
        "--profile",
        choices=["debug", "release"],
        default="debug",
        help="Cargo profile directory to search (default: debug).",
    )
    parser.add_argument(
        "--target-triple",
        help="Optional target triple (searches target/<triple>/<profile>).",
    )
    parser.add_argument(
        "--deps-subdir",
        help="Optional additional subdirectory (e.g., 'deps' for test binaries).",
    )
    parser.add_argument(
        "--match-mode",
        choices=MATCH_CHOICES,
        default="exact",
        help="Primary match mode for filenames (default: exact).",
    )
    parser.add_argument(
        "--fallback-mode",
        choices=MATCH_CHOICES,
        help="Optional secondary match mode if the primary fails.",
    )
    parser.add_argument(
        "--extension",
        action="append",
        dest="extensions",
        help=(
            "Explicit filename extensions to consider. Provide multiple times for "
            "multiple extensions. Defaults to ['', '.exe']."
        ),
    )
    parser.add_argument(
        "--search-root",
        action="append",
        dest="search_roots",
        help=(
            "Override search roots (relative to workspace). Provide multiple times "
            "to search several directories. Defaults to workspace 'target/' and "
            "'sea-core/target/'."
        ),
    )
    parser.add_argument(
        "--workspace",
        default=".",
        help="Workspace root (defaults to current working directory).",
    )
    parser.add_argument(
        "--require-executable",
        action="store_true",
        help="Only consider paths that are executable.",
    )
    return parser.parse_args()


def canonical_workspace(path: str) -> str:
    return os.path.abspath(os.path.expanduser(path))


def candidate_directories(
    workspace: str, roots: Sequence[str], target: Optional[str], profile: str, deps: Optional[str]
) -> List[str]:
    seen = set()
    directories: List[str] = []

    for root in roots:
        base = os.path.join(workspace, root)
        targets: Tuple[str, ...]
        if target:
            targets = (
                os.path.join(base, target, profile),
                os.path.join(base, profile),
            )
        else:
            targets = (os.path.join(base, profile),)

        for target_dir in targets:
            if deps:
                target_dir = os.path.join(target_dir, deps)
            normalized = os.path.normpath(target_dir)
            if normalized not in seen:
                seen.add(normalized)
                directories.append(normalized)
    return directories


def build_candidate_names(name: str, extensions: Sequence[str]) -> List[str]:
    if extensions:
        return [f"{name}{ext}" for ext in extensions]
    return [name, f"{name}.exe"]


def iter_sorted_files(directory: str) -> Iterable[str]:
    try:
        entries = os.listdir(directory)
    except FileNotFoundError:
        return []
    except NotADirectoryError:
        return []
    entries.sort(
        key=lambda entry: os.path.getmtime(os.path.join(directory, entry)),
        reverse=True,
    )
    return entries


def match_entry(entry: str, names: Sequence[str], mode: str) -> bool:
    entry_cmp = entry.lower()
    if mode == "exact":
        lowered = [n.lower() for n in names]
        return entry_cmp in lowered
    if mode == "prefix":
        lowered = [n.lower() for n in names]
        return any(entry_cmp.startswith(n) for n in lowered)
    if mode == "contains":
        lowered = [n.lower() for n in names]
        return any(n in entry_cmp for n in lowered)
    raise ValueError(f"Unsupported match mode: {mode}")


def find_binary(
    directories: Sequence[str],
    options: SearchOptions,
) -> Optional[str]:
    for directory in directories:
        for entry in iter_sorted_files(directory):
            path = os.path.join(directory, entry)
            if not os.path.isfile(path):
                continue
            if options.require_executable and not os.access(path, os.X_OK):
                continue
            if match_entry(entry, options.names, options.match_mode):
                return os.path.abspath(path)
    return None


def fail_with_diagnostics(
    name: str, directories: Sequence[str], match_mode: str, fallback_mode: Optional[str]
) -> None:
    rel_dirs = [os.path.relpath(d) for d in directories]
    print(
        "Unable to locate compiled binary.\n"
        f"  name: {name}\n"
        f"  match_mode: {match_mode}\n"
        f"  fallback_mode: {fallback_mode or 'none'}\n"
        f"  searched directories:\n    - " + "\n    - ".join(rel_dirs),
        file=sys.stderr,
    )
    sys.exit(1)


def main() -> None:
    args = parse_args()
    workspace = canonical_workspace(args.workspace)
    roots = args.search_roots or DEFAULT_SEARCH_ROOTS
    extensions = args.extensions or ["", ".exe"]

    directories = candidate_directories(
        workspace=workspace,
        roots=roots,
        target=args.target_triple,
        profile=args.profile,
        deps=args.deps_subdir,
    )

    options = SearchOptions(
        names=build_candidate_names(args.name, extensions),
        match_mode=args.match_mode,
        require_executable=args.require_executable,
    )

    result = find_binary(directories, options)
    if not result and args.fallback_mode:
        fallback_options = SearchOptions(
            names=options.names,
            match_mode=args.fallback_mode,
            require_executable=options.require_executable,
        )
        result = find_binary(directories, fallback_options)

    if not result:
        fail_with_diagnostics(
            name=args.name,
            directories=directories,
            match_mode=args.match_mode,
            fallback_mode=args.fallback_mode,
        )

    print(result)


if __name__ == "__main__":
    main()
