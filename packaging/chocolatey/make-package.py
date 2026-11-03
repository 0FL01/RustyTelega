#!/usr/bin/env python3
"""Write the Chocolatey package source for one Quill release.

    packaging/chocolatey/make-package.py 2026.1104.5000 OUT_DIR

OUT_DIR/quill.nuspec and OUT_DIR/tools/ wrap the release's
quill-windows-x86_64-setup.exe (packaging/windows/quill.iss), installed
silently. The checksum comes from the release's SHA256SUMS.txt, or from
QUILL_SUMS_FILE on disk; QUILL_SETUP_URL points the package at another copy
of the installer (CI uses a local file). `choco pack` in OUT_DIR builds the
.nupkg. docs/decisions/distribution-channels.md
"""
import os
import re
import sys
import urllib.request
from pathlib import Path
from xml.sax.saxutils import escape

REPO = "shinycake/quill"
ASSET = "quill-windows-x86_64-setup.exe"

NUSPEC = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>quill</id>
    <version>{version}</version>
    <title>Quill</title>
    <authors>Quill contributors</authors>
    <owners>shinycake</owners>
    <projectUrl>https://shinycake.github.io/quill/</projectUrl>
    <iconUrl>https://raw.githubusercontent.com/{repo}/main/site/img/icon-256.png</iconUrl>
    <licenseUrl>https://github.com/{repo}/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <projectSourceUrl>https://github.com/{repo}</projectSourceUrl>
    <packageSourceUrl>https://github.com/{repo}/tree/main/packaging/chocolatey</packageSourceUrl>
    <bugTrackerUrl>https://github.com/{repo}/issues</bugTrackerUrl>
    <releaseNotes>https://github.com/{repo}/releases/tag/v{version}</releaseNotes>
    <tags>telegram chat messenger messaging foss cross-platform</tags>
    <summary>An independent, unofficial Telegram client.</summary>
    <description>Quill is a Telegram client for Windows, macOS and Linux, written in Rust: chats, groups, channels, media, stickers, calls and several accounts in one window.

Quill updates itself from Settings. It is not affiliated with, endorsed by, or sponsored by Telegram.</description>
  </metadata>
  <files>
    <file src="tools\\**" target="tools" />
  </files>
</package>
"""

INSTALL = """$ErrorActionPreference = 'Stop'
# Quill's own installer, run silently: a per-user install into
# %LOCALAPPDATA%\\Programs\\Quill with a Start menu shortcut.
$packageArgs = @{{
  packageName    = $env:ChocolateyPackageName
  fileType       = 'exe'
  url64bit       = '{url}'
  checksum64     = '{sha256}'
  checksumType64 = 'sha256'
  silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-'
  validExitCodes = @(0)
}}
Install-ChocolateyPackage @packageArgs
"""

UNINSTALL = """$ErrorActionPreference = 'Stop'
$key = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Quill_is1'
if (-not (Test-Path $key)) {
  Write-Warning 'Quill is not installed for this user; nothing to uninstall.'
  return
}
$uninstaller = (Get-ItemProperty $key).UninstallString.Trim('"')
Uninstall-ChocolateyPackage -PackageName $env:ChocolateyPackageName -FileType 'exe' `
  -SilentArgs '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' -File $uninstaller
"""


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    version, out = sys.argv[1], Path(sys.argv[2])
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        sys.exit(f"unexpected version {version}")
    base = f"https://github.com/{REPO}/releases/download/v{version}"
    local = os.environ.get("QUILL_SUMS_FILE")
    if local:
        sums = Path(local).read_text()
    else:
        req = urllib.request.Request(f"{base}/SHA256SUMS.txt", headers={"User-Agent": "quill-choco"})
        with urllib.request.urlopen(req, timeout=60) as r:
            sums = r.read().decode()
    digest = next((l.split()[0] for l in sums.splitlines() if l.strip().endswith(f"  {ASSET}")), None)
    if not digest or not re.fullmatch(r"[0-9a-f]{64}", digest):
        sys.exit(f"no checksum for {ASSET}")
    url = os.environ.get("QUILL_SETUP_URL") or f"{base}/{ASSET}"
    (out / "tools").mkdir(parents=True, exist_ok=True)
    (out / "quill.nuspec").write_text(NUSPEC.format(version=escape(version), repo=REPO), encoding="utf-8")
    (out / "tools" / "chocolateyinstall.ps1").write_text(INSTALL.format(url=url, sha256=digest), encoding="utf-8")
    (out / "tools" / "chocolateyuninstall.ps1").write_text(UNINSTALL, encoding="utf-8")
    print(out)


if __name__ == "__main__":
    main()
