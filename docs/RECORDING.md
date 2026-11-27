# Recording an install

A VM whose screen is 1920x1080 from the firmware to the desktop, in a window
you can record. One script: `scripts/vm.sh`.

**Do one unrecorded pass first.** The window path (`--video`: QEMU's GTK
display, guest GL through `virtio-vga-gl`) was written on a machine whose QEMU
could not open a window, so it has never been run. What was run: every refusal
below, the VM itself headless on the standard VGA at 1920x1080 (boot menu,
splash, live desktop, the guest-agent pin), the disk, `save` / `restore`.

## 1 · Once: the QEMU pieces Arch ships apart

`qemu-system-x86` alone has no window, no virtio display, no sound and no
`qemu-img`. `vm.sh` names what is missing; on a bare install it is all of:

```sh
sudo pacman -S --needed qemu-ui-gtk qemu-ui-opengl qemu-hw-display-virtio-gpu qemu-hw-display-virtio-gpu-gl qemu-hw-display-virtio-vga qemu-hw-display-virtio-vga-gl qemu-audio-pipewire qemu-img
```

(If pacman cannot find a file, the mirrors have moved on: `sudo pacman -Syu`
first — no partial upgrades.)

## 2 · Start it

```sh
export EWE_VM_DIR=~/vm/ewe-video        # the disk lives here; not /tmp (memory)
scripts/vm.sh up --video path/to/ewe-<version>-x86_64.iso
```

8 GB, 6 CPUs, sound, a 40 GB blank disk (`EWE_VM_DISK_SIZE=128G` before the
first `up` for a bigger number on the Disk screen), guest GL. The window has no
menu bar and no tabs: it is the guest's screen and nothing else.

If the desktop glitches with guest GL — it is the untested half — the same
window without it: `scripts/vm.sh up --video --software <iso>`. That is the
standard VGA with the desktop on llvmpipe, what every headless pass has run
on: correct, but animations stutter at this size.

## 3 · Hold 1920x1080

QEMU's GTK display tells a virtio display the window's size, which becomes its
preferred mode, and ewe's desktop follows "preferred" (this is how QEMU is
built; it has not been watched here yet). So the window has to be exactly
1920x1080, on a display at scale 1 — the external one, not a scaled laptop
panel. On an ewe host, before `up`:

```sh
hyprctl eval 'hl.window_rule({ name = "ewe-vm", match = { class = "^qemu" }, float = true, center = true, size = { 1920, 1080 } })'
```

It lasts until Hyprland reloads its config. If the window still tiles, its
class is something else: `hyprctl clients | grep -B4 'QEMU (ewe)'` shows it.

Belt and braces, from the host while the **live** session is up (it has the
guest agent; this one was run against the 0.17 ISO):

```sh
scripts/vm.sh ga runuser -u ewe -- env XDG_RUNTIME_DIR=/run/user/1000 \
    hyprctl --instance 0 eval 'hl.monitor({ output = "", mode = "1920x1080@60", position = "0x0", scale = 1 })'
```

The installed system has no guest agent: there, Settings → Displays pins the
mode. With `--software` none of this matters — that display has one preferred
mode, the one it was started with.

## 4 · Record

QEMU records nothing itself. `gpu-screen-recorder` (an ewe dependency) through
the portal captures the window alone — no host bar, no notifications:

```sh
gpu-screen-recorder -w portal -f 60 -fm cfr -cursor yes -o ~/Videos/ewe-install.mkv
```

Pick the QEMU window in the picker; Ctrl+C ends the take. Add
`-a default_output` for the guest's sounds (and every other sound of the host).
A region works too when the window sits centred on a 2560x1440 display —
`-w region -region 1920x1080+320+180` — with Do Not Disturb on.

## 5 · The takes

One take: install, press **Reboot** in the installer, and the same window
should boot the installed disk — the firmware's boot order starts with it
after an install (that is why `up` resets the UEFI variables). Confirm it in
the unrecorded pass; if the firmware goes back to the ISO, use two takes.

Two takes, the second as often as you like:

```sh
scripts/vm.sh down                      # after the install finished
scripts/vm.sh save                      # the disk + the UEFI variables, as "installed"
scripts/vm.sh reboot-disk --video       # first boot, take 1
scripts/vm.sh down && scripts/vm.sh restore && scripts/vm.sh reboot-disk --video   # …take 2
```

A whole new install: `scripts/vm.sh down && scripts/vm.sh wipe`, then `up`.

## Before you press record

- **Not while an `[ewe]` publish runs.** It deletes and recreates the repository
  the install downloads from; an install that overlaps its last minutes fails.
  `gh run list -R prj786/ewe-repo --workflow publish.yml --limit 1` must say
  `completed`.
- **Ctrl+Alt+G** is QEMU's keyboard grab: Super and the other shortcuts of
  the host's own desktop should reach the guest only while it is on. The same
  keys give the keyboard back.
- **The installed system goes idle**: dim after 280 s without input, lock at
  300 s, suspend at 15 minutes. The live session never does.
- The pointer hides after 5 s without movement; blur is off in every VM (the
  desktop turns it off itself); a few minutes after the first boot eep fetches
  its graphics driver in the background.
- The guest's sound plays on the host's default output.
