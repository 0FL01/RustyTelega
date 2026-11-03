#!/usr/bin/env bash
# Package a signed (and, for releases, notarized and stapled) Quill.app as the
# drag-to-Applications disk image: a 660x400 window with the background from
# assets/macos/dmg/, the app on the left and an Applications link on the right.
#
#   scripts/macos-dmg.sh dist/Quill.app dist/macos/quill-macos-aarch64.dmg
#
# Signs the image with QUILL_CODESIGN_IDENTITY when set (package-macos.yml
# notarizes and staples it after). Requires dmgbuild (pip install dmgbuild),
# which writes the Finder layout without driving Finder, so it runs headless.
# docs/decisions/distribution-channels.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="${1:?usage: macos-dmg.sh Quill.app out.dmg}"
OUT="${2:?usage: macos-dmg.sh Quill.app out.dmg}"
DMGBUILD="${DMGBUILD:-dmgbuild}"
ART="$ROOT/assets/macos/dmg"

[[ -d "$APP/Contents/MacOS" ]] || { echo "error: $APP is not an app bundle" >&2; exit 2; }
command -v "$DMGBUILD" >/dev/null || { echo "error: dmgbuild not found (pip install dmgbuild)" >&2; exit 2; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# One TIFF with both resolutions, so Retina displays get the @2x art.
tiffutil -cathidpicheck "$ART/background.png" "$ART/background@2x.png" -out "$work/background.tiff" >/dev/null

mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"
"$DMGBUILD" -s "$ART/settings.py" \
  -D app="$APP" \
  -D background="$work/background.tiff" \
  -D volume_icon="$ROOT/assets/icons/Quill.icns" \
  "Quill" "$OUT"

if [[ -n "${QUILL_CODESIGN_IDENTITY:-}" ]]; then
  codesign --force --timestamp --sign "$QUILL_CODESIGN_IDENTITY" "$OUT"
  codesign --verify --verbose=2 "$OUT"
fi
hdiutil verify "$OUT" >/dev/null
du -h "$OUT"
