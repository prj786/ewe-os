#!/usr/bin/env bash
# Boot the newest built ISO in QEMU/KVM (UEFI) — a throwaway test VM.
# 4G RAM + virtio-gpu is enough for the live session; nothing is persisted.
set -euo pipefail
cd "$(dirname "$0")"

iso="$(ls -t out/*.iso 2>/dev/null | head -1)"
[ -n "$iso" ] || { echo "xx no ISO in out/ — run build.sh first." >&2; exit 1; }

# --bios: legacy boot (syslinux path) instead of UEFI — docs/TESTING.md's
# second row of the boot matrix.
firmware=()
if [ "${1:-}" != "--bios" ]; then
    ovmf=/usr/share/edk2/x64/OVMF_CODE.4m.fd
    [ -r "$ovmf" ] || ovmf=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd
    firmware=(-drive "if=pflash,format=raw,readonly=on,file=$ovmf")
fi

exec qemu-system-x86_64 \
    -enable-kvm -m 4G -smp 4 -cpu host \
    "${firmware[@]}" \
    -device virtio-gpu-pci -display gtk,gl=on \
    -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
    -audiodev pipewire,id=snd0 -device intel-hda -device hda-output,audiodev=snd0 \
    -cdrom "$iso"
