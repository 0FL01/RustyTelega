#!/usr/bin/env bash
# Capture a Quill screenshot demo on a KDE Plasma (Wayland) desktop session.
#
# GPUI's in-process capture (`render_to_image`) is not implemented on Linux,
# so the demo window is opened maximized and grabbed with Spectacle as the
# active window. The Telegram Desktop reference (artifacts/telegram-donor.png)
# was taken the same way: a maximized window on a 1920x1080 screen gives
# 1919x1034.
#
# Usage: capture.sh <quill-binary> <out.png> [kind]
set -euo pipefail

BIN=${1:?quill binary}
PNG=${2:?output png}
KIND=${3:-ready-tdesktop-parity}

OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT

# One capture at a time: Spectacle grabs whichever window is active.
exec 9>/tmp/quill-tdesktop-parity-capture.lock
flock 9

QUILL_DEMO_MAXIMIZED=1 QUILL_DEMO_WINDOW_SIZE=${QUILL_DEMO_WINDOW_SIZE:-1919x1034} \
  QUILL_DEMO_THEME=${QUILL_DEMO_THEME:-dark} \
  QUILL_DEMO_LINGER_MS=${QUILL_DEMO_LINGER_MS:-8000} QUILL_REDUCED_MOTION=1 \
  "$BIN" --screenshot-demo "$KIND" "$OUT" >"$OUT/log" 2>&1 &
APP=$!

for _ in $(seq 1 120); do
  [ -f "$OUT/.quill-ready-$KIND" ] && break
  if ! kill -0 "$APP" 2>/dev/null; then cat "$OUT/log" >&2; exit 1; fi
  sleep 0.5
done
sleep "${QUILL_CAPTURE_SETTLE:-2}"

spectacle --background --nonotify --activewindow --output "$PNG"
kill "$APP" 2>/dev/null || true
identify "$PNG"
