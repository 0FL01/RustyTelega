#!/usr/bin/env bash
# Capture Telegram Desktop's own screens as parity references.
#
# Runs the official Telegram Desktop binary in a nested, headless KWin
# (kwin_wayland --virtual, 1920x1036, scale 1: the same session capture.sh
# runs Quill in) with a fresh -workdir and a private HOME, then plays a
# scenario: eis_input.py commands (click, type, key, sleep, maximize, shot)
# that drive the window through KWin's EIS server. Nothing appears on the
# desktop, nothing touches the host seat, and the binary never sees Quill's
# or the user's data.
#
# The binary is the official build (td.telegram.org tsetup), extracted by
# fetch_reference_binary.sh into artifacts/tdesktop-bin/ (untracked).
# References land in the output directory, never in the repository.
#
# Usage: capture_reference.sh <scenario> <out-dir>
#   scenario: a file in scripts/tdesktop_parity/reference/ (without .txt)
#   or a path. "shot NAME" lines write <out-dir>/NAME.png.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
TG="${TDESKTOP_BIN:-$ROOT/artifacts/tdesktop-bin/app/Telegram/Telegram}"
[ -x "$TG" ] || { echo "no Telegram binary at $TG (run fetch_reference_binary.sh)" >&2; exit 1; }

SCENARIO=${1:?scenario}
[ -f "$SCENARIO" ] || SCENARIO="$HERE/reference/$SCENARIO.txt"
[ -f "$SCENARIO" ] || { echo "no scenario $1" >&2; exit 1; }
OUT=$(realpath -m "${2:?out dir}")
mkdir -p "$OUT"

WORK=$(mktemp -d)
[ -n "${TDESKTOP_KEEP_WORK:-}" ] && echo "work: $WORK" >&2 || trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/home" "$WORK/workdir" "$WORK/config"

# Shots are relative to the output directory.
sed -E "s|^shot +(.+)$|shot $OUT/\\1.png|" "$SCENARIO" >"$WORK/scenario.txt"

cat >"$WORK/inner.sh" <<EOF
#!/usr/bin/env bash
export HOME="$WORK/home" XDG_DATA_HOME="$WORK/home/.local/share" \\
  XDG_CACHE_HOME="$WORK/home/.cache" XDG_STATE_HOME="$WORK/home/.local/state"
"$TG" ${TDESKTOP_ARGS:-} -noupdate -workdir "$WORK/workdir/" >"$WORK/tg.log" 2>&1 &
sleep \${TDESKTOP_START_WAIT:-8}
/usr/bin/python3 -I "$HERE/eis_input.py" <"$WORK/scenario.txt" >"$WORK/eis.log" 2>&1
echo \$? >"$WORK/eis.status"
kill %1 2>/dev/null || true
EOF
chmod +x "$WORK/inner.sh"

XDG_CONFIG_HOME="$WORK/config" timeout "${TDESKTOP_TIMEOUT:-300}" dbus-run-session -- \
  kwin_wayland --virtual --no-lockscreen --width 1920 --height 1036 --scale 1 \
  --socket "tdesktop-ref-$$" --exit-with-session "$WORK/inner.sh" \
  >"$WORK/kwin.log" 2>&1 || true

if [ "$(cat "$WORK/eis.status" 2>/dev/null)" != 0 ]; then
  cat "$WORK/eis.log" >&2 2>/dev/null || true
  echo "scenario failed" >&2
  exit 1
fi
ls "$OUT"
