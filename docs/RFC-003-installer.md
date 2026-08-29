# RFC-003 — the ewe installer

*Status: accepted (owner + engineering review, 2026-08-29) · replaces the
archinstall wrapper as the 1.0-beta installer · lives in this repo*

## The decision

**A first-party graphical installer, Tauri (Svelte + Rust), named
`ewe-installer`** — the third first-party app, sharing Komble's and
ewe-settings' stack, design language, and privileged-helper pattern.

Why not the alternatives, concretely:

- **Calamares** — not even in Arch's official repos (AUR), Qt-widgets chrome
  that fights the design language, a plugin architecture built for
  distro-generic needs we don't have. Rejected.
- **TUI** — contradicts the product: the live ISO boots a full graphical
  desktop; dropping the user into a terminal for the most important flow of
  their first session says the polish is skin-deep. A TUI *fallback*
  (`ewe-install`, kept) covers headless/rescue.
- **Keep archinstall wrapper** — archinstall's own UI asks questions we want
  to own (profile? additional packages?) in someone else's voice, and its
  step order can't express our defaults. It remains the plumbing we call for
  nothing — we already do partitioning/pacstrap ourselves in the scripted
  path; the wrapper's value was speed, and its job is done.
- Cost check: webkit2gtk (Tauri's engine) is already on the ISO — Komble
  ships it. Marginal weight of a Tauri installer ≈ the app binary itself.

## The steps (one decision per screen, defaults pre-chosen)

1. **Welcome / language + keyboard** — layout applies live to the test field.
2. **Time & place** — auto-detected (the hardened two-provider rule; shown,
   never silently trusted), editable map-less pickers: region → zone, locale.
   Writes the same `/etc/ewe/manual-timezone` contract the desktop honours.
3. **Disk** — pick a disk. Default: **erase disk** (simple mode). Advanced:
   reuse/keep-home lands post-1.0; dual-boot detection shows what will be
   destroyed, in red, always.
4. **Filesystem** — default **btrfs** with `@` / `@home` subvolumes +
   zstd compression (snapshot-ready for a future rollback story); **ext4**
   offered as "simple". Both paths produce the same fstab contract.
5. **Hibernation** — default ON for laptops (battery present): swapfile
   sized RAM+2G, `nocow` on btrfs, `resume=`/`resume_offset=` wired —
   exactly phase 32's proven logic, ported.
6. **User** — name, username, password, avatar (feeds AccountsService, so
   the greeter shows it on first boot); autologin toggle.
7. **Summary** — every choice restated in plain words; the only screen with
   an Install button.
8. **Install** — streamed progress (pacstrap + the layering we already
   verified: [ewe] repo + key, ewe package, GPU-by-lspci, greeter stack,
   per-user `ewe-conf` seed with the timezone/locale/user choices baked into
   `[system]`).
9. **Done** — reboot.

## Architecture

- Frontend: Svelte, runs as the live user, one step-component per screen.
- Backend: the Rust side executes NOTHING privileged itself — it drives a
  root `ewe-install-helper` (pkexec, fixed-verb allowlist, argv-only: the
  Komble helper pattern) whose verbs are the scripted primitives we already
  field-tested: `partition`, `mkfs`, `mount`, `pacstrap`, `layer`, `user`,
  `hibernate`, `bootloader`. Every verb streams JSON progress lines.
- The TUI path (`ewe-install --layer-only` and friends) calls the same
  helper verbs — one implementation, two faces.
- `ewe.conf` is born in the installer: step choices land in `[system]` and
  `[desktop]`, so the first boot is already described by the one file.

## Relation to Dolly

1.0-beta ships with `ewe-installer` as the default path on the live ISO
("Install ewe" desktop entry points at it), `ewe-install` as the rescue/
headless path. Real-hardware verification of BOTH is a Dolly gate.

## Time & Place, post-install

The same step-2 UI ships as a **Time & Place pane in ewe-settings**
(auto/manual toggle backed by `/etc/ewe/manual-timezone`, zone + locale
pickers, NTP status). Future upgrade for auto mode: geoclue + WiFi-beacon
positioning with a `zone.tab` nearest-zone lookup — immune to the
roaming-IP lies that moved a real machine to London.
