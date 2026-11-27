# Installing ewe

ewe installs from one ISO in about ten minutes and asks only what is
personal: your language, your keyboard, your place, which disk, who you are. Everything
technical is decided for you (see *Decided by ewe* below).

## 1 · Get the ISO

<https://prj786.github.io/download/> says where the current image is. Today
that is the [GitHub release](https://github.com/prj786/ewe-os/releases):
GitHub caps a release file at 2 GiB and the image is larger, so it is split
— download every `.part` and `SHA256SUMS` into one folder, join the parts and
check them:

```sh
cat ewe-*-x86_64.iso.*.part > ewe-x86_64.iso
sha256sum -c SHA256SUMS --ignore-missing
```

(When the download page offers the image in one piece, there is nothing to
join: check it with the same `sha256sum` line.)

Write it to a USB stick (any tool works — it is a plain hybrid ISO):

```sh
sudo dd if=ewe-x86_64.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Replace `/dev/sdX` with the stick (`lsblk` shows it — pick the disk, not a
partition). Ventoy, balenaEtcher and GNOME Disks work too.

## 2 · Boot it

Boot the stick in **UEFI** mode (legacy BIOS boot is supported as a
fallback). If your firmware has Secure Boot on, turn it off — the ISO is
not signed.

You land straight on the ewe desktop, running from the stick, with
**Install ewe** pinned first in the dock and the launcher (and it opens by
itself). Everything you see is the real thing: try the Control Center
(Super + N), Komble, the settings. One difference from an installed ewe: the
live stick has the Dock plugin installed so *Install ewe* has a place to
sit — a fresh install starts without it unless you pick it on the Plugins
screen. Nothing is written to your disks until you press the red Install
button on the Summary screen.

## 3 · Install — eleven screens

1. **Language** — the first screen: every language glibc knows, by its own
   name and the English one (*ქართული — Georgian*, *日本語 — Japanese*), with
   a search box that finds a language by either name or by a country
   (*Brasil*). Picking one fills in the rest, and each part stays
   changeable:
   - **Region** — when the language has several (Spanish in Spain or
     Mexico …); that is the system locale (`es_MX.UTF-8`). US English is
     always set up next to it as the fallback.
   - **Spell checking** — the language's dictionary when Arch has one
     (ticked); when it does not, the screen says so plainly and English is
     checked (ewe Settings → Language can add one later, from the AUR when
     that is where it lives).
   - **Typing** — Chinese, Japanese, Korean, Vietnamese and the Indic
     languages get an input method (Pinyin, Mozc, Hangul, Unikey, InScript
     …), ticked; Ctrl+Space turns it on and off on the new computer.
   ewe's own screens stay in English for now; apps and the system use your
   language.
2. **Keyboard** — filled in from the language: the keyboard named after
   your country (German in Austria gets the Austrian one); a language whose
   keyboard cannot type the letters a–z (Georgian, Russian, Greek …) gets
   US English first and its own second, so your username and password type
   the same everywhere. Reorder, add or remove; the first is the one the
   computer starts with, at the login screen too, and Super+Space switches.
   The choice applies to the live session at once, so *Try it* tests the
   real thing.
3. **Network** — the install downloads the whole system, so this screen
   waits for a connection: a cable just works, Wi-Fi is joined right here
   (pick the network, type the password). Nothing past this screen opens
   until NetworkManager reports full connectivity.
4. **Time & place** — your zone is detected (two providers must agree;
   you always see what was chosen), and the **formats** — how dates, times,
   numbers and money look, filled in from the language (an example shows
   them) and changeable to any region's.
5. **Disk** — pick the disk. **The whole disk is erased**; the warning is
   red here and again on the Summary.
6. **Your login** — name, username, password, computer name.
7. **Your ewe account** — sign in to your own Nextcloud, or press Next to
   skip. Type your server's address, your username and your password: the
   password goes to your server once and is traded for an app password of
   this computer's own (revoke it any time in Nextcloud → Settings →
   Security); it is never kept. An account with two-step verification or
   single sign-on uses *Sign in with the browser instead*, which opens your
   server's own page. If the account holds a backup from
   another ewe computer, the screen says what comes back — which computer,
   when, your folders, how many apps, which plugins — and **Bring all of it
   to this computer** is ticked. The new computer then starts signed in
   (the app password goes straight into your login keyring, unlocked by
   your login password), with your settings and look, your folders coming
   down by themselves, your browser bringing back bookmarks and tabs, and
   your apps waiting in Komble. (Bringing a backup back also brings that
   computer's keyboard, typing and spelling settings, over this screen's.)
8. **Plugins** — the extras that ship with ewe but are not installed
   until you ask: the dock, the window switcher (Alt+Tab), Safe (ewe's
   password manager), Shaun (the desktop companion), clipboard history,
   screenshots, passwords, music, Places, your phone, mail, Slack, Cast to
   TV, Insomnia, system monitor, SSH, VPN. Signed in with a backup, the
   plugins that computer had are
   already ticked (*From your account*); otherwise nothing is pre-checked.
   Tick or untick anything (the Dock row says *Recommended if you like a
   dock*). You can add or remove these any time in **Komble → Plugins**, so
   an empty pick is a fine answer.
9. **Apps** — optional: Komble's popular apps as tiles (browsers, chat,
   music, video, office, games …). The ones from Arch's repositories
   install with the system. The ones marked **Komble** come from the AUR or
   as AppImages: the installer never builds those — they wait in Komble
   after your first sign-in (*For you → Apps you picked while installing
   ewe*), one click each, and an AUR one still shows its build recipe for
   review first. Picking Steam turns on Arch's 32-bit packages
   (`[multilib]`) with the right 32-bit graphics driver.
10. **Summary** — everything you chose (language, keyboards, formats,
    spelling, typing, your ewe account, plugins and apps included), the
    *Decided by ewe* panel, and the only Install button.
11. **Install** — a last connectivity check, then streamed progress (the
    log follows its last line; scroll up to read, back down to follow again):
    the base system, your language, formats and keyboards, the desktop,
    then **updating the installed system** so the first boot owes no
    updates, your typing and spelling, your apps, your ewe account (and the
    restore), your plugins, then **Reboot**.
    Remove the stick when the screen goes dark. If a step fails, the screen
    says why and offers **Retry** (the disk is partitioned again from
    scratch) or **Back**. A plugin, an app, the input method or a dictionary
    that could not be installed never fails the install: the Done screen
    names it and says where to get it later (Komble, ewe Settings).

The installed machine boots through the ewe splash into the graphical
greeter — ewe's own: Ewe Dark, Geist, your keyboards. Signed in at install,
the first login is already your desktop
from your account; Welcome skips what is done. Otherwise the first login
opens the **Welcome** flow: connect to the
internet (Wi-Fi is joined from the flow itself), install any updates that
are waiting, **sign in to your Nextcloud** (the card with the Nextcloud
logo — any provider, or a server you run; ewe opens your server's own login
page and never sees your password), restore a backup if your account
holds one from another ewe machine, and a sixty-second tour. Google is not
part of the flow: it is optional, for Gmail notifications and a Drive
folder, and needs your own OAuth client file (Settings → Account).

### Decided by ewe

| | |
|---|---|
| filesystem | btrfs, `@` / `@home` subvolumes, zstd compression (with `btrfs-progs`, so snapshots and scrub are there) |
| bootloader | systemd-boot, splash-only boot (no menu unless you hold a key) |
| hibernation | automatic when a battery is present: swapfile of RAM + 2 GB |
| kernel | `linux` (stock Arch) + your CPU's microcode |
| graphics | drivers matched to the GPU(s) found (Intel / AMD / NVIDIA) |
| sound / network / login | PipeWire / NetworkManager / greetd |
| updates | the `[ewe]` repository is preconfigured; `pacman -Syu` (or Komble → Updates) rolls everything forward |

## 4 · After the install

- **Updates**: Komble → Updates, or `sudo pacman -Syu`. The top-bar download
  glyph tells you when something is pending.
- **Your account**: **ewe-sync** if you skipped it in the installer and
  the Welcome flow — its Overview shows everything your account keeps
  (settings & apps, browser, passwords, folders) and *Bring everything
  back* on a computer that has a backup waiting.
- **Google (optional)**: Settings → Account → the Google card, only after
  you drop your own OAuth client at `~/.config/ewe/oauth-client.json`; it
  adds Gmail notifications and a Drive folder, nothing else.
- **Settings**: Super + , — or the gear in the Control Center.
- **Language later**: ewe Settings → Language (the language, spell
  checking) and Time and place → Formats; typing in other languages is in
  ewe Settings → Keyboard and mouse.

## Headless / rescue install

The graphical installer drives a root helper with fixed verbs
(`partition`, `mkfs`, `pacstrap`, `settz`, `setlocale`, `setkeymap`,
`sethostname`, `user`, `layer`, `upgrade`, `language`, `apps`, `account`,
`addons`, `hibernate`, `bootloader`). The same primitives are reachable from a terminal on the
live session as `ewe-install` (see `ewe-install --help`), which is also
the path for scripted or headless installs and for re-layering ewe onto an
existing Arch system (`ewe-install --layer-only`).

## Testing the ISO in a VM

QEMU/KVM with UEFI is the reference environment; the repo ships a harness:

```sh
scripts/vm.sh up --gui testing/ewe-x86_64.iso   # boot with a window + sound
scripts/vm.sh down
scripts/vm.sh reboot-disk --gui                  # boot the installed disk
```

The harness gives the VM a blank 40 GB disk and *writable* UEFI variables —
without those, systemd-boot's entry vanishes between boots and the firmware
drops into its device menu. See `docs/TESTING.md` for the release
checklist and `docs/TROUBLESHOOTING.md` when something looks wrong.
