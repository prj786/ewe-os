# Installing ewe

ewe installs from one ISO in about ten minutes and asks only what is
personal: your keyboard, your place, which disk, who you are. Everything
technical is decided for you (see *Decided by ewe* below).

## 1 · Get the ISO

Download the newest release from
<https://github.com/prj786/ewe-os/releases>. GitHub caps release files at
2 GiB, so the image comes as `.part` files plus `SHA256SUMS`. Join and
verify:

```sh
cat ewe-*-x86_64.iso.*.part > ewe-x86_64.iso
sha256sum -c SHA256SUMS --ignore-missing
```

Write it to a USB stick (any tool works — it is a plain hybrid ISO):

```sh
sudo dd if=ewe-x86_64.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Replace `/dev/sdX` with the stick (`lsblk` shows it — pick the disk, not a
partition). Ventoy, balenaEtcher and GNOME Disks work too.

## 2 · Boot it

Boot the stick in **UEFI** mode (legacy BIOS boot is supported as a
fallback). If your firmware has Secure Boot on, turn it off — the ISO is
not signed.

You land straight on the ewe desktop, running from the stick, with
**Install ewe** pinned first in the dock and the launcher. Everything you
see is the real thing: try the Control Center (Super + N), Komble, the
settings. Nothing is written to your disks until you press the red Install
button on the Summary screen.

## 3 · Install — seven screens

1. **Welcome** — keyboard layout, with a field to try it.
2. **Network** — the install downloads the whole system, so this screen
   waits for a connection: a cable just works, Wi-Fi is joined right here
   (pick the network, type the password). Nothing past this screen opens
   until NetworkManager reports full connectivity.
3. **Time & place** — your zone is detected (two providers must agree;
   you always see what was chosen), plus a language/locale picker.
4. **Disk** — pick the disk. **The whole disk is erased**; the warning is
   red here and again on the Summary.
5. **Your account** — name, username, password, computer name.
6. **Summary** — everything you chose, the *Decided by ewe* panel, and the
   only Install button.
7. **Install** — a last connectivity check, then streamed progress, then
   **Reboot**. Remove the stick when the screen goes dark. If a step
   fails, the screen says why and offers **Retry** (the disk is
   partitioned again from scratch) or **Back**.

The installed machine boots through the ewe splash into the graphical
greeter. The first login opens the **Welcome** flow: connect to the
internet (Wi-Fi is joined from the flow itself), sign in with Google
(optional, one consent for calendar, mail, sync, Drive), restore a backup
if your account has one from another ewe machine, and a sixty-second tour.

### Decided by ewe

| | |
|---|---|
| filesystem | btrfs, `@` / `@home` subvolumes, zstd compression |
| bootloader | systemd-boot, splash-only boot (no menu unless you hold a key) |
| hibernation | automatic when a battery is present: swapfile of RAM + 2 GB |
| kernel | `linux` (stock Arch) + your CPU's microcode |
| graphics | drivers matched to the GPU(s) found (Intel / AMD / NVIDIA) |
| sound / network / login | PipeWire / NetworkManager / greetd |
| updates | the `[ewe]` repository is preconfigured; `pacman -Syu` (or Komble → Updates) rolls everything forward |

## 4 · After the install

- **Updates**: Komble → Updates, or `sudo pacman -Syu`. The top-bar download
  glyph tells you when something is pending.
- **Google**: Settings → User → *Connect Google* if you skipped it in the
  Welcome flow. Signing in also mounts your Drive at `~/Google Drive`
  (Files → sidebar).
- **Settings**: Super + , — or the gear in the Control Center.
- **Language later**: Settings → Time & Place → Language.

## Headless / rescue install

The graphical installer drives a root helper with fixed verbs
(`partition`, `mkfs`, `mount`, `pacstrap`, `layer`, `user`, `hibernate`,
`bootloader`). The same primitives are reachable from a terminal on the
live session as `ewe-install` (see `ewe-install --help`), which is also
the path for scripted or headless installs and for re-layering ewe onto an
existing Arch system (`ewe-install --layer-only`).

## Testing the ISO in a VM

QEMU/KVM with UEFI is the reference environment; the repo ships a harness:

```sh
scripts/vm.sh up --gui testing/ewe-x86_64.iso   # boot with a window + sound
scripts/vm.sh down
scripts/vm.sh reboot-disk --gui                  # boot the installed disk
```

The harness gives the VM a blank 40 GB disk and *writable* UEFI variables —
without those, systemd-boot's entry vanishes between boots and the firmware
drops into its device menu. See `docs/TESTING.md` for the release
checklist and `docs/TROUBLESHOOTING.md` when something looks wrong.
