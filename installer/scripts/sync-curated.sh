#!/usr/bin/env bash
# The installer's Apps screen offers Komble's curated list — the same file,
# copied verbatim (it is plain data: komble-arch/src/lib/curated.js). Run this
# after the list changes in Komble; `--check` only says whether the copy is
# current (exit 1 when it is not), for a release checklist.
#
#   scripts/sync-curated.sh [--check] [path/to/komble-arch]
set -euo pipefail
cd "$(dirname "$0")/.."
check=0
[ "${1:-}" = "--check" ] && { check=1; shift; }
src="${1:-../../komble-arch}/src/lib/curated.js"
dst=src/lib/curated.js
[ -r "$src" ] || { echo "no Komble checkout at ${src%/src/lib/curated.js} — pass its path" >&2; exit 2; }
if [ "$check" = 1 ]; then
    if cmp -s "$src" "$dst"; then echo "curated.js is current"; exit 0; fi
    echo "curated.js differs from $src — run scripts/sync-curated.sh" >&2
    exit 1
fi
install -m644 "$src" "$dst"
echo "copied $src → $dst"
