#!/bin/sh
# Build the macOS .pkg and .dmg for one architecture or a universal binary.
#
# Usage: packaging/macos/build.sh <version> <universal|arm64|x86_64> \
#            <arm64dir|''> <x86dir|''> <outdir>
#
# The package installs to /usr/local/bin/domainforge. It is UNSIGNED: no Apple
# Developer ID certificate is used and it is not notarized. The executable is
# ad-hoc signed after lipo so the arm64 slice can run on Apple silicon.
set -eu

if [ "$#" -ne 5 ]; then
    echo "usage: $0 <version> <universal|arm64|x86_64> <arm64dir|''> <x86dir|''> <outdir>" >&2
    exit 2
fi

version=$1
label=$2
arm_dir=$3
x86_dir=$4
outdir=$5

case "$version" in
    '' | *[!0-9A-Za-z.+~-]*)
        echo "error: invalid version '$version'" >&2
        exit 2
        ;;
esac
case "$label" in
    universal | arm64 | x86_64) ;;
    *)
        echo "error: unsupported label '$label' (expected universal, arm64, or x86_64)" >&2
        exit 2
        ;;
esac

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
pkg_name="domainforge-${version}-macos-${label}.pkg"
dmg_name="domainforge-${version}-macos-${label}.dmg"

case "$label" in
    universal)
        [ -n "$arm_dir" ] && [ -n "$x86_dir" ] \
            || { echo "error: universal needs both arm64 and x86_64 binaries" >&2; exit 1; }
        ;;
    arm64)
        [ -n "$arm_dir" ] \
            || { echo "error: arm64 needs an arm64 binary directory" >&2; exit 1; }
        ;;
    x86_64)
        [ -n "$x86_dir" ] \
            || { echo "error: x86_64 needs an x86_64 binary directory" >&2; exit 1; }
        ;;
esac

if [ -n "$arm_dir" ] && [ ! -x "$arm_dir/domainforge" ]; then
    echo "error: missing executable $arm_dir/domainforge" >&2
    exit 1
fi
if [ -n "$x86_dir" ] && [ ! -x "$x86_dir/domainforge" ]; then
    echo "error: missing executable $x86_dir/domainforge" >&2
    exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

stage="$work/stage"
mkdir -p "$stage/bin" "$stage/share/doc/domainforge/examples"

if [ "$label" = universal ]; then
    lipo -create "$arm_dir/domainforge" "$x86_dir/domainforge" \
        -output "$stage/bin/domainforge"
else
    if [ "$label" = arm64 ]; then
        cp "$arm_dir/domainforge" "$stage/bin/domainforge"
    else
        cp "$x86_dir/domainforge" "$stage/bin/domainforge"
    fi
fi
chmod 0755 "$stage/bin/domainforge"
codesign --force --sign - "$stage/bin/domainforge" >/dev/null

install -m 0644 "$repo_root/LICENSE" "$stage/share/doc/domainforge/LICENSE"
install -m 0644 "$repo_root/domainforge-core/examples/basic.sea" \
    "$stage/share/doc/domainforge/examples/basic.sea"
install -m 0644 "$repo_root/fixtures/projection_cell/invalid/01_syntax_broken.sea" \
    "$stage/share/doc/domainforge/examples/invalid.sea"

mkdir -p "$outdir"
pkgbuild --root "$stage" \
    --identifier ai.godspeed.domainforge \
    --version "$version" \
    --install-location /usr/local \
    "$outdir/$pkg_name" >/dev/null

dmg_src="$work/dmg"
mkdir -p "$dmg_src"
cp "$outdir/$pkg_name" "$dmg_src/"
cp "$repo_root/LICENSE" "$dmg_src/LICENSE"
cat > "$dmg_src/README-install.txt" <<EOF
DomainForge ${version} for macOS (${label})

Install
  Double-click ${pkg_name}, or from Terminal:
    sudo installer -pkg ${pkg_name} -target /
  The domainforge command is placed in /usr/local/bin.

Unsigned package
  This package is NOT signed with an Apple Developer ID and is NOT notarized.
  Gatekeeper may refuse to open it. Either:
    - Control-click the package, choose Open, and confirm; or
    - remove the quarantine flag after download:
        xattr -d com.apple.quarantine ${pkg_name}
  Verify the download before installing:
    shasum -a 256 -c SHA256SUMS
    gh attestation verify ${pkg_name} --repo GodSpeedAI/DomainForge

Uninstall
  sudo rm -f /usr/local/bin/domainforge
  sudo rm -rf /usr/local/share/doc/domainforge
  sudo pkgutil --forget ai.godspeed.domainforge

Licence: see LICENSE in this image.
EOF

hdiutil create -volname "DomainForge ${version}" -srcfolder "$dmg_src" \
    -format UDZO "$outdir/$dmg_name" >/dev/null

test -s "$outdir/$pkg_name"
test -s "$outdir/$dmg_name"
echo "OK: built $outdir/$pkg_name and $outdir/$dmg_name"
