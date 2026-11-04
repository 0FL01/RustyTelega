#!/usr/bin/env bash
# Repackage the two already verified, same-run CI bundles; never download a
# Quill release or publish a package. See scripts/linux-package.sh for ABI.
set -euo pipefail

if (( $# != 4 )); then
  echo "usage: bash packaging/linux/package.sh <x86_64-bundle.tar.gz> <aarch64-bundle.tar.gz> <version> <output-dir>" >&2
  exit 2
fi
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
x86="$1" arm="$2" VERSION="$3" output="$4"
[[ "$VERSION" =~ ^[0-9]+(\.[0-9]+)*$ ]] || { echo "error: version must contain only numeric dot-separated components" >&2; exit 2; }
[[ "$(uname -m)" == x86_64 && "$(uname -s)" == Linux ]] || { echo "error: an x86_64 Linux host is required" >&2; exit 2; }
(( $(id -u) != 0 )) || { echo "error: run as a non-root user with access to Docker" >&2; exit 2; }
for tool in docker python3 tar sha256sum; do
  command -v "$tool" >/dev/null || { echo "error: missing $tool" >&2; exit 2; }
done
for bundle in "$x86" "$arm"; do
  [[ -s "$bundle" ]] || { echo "error: missing or empty bundle: $bundle" >&2; exit 2; }
done

work="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/quill-packages.XXXXXXXX")"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/stage" "$work/arch" "$work/out"
tar -xzf "$x86" -C "$work/stage"
mv "$work/stage/quill-linux-x86_64" "$work/stage/quill"
for path in quill quill-webview ffmpeg ffprobe lib/libtdjson.so lib/libquill_calls.so \
  lib/libquillvideo.so lib/libssl.so.3 lib/libcrypto.so.3 LICENSE THIRD_PARTY.md \
  THIRD_PARTY_LICENSES.md share/applications/org.shinycake.quill.desktop \
  share/icons/hicolor/256x256/apps/quill.png; do
  [[ -s "$work/stage/quill/$path" ]] || { echo "error: x86_64 bundle missing $path" >&2; exit 1; }
done
[[ -d "$work/stage/quill/licenses/ffmpeg" && -d "$work/stage/quill/licenses/quill-calls" ]] || {
  echo "error: x86_64 bundle is missing native library licenses" >&2; exit 1;
}
# The executables and *all* bundled libraries remain together; the loader's
# existing $ORIGIN/lib and $ORIGIN RUNPATHs must not be rewritten.
mv "$work/stage/quill/share" "$work/stage/share"
rm -f "$work/stage/quill/install.sh" "$work/stage/quill/README.txt"
cp "$root/packaging/linux/nfpm.yaml" "$work/nfpm.yaml"

# makepkg expects source names and both checksums, even though this package is
# x86_64 only. No source points at an upstream release or a network URL.
cp "$x86" "$work/arch/quill-linux-x86_64-bundle.tar.gz"
cp "$arm" "$work/arch/quill-linux-aarch64-bundle.tar.gz"
(cd "$work/arch" && sha256sum quill-linux-{x86_64,aarch64}-bundle.tar.gz > SHA256SUMS.txt)
QUILL_SUMS_FILE="$work/arch/SHA256SUMS.txt" python3 "$root/packaging/aur/make-pkgbuild.py" --local "$VERSION" "$work/arch"

export VERSION
rpm_vars=(RPM_GCC RPM_CXX RPM_ZLIB RPM_ALSA RPM_PULSE RPM_GTK RPM_X11_XCB RPM_XCB RPM_XCB_XKB
  RPM_XCOMPOSITE RPM_XDAMAGE RPM_XRANDR RPM_XTST RPM_XKB RPM_WAYLAND RPM_VULKAN RPM_FONTCONFIG
  RPM_FREETYPE RPM_WEBKIT RPM_SOUP)
nfpm_env=(--env VERSION)
for var in "${rpm_vars[@]}"; do nfpm_env+=(--env "$var"); done
nfpm() {
  docker run --rm --platform linux/amd64 --user "$(id -u):$(id -g)" \
    --mount "type=bind,src=$work,dst=/work" --workdir /work "${nfpm_env[@]}" \
    goreleaser/nfpm:v2.48.0 package --config /work/nfpm.yaml --packager "$1" --target "/work/out/$2"
}

# Export both mappings explicitly: the RPM metadata must name real native
# packages rather than silently treating Fedora and openSUSE as interchangeable.
export RPM_GCC=libgcc RPM_CXX=libstdc++ RPM_ZLIB=zlib-ng-compat RPM_ALSA=alsa-lib
export RPM_PULSE=pulseaudio-libs RPM_GTK=gtk3 RPM_X11_XCB=libX11-xcb RPM_XCB=libxcb RPM_XCB_XKB=libxcb
export RPM_XCOMPOSITE=libXcomposite RPM_XDAMAGE=libXdamage RPM_XRANDR=libXrandr RPM_XTST=libXtst
export RPM_XKB=libxkbcommon-x11 RPM_WAYLAND=libwayland-client RPM_VULKAN=vulkan-loader
export RPM_FONTCONFIG=fontconfig RPM_FREETYPE=freetype RPM_WEBKIT=webkit2gtk4.1 RPM_SOUP=libsoup3
nfpm deb quill-linux-x86_64.deb
nfpm rpm quill-linux-x86_64-fedora.rpm

export RPM_GCC=libgcc_s1 RPM_CXX=libstdc++6 RPM_ZLIB=libz1 RPM_ALSA=libasound2
export RPM_PULSE=libpulse0 RPM_GTK=libgtk-3-0 RPM_X11_XCB=libX11-xcb1 RPM_XCB=libxcb1 RPM_XCB_XKB=libxcb-xkb1
export RPM_XCOMPOSITE=libXcomposite1 RPM_XDAMAGE=libXdamage1 RPM_XRANDR=libXrandr2 RPM_XTST=libXtst6
export RPM_XKB=libxkbcommon-x11-0 RPM_WAYLAND=libwayland-client0 RPM_VULKAN=libvulkan1
export RPM_FONTCONFIG=fontconfig RPM_FREETYPE=libfreetype6 RPM_WEBKIT=libwebkit2gtk-4_1-0 RPM_SOUP=libsoup-3_0-0
nfpm rpm quill-linux-x86_64-opensuse.rpm

# The pinned Arch image has makepkg/fakeroot. Resolve the PKGBUILD's real
# dependencies with pacman, then run makepkg as the invoking non-root UID.
# A software Vulkan provider makes the virtual vulkan-driver dependency
# resolvable in a headless image; it does not replace a user's GPU driver.
docker run --rm --platform linux/amd64 \
  --mount "type=bind,src=$work,dst=/work" --workdir /work/arch \
  --env "BUILD_UID=$(id -u)" --env "BUILD_GID=$(id -g)" \
  'archlinux:base-devel@sha256:767de6eaa40dcfeda5263a48737ea03a782caabdf3a403b6e6a203c7fd333681' \
  bash -eo pipefail -c '
    source ./PKGBUILD
    # A virtual name passed as an explicit target makes pacman choose its
    # first provider (currently NVIDIA) even when swrast is also a target.
    packages=()
    for dep in "${depends[@]}"; do
      [[ "$dep" == vulkan-driver ]] || packages+=("$dep")
    done
    pacman -Syu --noconfirm --needed vulkan-swrast "${packages[@]}"
    groupadd -o -g "$BUILD_GID" quill-builder
    useradd -o -u "$BUILD_UID" -g "$BUILD_GID" -M -d /tmp -s /bin/bash quill-builder
    runuser -u quill-builder -- env HOME=/tmp \
      "PACKAGER=Andrey Bash <bash8de@proton.me>" makepkg --noconfirm
  '
shopt -s nullglob
arch_packages=("$work"/arch/quill-bin-*.pkg.tar.zst)
(( ${#arch_packages[@]} == 1 )) || { echo "error: expected exactly one makepkg package" >&2; exit 1; }
cp "${arch_packages[0]}" "$work/out/quill-linux-x86_64.pkg.tar.zst"

mkdir -p "$output"
for name in quill-linux-x86_64.deb quill-linux-x86_64-fedora.rpm \
  quill-linux-x86_64-opensuse.rpm quill-linux-x86_64.pkg.tar.zst; do
  [[ -s "$work/out/$name" ]] || { echo "error: package missing: $name" >&2; exit 1; }
  install -m 644 "$work/out/$name" "$output/$name"
done
echo "Built four x86_64 packages in $output"
