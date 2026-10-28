#!/bin/sh
# Build domainforge_<version>_<arch>.deb from the native release binary.
#
# Usage: packaging/deb/build.sh <version> <amd64|arm64> <bindir> <outdir>
#
# Depends is computed from the binary with dpkg-shlibdeps against the runner's
# package database, so the package declares real library dependencies instead
# of hand-written guesses. Build on Ubuntu 22.04 (glibc 2.35) for the widest
# supported ABI.
set -eu

if [ "$#" -ne 4 ]; then
    echo "usage: $0 <version> <amd64|arm64> <bindir> <outdir>" >&2
    exit 2
fi

version=$1
arch=$2
bindir=$(CDPATH= cd -- "$3" && pwd)
outdir=$4

case "$version" in
    '' | *[!0-9A-Za-z.+~-]*)
        echo "error: invalid version '$version'" >&2
        exit 2
        ;;
esac
case "$arch" in
    amd64 | arm64) ;;
    *)
        echo "error: unsupported architecture '$arch' (expected amd64 or arm64)" >&2
        exit 2
        ;;
esac

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
binary="$bindir/domainforge"
if [ ! -x "$binary" ]; then
    echo "error: missing executable $binary" >&2
    exit 1
fi

sh "$repo_root/packaging/check-linux-binary.sh" "$binary" "$arch"
command -v dpkg-shlibdeps >/dev/null 2>&1 || {
    echo "error: dpkg-shlibdeps is required (install dpkg-dev)" >&2
    exit 1
}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/shlibs/debian"
cat > "$work/shlibs/debian/control" <<EOF
Source: domainforge
Section: utils
Priority: optional
Maintainer: GodSpeedAI <https://github.com/GodSpeedAI/DomainForge>
Package: domainforge
Architecture: $arch
Description: shlibs probe
 probe
EOF

(
    cd "$work/shlibs"
    dpkg-shlibdeps -O -e"$binary" > "$work/shlibs.out" 2> "$work/shlibs.err"
) || {
    cat "$work/shlibs.err" >&2
    exit 1
}

deps=$(sed -n 's/^shlibs:Depends=//p' "$work/shlibs.out")
case "$deps" in
    *libc6*) ;;
    *)
        echo "error: dpkg-shlibdeps did not report libc6 (got '$deps')" >&2
        exit 1
        ;;
esac

root="$work/domainforge_${version}_${arch}"
mkdir -p "$root/DEBIAN" "$root/usr/bin" \
    "$root/usr/share/doc/domainforge/examples"
install -m 0755 "$binary" "$root/usr/bin/domainforge"
install -m 0644 "$repo_root/LICENSE" "$root/usr/share/doc/domainforge/copyright"
install -m 0644 "$repo_root/domainforge-core/examples/basic.sea" \
    "$root/usr/share/doc/domainforge/examples/basic.sea"
install -m 0644 "$repo_root/fixtures/projection_cell/invalid/01_syntax_broken.sea" \
    "$root/usr/share/doc/domainforge/examples/invalid.sea"

installed_kib=$(du -sk "$root/usr" | cut -f1)
cat > "$root/DEBIAN/control" <<EOF
Package: domainforge
Version: $version
Architecture: $arch
Maintainer: GodSpeedAI <https://github.com/GodSpeedAI/DomainForge>
Installed-Size: $installed_kib
Depends: $deps
Section: utils
Priority: optional
Homepage: https://github.com/GodSpeedAI/DomainForge
Description: semantic compiler and projection CLI for the SEA DSL
 DomainForge parses and validates SEA (Semantic Enterprise Architecture)
 models and projects them into code, architecture, process, and verification
 formats. This package installs the standalone domainforge command; it does
 not require a Rust toolchain, Node.js, Python, or any other language runtime.
EOF
chmod 0644 "$root/DEBIAN/control"

mkdir -p "$outdir"
deb="$outdir/domainforge_${version}_${arch}.deb"
dpkg-deb --build --root-owner-group "$root" "$deb" >/dev/null

test "$(dpkg-deb --field "$deb" Package)" = "domainforge"
test "$(dpkg-deb --field "$deb" Version)" = "$version"
test "$(dpkg-deb --field "$deb" Architecture)" = "$arch"
dpkg-deb --contents "$deb" | grep -q './usr/bin/domainforge$'
dpkg-deb --info "$deb" | sed -n '/Package:/,/Description:/p'
echo "OK: built $deb"
