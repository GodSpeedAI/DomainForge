#!/usr/bin/env python3
"""Check that release workflow gates are correct.

Verifies:
- Every upload-artifact job is referenced in create-release needs
- Dependabot automerge does not treat empty check results as success
"""
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("PyYAML is required: pip install pyyaml")
    sys.exit(1)


def check_artifact_gates(path):
    errors = []
    with open(path) as f:
        wf = yaml.safe_load(f)

    if wf is None:
        return []

    jobs = wf.get("jobs", {})
    create_release = jobs.get("create-release", {})
    needs = create_release.get("needs", [])
    if isinstance(needs, str):
        needs = [needs]

    for job_name, job in jobs.items():
        if job_name == "create-release":
            continue
        steps = job.get("steps", [])
        has_upload = any(
            "upload-artifact" in str(step.get("uses", ""))
            for step in steps
        )
        if has_upload and job_name not in needs:
            errors.append(
                f"{path.name}: job '{job_name}' uploads artifacts "
                f"but is not in create-release needs (found: {needs})"
            )

    return errors


def check_dependabot_gates(path):
    errors = []
    if "dependabot" not in path.name:
        return []

    with open(path) as f:
        wf = yaml.safe_load(f)

    if wf is None:
        return []

    for job_name, job in wf.get("jobs", {}).items():
        for step in job.get("steps", []):
            run = step.get("run", "")
            if not isinstance(run, str):
                continue
            if "checks" in run.lower() or "status" in run.lower():
                if "[]" in run and "exit 0" not in run.split("[]")[0]:
                    lines_after = run.split("[]")
                    remaining = lines_after[1] if len(lines_after) > 1 else ""
                    if 'exit 0' in remaining and 'exit 1' not in remaining:
                        continue

                has_empty_fail = "[]" in run and "exit 1" in run
                if not has_empty_fail:
                    if "[]" in run:
                        next_chunk = run.split("[]", 1)[1] if "[]" in run else ""
                        if "exit 1" not in next_chunk:
                            has_rejection = "no check" in run.lower() and "exit 1" in run
                            if not has_rejection:
                                errors.append(
                                    f"{path.name}:{job_name}: "
                                    f"dependabot check may treat empty [] as success "
                                    f"without explicit failure"
                                )

    return errors


def main():
    workflows_dir = Path(".github/workflows")
    if not workflows_dir.exists():
        print("No .github/workflows directory found")
        sys.exit(0)

    all_errors = []
    for wf in sorted(workflows_dir.glob("*.yml")):
        all_errors.extend(check_artifact_gates(wf))
        all_errors.extend(check_dependabot_gates(wf))
    for wf in sorted(workflows_dir.glob("*.yaml")):
        all_errors.extend(check_artifact_gates(wf))
        all_errors.extend(check_dependabot_gates(wf))

    if all_errors:
        print("Workflow gate lint FAILURES:")
        for e in all_errors:
            print(f"  - {e}")
        sys.exit(1)
    else:
        print("All workflow gate lints passed")
        sys.exit(0)


if __name__ == "__main__":
    main()
