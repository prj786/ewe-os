# Roadmap

The distro's version line (the DE keeps its own 0.8.x line). Each release has
explicit criteria; an ISO ships only after the full checklist in
[docs/TESTING.md](docs/TESTING.md) passes. Work is tracked as GitHub issues on
this repo, one milestone per release.

## 0.1-alpha — proof of life ✅ (2026-08-27)

- [x] archiso profile (releng-derived) with the [ewe] repo baked in
- [x] `ewe` pulls the whole DE as a pacman dependency tree
- [x] live boot → autologin → first-boot deploy → full desktop (QEMU-verified)
- [x] `ewe-install` written (archinstall + ewe layering) — **untested**

## 0.2-alpha — looks like ewe, installs ewe ✅ (2026-08-27)

- [x] icon/cursor themes as packages (`reversal-icon-theme`, `mocu-cursor-theme`
      via ewe-repo `pkgbuilds/`) — fixes the pink/black icons on the live ISO
- [x] branded boot: menu titles + syslinux art, plymouth `ewe` splash in the
      ISO initramfs, splash → greetd handoff (no black gap at first boot)
- [x] live session never idle-locks (empty-password user can't unlock)
- [x] "Install ewe" entry in the live launcher/dock
- [x] `ewe-install` verified end to end in QEMU (disk, UEFI): installed system
      boots to the greeter with the desktop deployed
- [x] BIOS boot path verified (only UEFI tested so far)

## 0.3-alpha — trust and polish

- [ ] repo + package signing (drop `SigLevel = Optional TrustAll`)
- [ ] ISO built by CI, checksummed, published as a GitHub release
- [ ] installer UX decision: keep the archinstall wrapper vs. GUI
      (Calamares or an ewe-settings-style Tauri app)
- [ ] wallpapers + default look on the live session

## beta — when someone else can run it

- install verified on real hardware (not just QEMU)
- upgrade path proven across at least two releases (pacman -Syu only)
- docs: install guide, troubleshooting, project site
