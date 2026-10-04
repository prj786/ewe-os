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
  **dock add-on**, so *Install ewe* has a dock to be pinned in (since ewe
  0.25 a fresh account has no dock; `ewe-live-deploy`). A root rescue
  console lives on tty3 (Ctrl+Alt+F3); tty1 belongs to the greeter.
- **`ewe-installer`** — the graphical installer (Tauri; RFC-003 in
  `docs/`): keyboard, network, time & place, disk, account, **add-ons**,
  summary, install. The Add-ons step lists every first-party extra the
  payload carries (dock, clipboard, screenshots, music, Places, phone,
  mail, Cast to TV, …) with nothing pre-checked — add-ons are opt-in — and
  the picked ones are installed for the new account at the end of the run,
  each one best-effort.
- **`ewe-install`** — the same install from a terminal (rescue/headless):
  archinstall handles disks, locale, users and bootloader; the wrapper then
  layers the `[ewe]` repo, the `ewe` package, the greeter stack (greetd →
  cage → Quickshell greeter), the per-user deploy for every created account
  and the add-ons (`--addons ewe.dock,ewe.media`, or the prompt). The
  installed machine boots to the graphical greeter with the desktop ready.

## Build

```sh
sudo pacman -S archiso
sudo ./build.sh          # → out/ewe-0.1-alpha-x86_64.iso
./run-iso.sh             # boot it in QEMU/KVM (UEFI)
```

The profile (`iso/`) derives from archiso's releng v89: its full rescue
toolbox is kept, networkd/iwd are swapped for NetworkManager (what the DE
uses), and greetd + the live user service are enabled on top.

## Repos of the project

| repo | role |
|---|---|
| [ewe](https://github.com/prj786/ewe) | the desktop environment (Hyprland + Quickshell) + `ewe` package |
| [komble-arch](https://github.com/prj786/komble-arch) | Komble — the software manager |
| [ewe-settings](https://github.com/prj786/ewe-settings) | the Settings app |
| [ewe-repo](https://github.com/prj786/ewe-repo) | the `[ewe]` pacman repository |
| ewe-os | this — the ISO |
