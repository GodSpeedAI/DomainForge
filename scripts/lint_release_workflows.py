#!/usr/bin/env python3
"""Lint release workflows for security best practices."""
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("PyYAML is required: pip install pyyaml")
    sys.exit(1)


GH_CALL = re.compile(r"(^|[\s;&|(`$])gh\s", re.MULTILINE)
GH_REPO_FLAG = re.compile(r"\s(--repo|-R)[\s=]")
NPM_INSTALL = re.compile(r"npm install(?!\s+-g\s+npm@)")


def lint_workflow(path):
    errors = []
    try:
        with open(path) as f:
            wf = yaml.safe_load(f)
    except Exception as e:
        return [f"{path.name}: failed to parse YAML: {e}"]

    if wf is None:
        return []

    workflow_env = wf.get("env") or {}
    for job_name, job in wf.get("jobs", {}).items():
        steps = job.get("steps", [])
        has_checkout = any(
            str(s.get("uses", "")).startswith("actions/checkout@") for s in steps
        )
        job_env = job.get("env") or {}
        for i, step in enumerate(steps):
            run = step.get("run", "")
            name = step.get("name", step.get("uses", "unnamed"))

            if step.get("continue-on-error"):
                run_lower = str(run).lower()
                name_lower = str(name).lower()
                if "publish" in name_lower or "publish" in run_lower:
                    errors.append(
                        f"{path.name}:{job_name}/step[{i}]: "
                        f"continue-on-error on publish step '{name}'"
                    )

            if isinstance(run, str) and "curl" in run:
                piped = run.split("|")[-1].strip() if "|" in run else ""
                if piped.endswith("sh") or "sops" in run:
                    if "sha256" not in run.lower() and "checksum" not in run.lower():
                        errors.append(
                            f"{path.name}:{job_name}/step[{i}]: "
                            f"curl download without checksum verification"
                        )

            # Without a checkout there is no .git for gh to infer the repository
            # from, so it fails with "not a git repository" (v0.20.0, v0.21.0).
            if isinstance(run, str) and GH_CALL.search(run) and not has_checkout:
                step_env = step.get("env") or {}
                has_repo = any("GH_REPO" in e for e in (workflow_env, job_env, step_env))
                if not has_repo and not GH_REPO_FLAG.search(run):
                    errors.append(
                        f"{path.name}:{job_name}/step[{i}]: "
                        f"gh call in job without checkout needs GH_REPO or --repo"
                    )

            # A global npm self-upgrade installs no project dependencies.
            if isinstance(run, str) and NPM_INSTALL.search(run):
                if "--frozen" not in run and "npm ci" not in run:
                    if "release" in path.name:
                        errors.append(
                            f"{path.name}:{job_name}/step[{i}]: "
                            f"unfrozen npm install in release workflow"
                        )

    return errors


def main():
    workflows_dir = Path(".github/workflows")
    if not workflows_dir.exists():
        print("No .github/workflows directory found")
        sys.exit(0)

    all_errors = []
    for wf in sorted(workflows_dir.glob("*.yml")):
        all_errors.extend(lint_workflow(wf))
    for wf in sorted(workflows_dir.glob("*.yaml")):
        all_errors.extend(lint_workflow(wf))

    if all_errors:
        print("Release workflow lint FAILURES:")
        for e in all_errors:
            print(f"  - {e}")
        sys.exit(1)
    else:
        print("All release workflow lints passed")
        sys.exit(0)


if __name__ == "__main__":
    main()
