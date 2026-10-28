#!/bin/sh
# Reject a Linux CLI binary that would not run on the supported baseline
# (Ubuntu 22.04 / glibc 2.35) or that carries an accidental build-environment
# dependency.
#
# Usage: packaging/check-linux-binary.sh <binary> <amd64|arm64>
set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <binary> <amd64|arm64>" >&2
    exit 2
fi

bin=$1
arch=$2

if [ ! -f "$bin" ]; then
    echo "error: binary not found: $bin" >&2
    exit 1
fi

case "$arch" in
    amd64)
        expected_machine='Advanced Micro Devices X86-64'
        expected_interpreter='/lib64/ld-linux-x86-64.so.2'
        ;;
    arm64)
        expected_machine='AArch64'
        expected_interpreter='/lib/ld-linux-aarch64.so.1'
        ;;
    *)
        echo "error: unsupported architecture '$arch' (expected amd64 or arm64)" >&2
        exit 2
        ;;
esac

header=$(readelf -h "$bin")
machine=$(printf '%s\n' "$header" | sed -n 's/.*Machine:[[:space:]]*//p')
class=$(printf '%s\n' "$header" | sed -n 's/.*Class:[[:space:]]*//p')
type=$(printf '%s\n' "$header" | sed -n 's/.*Type:[[:space:]]*//p')

case "$machine" in
    *"$expected_machine"*) ;;
    *)
        echo "error: $bin has ELF machine '$machine', expected '$expected_machine'" >&2
        exit 1
        ;;
esac
case "$class" in
    *ELF64*) ;;
    *)
        echo "error: $bin is not an ELF64 binary (class: $class)" >&2
        exit 1
        ;;
esac
case "$type" in
    *EXEC* | *DYN*) ;;
    *)
        echo "error: $bin is not an executable ELF (type: $type)" >&2
        exit 1
        ;;
esac

interpreter=$(readelf -l "$bin" \
    | sed -n 's/.*Requesting program interpreter: \(.*\)]/\1/p' | head -1)
if [ "$interpreter" != "$expected_interpreter" ]; then
    echo "error: $bin interpreter is '$interpreter', expected '$expected_interpreter'" >&2
    exit 1
fi

dynamic=$(readelf -d "$bin" 2>/dev/null || true)
if printf '%s\n' "$dynamic" | grep -Eq 'RPATH|RUNPATH'; then
    echo "error: $bin carries an RPATH/RUNPATH:" >&2
    printf '%s\n' "$dynamic" | grep -E 'RPATH|RUNPATH' >&2
    exit 1
fi

if grep -aqF '/nix/store' "$bin"; then
    echo "error: $bin references the Nix store" >&2
    exit 1
fi

needed=$(printf '%s\n' "$dynamic" \
    | sed -n 's/.*Shared library: \[\(.*\)\]/\1/p' | sort -u)
if [ -z "$needed" ]; then
    echo "error: $bin has no dynamic dependencies; expected a glibc-linked binary" >&2
    exit 1
fi

for lib in $needed; do
    case "$lib" in
        libc.so.6 | libm.so.6 | libdl.so.2 | libpthread.so.0 | librt.so.1 | \
            libgcc_s.so.1 | ld-linux-aarch64.so.1 | ld-linux-x86-64.so.2 | \
            linux-vdso.so.1)
            ;;
        *)
            echo "error: $bin has unexpected dynamic dependency '$lib'" >&2
            exit 1
            ;;
    esac
done

symbols=$(readelf --dyn-syms --wide "$bin")
if printf '%s\n' "$symbols" | grep -Eq 'GLIBCXX_|CXXABI_'; then
    echo "error: $bin requires the C++ runtime" >&2
    exit 1
fi

required_glibc=$(printf '%s\n' "$symbols" \
    | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/^GLIBC_//' | sort -Vu | tail -1)
if [ -z "$required_glibc" ]; then
    echo "error: $bin exposes no versioned GLIBC symbols; cannot verify the ABI baseline" >&2
    exit 1
fi
baseline='2.35'
highest=$(printf '%s\n%s\n' "$required_glibc" "$baseline" | sort -V | tail -1)
if [ "$highest" != "$baseline" ]; then
    echo "error: $bin requires GLIBC_$required_glibc, above the $baseline baseline" >&2
    exit 1
fi

echo "OK: $bin is ELF64 $arch, interpreter $interpreter, max GLIBC_$required_glibc"
printf '    NEEDED: %s\n' "$(printf '%s' "$needed" | tr '\n' ' ')"
