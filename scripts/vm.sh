#!/usr/bin/env bash
# Headless ISO test harness — boots an ISO in QEMU/KVM with a QMP socket and
# a guest-agent channel, so an agent (human or Claude) can screenshot, type,
# and run commands inside the guest without any display.
#
#   scripts/vm.sh up [iso]        boot newest testing/*.iso|out/*.iso (UEFI) + blank target disk
#   scripts/vm.sh up --bios [iso] legacy boot (syslinux path)
#   scripts/vm.sh up --gui [iso]  same, but with a visible window + sound (human test drive)
#   scripts/vm.sh up --vnc [iso]  VNC on 127.0.0.1:5900 instead (hosts whose QEMU has no
#                                 GTK/SDL module: view it with any VNC client or noVNC)
#   scripts/vm.sh up --video [iso]  the capture VM (docs/RECORDING.md): a window that
#                                 holds 1920x1080, guest GL through virtio-vga-gl, 8G,
#                                 6 CPUs, sound. With --software: the same window on the
#                                 standard VGA (no guest GL — the known-good fallback)
#   scripts/vm.sh shot [name]     QMP screendump → $EWE_VM_DIR/<name>.png
#   scripts/vm.sh key <keys…>     QMP send-key (e.g. `key ret`, `key ctrl-alt-f3`)
#   scripts/vm.sh type <text>     type a string via send-key
#   scripts/vm.sh ga <cmd…>       run a command in the guest via qemu-guest-agent
#                                 (live user is root on the ISO), prints stdout
#   scripts/vm.sh status          is it running?
#   scripts/vm.sh down            kill the VM (target disk is kept for reboot tests)
#   scripts/vm.sh reboot-disk     boot from the target DISK (no ISO) — the
#                                 post-install verification boot; takes the same
#                                 --gui / --vnc / --video [--software] as `up`
#   scripts/vm.sh save [name]     VM down: keep the disk as it is now + the UEFI
#                                 variables (default name: installed)
#   scripts/vm.sh restore [name]  VM down: put both back — the same first boot again
#   scripts/vm.sh saves           list what was saved
#   scripts/vm.sh wipe            VM down: delete the target disk and its saves
#
# Knobs (environment):
#   EWE_VM_DIR        sockets, screenshots, the target disk (default /tmp/ewe-vm)
#   EWE_VM_MEM        guest memory (6G; --video: 8G)
#   EWE_VM_SMP        guest CPUs (4; --video: 6)
#   EWE_VM_DISK_SIZE  the target disk, when it is created (40G)
#   EWE_VM_RES        WxH the display prefers, e.g. 1920x1080 (--video: that;
#                     otherwise the device's own default, 1280x800 under OVMF)
#   EWE_VM_VGA        the display device (virtio-vga; --video: virtio-vga-gl,
#                     --video --software: VGA)
#   EWE_VM_QEMU_IMG   a qemu-img to use (hosts without the package)
#   EWE_VM_GA_TIMEOUT seconds a `ga` command may take (60)
#   EWE_VM_DRYRUN=1   print the QEMU command line and stop: nothing is checked,
#                     created or started
#
# Gotchas encoded here (docs/TESTING.md):
#   * -device virtio-vga, NOT virtio-gpu-pci: gpu-pci's screendump is black.
#     EWE_VM_VGA=VGA when the host lacks qemu-hw-display-virtio-vga (Arch
#     ships it apart): the guest renders in software either way (no virgl).
#   * qemu-guest-agent is in the live package set; it needs the virtserialport
#     named org.qemu.guest_agent.0. The INSTALLED system has none: for the
#     post-install pass, add it to the target before pressing Reboot
#     (docs/TESTING.md, "Install path").
#   * EWE_VM_DIR (default /tmp/ewe-vm) holds the sockets, the screenshots and
#     the target disk — on a tmpfs /tmp an install fills RAM, so point it at a
#     real disk (--video refuses a tmpfs). On btrfs the folder is marked
#     no-copy-on-write before the disk is created: a qcow2 on a CoW,
#     compressing filesystem fragments with every guest write.
#   * A window (--gui, --video) needs QEMU's GTK module and the display
#     device's module; Arch ships both apart from qemu-system-x86. vm.sh says
#     which packages are missing instead of letting QEMU fail on its own words.
set -euo pipefail
cd "$(dirname "$0")/.."

WORK="${EWE_VM_DIR:-/tmp/ewe-vm}"
QMP="$WORK/qmp.sock"
GA="$WORK/ga.sock"
DISK="$WORK/target.qcow2"
PIDF="$WORK/qemu.pid"
QEMU=qemu-system-x86_64
QEMU_IMG="${EWE_VM_QEMU_IMG:-qemu-img}"
DRYRUN="${EWE_VM_DRYRUN:-0}"
[ "$DRYRUN" = 1 ] || mkdir -p "$WORK"

die() { printf 'xx %s\n' "$*" >&2; exit 1; }

# qemu_has display gtk | device virtio-vga-gl | audiodev pipewire — what THIS
# QEMU can do. Its modules are separate packages on Arch, and a device or a
# display whose module is missing is simply not in these lists.
qemu_has() {
    local out
    case "$1" in
        display)  out="$("$QEMU" -display help 2>/dev/null)";  grep -qx -- "$2" <<<"$out" ;;
        device)   out="$("$QEMU" -device help 2>/dev/null)";   grep -q -- "^name \"$2\"" <<<"$out" ;;
        audiodev) out="$("$QEMU" -audiodev help 2>/dev/null)"; grep -qx -- "$2" <<<"$out" ;;
        *) return 2 ;;
    esac
}

# the packages a display device comes from (its module loads the others)
device_pkgs() {
    case "$1" in
        virtio-vga-gl) echo "qemu-hw-display-virtio-gpu qemu-hw-display-virtio-gpu-gl qemu-hw-display-virtio-vga qemu-hw-display-virtio-vga-gl" ;;
        virtio-vga)    echo "qemu-hw-display-virtio-gpu qemu-hw-display-virtio-vga" ;;
        *)             echo "" ;;
    esac
}

# need_qemu <device> <window: 0|1> — stop with a sentence, and the one line
# that fixes it, when this QEMU cannot do what was asked
need_qemu() {
    local dev="$1" window="$2" missing=() pkgs=() line
    command -v "$QEMU" >/dev/null 2>&1 || die "$QEMU is not installed: sudo pacman -S qemu-system-x86"
    if [ "$window" = 1 ] && ! qemu_has display gtk; then
        missing+=("no \"gtk\" display (no window)"); pkgs+=(qemu-ui-gtk qemu-ui-opengl)
    fi
    if ! qemu_has device "$dev"; then
        missing+=("no \"$dev\" device")
        # shellcheck disable=SC2207  # package names: plain words
        pkgs+=($(device_pkgs "$dev"))
    fi
    [ "${#missing[@]}" -eq 0 ] && return 0
    # the line below should be the whole fix: add what a window would also
    # want (sound) and what the disk needs, when those are missing too
    if [ "$window" = 1 ] && ! qemu_has audiodev pipewire; then pkgs+=(qemu-audio-pipewire); fi
    if [ ! -f "$DISK" ] && ! command -v "$QEMU_IMG" >/dev/null 2>&1; then pkgs+=(qemu-img); fi
    {
        printf 'xx the QEMU on this machine cannot start this VM:\n'
        for line in "${missing[@]}"; do printf '     it has %s\n' "$line"; done
        if [ "${#pkgs[@]}" -gt 0 ]; then
            printf '   Arch ships those apart from qemu-system-x86. One line installs what is missing:\n'
            printf '     sudo pacman -S --needed %s\n' "${pkgs[*]}"
        fi
        printf '   Without them: EWE_VM_VGA=VGA scripts/vm.sh up --vnc  (the screen on 127.0.0.1:5900;\n'
        printf '   the standard VGA and VNC are built into every QEMU).\n'
    } >&2
    exit 1
}

# the target disk, created once
make_disk() {
    [ -f "$DISK" ] && return 0
    command -v "$QEMU_IMG" >/dev/null 2>&1 \
        || die "no qemu-img to create the target disk with (Arch ships it apart from qemu-system-x86): sudo pacman -S qemu-img — or point EWE_VM_QEMU_IMG at one"
    # btrfs: no copy-on-write (and so no compression) for what is created in
    # this folder from now on — the attribute only takes on new, empty files
    if [ "$(stat -f -c %T "$WORK" 2>/dev/null)" = btrfs ]; then
        chattr +C "$WORK" 2>/dev/null || echo "note: could not mark $WORK no-copy-on-write (chattr +C); the disk will fragment more" >&2
    fi
    "$QEMU_IMG" create -f qcow2 "$DISK" "${EWE_VM_DISK_SIZE:-40G}" >/dev/null
}

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
    python3 - "$GA" "${EWE_VM_GA_TIMEOUT:-60}" "$@" <<'PY'
import base64, json, socket, sys, time
path, limit, argv = sys.argv[1], float(sys.argv[2]), sys.argv[3:]
s = socket.socket(socket.AF_UNIX); s.settimeout(30); s.connect(path)
f = s.makefile("rw")
def call(cmd):
    f.write(json.dumps(cmd) + "\n"); f.flush()
    return json.loads(f.readline())
call({"execute": "guest-sync", "arguments": {"id": 1}})
r = call({"execute": "guest-exec", "arguments": {
    "path": argv[0], "arg": argv[1:], "capture-output": True}})
pid = r["return"]["pid"]
for _ in range(int(limit * 2)):
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

running() { [ -r "$PIDF" ] && kill -0 "$(cat "$PIDF" 2>/dev/null)" 2>/dev/null; }

boot() { # boot <cdrom-args...>
    [ "$DRYRUN" = 1 ] || { running && { echo "already running (pid $(cat "$PIDF")) — vm.sh down first" >&2; exit 1; }; }
    local vga="${EWE_VM_VGA:-virtio-vga}" mem="${EWE_VM_MEM:-6G}" smp="${EWE_VM_SMP:-4}" res="${EWE_VM_RES:-}"
    local window=0 sound=0 title=()
    # --gui: a visible window + sound for a human at the keyboard; the QMP/GA
    # sockets stay up either way, so shot/key/ga keep working alongside it
    local display=(-display none)
    [ "${VNC:-0}" = 1 ] && display=(-display none -vnc 127.0.0.1:0)
    # shellcheck disable=SC2054  # QEMU's own comma-separated option values
    [ "${GUI:-0}" = 1 ] && { display=(-display gtk,gl=on); window=1; sound=1; }
    if [ "${VIDEO:-0}" = 1 ]; then
        # The capture VM (docs/RECORDING.md). The window IS the guest's screen:
        # no menu bar, no tabs, and zoom-to-fit, so the frame keeps the size
        # the host gives the window through firmware, splash and desktop alike
        # — and a virtio display is told that size as its preferred mode.
        # window-close=off: a stray close would cut the disk off mid-write.
        local gl=on
        mem="${EWE_VM_MEM:-8G}"; smp="${EWE_VM_SMP:-6}"; res="${EWE_VM_RES:-1920x1080}"
        if [ "${SOFTWARE:-0}" = 1 ]; then
            # no guest GL: the standard VGA, the desktop on llvmpipe — what
            # every headless pass ran on. Its mode comes from the EDID alone
            # (xres/yres), so the resolution cannot follow the window.
            vga="${EWE_VM_VGA:-VGA}"; gl=off
        else
            vga="${EWE_VM_VGA:-virtio-vga-gl}"
        fi
        display=(-display "gtk,gl=$gl,show-menubar=off,show-tabs=off,zoom-to-fit=on,window-close=off")
        window=1; sound=1; title=(-name ewe)
        if [ "$DRYRUN" != 1 ] && [ "$(stat -f -c %T "$WORK" 2>/dev/null)" = tmpfs ]; then
            die "$WORK is in memory (tmpfs) and an install writes about 10 GB to the target disk — export EWE_VM_DIR=~/vm/ewe-video (any folder on a real disk) first"
        fi
    fi
    local dev="$vga"
    if [ -n "$res" ]; then
        [[ "$res" =~ ^[0-9]+x[0-9]+$ ]] || die "EWE_VM_RES is WIDTHxHEIGHT (1920x1080), not '$res'"
        dev="$vga,xres=${res%x*},yres=${res#*x}"
    fi
    [ "$DRYRUN" = 1 ] || need_qemu "$vga" "$window"
    local audio=()
    if [ "$sound" = 1 ]; then
        if [ "$DRYRUN" = 1 ] || qemu_has audiodev pipewire; then
            # shellcheck disable=SC2054  # QEMU's own comma-separated option values
            audio=(-audiodev pipewire,id=snd0 -device intel-hda -device hda-output,audiodev=snd0)
        else
            echo "note: no sound — this QEMU has no pipewire audio (sudo pacman -S qemu-audio-pipewire)" >&2
        fi
    fi
    local firmware=()
    if [ "${BIOS:-0}" != 1 ]; then
        local ovmf=/usr/share/edk2/x64/OVMF_CODE.4m.fd
        local vars_src=/usr/share/edk2/x64/OVMF_VARS.4m.fd
        [ -r "$ovmf" ] || { ovmf=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd; vars_src=/usr/share/edk2-ovmf/x64/OVMF_VARS.fd; }
        # writable NVRAM: without it bootctl's boot entry evaporates between
        # boots and the firmware drops into its boot-device menu
        if [ "$DRYRUN" != 1 ]; then [ -f "$WORK/OVMF_VARS.fd" ] || cp "$vars_src" "$WORK/OVMF_VARS.fd"; fi
        firmware=(-drive "if=pflash,format=raw,readonly=on,file=$ovmf"
                  -drive "if=pflash,format=raw,file=$WORK/OVMF_VARS.fd")
    fi
    # shellcheck disable=SC2054  # QEMU's own comma-separated option values
    local cmd=("$QEMU"
        -enable-kvm -m "$mem" -smp "$smp" -cpu host
        "${title[@]}" "${firmware[@]}"
        -device "$dev" "${display[@]}" "${audio[@]}"
        -qmp "unix:$QMP,server=on,wait=off"
        -chardev "socket,path=$GA,server=on,wait=off,id=qga0"
        -device virtio-serial -device virtserialport,chardev=qga0,name=org.qemu.guest_agent.0
        -device virtio-net-pci,netdev=n0 -netdev user,id=n0
        -device qemu-xhci -device usb-tablet
        -drive "file=$DISK,if=virtio,format=qcow2"
        -pidfile "$PIDF" -daemonize
        "$@")
    if [ "$DRYRUN" = 1 ]; then printf '%s\n' "${cmd[*]}"; return 0; fi
    make_disk
    "${cmd[@]}"
    echo "up — qmp: $QMP  ga: $GA  disk: $DISK"
}

# boot_flags <args…> — the mode flags `up` and `reboot-disk` share; what is
# left (an ISO for `up`) lands in REST
boot_flags() {
    REST=()
    while [ $# -gt 0 ]; do
        case "$1" in
            --bios) BIOS=1 ;;
            --gui) GUI=1 ;;
            --vnc) VNC=1 ;;
            --video) VIDEO=1 ;;
            --software) SOFTWARE=1 ;;
            --*) die "unknown option: $1 (--bios --gui --vnc --video --software)" ;;
            *) REST+=("$1") ;;
        esac
        shift
    done
    [ "${SOFTWARE:-0}" != 1 ] || [ "${VIDEO:-0}" = 1 ] || die "--software goes with --video (plain 'up' and --gui already choose the device with EWE_VM_VGA)"
}

# the saves need the VM down, the disk, and a qemu-img
save_ready() {
    running && die "vm.sh down first — a disk that is being written to cannot be saved or put back"
    [ -f "$DISK" ] || die "no target disk in $WORK yet"
    command -v "$QEMU_IMG" >/dev/null 2>&1 \
        || die "saves use qemu-img (Arch ships it apart from qemu-system-x86): sudo pacman -S qemu-img — or point EWE_VM_QEMU_IMG at one"
    [[ "$1" =~ ^[A-Za-z0-9._-]+$ ]] || die "a save's name is letters, digits, dots and dashes — not '$1'"
}

case "${1:-status}" in
up)
    shift
    boot_flags "$@"
    # shellcheck disable=SC2012  # newest first; ISO names are plain
    # (|| true: with no image at all `ls` fails, and under errexit + pipefail
    # that ended the script before it could say so)
    iso="${REST[0]:-$(ls -t testing/*.iso out/*.iso 2>/dev/null | head -1 || true)}"
    [ -n "$iso" ] || die "no ISO to boot: none in testing/ or out/ — build one (build.sh), download a test build (docs/TESTING.md), or name a file"
    [ -r "$iso" ] || die "no such ISO: $iso"
    [ "$DRYRUN" = 1 ] || echo "booting $iso"
    # fresh UEFI variables for every ISO boot: after an install the saved
    # BootOrder starts with the installed disk's entry and OVMF never falls
    # back to the CD (it tries PXE/HTTP instead — found in the 0.17 pass);
    # the install then records its entry in these new ones for reboot-disk
    [ "$DRYRUN" = 1 ] || rm -f "$WORK/OVMF_VARS.fd"
    boot -cdrom "$iso" -boot d
    ;;
reboot-disk)
    shift
    boot_flags "$@"
    [ "${#REST[@]}" -eq 0 ] || die "reboot-disk boots the target disk — it takes no ISO (${REST[*]})"
    boot -boot c
    ;;
save)
    # The disk and the firmware's variables, as they are now — taken right
    # after an install (VM down), this is "the moment before the first boot":
    # restore brings it back as often as a take needs. The UEFI variables go
    # with it because that is where bootctl recorded the installed system.
    name="${2:-installed}"
    save_ready "$name"
    "$QEMU_IMG" snapshot -d "$name" "$DISK" >/dev/null 2>&1 || true   # a save of that name is replaced
    "$QEMU_IMG" snapshot -c "$name" "$DISK"
    if [ -f "$WORK/OVMF_VARS.fd" ]; then cp "$WORK/OVMF_VARS.fd" "$WORK/OVMF_VARS.$name.fd"; fi
    echo "saved '$name' (disk + UEFI variables) — vm.sh restore $name puts it back"
    ;;
restore)
    name="${2:-installed}"
    save_ready "$name"
    "$QEMU_IMG" snapshot -a "$name" "$DISK" || die "no save called '$name' on this disk (vm.sh saves)"
    if [ -f "$WORK/OVMF_VARS.$name.fd" ]; then cp "$WORK/OVMF_VARS.$name.fd" "$WORK/OVMF_VARS.fd"; fi
    echo "restored '$name' — vm.sh reboot-disk boots it"
    ;;
saves)
    [ -f "$DISK" ] || die "no target disk in $WORK yet"
    command -v "$QEMU_IMG" >/dev/null 2>&1 || die "saves use qemu-img: sudo pacman -S qemu-img — or point EWE_VM_QEMU_IMG at one"
    "$QEMU_IMG" snapshot -l "$DISK"
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
    rm -f "$DISK" "$WORK"/OVMF_VARS.*.fd && echo "target disk wiped"
    ;;
*)
    echo "usage: vm.sh up [--bios] [--gui|--vnc|--video [--software]] [iso] | reboot-disk [--gui|--vnc|--video [--software]] | save [name] | restore [name] | saves | shot [name] | key <k…> | click <x> <y> | type <text> | ga <cmd…> | status | down | wipe" >&2
    exit 1
    ;;
esac
