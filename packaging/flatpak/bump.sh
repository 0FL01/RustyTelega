#!/usr/bin/env bash
# Point the Flatpak manifest at a Quill release: rewrites both tarball URLs
# and their sha256 from the release's SHA256SUMS.txt.
#
#   bash packaging/flatpak/bump.sh v2026.1103.4940
#
# flatpak-external-data-checker does the same on Flathub; CI runs this on
# `release: published` to build the new release before Flathub picks it up.
set -euo pipefail

TAG="${1:?usage: bump.sh vYEAR.MONTHDAY.COMMITS}"
[[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "error: unexpected tag $TAG" >&2; exit 2; }
MANIFEST="$(dirname "${BASH_SOURCE[0]}")/io.github.shinycake.Quill.yml"
BASE="https://github.com/shinycake/quill/releases/download/$TAG"

sums="$(curl -fsSL --retry 3 "$BASE/SHA256SUMS.txt")"
for arch in x86_64 aarch64; do
  asset="quill-linux-$arch-bundle.tar.gz"
  sum="$(awk -v name="$asset" '$2 == name { print $1 }' <<<"$sums")"
  [[ "$sum" =~ ^[0-9a-f]{64}$ ]] || { echo "error: no sha256 for $asset in $TAG" >&2; exit 2; }
  # The sha256 line follows its url line.
  awk -v asset="$asset" -v url="$BASE/$asset" -v sum="$sum" '
    index($0, "url: ") && index($0, "/" asset) { sub(/url: .*/, "url: " url); next_sum = 1; print; next }
    next_sum && index($0, "sha256: ") { sub(/sha256: .*/, "sha256: " sum); next_sum = 0 }
    { print }
  ' "$MANIFEST" > "$MANIFEST.tmp"
  mv "$MANIFEST.tmp" "$MANIFEST"
  grep -q "$BASE/$asset" "$MANIFEST" || { echo "error: $asset url not updated" >&2; exit 2; }
  grep -q "sha256: $sum" "$MANIFEST" || { echo "error: $asset sha256 not updated" >&2; exit 2; }
done
echo "Manifest now builds $TAG"
