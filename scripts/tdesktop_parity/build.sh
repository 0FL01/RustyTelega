#!/usr/bin/env bash
# Build the Quill binary that capture.sh runs, in its own target directory.
#
# `cargo test` relinks target/debug/quill with the test-support feature,
# and such a binary refuses to start (src/main.rs refuse_test_build). A
# separate target directory keeps the capture binary intact.
#
# Without a system alsa-lib-devel, point QUILL_NATIVE_ROOT at a directory
# where its RPM is unpacked (usr/lib64/pkgconfig/alsa.pc inside).
#
# Usage: build.sh   (prints the binary path)
set -euo pipefail
cd "$(dirname "$0")/../.."
if [ -n "${QUILL_NATIVE_ROOT:-}" ]; then
  export PKG_CONFIG_PATH="$QUILL_NATIVE_ROOT/usr/lib64/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  export PKG_CONFIG_SYSROOT_DIR="$QUILL_NATIVE_ROOT"
fi
CARGO_TARGET_DIR=target/parity cargo build --bin quill >&2
echo "$PWD/target/parity/debug/quill"
