#!/bin/sh
# Print the one repository version and refuse to continue when the manifests
# that Release Please synchronizes have drifted apart.
#
# Usage: packaging/version.sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ ! -f "$root/version.txt" ]; then
    echo "error: missing $root/version.txt" >&2
    exit 1
fi

version=$(tr -d '[:space:]' < "$root/version.txt")
case "$version" in
    '' | *[!0-9A-Za-z.+~-]*)
        echo "error: invalid version '$version' in version.txt" >&2
        exit 1
        ;;
esac

crate_version=$(sed -n '/^\[package\]/,/^\[/ s/^version = "\([^"]*\)"/\1/p' \
    "$root/domainforge-core/Cargo.toml" | head -1)
python_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' \
    "$root/domainforge-python/pyproject.toml" | head -1)
npm_version=$(sed -n 's/^[[:space:]]*"version": "\([^"]*\)".*/\1/p' \
    "$root/domainforge-typescript/package.json" | head -1)
lock_version=$(grep -A2 '^name = "domainforge-core"$' "$root/Cargo.lock" \
    | sed -n 's/^version = "\([^"]*\)"/\1/p' | head -1)

for pair in \
    "domainforge-core/Cargo.toml:$crate_version" \
    "domainforge-python/pyproject.toml:$python_version" \
    "domainforge-typescript/package.json:$npm_version" \
    "Cargo.lock:$lock_version"; do
    file=${pair%%:*}
    actual=${pair#*:}
    if [ "$actual" != "$version" ]; then
        echo "error: $file version '$actual' differs from version.txt '$version'" >&2
        exit 1
    fi
done

printf '%s\n' "$version"
