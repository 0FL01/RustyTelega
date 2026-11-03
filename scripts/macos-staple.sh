#!/usr/bin/env bash
# Staples a notarized bundle or disk image and validates the ticket.
# Right after notarization Apple's ticket lookup (api.apple-cloudkit.com)
# sometimes drops the connection (stapler error 68, in staple or validate), so this tries again
# with a growing wait before giving up.
set -euo pipefail
target="$1"
for attempt in 1 2 3 4 5; do
  if xcrun stapler staple "$target" && xcrun stapler validate "$target"; then
    exit 0
  fi
  [ "$attempt" = 5 ] && break
  wait=$((attempt * 30))
  echo "stapling $target failed (attempt $attempt), trying again in ${wait}s"
  sleep "$wait"
done
echo "::error::could not staple $target" >&2
exit 1
