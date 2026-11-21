# Release checklist

Every ISO passes ALL of this before it ships. Run it after `./build.sh`
(remember `sudo ./build.sh clean` first after profile edits — mkarchiso
silently reuses a stale work dir).

## Boot matrix

| | how | pass = |
|---|---|---|
| UEFI | `./run-iso.sh` (OVMF) | boot menu → live desktop |
| BIOS | `./run-iso.sh --bios` | same |

Headless (no window, screenshots over QMP): `scripts/vm.sh` — boots the
newest `testing/*.iso`/`out/*.iso` with a QMP socket, a qemu-guest-agent
channel (root shell into the live system: `vm.sh ga <cmd…>`), and a
persistent 40G blank target disk. `vm.sh shot` screendumps to PNG,
`vm.sh key`/`type` synthesise input, `vm.sh reboot-disk` boots the
installed disk for the post-install verification pass. It uses
`-device virtio-vga`, **not** `virtio-gpu-pci` — gpu-pci's screendump
reads solid black even while the guest renders. `EWE_VM_DIR` moves its
work folder (sockets, screenshots, the 40G disk) off a tmpfs `/tmp` — an
install there fills RAM; `EWE_VM_GA_TIMEOUT` lengthens `ga` for slow
commands (pacman).

**A test ISO without a release:** `gh workflow run release-iso.yml --ref
<branch> -f test=true` builds the image from that branch in CI and keeps it
as the run's artifact (`gh run download <run> -n ewe-test-iso -D testing/`)
— nothing is uploaded to the Archive and no release is touched.

## Live session

- [ ] boot reaches the desktop with no interaction (autologin, first-boot
      deploy runs; a short plymouth-covered pause is OK, black screen is not)
- [ ] bar, dock, launcher (Super+D), Quick Settings all open — the dock is
      the `ewe.dock` plugin, installed for the live user by
      `ewe-live-deploy` (a fresh account has none since ewe 0.25); "Install
      ewe" sits first in it and the installer opened by itself
- [ ] folder/app icons render (Reversal) — pink/black tiles = theme missing
- [ ] network up (NetworkManager); Komble opens and loads the AM catalog
- [ ] `sudo pacman -Syu` resolves against `[ewe]` without errors
- [ ] leave idle 12+ min: session must NOT lock
- [ ] tty3 root rescue console reachable (Ctrl+Alt+F3; tty1 stays blank —
      it belongs to the plymouth→greeter handoff)

## Language, keyboard, apps (0.17)

- [ ] the installer opens on **Language**: languages by their own name and
      the English one; every native name draws (no boxes: Georgian,
      Japanese, Arabic joined, Hindi); search finds "georg", "日本",
      "brasil" (→ Portuguese, Brazil)
- [ ] Georgian: Region Georgia; Spell checking says plainly that Arch has
      no Georgian dictionary; no Typing row. Keyboard screen: English (US)
      first, Georgian second; *Try it* types Latin, Super+Space Georgian
- [ ] Japanese: Typing ticked (Mozc); Keyboard screen: Japanese (`jp`)
- [ ] German (Austria): Spell checking ticked (hunspell-de); keyboard
      German (Austria)
- [ ] Time & place shows Formats = the language's region, with an example
- [ ] Apps: tiles by category; "Komble" marks AUR/AppImage ones; the
      Summary lists language, keyboards, formats, spelling, typing, apps
- [ ] plain English + US keyboard: the install log has no `language` step

## Install path

- [ ] QEMU with a blank virtio disk: run "Install ewe" (or `ewe-install`),
      minimal archinstall config, one user account
- [ ] the Plugins step lists the payload's 16 plugins, nothing pre-checked;
      pick 2–3 (say the dock, clipboard, music); the Summary names them
- [ ] installer completes; target has `[ewe]` in pacman.conf, greetd enabled,
      wayland session entry, user's home deployed (`~/.local/share/ewe`)
- [ ] on the installed disk: `ewe-plugin list` shows exactly the picked
      plugins (installed, enabled, source bundled), nothing else;
      `~/.local/state/ewe/addons-migrated` names all 16 as considered
      (nothing migrated); the dock is there only if it was picked
- [ ] `ewe-install --help` prints the plugin options; `ewe-install --addons
      ewe.dock` from tty3 works the same way (rescue path)
- [ ] for the checks below, give the installed system a guest agent
      BEFORE pressing Reboot (the target is still mounted):
      `vm.sh ga arch-chroot /mnt/ewe-install pacman -S --noconfirm
      qemu-guest-agent`, then `vm.sh down && vm.sh reboot-disk`
- [ ] reboot from disk: greeter lists "Ewe", login lands in the desktop
- [ ] the login screen starts on the first keyboard (the password typed in
      the installer works); with two, Super+Space switches
- [ ] `localectl`: System Locale LANG + LC_TIME… = the formats; X11 Layout =
      the keyboards; VC Keymap = the first Latin one. `/etc/locale.conf`
      says the same; `locale -a` has the locale, the formats' and en_US
- [ ] with an input method: `pacman -Q fcitx5 fcitx5-<engine>`; as the
      user `ewe-conf get desktop.ime` = `{"enabled": true, "engines": […]}`;
      fcitx5 runs after sign-in, Ctrl+Space types the language (the bar's
      layout indicator shows its mark)
- [ ] with a dictionary: `pacman -Q hunspell-<x>`; `ewe-conf get
      desktop.language.spellcheck` = `["<x>", "en_US"]`
- [ ] the desktop's keyboard (`ewe-conf get desktop.input.kb_layout`) =
      the Keyboard screen's list
- [ ] apps: repo picks `pacman -Q`'d; AUR/AppImage picks in
      `~/.local/state/ewe/install-picks.json`, and Komble opens on For you
      → *Apps you picked while installing ewe* with Review…/Install
- [ ] on the installed system: `sudo pacman -Syu` works; lock screen unlocks
      with the user's password
- [ ] target is fully updated: `pacman -Qu` is empty at first boot
- [ ] first login: `journalctl -b | grep gkr-pam` shows the keyring
      unlocked/created, no "invalid"; `~/.local/share/keyrings/login.keyring`
      exists
- [ ] Welcome → Sign in to Nextcloud completes against the test mock
      (`tests/mock-nextcloud.py` in the ewe repo, `EWE_CLOUD_SERVER_INSECURE=1`)
      or a real account: the app password lands in the keyring with no
      keyring prompt, `ewe-cloud status` says signed in, `ewe-conf push`
      lands `ewe/ewe.conf` in the account
- [ ] Settings → Account: the Google card shows **Not configured** (no
      client file) — the package must ship none

## Updates

- [ ] on the previous release's installed system, `pacman -Syu` alone brings
      in the new ewe payload and the session self-refreshes at next login
