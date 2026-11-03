#!/usr/bin/env bash
# Submit a signed Quill.app zip or disk image to Apple's notary service and
# wait for the verdict; on anything but Accepted, print the notary log and
# fail. The caller staples the ticket (to the .app, or to the .dmg itself).
#
#   NOTARY_KEY=<base64 .p8> NOTARY_KEY_ID=... NOTARY_ISSUER_ID=... \
#     scripts/macos-notarize.sh dist/macos/quill-macos-aarch64.dmg
#
# docs/decisions/macos-developer-id.md
set -euo pipefail

FILE="${1:?usage: macos-notarize.sh <file.zip|file.dmg>}"
: "${NOTARY_KEY:?}" "${NOTARY_KEY_ID:?}" "${NOTARY_ISSUER_ID:?}"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
key="$work/key.p8"
printf '%s' "$NOTARY_KEY" | base64 --decode > "$key"
auth=(--key "$key" --key-id "$NOTARY_KEY_ID" --issuer "$NOTARY_ISSUER_ID")

result="$work/result.json"
xcrun notarytool submit "$FILE" "${auth[@]}" --wait --timeout 1h --output-format json | tee "$result"
id="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["id"])' "$result")"
status="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"])' "$result")"
if [ "$status" != Accepted ]; then
  xcrun notarytool log "$id" "${auth[@]}" || true
  echo "::error::notarization of $(basename "$FILE") $status"
  exit 1
fi
echo "notarized $(basename "$FILE") ($id)"
