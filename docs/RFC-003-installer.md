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

## The product rule (owner decision, 2026-08-29): ewe DECIDES the stack

The simplest distro asks no technical questions. Users answer only what is
personal — keyboard, place, which disk, who they are. Everything else is a
DECISION, not an option, chosen for speed and stability first:

| decided | answer |
|---|---|
| filesystem | btrfs — @/@home subvolumes, zstd (snapshot-ready) |
| bootloader | systemd-boot |
| hibernation | automatic: battery present → on (swapfile RAM+2G) |
| kernel | linux (stock) |
| GPU drivers | matched to hardware via lspci |
| audio / network / login | pipewire / NetworkManager / greetd |
| desktop | ewe |

Free, but premium: no headaches, no menu of ways to hold it wrong. The
Summary shows the decided stack in a read-only "Decided by ewe" panel so
the choices are honest, just not negotiable. (Power users have arch — and
the TUI rescue path.)

## The steps (six screens, all personal)

1. **Welcome** — keyboard layout, live test field.
2. **Time & place** — auto-detected (two-provider agreement; shown, never
   silently trusted), zone + locale pickers. Writes the desktop's
   `/etc/ewe/manual-timezone` contract.
3. **Disk** — which disk (preselected when there is only one); whole-disk,
   the erase warning in red here and again on the Summary.
4. **Your account** — name, username, password, computer name.
5. **Summary** — personal choices restated + the read-only decided stack;
   the only screen with an (explicitly red) Install button.
6. **Install** — streamed progress, then Reboot.

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
