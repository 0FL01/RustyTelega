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
# Usage: capture.sh <quill-binary> <out.png> [kind]
set -euo pipefail

BIN=$(realpath "${1:?quill binary}")
PNG=$(realpath -m "${2:?output png}")
KIND=${3:-ready-tdesktop-parity}

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

cat >"$WORK/inner.sh" <<EOF
#!/usr/bin/env bash
QUILL_DEMO_MAXIMIZED=1 QUILL_DEMO_HIDE_STATUS=1 QUILL_DEMO_WINDOW_SIZE=1919x1034 \\
  QUILL_DEMO_THEME=\${QUILL_DEMO_THEME:-dark} QUILL_DEMO_LINGER_MS=15000 QUILL_REDUCED_MOTION=\${QUILL_REDUCED_MOTION:-1} \\
  "$BIN" --screenshot-demo "$KIND" "$WORK/out" >"$WORK/quill.log" 2>&1 &
for _ in \$(seq 1 120); do
  [ -f "$WORK/out/.quill-ready-$KIND" ] && break
  sleep 0.5
done
sleep \${QUILL_CAPTURE_SETTLE:-3}
spectacle --background --nonotify --activewindow --output "$PNG" >"$WORK/spectacle.log" 2>&1
kill %1 2>/dev/null || true
EOF
chmod +x "$WORK/inner.sh"
mkdir -p "$WORK/out" "$WORK/config"

# A clean config: the user's KWin output settings (scale) must not apply.
XDG_CONFIG_HOME="$WORK/config" timeout 120 dbus-run-session -- \
  kwin_wayland --virtual --no-lockscreen --width 1920 --height 1036 --scale 1 \
  --socket "quill-capture-$$" --exit-with-session "$WORK/inner.sh" \
  >"$WORK/kwin.log" 2>&1 || true

if [ ! -s "$PNG" ]; then
  cat "$WORK/quill.log" "$WORK/spectacle.log" >&2 2>/dev/null || true
  echo "capture failed" >&2
  exit 1
fi
identify "$PNG"
