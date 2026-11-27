# ewe OS

The distro layer of [ewe](https://github.com/prj786/ewe): the archiso profile
that builds the **live/install ISO**. The DE, apps, and packaging live in
their own repos — this one turns them into a bootable distribution.

**Version:** the `VERSION` file (the distro has its own line; the DE's
`ewe` package has another). **Install guide: [docs/INSTALL.md](docs/INSTALL.md)
· problems: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md).** The ISO pins nothing — it preconfigures the
[\[ewe\] pacman repo](https://github.com/prj786/ewe-repo) so both the live
session and installed systems roll forward with plain `pacman -Syu`.

## What the ISO does

- **Live session** — boots through greetd straight into a full ewe desktop
  (autologin as the `ewe` live user; the DE deploys itself on first start via
  `ewe-setup` from the preinstalled `ewe` package, at boot, under the
  plymouth splash). The live user — and only the live user — also gets the
  **Dock plugin**, so *Install ewe* has a dock to be pinned in (since ewe
  0.25 a fresh account has no dock; `ewe-live-deploy`). A root rescue
  console lives on tty3 (Ctrl+Alt+F3); tty1 belongs to the greeter.
- **`ewe-installer`** — the graphical installer (Tauri; RFC-003 in
  `docs/`): **language** (by its own name; it fills in the rest), keyboard,
  network, time & place (with the formats), disk, your login, **your ewe
  account**, **plugins**, **apps**, summary, install. The language brings its
  region, keyboards (US English first when the language's own cannot type
  a–z), formats, a spelling dictionary when Arch has one and, for Chinese,
  Japanese, Korean, Vietnamese and the Indic languages, an input method
  (fcitx5, Ctrl+Space) — all changeable, all installed in one pacman run
  and saved through `ewe-conf` as the new user. The Apps screen offers
  Komble's curated list: repo apps install with the system, AUR and
  AppImage picks wait in Komble (`~/.local/state/ewe/install-picks.json`,
  *For you*) — nothing is built from the AUR in the installer. *Your ewe account* is sign in or
  skip: signed in to your Nextcloud, the new computer starts signed in (the
  app password goes into the new login keyring) and, when the account holds
  a backup, with everything from it — settings, the folders to sync, the
  apps for Komble. The Plugins step lists every first-party plugin the
  payload carries (dock, window switcher, ewe Safe, Shaun, clipboard,
  screenshots, music, Places, phone, mail, Slack, Cast to TV, …): the ones
  your backup had are
  pre-checked, otherwise none — they are opt-in — and the picked ones are
  installed for the new account at the end of the run, each one best-effort.
- **`ewe-install`** — the same install from a terminal (rescue/headless):
  archinstall handles disks, locale, users and bootloader; the wrapper then
  layers the `[ewe]` repo, the `ewe` package, the greeter stack (greetd →
  cage → Quickshell greeter), the per-user deploy for every created account
  and the plugins (`--addons ewe.dock,ewe.media`, or the prompt). The
  installed machine boots to the graphical greeter with the desktop ready.

## Build

```sh
sudo pacman -S archiso
sudo ./build.sh          # → out/ewe-<VERSION>-x86_64.iso
./run-iso.sh             # boot it in QEMU/KVM (UEFI), with a disk to install onto
```

The released images are built by CI (`.github/workflows/release-iso.yml`),
which also compiles the graphical installer into the image; `build.sh` does
not build it. A test image without a release: `gh workflow run
release-iso.yml --ref <branch> -f test=true` ([docs/TESTING.md](docs/TESTING.md)).
`run-iso.sh` is a front for `scripts/vm.sh`, the test harness — headless with
screenshots and a guest agent, or `--video` for the VM an install is recorded
in ([docs/RECORDING.md](docs/RECORDING.md)).

The profile (`iso/`) derives from archiso's releng v89: its full rescue
toolbox is kept, networkd/iwd are swapped for NetworkManager (what the DE
uses), and greetd + the live user service are enabled on top.

## Repos of the project

| repo | role |
|---|---|
| [ewe](https://github.com/prj786/ewe) | the desktop environment (Hyprland + Quickshell) + `ewe` package |
| [komble-arch](https://github.com/prj786/komble-arch) | Komble — the software manager |
| [ewe-settings](https://github.com/prj786/ewe-settings) | ewe Settings — the settings app |
| [ewe-sync](https://github.com/prj786/ewe-sync) | ewe Sync — your account: your Nextcloud, sync, folders |
| [ewe-browser](https://github.com/prj786/ewe-browser) | ewe Browser — the desktop's browser |
| [ewe-safe](https://github.com/prj786/ewe-safe) | ewe Safe — ewe's password manager (opt-in, with the ewe Safe plugin) |
| [ewe-companion](https://github.com/prj786/ewe-companion) | Shaun — the desktop companion (opt-in, with the Shaun plugin) |
| [ewe-cast](https://github.com/prj786/ewe-cast) | ewe Cast — casting to a TV (the Cast to TV plugin's engine) |
| [ewe-repo](https://github.com/prj786/ewe-repo) | the `[ewe]` pacman repository |
| ewe-os | this — the ISO |
