#!/usr/bin/env bash
# Grammar-change gate: if this diff touches SEA grammar/AST/schema/formatter
# files, it must also touch an ADR and a test or fixture file. Run in CI's
# `lint` job. See docs/reference/sea-language-evolution-policy.md.
#
# This is deliberately a coarse presence check (files touched), not a
# semantic review of content.
# ponytail: presence gate, not content review — tighten to check the ADR
# actually documents the touched declarations if this gate ever misses a
# real regression.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

BASE_REF="${GITHUB_BASE_REF:-}"
if [[ -z "$BASE_REF" ]]; then
  echo "check-sea-language-change: no GITHUB_BASE_REF set (not running in a PR context) — skipping"
  exit 0
fi

# Fail closed: a fetch or base-ref resolution failure must abort the gate,
# not silently widen the set of changes that skip review.
if ! git fetch origin "$BASE_REF" --depth=1; then
  echo "check-sea-language-change: FAILED — could not fetch origin/$BASE_REF" >&2
  exit 1
fi

DIFF_BASE="origin/$BASE_REF"
if ! git rev-parse --verify "$DIFF_BASE" >/dev/null 2>&1; then
  echo "check-sea-language-change: FAILED — could not resolve $DIFF_BASE after fetch" >&2
  exit 1
fi

if ! CHANGED="$(git diff --name-only "$DIFF_BASE"...HEAD)"; then
  echo "check-sea-language-change: FAILED — could not compute diff against $DIFF_BASE" >&2
  exit 1
fi

WATCHED_PATTERN='^domainforge-core/grammar/.*\.pest$|^domainforge-core/src/parser/ast(_schema|_convert)?\.rs$|^domainforge-core/src/formatter/printer\.rs$|^domainforge-core/src/parser/printer\.rs$|^schemas/ast-.*\.schema\.json$'

if ! grep -qE "$WATCHED_PATTERN" <<<"$CHANGED"; then
  echo "check-sea-language-change: no watched grammar/AST/schema/formatter files touched — OK"
  exit 0
fi

echo "check-sea-language-change: watched files touched:"
grep -E "$WATCHED_PATTERN" <<<"$CHANGED" | sed 's/^/  - /'

missing=()

if ! grep -qE '^docs/specs/ADR-.*\.md$' <<<"$CHANGED"; then
  missing+=("an ADR under docs/specs/ADR-*.md documenting the grammar change (see docs/templates/ADR-template-sea-language-change.md)")
fi

if ! grep -qE '^domainforge-core/tests/.*\.rs$|^fixtures/' <<<"$CHANGED"; then
  missing+=("a test file under domainforge-core/tests/ or a fixture under fixtures/ demonstrating the change")
fi

if [[ ${#missing[@]} -gt 0 ]]; then
  echo ""
  echo "check-sea-language-change: FAILED — grammar/AST/schema/formatter changed without:"
  for m in "${missing[@]}"; do
    echo "  - $m"
  done
  echo ""
  echo "See docs/reference/sea-language-evolution-policy.md for the full requirements."
  exit 1
fi

echo "check-sea-language-change: PASSED — ADR and test/fixture evidence present"
