#!/usr/bin/env bash
# The helper's pure parts, without root and without a target disk: the
# argument checks every verb relies on, and the console keymap that
# `setkeymap` picks from the keyboards (against a fake target tree).
#
#   installer/tests/helper-test.sh
set -uo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
ok()  { printf 'ok   %s\n' "$*"; }
bad() { printf 'FAIL %s\n' "$*"; fail=1; }
yes() { if "$@"; then ok "$*"; else bad "$* (should pass)"; fi; }
no()  { if "$@"; then bad "$* (should be refused)"; else ok "refused: $*"; fi; }

# shellcheck source=/dev/null  # the helper under test, sourced for its functions
EWE_HELPER_LIB=1 . "$here/helper/ewe-install-helper"
set +e

yes valid_locale en_US.UTF-8
yes valid_locale sr_RS.UTF-8@latin
yes valid_locale aa_ER
no  valid_locale 'en_US.UTF-8;rm'
no  valid_locale 'x y'

yes valid_layouts us ""
yes valid_layouts us,ge ""
yes valid_layouts us,ge ,qwerty
yes valid_layouts de nodeadkeys
yes valid_layouts us,in ,hin-wx
no  valid_layouts "" ""
no  valid_layouts us,ge x            # two layouts, one variant
no  valid_layouts 'us"' ""
no  valid_layouts us,a,b,c,d ""      # at most four
no  valid_layouts us 'a" }) os.execute("x'

yes valid_lang_pkg hunspell-de
yes valid_lang_pkg fcitx5
yes valid_lang_pkg fcitx5-mozc
yes valid_lang_pkg fcitx5-chinese-addons
no  valid_lang_pkg firefox
no  valid_lang_pkg 'fcitx5;rm'
no  valid_lang_pkg hunspell-

yes valid_pkg vlc
yes valid_pkg telegram-desktop
yes valid_pkg 0ad
yes valid_pkg libreoffice-fresh
no  valid_pkg -rf
no  valid_pkg 'a b'

# a fake target: systemd's map + a few kbd keymaps
MNT="$(mktemp -d)"
trap 'rm -rf "$MNT"' EXIT
install -d "$MNT/usr/share/systemd" "$MNT/usr/share/kbd/keymaps/i386/qwerty" "$MNT/usr/share/kbd/keymaps/i386/qwertz"
cat > "$MNT/usr/share/systemd/kbd-model-map" <<'MAP'
# consolelayout		xlayout	xmodel		xvariant	xoptions
us			us	pc105+inet	-		terminate:ctrl_alt_bksp
de			de	pc105		-		terminate:ctrl_alt_bksp
de-latin1-nodeadkeys	de	pc105		nodeadkeys	terminate:ctrl_alt_bksp
jp106			jp	jp106		-		terminate:ctrl_alt_bksp
ge			ge,us	pc105		-		terminate:ctrl_alt_bksp
MAP
for k in us jp106 ge; do : | gzip > "$MNT/usr/share/kbd/keymaps/i386/qwerty/$k.map.gz"; done
for k in de de-latin1-nodeadkeys; do : | gzip > "$MNT/usr/share/kbd/keymaps/i386/qwertz/$k.map.gz"; done

check() { if [ "$1" = "$2" ]; then ok "$3"; else bad "$3 — got '$1', wanted '$2'"; fi; }
expect() { check "$(console_keymap "$1" "$2")" "$3" "console keymap for '$1' '$2'"; }
expect us "" us
expect de "" de
expect de nodeadkeys de-latin1-nodeadkeys
expect jp "" jp106
expect us,ge "" us
expect ge,us "" us          # Georgian first: the console still starts Latin
expect ge "" us             # nothing Latin at all: US English
expect fr "" us             # no keymap on this target: US English
expect ru,de "" de

# the greeter wrapper's reading of localectl's X11 file: the block
# ewe-install's setup_greeter splices into a wrapper that has none (an ewe
# payload up to 0.50.1; later ones carry their own, tested in the ewe repo)
snip="$(sed -n "/^[[:space:]]*greeter_kb=\"\$(cat <<'KB'/,/^KB\$/p" "$here/../iso/airootfs/usr/local/bin/ewe-install" | sed '1d;$d')"
if [ -n "$snip" ]; then ok "found the keyboard block in ewe-install"; else bad "the keyboard block is not where this test looks for it"; fi
conf="$MNT/etc/X11/xorg.conf.d/00-keyboard.conf"
install -d "$(dirname "$conf")"
printf 'Section "InputClass"\n        Identifier "system-keyboard"\n        Option "XkbLayout" "us,ge"\n        Option "XkbVariant" ",qwerty"\nEndSection\n' > "$conf"
got="$(env -i sh -c "$(printf '%s\n' "$snip" | sed "s|/etc/X11/xorg.conf.d/00-keyboard.conf|$conf|"); echo \"\$XKB_DEFAULT_LAYOUT|\$XKB_DEFAULT_VARIANT|\$XKB_DEFAULT_OPTIONS\"")"
check "$got" "us,ge|,qwerty|grp:win_space_toggle" "greeter keyboard, two layouts"
printf 'Section "InputClass"\n        Option "XkbLayout" "de"\nEndSection\n' > "$conf"
got="$(env -i sh -c "$(printf '%s\n' "$snip" | sed "s|/etc/X11/xorg.conf.d/00-keyboard.conf|$conf|"); echo \"\$XKB_DEFAULT_LAYOUT|\$XKB_DEFAULT_VARIANT|\$XKB_DEFAULT_OPTIONS\"")"
check "$got" "de||" "greeter keyboard, one layout"
rm -f "$conf"
got="$(env -i sh -c "$(printf '%s\n' "$snip" | sed "s|/etc/X11/xorg.conf.d/00-keyboard.conf|$conf|"); echo \"\$XKB_DEFAULT_LAYOUT|\"")"
check "$got" "|" "greeter keyboard without a file: xkb's default"

if [ "$fail" = 0 ]; then echo "all passed"; else echo "FAILED"; exit 1; fi
