#!/usr/bin/env bash
# The login screen ewe-install lays down (setup_greeter), run against a real
# ewe payload — without an ISO, a VM or root on this machine.
#
#   installer/tests/greeter-test.sh <payload> [<payload> …]
#
# <payload> is what the target's /usr/share/ewe will hold: an `ewe` package
# file (ewe-*.pkg.tar.zst, e.g. the one `[ewe]` serves) or a directory laid
# out like it (an ewe checkout works: system/, bin/, dotfiles/ …). Give it
# both the published package and the next one: the ISO and the ewe release
# can land in either order, and the login screen has to be right with each.
#
# setup_greeter writes to /etc, /usr/local and /usr/share/fonts, so each
# payload gets its own throwaway Arch container (podman, rootless, no
# network): the container's / is "the target", chroot_root runs in place, a
# fake systemctl writes down what would be enabled. Then the installed wrapper
# is RUN there with a fake cage and a two-keyboard X11 keymap — the login
# keyboards are checked by what cage would get, whichever wrapper it is.
#
# Skips (exit 0) without podman or the image; never pulls.
set -u
cd "$(dirname "$0")/../.." || exit 1
IMAGE="${EWE_TEST_IMAGE:-docker.io/library/archlinux:base-devel}"   # has python3

[ $# -ge 1 ] || { sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
command -v podman >/dev/null 2>&1 || { echo "skip: needs podman"; exit 0; }
podman image exists "$IMAGE" 2>/dev/null \
    || { echo "skip: needs the image $IMAGE (podman pull $IMAGE)"; exit 0; }

bash -n iso/airootfs/usr/local/bin/ewe-install || { echo "FAIL  ewe-install does not parse"; exit 1; }

fail=0
for payload in "$@"; do
    echo "== $payload"
    if [ -d "$payload" ]; then
        mount=(-v "$(realpath "$payload"):/payload:ro"); kind=dir
    elif [ -f "$payload" ]; then
        mount=(-v "$(realpath "$payload"):/payload.pkg.tar.zst:ro"); kind=pkg
    else
        echo "FAIL  no such payload: $payload"; fail=1; continue
    fi
    podman run --rm --network none "${mount[@]}" -e KIND="$kind" \
        -v "$PWD/iso/airootfs/usr/local/bin/ewe-install:/ewe-install:ro" \
        "$IMAGE" bash -c '
set -u
fail=0
ok() { echo "ok    $*"; }
no() { echo "FAIL  $*"; fail=1; }

# ── the target: this container. Its /usr/share/ewe is the payload ────────────
mkdir -p /usr/share/ewe
if [ "$KIND" = pkg ]; then
    bsdtar -xf /payload.pkg.tar.zst -C / usr/share/ewe || { echo "FAIL  not an ewe package"; exit 1; }
else
    for d in system bin dotfiles; do cp -a "/payload/$d" /usr/share/ewe/ || { echo "FAIL  the payload has no $d/"; exit 1; }; done
    cp /payload/VERSION /usr/share/ewe/ 2>/dev/null
fi
P=/usr/share/ewe
echo "      payload: ewe $(cat $P/VERSION 2>/dev/null || echo "?")"
SHIPS_KB=0; grep -q XKB_DEFAULT_LAYOUT $P/system/greeter/ewe-greeter 2>/dev/null && SHIPS_KB=1
# what a real target has and this image does not
mkdir -p /usr/lib/security /fake; : > /usr/lib/security/pam_gnome_keyring.so
printf "#!/bin/sh\necho \"systemctl \$*\" >> /fake/systemctl.log\n" > /fake/systemctl
printf "#!/bin/sh\necho \"fc-cache \$*\" >> /fake/fc-cache.log\n" > /fake/fc-cache
printf "#!/bin/sh\nenv | sort > /fake/cage.env\necho \"cage \$*\" > /fake/cage.args\n" > /fake/cage
printf "#!/bin/sh\nexit 0\n" > /fake/systemd-detect-virt   # a VM: this is where the ISO is smoke-tested
chmod +x /fake/*
export PATH="/fake:$PATH"

# ── run it the way ewe-install does, with the chroot seam pointed here ──────
export EWE_INSTALL_MNT=/ EWE_INSTALL_LIB=1
. /ewe-install
chroot_root() { "$@"; }
ok() { echo "ok    $*"; }      # ewe-install has its own ok(): take ours back
out="$(setup_greeter 2>&1)"; rc=$?
[ "$rc" = 0 ] && ok "setup_greeter succeeds" || no "setup_greeter failed ($rc): $out"
case "$out" in *"!!"*) no "it warned: $out" ;; *) ok "…with no warning" ;; esac

G=/etc/xdg/quickshell/ewe-greeter
cmp -s /etc/greetd/config.toml $P/system/greetd/config.toml && cmp -s /etc/greetd/regreet.toml $P/system/greetd/regreet.toml \
    && ok "greetd config.toml + regreet.toml are the payload files" || no "greetd config differs"
cmp -s $G/shell.qml $P/system/greeter/shell.qml && ok "the greeter QML is the payload file" || no "shell.qml differs"

# colours: the roles shell.qml reads, as JSON, world-readable (the greeter user reads it)
if python3 - $G/theme-tokens.json <<PY
import json, sys
t = json.load(open(sys.argv[1]))
need = ["surface-base", "text-primary", "text-muted", "surface-sunken", "accent-text", "border-strong"]
missing = [k for k in need if k not in t.get("color", {})]
assert not missing, missing
assert t["color"]["surface-base"].lower() not in ("black", "#000000"), t["color"]["surface-base"]
PY
then ok "theme-tokens.json holds Ewe Dark (surface-base $(python3 -c "import json;print(json.load(open(\"$G/theme-tokens.json\"))[\"color\"][\"surface-base\"])"))"
else no "theme-tokens.json is missing or not the tokens shell.qml reads"; fi
[ "$(stat -c %a $G/theme-tokens.json 2>/dev/null)" = 644 ] && [ ! -e $G/theme-tokens.json.tmp ] \
    && ok "…mode 644, no .tmp left behind" || no "theme-tokens.json mode/leftover: $(ls -la $G)"

# type: Geist + Geist Mono + the licence, and the Georgian fallback rule
n=0; for f in $P/dotfiles/quickshell/fonts/geist/*.woff2 $P/dotfiles/quickshell/fonts/geist/GEIST-LICENSE; do
    cmp -s "$f" "/usr/share/fonts/ewe/${f##*/}" && n=$((n + 1)) || no "font not installed: ${f##*/}"
done
[ "$n" -ge 5 ] && ok "Geist, Geist Mono and their licence are in /usr/share/fonts/ewe ($n files)" || no "only $n font files"
cmp -s /etc/fonts/conf.d/60-ewe-geist.conf $P/system/fontconfig/60-ewe-geist.conf \
    && ok "60-ewe-geist.conf is in /etc/fonts/conf.d" || no "60-ewe-geist.conf missing"
grep -qx "fc-cache -f /usr/share/fonts/ewe" /fake/fc-cache.log 2>/dev/null && ok "the font cache is refreshed" || no "fc-cache was not run"

# the wrapper
W=/usr/local/bin/ewe-greeter
[ -x $W ] && sh -n $W && ok "the wrapper is installed, executable, and parses" || no "wrapper: $(ls -la $W 2>&1)"
if [ "$SHIPS_KB" = 1 ]; then
    cmp -s $W $P/system/greeter/ewe-greeter \
        && ok "this payload carries the keyboards itself: the wrapper is its file, byte for byte" \
        || no "the payload wrapper was changed: $(diff $P/system/greeter/ewe-greeter $W | head -5)"
else
    grep -q "^printf .\\\\033\[H" $W && grep -q "greeter.log" $W \
        && ok "an older payload: its wrapper (VT wipe, log file)…" || no "not the payload wrapper: $(head -3 $W)"
    [ "$(grep -c XKB_DEFAULT_LAYOUT $W)" -ge 1 ] && [ "$(tail -n 1 $W | cut -c1-9)" = "exec cage" ] \
        && ok "…with the keyboard block spliced in before exec cage" || no "the splice is missing: $(tail -4 $W)"
fi
grep -q "^exec cage -s -- sh -c \"qs -c ewe-greeter || regreet\"" $W && ok "it ends in cage → the greeter, ReGreet as fallback" || no "exec line: $(tail -1 $W)"

# run it: what cage gets when the installer chose two keyboards
mkdir -p /etc/X11/xorg.conf.d
printf "Section \"InputClass\"\n        Identifier \"system-keyboard\"\n        MatchIsKeyboard \"on\"\n        Option \"XkbLayout\" \"us,ge\"\nEndSection\n" > /etc/X11/xorg.conf.d/00-keyboard.conf
vt="$(env -i PATH="/fake:/usr/bin" XDG_CACHE_HOME=/tmp/greeter-cache sh $W 2>&1)"
grep -qx "XKB_DEFAULT_LAYOUT=us,ge" /fake/cage.env 2>/dev/null && grep -qx "XKB_DEFAULT_OPTIONS=grp:win_space_toggle" /fake/cage.env \
    && ok "run with a us,ge keymap: cage starts on us, Super+Space switches" || no "XKB env: $(grep XKB /fake/cage.env 2>&1)"
grep -qx "WLR_RENDERER_ALLOW_SOFTWARE=1" /fake/cage.env 2>/dev/null \
    && ok "in a VM it allows software GL" || no "WLR_RENDERER_ALLOW_SOFTWARE is not set in a VM"
[ "$vt" = "$(printf "\033[H\033[2J\033[3J")" ] && ok "the VT gets the wipe and nothing else" || no "the VT got: $(printf %s "$vt" | cat -v)"
rm -f /etc/X11/xorg.conf.d/00-keyboard.conf /fake/cage.env
env -i PATH="/fake:/usr/bin" XDG_CACHE_HOME=/tmp/greeter-cache sh $W >/dev/null 2>&1
! grep -q "^XKB_DEFAULT" /fake/cage.env 2>/dev/null && [ -s /fake/cage.env ] \
    && ok "without a keymap file: no XKB_DEFAULT_* (US, as before)" || no "XKB env without a file: $(grep XKB /fake/cage.env 2>&1)"

# the rest of the login path
cmp -s /etc/pam.d/greetd $P/system/pam.d/greetd && grep -q pam_gnome_keyring /etc/pam.d/greetd \
    && ok "PAM: the payload file, with pam_gnome_keyring" || no "/etc/pam.d/greetd"
[ -x /usr/local/bin/ewe-share-picker ] && cmp -s /etc/systemd/system/greetd.service.d/plymouth.conf $P/system/greetd/plymouth.conf \
    && ok "share picker + the greetd drop-in" || no "share picker / drop-in"
want="systemctl enable greetd.service getty@tty3.service NetworkManager.service bluetooth.service power-profiles-daemon.service systemd-timesyncd.service avahi-daemon.service"
[ "$(cat /fake/systemctl.log 2>/dev/null)" = "$want" ] && ok "the same seven units are enabled" || no "systemctl: $(cat /fake/systemctl.log 2>&1)"

# a second run changes nothing (resume after an interrupted install)
sum1="$(cat $W $G/theme-tokens.json | sha256sum)"
setup_greeter >/dev/null 2>&1; sum2="$(cat $W $G/theme-tokens.json | sha256sum)"
[ "$sum1" = "$sum2" ] && ok "a second run leaves the wrapper and the tokens as they were" || no "a second run changed the wrapper or the tokens"

# the login path still fails loudly: no PAM module → the install must stop
rm -f /usr/lib/security/pam_gnome_keyring.so
setup_greeter >/dev/null 2>&1 && no "a target without pam_gnome_keyring.so passed" || ok "a target without pam_gnome_keyring.so fails the step"
exit $fail
' || fail=1
done

[ "$fail" = 0 ] && echo "all good" || echo "FAILED"
exit "$fail"
