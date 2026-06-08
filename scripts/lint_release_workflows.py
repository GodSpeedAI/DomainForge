#!/usr/bin/env python3
"""Lint release workflows for security best practices."""
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("PyYAML is required: pip install pyyaml")
    sys.exit(1)


def lint_workflow(path):
    errors = []
    try:
        with open(path) as f:
            wf = yaml.safe_load(f)
    except Exception as e:
        return [f"{path.name}: failed to parse YAML: {e}"]

    if wf is None:
        return []

    for job_name, job in wf.get("jobs", {}).items():
        steps = job.get("steps", [])
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

            if isinstance(run, str) and "npm install" in run:
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
