# ewe OS

The distro layer of [ewe](https://github.com/prj786/ewe): the archiso profile
that builds the **live/install ISO**. The DE, apps, and packaging live in
their own repos — this one turns them into a bootable distribution.

**Version: 0.1-alpha.** The distro has its own version line; the DE stays on
its 0.8.x line. The ISO pins nothing — it preconfigures the
[\[ewe\] pacman repo](https://github.com/prj786/ewe-repo) so both the live
session and installed systems roll forward with plain `pacman -Syu`.

## What the ISO does

- **Live session** — boots through greetd straight into a full ewe desktop
  (autologin as the `ewe` live user; the DE deploys itself on first start via
  `ewe-setup` from the preinstalled `ewe` package). A root rescue console
  stays on tty1.
- **`ewe-install`** — guided disk install: archinstall handles disks, locale,
  users and bootloader; the wrapper then layers the `[ewe]` repo, the `ewe`
  package, the greeter stack (greetd → cage → Quickshell greeter), and the
  per-user deploy for every created account. The installed machine boots to
  the graphical greeter with the desktop ready.

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
