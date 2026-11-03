#!/usr/bin/env python3
"""Write the winget manifests for one Quill release.

    packaging/winget/make-manifests.py 2026.1104.5000 OUT_DIR

Reads the release's SHA256SUMS.txt for the installer's checksum and writes
OUT_DIR/manifests/s/shinycake/Quill/<version>/ in the layout of
microsoft/winget-pkgs, ready to copy into a pull request there. Later releases
are submitted by release-published.yml with Komac, which keeps these fields.
docs/decisions/distribution-channels.md
"""
import os
import re
import sys
import urllib.request
from pathlib import Path

REPO = "shinycake/quill"
ID = "shinycake.Quill"
ASSET = "quill-windows-x86_64-setup.exe"
SCHEMA = "1.10.0"


def get(url: str) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "quill-winget"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read().decode()


def release_date(version: str) -> str:
    """2026.1103.4940 -> 2026-11-03 (the build commit's UTC date)."""
    year, monthday, _ = version.split(".")
    return f"{year}-{int(monthday[:-2]):02d}-{monthday[-2:]}"


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    version, out = sys.argv[1], Path(sys.argv[2])
    if not re.fullmatch(r"\d{4}\.\d{3,4}\.\d+", version):
        sys.exit(f"version must be YEAR.MONTHDAY.COMMITS, got {version}")
    base = f"https://github.com/{REPO}/releases/download/v{version}"
    local = os.environ.get("QUILL_SUMS_FILE")  # a release's SHA256SUMS.txt on disk (tests)
    sums = Path(local).read_text() if local else get(f"{base}/SHA256SUMS.txt")
    digest = next((l.split()[0] for l in sums.splitlines() if l.endswith(f"  {ASSET}")), None)
    if not digest:
        sys.exit(f"v{version} has no {ASSET}")
    head = f"# yaml-language-server: $schema=https://aka.ms/winget-manifest.{{kind}}.{SCHEMA}.schema.json\n\n"
    common = f"PackageIdentifier: {ID}\nPackageVersion: {version}\n"
    files = {
        f"{ID}.yaml": head.format(kind="version") + common + f"""DefaultLocale: en-US
ManifestType: version
ManifestVersion: {SCHEMA}
""",
        f"{ID}.installer.yaml": head.format(kind="installer") + common + f"""Platform:
- Windows.Desktop
MinimumOSVersion: 10.0.17763.0
InstallerType: inno
Scope: user
InstallModes:
- interactive
- silent
UpgradeBehavior: install
ReleaseDate: {release_date(version)}
Installers:
- Architecture: x64
  InstallerUrl: {base}/{ASSET}
  InstallerSha256: {digest.upper()}
ManifestType: installer
ManifestVersion: {SCHEMA}
""",
        f"{ID}.locale.en-US.yaml": head.format(kind="defaultLocale") + common + f"""PackageLocale: en-US
Publisher: shinycake
PublisherUrl: https://github.com/shinycake
PublisherSupportUrl: https://github.com/{REPO}/issues
PackageName: Quill
PackageUrl: https://shinycake.github.io/quill/
License: MIT
LicenseUrl: https://github.com/{REPO}/blob/main/LICENSE
Copyright: Copyright (c) Quill contributors
ShortDescription: An independent, unofficial Telegram client.
Description: |-
  Quill is a Telegram client for Windows, macOS and Linux, written in Rust: chats, groups,
  channels, media, stickers, calls and several accounts in one window.
  Quill is not affiliated with, endorsed by, or sponsored by Telegram.
Moniker: quill
Tags:
- chat
- messaging
- messenger
- telegram
- telegram-client
ReleaseNotesUrl: https://github.com/{REPO}/releases/tag/v{version}
ManifestType: defaultLocale
ManifestVersion: {SCHEMA}
""",
    }
    dest = out / "manifests" / "s" / "shinycake" / "Quill" / version
    dest.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (dest / name).write_text(text, encoding="utf-8")
    print(dest)


if __name__ == "__main__":
    main()
