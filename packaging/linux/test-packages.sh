#!/usr/bin/env bash
# Install and test the actual assets, with dependencies resolved by each OS.
set -euo pipefail
if (( $# != 1 )); then
  echo 'usage: bash packaging/linux/test-packages.sh <output-dir>' >&2
  exit 2
fi
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
[[ -d "$1" ]] || { echo "error: not a directory: $1" >&2; exit 2; }
output="$(cd "$1" && pwd)"
for name in quill-linux-x86_64.deb quill-linux-x86_64-fedora.rpm \
  quill-linux-x86_64-opensuse.rpm quill-linux-x86_64.pkg.tar.zst; do
  [[ -s "$output/$name" ]] || { echo "error: package missing or empty: $name" >&2; exit 1; }
done
command -v docker >/dev/null || { echo 'error: Docker is required' >&2; exit 2; }

# Concrete versions AND immutable manifests. Repositories are still rolling:
# their package updates and network availability are intentionally not faked.
debian='debian:13-slim@sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4'
ubuntu='ubuntu:26.04@sha256:6a06111c3b0a8600feadb270aec771dc239bf407b25625f93f44ab00e11f4d3c'
fedora='fedora:45@sha256:a7d72ba2745b100a3351087472e5ee7b2450af7cf331e7673007e3817612839c'
opensuse='opensuse/leap:16.1@sha256:756634f02c733fc52d5c9aa09aca6374c91c53c675dd445743736e13a8428858'
arch='archlinux:base@sha256:ece6c971fec3c532fd508bce299a446deaf469feecbfe7c353e995bc1a5e7bba'
run() {
  local image="$1" command="$2"
  echo "=== Installing and testing in $image ==="
  docker run --rm --platform linux/amd64 \
    --mount "type=bind,src=$output,dst=/packages,readonly" \
    --mount "type=bind,src=$root/packaging/linux/smoke-installed.sh,dst=/smoke-installed.sh,readonly" \
    --entrypoint /bin/bash "$image" -e -o pipefail -c "$command"
}

apt_test='
  p=/packages/quill-linux-x86_64.deb
  test "$(dpkg-deb -f "$p" Package)" = quill
  test "$(dpkg-deb -f "$p" Architecture)" = amd64
  dpkg-deb -I "$p"
  apt-get update
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "$p"
  dpkg-query -W quill
  /bin/sh /smoke-installed.sh
'
run "$debian" "$apt_test"
run "$ubuntu" "$apt_test"

run "$fedora" '
  p=/packages/quill-linux-x86_64-fedora.rpm
  test "$(rpm -qp --qf "%{NAME} %{ARCH}" "$p")" = "quill x86_64"
  rpm -qip "$p"; rpm -qp --requires "$p"
  dnf install -y "$p"
  rpm -q quill
  /bin/sh /smoke-installed.sh
'
run "$opensuse" '
  p=/packages/quill-linux-x86_64-opensuse.rpm
  test "$(rpm -qp --qf "%{NAME} %{ARCH}" "$p")" = "quill x86_64"
  rpm -qip "$p"; rpm -qp --requires "$p"
  zypper --non-interactive install --allow-unsigned-rpm "$p"
  rpm -q quill
  /bin/sh /smoke-installed.sh
'
run "$arch" '
  p=/packages/quill-linux-x86_64.pkg.tar.zst
  listing="$(bsdtar -tf "$p")"
  grep -qx "\.PKGINFO" <<<"$listing"
  grep -qx "\.MTREE" <<<"$listing"
  pacman -Qip "$p"
  test "$(bsdtar -xOf "$p" .PKGINFO | sed -n "s/^arch = //p")" = x86_64
  # Select a software provider for the virtual vulkan-driver dependency on
  # this headless system, rather than pacman selecting an arbitrary GPU.
  pacman -Syu --noconfirm --needed vulkan-swrast
  pacman -U --noconfirm "$p"
  pacman -Q quill-bin
  /bin/sh /smoke-installed.sh
'
echo 'All five package installations passed'
