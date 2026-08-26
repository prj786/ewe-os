# Release checklist

Every ISO passes ALL of this before it ships. Run it after `./build.sh`
(remember `sudo ./build.sh clean` first after profile edits — mkarchiso
silently reuses a stale work dir).

## Boot matrix

| | how | pass = |
|---|---|---|
| UEFI | `./run-iso.sh` (OVMF) | boot menu → live desktop |
| BIOS | `./run-iso.sh --bios` | same |

Headless (no window, screenshots over QMP): use `-device virtio-vga`, **not**
`virtio-gpu-pci` — its screendump reads solid black even while the guest
renders. `scripts/` has the QMP screendump helper from the 0.1 bring-up.

## Live session

- [ ] boot reaches the desktop with no interaction (autologin, first-boot
      deploy runs; a short plymouth-covered pause is OK, black screen is not)
- [ ] bar, dock, launcher (Super+D), Quick Settings all open
- [ ] folder/app icons render (Reversal) — pink/black tiles = theme missing
- [ ] network up (NetworkManager); Komble opens and loads the AM catalog
- [ ] `sudo pacman -Syu` resolves against `[ewe]` without errors
- [ ] leave idle 12+ min: session must NOT lock
- [ ] tty1 root rescue console reachable (Ctrl+Alt+F1)

## Install path

- [ ] QEMU with a blank virtio disk: run "Install ewe" (or `ewe-install`),
      minimal archinstall config, one user account
- [ ] installer completes; target has `[ewe]` in pacman.conf, greetd enabled,
      wayland session entry, user's home deployed (`~/.local/share/ewe`)
- [ ] reboot from disk: greeter lists "Ewe", login lands in the desktop
- [ ] on the installed system: `sudo pacman -Syu` works; lock screen unlocks
      with the user's password

## Updates

- [ ] on the previous release's installed system, `pacman -Syu` alone brings
      in the new ewe payload and the session self-refreshes at next login
