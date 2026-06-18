#!/usr/bin/env sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <commit_message_file>" >&2
  exit 2
fi

msg_file="$1"

if [ -x "./node_modules/.bin/commitlint" ]; then
  exec ./node_modules/.bin/commitlint --edit "$msg_file"
fi

if command -v bunx >/dev/null 2>&1; then
  exec bunx --yes commitlint --edit "$msg_file"
fi

if command -v npx >/dev/null 2>&1; then
  exec npx --yes commitlint --edit "$msg_file"
fi

if command -v npm >/dev/null 2>&1; then
  exec npm exec --yes commitlint -- --edit "$msg_file"
fi

echo "commitlint is required but no compatible runner was found (node_modules/.bin, bunx, npx, npm)." >&2
echo "Run 'just setup' or install Node.js/Bun, then retry." >&2
exit 1
