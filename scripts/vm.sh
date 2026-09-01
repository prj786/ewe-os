#!/usr/bin/env bash
# Headless ISO test harness — boots an ISO in QEMU/KVM with a QMP socket and
# a guest-agent channel, so an agent (human or Claude) can screenshot, type,
# and run commands inside the guest without any display.
#
#   scripts/vm.sh up [iso]        boot newest testing/*.iso|out/*.iso (UEFI) + blank target disk
#   scripts/vm.sh up --bios [iso] legacy boot (syslinux path)
#   scripts/vm.sh shot [name]     QMP screendump → /tmp/ewe-vm/<name>.png
#   scripts/vm.sh key <keys…>     QMP send-key (e.g. `key ret`, `key ctrl-alt-f3`)
#   scripts/vm.sh type <text>     type a string via send-key
#   scripts/vm.sh ga <cmd…>       run a command in the guest via qemu-guest-agent
#                                 (live user is root on the ISO), prints stdout
#   scripts/vm.sh status          is it running?
#   scripts/vm.sh down            kill the VM (target disk is kept for reboot tests)
#   scripts/vm.sh reboot-disk     boot from the target DISK (no ISO) — the
#                                 post-install verification boot
#
# Gotchas encoded here (docs/TESTING.md):
#   * -device virtio-vga, NOT virtio-gpu-pci: gpu-pci's screendump is black.
#   * qemu-guest-agent is in the live package set; it needs the virtserialport
#     named org.qemu.guest_agent.0.
set -euo pipefail
cd "$(dirname "$0")/.."

WORK=/tmp/ewe-vm
QMP="$WORK/qmp.sock"
GA="$WORK/ga.sock"
DISK="$WORK/target.qcow2"
PIDF="$WORK/qemu.pid"
mkdir -p "$WORK"

qmp() { # qmp '<json command>' — handshake + one command, prints the reply line
    python3 - "$QMP" "$1" <<'PY'
import json, socket, sys
s = socket.socket(socket.AF_UNIX); s.settimeout(20); s.connect(sys.argv[1])
f = s.makefile("rw")
f.readline()                                    # greeting
f.write(json.dumps({"execute": "qmp_capabilities"}) + "\n"); f.flush(); f.readline()
f.write(sys.argv[2] + "\n"); f.flush()
while True:
    line = f.readline()
    if not line: break
    j = json.loads(line)
    if "return" in j or "error" in j:
        print(json.dumps(j)); break
PY
}

ga_exec() { # run argv in the guest through the guest agent, print stdout
    python3 - "$GA" "$@" <<'PY'
import base64, json, socket, sys, time
path, argv = sys.argv[1], sys.argv[2:]
s = socket.socket(socket.AF_UNIX); s.settimeout(30); s.connect(path)
f = s.makefile("rw")
def call(cmd):
    f.write(json.dumps(cmd) + "\n"); f.flush()
    return json.loads(f.readline())
call({"execute": "guest-sync", "arguments": {"id": 1}})
r = call({"execute": "guest-exec", "arguments": {
    "path": argv[0], "arg": argv[1:], "capture-output": True}})
pid = r["return"]["pid"]
for _ in range(120):
    time.sleep(0.5)
    st = call({"execute": "guest-exec-status", "arguments": {"pid": pid}})["return"]
    if st.get("exited"):
        out = base64.b64decode(st.get("out-data", "")).decode(errors="replace")
        err = base64.b64decode(st.get("err-data", "")).decode(errors="replace")
        sys.stdout.write(out)
        if err: sys.stderr.write(err)
        sys.exit(st.get("exitcode", 0))
print("guest-exec: timed out", file=sys.stderr); sys.exit(124)
PY
}

running() { [ -r "$PIDF" ] && kill -0 "$(cat "$PIDF")" 2>/dev/null; }

boot() { # boot <cdrom-args...>
    running && { echo "already running (pid $(cat "$PIDF")) — vm.sh down first" >&2; exit 1; }
    [ -f "$DISK" ] || qemu-img create -f qcow2 "$DISK" 40G >/dev/null
    local firmware=()
    if [ "${BIOS:-0}" != 1 ]; then
        local ovmf=/usr/share/edk2/x64/OVMF_CODE.4m.fd
        local vars_src=/usr/share/edk2/x64/OVMF_VARS.4m.fd
        [ -r "$ovmf" ] || { ovmf=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd; vars_src=/usr/share/edk2-ovmf/x64/OVMF_VARS.fd; }
        # writable NVRAM: without it bootctl's boot entry evaporates between
        # boots and the firmware drops into its boot-device menu
        [ -f "$WORK/OVMF_VARS.fd" ] || cp "$vars_src" "$WORK/OVMF_VARS.fd"
        firmware=(-drive "if=pflash,format=raw,readonly=on,file=$ovmf"
                  -drive "if=pflash,format=raw,file=$WORK/OVMF_VARS.fd")
    fi
    qemu-system-x86_64 \
        -enable-kvm -m 6G -smp 4 -cpu host \
        "${firmware[@]}" \
        -device virtio-vga -display none \
        -qmp "unix:$QMP,server=on,wait=off" \
        -chardev "socket,path=$GA,server=on,wait=off,id=qga0" \
        -device virtio-serial -device virtserialport,chardev=qga0,name=org.qemu.guest_agent.0 \
        -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
        -device qemu-xhci -device usb-tablet \
        -drive "file=$DISK,if=virtio,format=qcow2" \
        -pidfile "$PIDF" -daemonize \
        "$@"
    echo "up — qmp: $QMP  ga: $GA  disk: $DISK"
}

case "${1:-status}" in
up)
    shift; [ "${1:-}" = "--bios" ] && { BIOS=1; shift; }
    iso="${1:-$(ls -t testing/*.iso out/*.iso 2>/dev/null | head -1)}"
    [ -n "$iso" ] && [ -r "$iso" ] || { echo "no ISO found (testing/ or out/)" >&2; exit 1; }
    echo "booting $iso"
    boot -cdrom "$iso" -boot d
    ;;
reboot-disk)
    boot -boot c
    ;;
shot)
    name="${2:-vm}"
    qmp "{\"execute\":\"screendump\",\"arguments\":{\"filename\":\"$WORK/$name.png\",\"format\":\"png\"}}" >/dev/null
    echo "$WORK/$name.png"
    ;;
key)
    shift
    for k in "$@"; do
        keys=$(python3 -c "import json,sys; print(json.dumps([{'type':'qcode','data':q} for q in sys.argv[1].split('-')]))" "$k")
        qmp "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[]}}" >/dev/null 2>&1 || true
        qmp "{\"execute\":\"send-key\",\"arguments\":{\"keys\":$keys}}" >/dev/null
        sleep 0.1
    done
    ;;
click) # click <x> <y> — absolute pixel coords on the guest framebuffer
    # (needs the usb-tablet the boot args add — rel mice can't teleport)
    x="$2"; y="$3"
    # abs axes are scaled 0..32767 over the display size; probe via screendump
    qmp "{\"execute\":\"screendump\",\"arguments\":{\"filename\":\"$WORK/.probe.png\",\"format\":\"png\"}}" >/dev/null
    dims=$(python3 - "$WORK/.probe.png" <<'PY'
import struct, sys
try:
    with open(sys.argv[1], 'rb') as f:
        f.read(16)
        w, h = struct.unpack('>II', f.read(8))
    print(w, h)
except Exception:
    print(1280, 800)
PY
)
    read -r W H <<<"$dims"
    ax=$(( x * 32767 / W )); ay=$(( y * 32767 / H ))
    qmp "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[
        {\"type\":\"abs\",\"data\":{\"axis\":\"x\",\"value\":$ax}},
        {\"type\":\"abs\",\"data\":{\"axis\":\"y\",\"value\":$ay}}]}}" >/dev/null
    qmp "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[
        {\"type\":\"btn\",\"data\":{\"button\":\"left\",\"down\":true}}]}}" >/dev/null
    qmp "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[
        {\"type\":\"btn\",\"data\":{\"button\":\"left\",\"down\":false}}]}}" >/dev/null
    ;;
type)
    shift
    python3 - "$0" "$*" <<'PY'
import subprocess, sys
MAP = {' ': 'spc', '-': 'minus', '.': 'dot', '/': 'slash', '_': 'shift-minus',
       ':': 'shift-semicolon', '@': 'shift-2', ',': 'comma', '=': 'equal'}
for ch in sys.argv[2]:
    if ch.isupper():
        key = 'shift-' + ch.lower()
    else:
        key = MAP.get(ch, ch)
    subprocess.run([sys.argv[1], 'key', key], check=True)
PY
    ;;
ga)
    shift; ga_exec "$@"
    ;;
status)
    running && echo "running (pid $(cat "$PIDF"))" || echo "not running"
    ;;
down)
    # graceful first: a SIGTERM'd guest never flushes its dirty FAT buffers —
    # that once zero-lengthed a freshly-installed ESP's loader files
    if running; then
        qmp '{"execute":"system_powerdown"}' >/dev/null 2>&1 || true
        for _ in $(seq 1 20); do running || break; sleep 0.5; done
        running && kill "$(cat "$PIDF")"
        echo "stopped"
    else
        echo "not running"
    fi
    ;;
wipe)
    running && { echo "vm.sh down first" >&2; exit 1; }
    rm -f "$DISK" && echo "target disk wiped"
    ;;
*)
    echo "usage: vm.sh up [--bios] [iso] | reboot-disk | shot [name] | key <k…> | type <text> | ga <cmd…> | status | down | wipe" >&2
    exit 1
    ;;
esac
