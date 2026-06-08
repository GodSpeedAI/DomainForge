#!/usr/bin/env python3
"""Lint release workflows for publish safety and security."""
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("PyYAML is required: pip install pyyaml")
    sys.exit(1)


def lint_release_security(path):
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

            if isinstance(run, str) and "curl" in run:
                piped = run.split("|")[-1].strip() if "|" in run else ""
                if piped.endswith("sh") or piped.endswith("bash"):
                    if "sha256" not in run.lower() and "checksum" not in run.lower():
                        errors.append(
                            f"{path.name}:{job_name}/step[{i}]: "
                            f"curl | sh without checksum verification"
                        )

            if step.get("continue-on-error"):
                name_lower = str(name).lower()
                run_lower = str(run).lower()
                if "publish" in name_lower or "publish" in run_lower or "release" in name_lower:
                    errors.append(
                        f"{path.name}:{job_name}/step[{i}]: "
                        f"continue-on-error on publish/release step '{name}'"
                    )

            if isinstance(run, str) and "publish" in run.lower():
                if "|| echo" in run or "|| true" in run:
                    if "already" not in run.lower() and "skip" not in run.lower():
                        errors.append(
                            f"{path.name}:{job_name}/step[{i}]: "
                            f"publish failure masked by || echo/|| true in '{name}'"
                        )

            env_block = step.get("env", {})
            uses = step.get("uses", "")
            is_build = isinstance(run, str) and any(
                kw in run for kw in ["cargo build", "npm run build", "wasm-pack build"]
            )
            is_publish = isinstance(run, str) and "publish" in run.lower()
            if is_build and not is_publish:
                token_keys = [k for k in env_block if "token" in k.lower() or "key" in k.lower()]
                if token_keys:
                    errors.append(
                        f"{path.name}:{job_name}/step[{i}]: "
                        f"secret env ({', '.join(token_keys)}) exposed during build step '{name}'"
                    )

    return errors


def main():
    workflows_dir = Path(".github/workflows")
    if not workflows_dir.exists():
        print("No .github/workflows directory found")
        sys.exit(0)

    all_errors = []
    for wf in sorted(workflows_dir.glob("*.yml")):
        all_errors.extend(lint_release_security(wf))
    for wf in sorted(workflows_dir.glob("*.yaml")):
        all_errors.extend(lint_release_security(wf))

    if all_errors:
        print("Release security lint FAILURES:")
        for e in all_errors:
            print(f"  - {e}")
        sys.exit(1)
    else:
        print("All release security lints passed")
        sys.exit(0)


if __name__ == "__main__":
    main()
