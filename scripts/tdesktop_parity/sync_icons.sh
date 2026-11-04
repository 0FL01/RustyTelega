#!/usr/bin/env bash
# Copy the tdesktop icon masks listed in src/ui/td/icon.rs (1x, 2x, 3x)
# from a tdesktop checkout into assets/tdesktop-icons.
#
# Usage: sync_icons.sh [tdesktop-checkout]   (default: artifacts/tdesktop)
set -euo pipefail
cd "$(dirname "$0")/../.."
TD=${1:-artifacts/tdesktop}
DEST=assets/tdesktop-icons
grep -o '=> "[^"]*"' src/ui/td/icon.rs | sed 's/=> "\(.*\)"/\1/' | while read -r path; do
  for suffix in "" "@2x" "@3x"; do
    src=""
    for base in "$TD/Telegram/Resources/icons" "$TD/Telegram/lib_ui/icons"; do
      [ -f "$base/$path$suffix.png" ] && src="$base/$path$suffix.png" && break
    done
    if [ -z "$src" ]; then echo "missing: $path$suffix.png" >&2; exit 1; fi
    mkdir -p "$DEST/$(dirname "$path")"
    cp "$src" "$DEST/$path$suffix.png"
  done
done
find "$DEST" -name '*.png' | wc -l
