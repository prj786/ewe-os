# Release checklist

Every ISO passes ALL of this before it ships. Run it after `./build.sh`
(remember `sudo ./build.sh clean` first after profile edits — mkarchiso
silently reuses a stale work dir).

## Boot matrix

| | how | pass = |
|---|---|---|
| UEFI | `./run-iso.sh` (OVMF) | boot menu → live desktop |
| BIOS | `./run-iso.sh --bios` | same |

Headless (no window, screenshots over QMP): `scripts/vm.sh` — boots the
newest `testing/*.iso`/`out/*.iso` with a QMP socket, a qemu-guest-agent
channel (root shell into the live system: `vm.sh ga <cmd…>`), and a
persistent 40G blank target disk. `vm.sh shot` screendumps to PNG,
`vm.sh key`/`type` synthesise input, `vm.sh reboot-disk` boots the
installed disk for the post-install verification pass. It uses
`-device virtio-vga`, **not** `virtio-gpu-pci` — gpu-pci's screendump
reads solid black even while the guest renders.

## Live session

- [ ] boot reaches the desktop with no interaction (autologin, first-boot
      deploy runs; a short plymouth-covered pause is OK, black screen is not)
- [ ] bar, dock, launcher (Super+D), Quick Settings all open
- [ ] folder/app icons render (Reversal) — pink/black tiles = theme missing
- [ ] network up (NetworkManager); Komble opens and loads the AM catalog
- [ ] `sudo pacman -Syu` resolves against `[ewe]` without errors
- [ ] leave idle 12+ min: session must NOT lock
- [ ] tty3 root rescue console reachable (Ctrl+Alt+F3; tty1 stays blank —
      it belongs to the plymouth→greeter handoff)

## Install path

- [ ] QEMU with a blank virtio disk: run "Install ewe" (or `ewe-install`),
      minimal archinstall config, one user account
- [ ] installer completes; target has `[ewe]` in pacman.conf, greetd enabled,
      wayland session entry, user's home deployed (`~/.local/share/ewe`)
- [ ] reboot from disk: greeter lists "Ewe", login lands in the desktop
- [ ] on the installed system: `sudo pacman -Syu` works; lock screen unlocks
      with the user's password
- [ ] target is fully updated: `pacman -Qu` is empty at first boot
- [ ] first login: `journalctl -b | grep gkr-pam` shows the keyring
      unlocked/created, no "invalid"; `~/.local/share/keyrings/login.keyring`
      exists and Google sign-in stores its token with no keyring prompt

## Updates

- [ ] on the previous release's installed system, `pacman -Syu` alone brings
      in the new ewe payload and the session self-refreshes at next login
