#!/usr/bin/env python3
"""Point Casks/quill-app.rb at Quill's latest GitHub release.

Reads the release's SHA256SUMS.txt (the checksums the release pipeline
publishes) and rewrites the cask's version and sha256 lines. Prints the new
version when the cask changed, nothing when it was already current.
"""
import json
import re
import sys
import urllib.request
from pathlib import Path

REPO = "shinycake/quill"
ASSET = "quill-macos-aarch64.zip"
CASK = Path(__file__).resolve().parent.parent / "Casks" / "quill-app.rb"


def get(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "shinycake-homebrew-tap"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()


release = json.loads(get(f"https://api.github.com/repos/{REPO}/releases/latest"))
tag = release["tag_name"]
version = tag.removeprefix("v")
if not re.fullmatch(r"\d{4}\.\d{3,4}\.\d+", version):
    sys.exit(f"unexpected tag {tag}")
sums = get(f"https://github.com/{REPO}/releases/download/{tag}/SHA256SUMS.txt").decode()
digest = next((line.split()[0] for line in sums.splitlines() if line.endswith(f"  {ASSET}")), None)
if not digest or not re.fullmatch(r"[0-9a-f]{64}", digest):
    sys.exit(f"no checksum for {ASSET} in {tag}")

text = CASK.read_text()
new = re.sub(r'(?m)^  version ".*"$', f'  version "{version}"', text)
new = re.sub(r'(?m)^  sha256 ".*"$', f'  sha256 "{digest}"', new)
if new != text:
    CASK.write_text(new)
    print(version)
