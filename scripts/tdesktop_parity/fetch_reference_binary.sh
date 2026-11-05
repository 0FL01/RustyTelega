#!/usr/bin/env bash
# Fetch the official Telegram Desktop Linux build that matches the tdesktop
# sources in artifacts/tdesktop (7.3.5) into artifacts/tdesktop-bin/, which
# is untracked. The archive goes to its own directory and is checked against
# the SHA-256 recorded when the parity work started.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION=${TDESKTOP_VERSION:-7.3.5}
SHA256=${TDESKTOP_SHA256:-62082a56b212f22d75106dd13b7e3b9bc4ecdbfb472146b56c74da74199855b7}
DEST="$ROOT/artifacts/tdesktop-bin"
mkdir -p "$DEST/dl" "$DEST/app"
ARCHIVE="$DEST/dl/td-setup-linux-x64-$VERSION.tar.xz"
[ -f "$ARCHIVE" ] || curl -fsSL -o "$ARCHIVE" \
  "https://td.telegram.org/linux-x64/td-setup-linux-x64-$VERSION.tar.xz"
echo "$SHA256  $ARCHIVE" | sha256sum -c -
# Only the two expected members, nothing outside Telegram/.
MEMBERS=$(tar -tJf "$ARCHIVE")
if grep -qvxE 'Telegram/|Telegram/Updater|Telegram/Telegram' <<<"$MEMBERS"; then
  echo "unexpected archive members" >&2
  exit 1
fi
tar -xJf "$ARCHIVE" -C "$DEST/app" Telegram/Telegram
echo "$DEST/app/Telegram/Telegram"
