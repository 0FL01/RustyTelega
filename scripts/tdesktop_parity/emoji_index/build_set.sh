#!/usr/bin/env bash
# Build the bundled "Telegram Desktop" emoji set (assets/emoji/tdesktop/) from
# a tdesktop checkout: the sheets Telegram/Resources/emoji/emoji_1..8.webp and
# the index codegen_emoji gives each emoji of Telegram/lib_ui/emoji.txt
# (sheet = i >> 9, cell = i & 511; lib_ui/ui/emoji_config.h).
#
# Needs g++, Qt 6 Core (pkg-config Qt6Core) and the tdesktop submodules
# Telegram/codegen and Telegram/lib_base checked out.
#
# Usage: build_set.sh <tdesktop checkout>
set -euo pipefail
TD=$(realpath "${1:?tdesktop checkout}")
HERE=$(cd "$(dirname "$0")" && pwd)
OUT="$HERE/../../../assets/emoji/tdesktop"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
T="$TD/Telegram"
g++ -std=c++20 -O1 -fPIC -I"$T/codegen" -I"$T/lib_base" $(pkg-config --cflags Qt6Core) \
  "$HERE/driver.cpp" "$T/codegen/codegen/emoji/data.cpp" "$T/codegen/codegen/emoji/data_read.cpp" \
  "$T/codegen/codegen/emoji/data_old.cpp" "$T/codegen/codegen/common/logging.cpp" \
  $(pkg-config --libs Qt6Core) -o "$WORK/emoji-list"
"$WORK/emoji-list" "$T/lib_ui/emoji.txt" > "$WORK/list.tsv"
mkdir -p "$OUT"
python3 -I "$HERE/write_config.py" "$WORK/list.tsv" "$T/Resources/emoji" "$OUT"
