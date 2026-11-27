#!/usr/bin/env bash
# Boot the newest built ISO (testing/*.iso, out/*.iso) in a QEMU/KVM window,
# with a blank target disk, so "Install ewe" has something to install onto.
#
#   ./run-iso.sh             UEFI, a window with sound      (vm.sh up --gui)
#   ./run-iso.sh --bios      legacy boot, the syslinux path — docs/TESTING.md's
#                            second row of the boot matrix
#   ./run-iso.sh --video     the 1920x1080 capture VM       (docs/RECORDING.md)
#   ./run-iso.sh … <iso>     that image instead of the newest
#
# A front for scripts/vm.sh, which holds the one machine definition (this
# script used to carry its own: no disk — the installer's Disk screen was
# empty — and no UEFI variables). The VM runs in the background:
# `scripts/vm.sh down` stops it, `scripts/vm.sh reboot-disk --gui` boots what
# was installed. EWE_VM_DIR is where its disk lives (default /tmp/ewe-vm —
# usually memory; give it a real folder before installing). vm.sh says which
# QEMU packages are missing when this machine cannot open a window.
set -euo pipefail
cd "$(dirname "$0")"

mode=(--gui)
rest=()
for a in "$@"; do
    case "$a" in
        --video|--vnc) mode=("$a") ;;
        *) rest+=("$a") ;;
    esac
done
exec scripts/vm.sh up "${mode[@]}" "${rest[@]}"
