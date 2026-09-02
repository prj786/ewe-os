# Troubleshooting the ISO and a fresh install

The DE's own guide (`ewe/docs/TROUBLESHOOTING.md`) covers the desktop once
it runs. This one is for the ISO, the installer and the first boots.

## Getting to a console

- **Live ISO**: `Ctrl+Alt+F3` is a root rescue console. tty1 belongs to the
  greeter/desktop — never expect a prompt there.
- **Installed system**: `Ctrl+Alt+F3` is a login console (your user, then
  `sudo`). Back to the desktop with `Ctrl+Alt+F1`.
- Logs: `journalctl -b -p err` for this boot's errors; `journalctl -b -u
  greetd` for the login stack; `journalctl --user -u ewe.service` for the
  desktop shell.

## The stick does not boot

- Boot in UEFI mode; disable Secure Boot (the ISO is unsigned).
- Re-check the image: `sha256sum -c SHA256SUMS --ignore-missing` after
  joining the parts. A truncated download boots to a black screen or an
  "invalid or corrupt" message.
- Legacy BIOS boot exists as a fallback but the installed system is
  UEFI-only (systemd-boot) — install from UEFI.

## Black screen instead of the live desktop

- Wait ~60 s on first boot: the desktop deploys itself under the splash;
  slow USB sticks stretch this.
- `Ctrl+Alt+F3` → `journalctl -b -u ewe-live-user -u greetd --no-pager`.
  If `ewe-live-user.service` failed, run `/usr/local/bin/ewe-live-deploy`
  by hand as the `ewe` user (`runuser -u ewe -- /usr/local/bin/ewe-live-deploy`)
  and `systemctl restart greetd`.
- Virtual machines: the desktop needs a GPU with 3D — in QEMU use
  `virtio-vga`/`virtio-gpu` with `gl=on`; in VirtualBox enable 3D
  acceleration and give the VM ≥ 64 MB of video memory.

## The installer

- **"No disk"** — the disk list shows whole disks only. An NVMe drive
  appears as `nvme0n1`; USB sticks (including the one you booted) are
  hidden. Check `lsblk` on tty3.
- **Next is greyed out on the Network screen** — the installer refuses to
  continue until NetworkManager reports *full* connectivity. Join a Wi-Fi
  network from the list (passwords are 8+ characters), or plug in a cable.
  "Connected, but no internet" means the link is up and the route is not:
  a captive portal (open a browser from the dock, sign in, come back) or a
  router without uplink. `nmcli device` and `nmcli networking connectivity`
  on tty3 show what the installer sees.
- **Time & place shows the wrong city** — the two location providers
  disagreed or the network was down. Pick the zone by hand; the choice is
  saved as manual (`/etc/ewe/manual-timezone`) and can be changed later in
  Settings → Time & Place.
- **"Lost the connection — go back to Network"** — the last check before
  the disk is touched failed. Nothing was erased; reconnect and press
  Install again.
- **"pacstrap failed" / "the ewe layer failed"** — the download died
  mid-install (network, a mirror, the `[ewe]` repo). The message is the
  helper's own; the log above it has the last lines from pacman. Press
  **Retry**: the disk is partitioned again from scratch, nothing is left
  half-done. **Back** returns to the Summary.
- **"Erase" confirmation** — there is no dual-boot mode. The chosen disk is
  wiped entirely. Keep other disks unplugged if in doubt.

## First boot of the installed system

- **Firmware boot menu instead of ewe** — the UEFI entry was not recorded.
  Boot the stick, open tty3, mount the target's ESP and run
  `bootctl --esp-path=/mnt/boot install`. In QEMU this happens when the
  VM has no writable OVMF variables (use `scripts/vm.sh`, which provides
  them).
- **Splash, then nothing** — greetd could not take VT1. tty3 →
  `journalctl -b -u greetd`. Known trap: a lingering `plymouthd`; the greetd
  drop-in guards against it (`plymouth quit` before start) — if a stale
  drop-in was carried over, reinstall the DE's `greetd` files
  (`ewe-install --layer-only`).
- **Greeter shows, login fails** — the password is the one typed in the
  installer; keyboard layout at the greeter is the one chosen on the
  Welcome screen. Wrong layout: log in via tty3 and run `localectl
  set-x11-keymap <layout>`.
- **Desktop loads but Settings → User says "Not configured"** — the ewe
  package lacks its Google client: `sudo pacman -Syu ewe` (fixed since
  0.9.9-2; CI now asserts the file is in every package).

## Upgrades

- `sudo pacman -Syu` upgrades everything, including the desktop. The
  shell restarts itself on the next login; apps pick new versions up at
  once.
- Komble → Updates is the same operation with a progress view.
- If the `[ewe]` repo signature is reported as untrusted (a reinstalled
  keyring), re-trust the project key that ships with the DE:
  `sudo pacman-key --add /usr/share/ewe/system/ewe.gpg && sudo pacman-key
  --lsign-key 639CA544B61509B3FC2621ABADBCD432FC0763C1`.

## Reporting

Include: the ISO file name and the DE version (`cat /usr/share/ewe/VERSION`),
`journalctl -b -p err --no-pager | tail -50`, and for installer problems the
text of the Install screen's progress log (it is selectable) or, for the
CLI path, the terminal output of `ewe-install`.
Issues: <https://github.com/prj786/ewe-os/issues>.
