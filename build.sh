#!/usr/bin/env bash
# Build the ewe ISO — a thin mkarchiso wrapper. Needs the `archiso` package
# and root (mkarchiso mounts loop devices). Output: out/ewe-<VERSION>-x86_64.iso
#
#   sudo ./build.sh            build
#   sudo ./build.sh clean      drop the work dir (do this after profile edits;
#                              mkarchiso reuses a stale work dir silently)
set -euo pipefail
cd "$(dirname "$0")"

command -v mkarchiso >/dev/null 2>&1 || { echo "xx install the 'archiso' package first." >&2; exit 1; }
[ "$(id -u)" = 0 ] || { echo "xx run with sudo (mkarchiso needs root)." >&2; exit 1; }

if [ "${1:-}" = "clean" ]; then rm -rf work; echo "ok work/ removed"; exit 0; fi

mkarchiso -v -w work -o out iso
echo
ls -lh out/*.iso | tail -1
echo ":: test it:  run-iso.sh (QEMU/KVM, UEFI)"
