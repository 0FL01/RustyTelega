#!/bin/sh
# Run inside an installed-package test image, with no host library overrides.
set -eu
unset LD_LIBRARY_PATH QUILL_TDJSON_PATH
libdir=/usr/lib/quill/lib

test -L /usr/bin/quill
test "$(readlink -f /usr/bin/quill)" = /usr/lib/quill/quill
case "$(quill --version)" in 'Quill '?*) ;; *) echo 'bad quill --version' >&2; exit 1;; esac
test "$(quill --build-info)" = ui
for f in /usr/lib/quill/quill /usr/lib/quill/quill-webview /usr/lib/quill/ffmpeg /usr/lib/quill/ffprobe; do
  test -x "$f"
done
for name in libtdjson.so libquill_calls.so libquillvideo.so libssl.so.3 libcrypto.so.3; do
  test -s "$libdir/$name"
done
for stem in libavcodec libavformat libavutil libavfilter libavdevice libswscale libswresample; do
  found=0
  for f in "$libdir/$stem".so.*; do [ -f "$f" ] && found=1; done
  test "$found" = 1 || { echo "missing $stem" >&2; exit 1; }
done

# ldd can exit 0 even when a dependency is missing. Check the actual bundled
# consumers' OpenSSL/FFmpeg resolution without forcing unrelated *system*
# WebKit/GTK dependencies to use our private copies.
for f in /usr/lib/quill/quill /usr/lib/quill/quill-webview /usr/lib/quill/ffmpeg /usr/lib/quill/ffprobe "$libdir"/*.so*; do
  [ -f "$f" ] || continue
  if ! deps="$(ldd "$f" 2>&1)"; then echo "ldd failed: $f: $deps" >&2; exit 1; fi
  printf '%s\n' "ldd $f" "$deps"
  if printf '%s\n' "$deps" | grep -q 'not found'; then exit 1; fi
  case "$f" in
    "$libdir/libtdjson.so")
      printf '%s\n' "$deps" | awk '
        $2 == "=>" && $1 ~ /^lib(ssl|crypto)\.so\.3$/ {
          seen++; if ($3 !~ /^\/usr\/lib\/quill\/lib\//) { print "wrong OpenSSL: " $0 > "/dev/stderr"; bad = 1 }
        }
        END { if (seen != 2 || bad) exit 1 }
      ' ;;
    /usr/lib/quill/ffmpeg|/usr/lib/quill/ffprobe|"$libdir/libquillvideo.so")
      printf '%s\n' "$deps" | awk '
        $2 == "=>" && $1 ~ /^(libavcodec|libavformat|libavutil|libavfilter|libavdevice|libswscale|libswresample)\.so\./ {
          seen++; if ($3 !~ /^\/usr\/lib\/quill\/lib\//) { print "wrong FFmpeg: " $0 > "/dev/stderr"; bad = 1 }
        }
        END { if (!seen || bad) exit 1 }
      ' ;;
  esac
done

desktop=/usr/share/applications/org.shinycake.quill.desktop
test -s "$desktop"
grep -qx 'Exec=quill %u' "$desktop"
grep -qx 'Icon=quill' "$desktop"
test -s /usr/share/icons/hicolor/256x256/apps/quill.png
for f in LICENSE THIRD_PARTY.md THIRD_PARTY_LICENSES.md; do test -s "/usr/lib/quill/$f"; done
test -d /usr/lib/quill/licenses/ffmpeg
test -d /usr/lib/quill/licenses/quill-calls
echo 'Installed package smoke checks passed'
