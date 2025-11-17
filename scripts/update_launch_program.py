#!/usr/bin/env python3
"""
Update `.vscode/launch.json` program path for "Debug Rust Test (auto)" configuration
to point at the prepared debug binary `target/debug/deps/sea_debug_test`.

Usage:
  python3 scripts/update_launch_program.py --program <path>

If the file does not exist, writes the default program path.
"""

import argparse
import json
import os
import sys


def find_config_index(configs, name):
    for idx, c in enumerate(configs):
        if c.get("name") == name:
            return idx
    return -1


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--program", help="Path to the rust test binary (symlink)")
    parser.add_argument("--launch", default=".vscode/launch.json")
    parser.add_argument("--config-name", default="Debug Rust Test (auto)")
    args = parser.parse_args()

    # Prefer a workspace-relative program path so launch.json remains portable
    program_path = args.program or os.path.join(
        "${workspaceFolder}", "target", "debug", "deps", "sea_debug_test"
    )
    # If program_path is absolute and inside the workspace, replace the prefix with ${workspaceFolder}
    workspace_prefix = os.path.abspath(".")
    if "${workspaceFolder}" not in program_path:
        abs_program_path = os.path.abspath(program_path)
        normalized_workspace = os.path.normcase(os.path.normpath(workspace_prefix))
        normalized_program = os.path.normcase(os.path.normpath(abs_program_path))
        if normalized_program.startswith(normalized_workspace):
            rel_path = os.path.relpath(abs_program_path, workspace_prefix)
            program_path = os.path.join("${workspaceFolder}", rel_path).replace("\\", "/")
        else:
            program_path = abs_program_path.replace("\\", "/")
    launch_path = args.launch

    if not os.path.exists(launch_path):
        print(
            f"Warning: {launch_path} not found, creating a new launch.json with default config"
        )
        launch = {"version": "0.2.0", "configurations": []}
    else:
        try:
            with open(launch_path, "r") as f:
                launch = json.load(f)
        except json.JSONDecodeError as err:
            print(f"Failed to parse {launch_path}: {err}")
            sys.exit(1)

    configs = launch.setdefault("configurations", [])
    idx = find_config_index(configs, args.config_name)

    if idx == -1:
        # If not found, add a new configuration stub
        configs.append(
            {
                "name": args.config_name,
                "type": "codelldb",
                "request": "launch",
                "program": program_path,
                "args": ["--nocapture", "${input:rustTestName}"],
                "cwd": "${workspaceFolder}",
                "stopAtEntry": False,
                "terminal": "integrated",
            }
        )
        print(f"Added {args.config_name} to launch.json with program={program_path}")
    else:
        configs[idx]["program"] = program_path
        print(f"Updated {args.config_name} program to {program_path}")

    launch_dir = os.path.dirname(launch_path)
    if launch_dir:
        os.makedirs(launch_dir, exist_ok=True)
    with open(launch_path, "w") as f:
        json.dump(launch, f, indent=2)
    print(f"Wrote updated launch.json: {launch_path}")


if __name__ == "__main__":
    main()
