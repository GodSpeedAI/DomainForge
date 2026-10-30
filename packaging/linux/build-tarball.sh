#!/bin/sh
# Build a self-contained Linux tarball that does not depend on the .deb.
#
# Usage: packaging/linux/build-tarball.sh <version> <target-triple> <bindir> <outdir>
#
# The archive is reproducible: normalized ownership, timestamps, sort order, and
# gzip -n (no embedded name or mtime).
set -eu

if [ "$#" -ne 4 ]; then
    echo "usage: $0 <version> <target-triple> <bindir> <outdir>" >&2
    exit 2
fi

version=$1
target=$2
bindir=$(CDPATH= cd -- "$3" && pwd)
outdir=$4

case "$version" in
    '' | *[!0-9A-Za-z.+~-]*)
        echo "error: invalid version '$version'" >&2
        exit 2
        ;;
esac
case "$target" in
    x86_64-unknown-linux-gnu | aarch64-unknown-linux-gnu) ;;
    *)
        echo "error: unsupported target '$target'" >&2
        exit 2
        ;;
esac

binary="$bindir/domainforge"
if [ ! -x "$binary" ]; then
    echo "error: missing executable $binary" >&2
    exit 1
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

name="domainforge-${version}-${target}"
stage="$work/$name"
mkdir -p "$stage/examples"
install -m 0755 "$binary" "$stage/domainforge"
install -m 0644 "$repo_root/LICENSE" "$stage/LICENSE"
install -m 0644 "$repo_root/domainforge-core/examples/basic.sea" \
    "$stage/examples/basic.sea"
install -m 0644 "$repo_root/fixtures/projection_cell/invalid/01_syntax_broken.sea" \
    "$stage/examples/invalid.sea"
cat > "$stage/README.txt" <<EOF
DomainForge ${version} (${target})

This archive contains the standalone domainforge command. It needs a glibc
2.35-or-newer Linux system (Ubuntu 22.04 or newer). No Rust toolchain, Node.js,
Python, or other language runtime is required.

Run:
  ./domainforge --version
  ./domainforge validate --format human examples/basic.sea
  ./domainforge parse --ast --format json examples/basic.sea
  ./domainforge project --format domain-python examples/basic.sea ./out

examples/invalid.sea is deliberately invalid and must exit non-zero.

Licence: Apache-2.0, see LICENSE.
EOF

mkdir -p "$outdir"
archive="$outdir/${name}.tar.gz"
tar --sort=name --mtime='UTC 1970-01-01' --owner=0 --group=0 --numeric-owner \
    -C "$work" -cf - "$name" | gzip -9n > "$archive"

test -s "$archive"
tar -tzf "$archive" | grep -q "^${name}/domainforge$"
echo "OK: built $archive"
