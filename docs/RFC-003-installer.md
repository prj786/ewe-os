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

### Addendum — the Add-ons step (ewe-os 0.13.0-beta, 2026-10-04)

ewe 0.25 made every first-party extra an **add-on**: shipped inside the
payload (`/usr/share/ewe/plugins/<id>/` + `bundle.json`), installed on
nothing by default, one click away in Komble → Add-ons or the desktop's
Welcome screen. The installer gets the same catalogue as a step of its own,
**between *Your account* and the Summary** — after the account exists (the
add-ons are installed *for* it) and before the one screen that restates
every choice:

- the list is read from the **live system's payload** by the backend
  (`addons` command: `bundle.json` + each manifest's name, description,
  icon, category) — no user config has to exist, and an ISO carrying an
  older ewe (no `bundle.json`) has no such step at all;
- **nothing is pre-checked** (the opt-in rule is the product; the user's
  words: *keep as it is now*); the Dock row carries *Recommended if you like
  a dock*, as on Welcome; the footer says *You can add or remove these any
  time in Komble → Add-ons*;
- the picks are ids in `choices.addons`, shown on the Summary, and run as
  one helper verb, **`addons <id,id,…>`**, after `layer`/`upgrade` and
  before `bootloader`. The verb delegates to `ewe-install --addons-only`
  (one implementation, two faces): for every account on the target,
  `ewe-plugin install <id> --no-restart` **as that user in the chroot**
  (`runuser`, with the Hyprland/Wayland/D-Bus variables dropped — in a
  chroot there is nothing to reload, and `--no-restart` keeps the tool away
  from `systemctl --user`). `ewe-setup` ran just before and, on a fresh
  account, recorded the add-ons as considered without installing any
  (`migrate --fresh`) — so nothing is migrated, only what was picked goes in;
- **an add-on can never fail the OS install**: each one is best-effort, a
  failure is one `{"phase":"addons","addon":id,"ok":false,"msg":…}` line the
  Done screen summarises ("add it later in Komble → Add-ons"), and the verb
  exits 0. Only a malformed id list or an unmounted target is an error.

The TUI face: `ewe-install --addons id,id` (guided or `--layer-only`), an
interactive prompt in guided mode when no `--addons` was given (Enter =
none), and `--addons-only id,id` for the helper.

**The live session** is the one place that is *not* opt-in: a fresh 0.25
account has no dock, and the live stick pins *Install ewe* in the dock. So
`ewe-live-deploy` installs `ewe.dock` — only that, only for the live user —
right after `ewe-setup`; the installer also autostarts and sits first in the
launcher, as before. What an installed system gets is unchanged.

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
