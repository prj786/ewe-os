# Roadmap — ewe OS

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

## 0.3-alpha — one file

The machine becomes one declarative document: `~/.config/ewe/ewe.conf`
(see [RFC-001](https://github.com/prj786/ewe/blob/main/docs/RFC-001-one-config.md)).

- [ ] `ewe-conf` tool + schema + import/apply (RFC Phase 1)
- [ ] in-shell Settings writes through `ewe-conf` (Phase 2)
- [ ] ewe-settings writes through `ewe-conf` (Phase 3)
- [ ] generators consolidated into `ewe-conf` (Phase 4)
- [ ] `[system]` split: installer records profiles, gaming/dev lists move here (Phase 5)
- [ ] wallpapers + default look on the live session

## 0.4-alpha — connected

- [ ] `ewe-auth` token broker: one Google OAuth for shell + komble + future apps (RFC Phase 6)
- [ ] `ewe.conf` Drive sync — log in, get your machine back
- [ ] `[apps.installed]` manifest: komble writes it, restore reinstalls it
- [ ] repo + package signing (drop `SigLevel = Optional TrustAll`)
- [ ] ISO built and released by CI (no more laptop-bandwidth uploads)

## 1.0-beta — "Dolly"

Named for the sheep that proved you can clone the whole animal from one cell.

- install + upgrade verified on real hardware (not just QEMU)
- installer UX decision resolved and shipped
- restore-from-file demo: fresh install → sign in → your desktop and apps return
- docs: install guide, troubleshooting, project site
