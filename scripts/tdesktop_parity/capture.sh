#!/usr/bin/env bash
# Capture a Quill screenshot demo the way the Telegram Desktop reference was
# taken: a maximized window on a 1920x1080 KDE Plasma screen whose panel
# leaves 1920x1036 (artifacts/telegram-donor.png is that window, 1919x1034).
#
# GPUI's in-process capture (`render_to_image`) is not implemented on Linux,
# so the demo runs in a nested, headless KWin (`kwin_wayland --virtual`) with
# a 1920x1036 output, and Spectacle grabs the window from inside that
# session. Nothing appears on the desktop. The nested session must not lock
# with the desktop's (logind locks every session of the user): a locked
# KWin stops sending frame callbacks, the window freezes on its first frame
# and pictures decoded after it never show, hence --no-lockscreen. Needs
# kwin_wayland, spectacle and dbus-run-session.
#
# The demo cuts its pictures (avatars, photo, GIF frame, mini previews) from
# the reference when QUILL_PARITY_DONOR names it; this script sets it to
# artifacts/telegram-donor.png unless it is set already or the file is
# missing.
#
# Keep the main reference's original 1920x1036 viewport; its donor omits
# the final column and two bottom rows. Crop that capture explicitly to
# 1919x1034 before align/diff, which now reject mismatched image sizes.
# Intro references already have the full 1920x1036 size.
#
# Usage: capture.sh <quill-binary> <out.png> [kind] [WIDTHxHEIGHT]
set -euo pipefail
DONOR_DEFAULT="$(cd "$(dirname "$0")/../.." && pwd)/artifacts/telegram-donor.png"
if [ -z "${QUILL_PARITY_DONOR+set}" ] && [ -f "$DONOR_DEFAULT" ]; then
  export QUILL_PARITY_DONOR="$DONOR_DEFAULT"
fi

BIN=$(realpath "${1:?quill binary}")
PNG=$(realpath -m "${2:?output png}")
KIND=${3:-ready-tdesktop-parity}
SIZE=${4:-1920x1036}
[[ "$SIZE" =~ ^([1-9][0-9]*)x([1-9][0-9]*)$ ]] || {
  echo "invalid capture size: $SIZE (expected WIDTHxHEIGHT)" >&2; exit 1;
}
WIDTH=${BASH_REMATCH[1]}
HEIGHT=${BASH_REMATCH[2]}

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

cat >"$WORK/inner.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
QUILL_DEMO_MAXIMIZED=1 QUILL_DEMO_HIDE_STATUS=1 QUILL_DEMO_WINDOW_SIZE=$SIZE \\
  QUILL_DEMO_THEME=\${QUILL_DEMO_THEME:-dark} QUILL_DEMO_LINGER_MS=15000 QUILL_REDUCED_MOTION=\${QUILL_REDUCED_MOTION:-1} \\
  "$BIN" --screenshot-demo "$KIND" "$WORK/out" >"$WORK/quill.log" 2>&1 &
APP=\$!
trap 'kill "\$APP" 2>/dev/null || true; wait "\$APP" 2>/dev/null || true' EXIT
for _ in \$(seq 1 120); do
  [ -f "$WORK/out/.quill-ready-$KIND" ] && break
  kill -0 "\$APP" 2>/dev/null || exit 1
  sleep 0.5
done
[ -f "$WORK/out/.quill-ready-$KIND" ] || exit 1
sleep \${QUILL_CAPTURE_SETTLE:-3}
spectacle --background --nonotify --activewindow --output "$WORK/capture.png" >"$WORK/spectacle.log" 2>&1
EOF
chmod +x "$WORK/inner.sh"
mkdir -p "$WORK/out" "$WORK/config"

# A clean config: the user's KWin output settings (scale) must not apply.
XDG_CONFIG_HOME="$WORK/config" timeout 120 dbus-run-session -- \
  kwin_wayland --virtual --no-lockscreen --width "$WIDTH" --height "$HEIGHT" --scale 1 \
  --socket "quill-capture-$$" --exit-with-session "$WORK/inner.sh" \
  >"$WORK/kwin.log" 2>&1 || true

if [ ! -s "$WORK/capture.png" ]; then
  cat "$WORK/quill.log" "$WORK/spectacle.log" >&2 2>/dev/null || true
  echo "capture failed" >&2
  exit 1
fi
mv "$WORK/capture.png" "$PNG"
identify "$PNG"
