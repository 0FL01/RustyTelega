#!/usr/bin/env bash
# Generate a CI manifest from local bundles, or bump the checked-in manifest to
# a published RustyTelega release. See README.md; never build the checked-in
# upstream URLs by accident.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
exec python3 - "$script_dir/io.github.shinycake.Quill.yml" "$@" <<'PY'
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile


def fail(message):
    sys.exit(f"error: {message}")


usage = ("usage: bump.sh --local <bundle-directory> <output.yml>\n"
         "       bump.sh vYEAR.MONTHDAY.COMMITS")
manifest = Path(sys.argv[1])
args = sys.argv[2:]
arches = ("x86_64", "aarch64")
assets = {arch: f"quill-linux-{arch}-bundle.tar.gz" for arch in arches}

# This deliberately accepts only the known manifest layout. In particular,
# an extra URL, mirror, source, or architecture must not survive unnoticed:
# Flatpak Builder prefers a URL over a path when both are present. If the
# manifest's source layout changes, update this check along with the manifest.
try:
    original = manifest.read_text(encoding="utf-8")
except OSError as e:
    fail(f"cannot read {manifest}: {e}")
source_headers = re.findall(r"(?m)^[ \t]*(?:-[ \t]+)?sources[ \t]*:", original)
module_headers = re.findall(r"(?m)^[ \t]*(?:-[ \t]+)?modules[ \t]*:", original)
if len(source_headers) != 1 or len(module_headers) != 1 or original.count("\nmodules:\n") != 1:
    fail("unexpected modules/sources layout in manifest")
marker = "    sources:\n"
if original.count(marker) != 1:
    fail("unexpected sources block in manifest")
prefix, source_block = original.split(marker)
module_section = prefix.split("\nmodules:\n", 1)[1]
module_entries = re.findall(r"(?m)^  -[ \t]+.*$", module_section)
if module_entries != ["  - name: quill"]:
    fail("expected exactly one quill module")
pattern = re.compile(
    r"      - type: archive\n"
    r"        only-arches: \[(?P<arch1>x86_64|aarch64)\]\n"
    r"        url: (?P<url1>[^\n]+)\n"
    r"        sha256: (?:[0-9a-f]{64}|\"[0-9a-f]{64}\")\n"
    r"      - type: archive\n"
    r"        only-arches: \[(?P<arch2>x86_64|aarch64)\]\n"
    r"        url: (?P<url2>[^\n]+)\n"
    r"        sha256: (?:[0-9a-f]{64}|\"[0-9a-f]{64}\")\n"
    r"      - type: file\n"
    r"        path: io\.github\.shinycake\.Quill\.metainfo\.xml\n"
)
match = pattern.fullmatch(source_block)
if not match or {match["arch1"], match["arch2"]} != set(arches):
    fail("expected exactly two distinct architecture archives and the AppStream file")
for i in (1, 2):
    arch = match[f"arch{i}"]
    # Accept the original upstream URLs only as input for this migration;
    # neither mode ever fetches from them or leaves them in its output.
    expected_url = (r"https://github\.com/(?:shinycake/quill|0FL01/RustyTelega)/"
                    r"releases/download/v[0-9]+\.[0-9]+\.[0-9]+/" + re.escape(assets[arch]))
    if not re.fullmatch(expected_url, match[f"url{i}"]):
        fail(f"unexpected manifest URL for {arch}")


def sha256_file(path):
    digest = hashlib.sha256()
    try:
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
    except OSError as e:
        fail(f"cannot read {path}: {e}")
    return digest.hexdigest()


def atomic_write(path, content):
    temp = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", newline="\n",
                                         dir=path.parent, prefix=".bump-", delete=False) as stream:
            temp = Path(stream.name)
            stream.write(content)
        os.chmod(temp, stat.S_IMODE(manifest.stat().st_mode))
        os.replace(temp, path)
    except OSError as e:
        fail(f"cannot write {path}: {e}")
    finally:
        if temp is not None:
            temp.unlink(missing_ok=True)


if len(args) == 3 and args[0] == "--local":
    try:
        bundle_dir = Path(args[1]).resolve(strict=True)
        output_arg = Path(args[2])
        output = output_arg.parent.resolve(strict=True) / output_arg.name
    except (OSError, RuntimeError) as e:
        fail(f"invalid input or output path: {e}")
    if not bundle_dir.is_dir():
        fail(f"not a bundle directory: {bundle_dir}")
    if output.parent != manifest.parent or output.suffix not in (".yml", ".yaml"):
        fail("output must be a .yml or .yaml file next to the original manifest")
    if output == manifest or output.is_symlink():
        fail("output must not be the original manifest or a symlink")

    values = {}
    for arch, asset in assets.items():
        path = bundle_dir / asset
        if path.is_symlink() or not path.is_file():
            fail(f"missing regular bundle file: {path}")
        relative = os.path.relpath(path, manifest.parent)
        if not all(character.isprintable() for character in relative):
            fail(f"bundle path contains non-printable characters: {path!s}")
        # A JSON string is a YAML double-quoted scalar; this also safely handles
        # spaces, quotes, and other punctuation in directory names.
        values[arch] = ("path", json.dumps(relative, ensure_ascii=False), sha256_file(path))
    label = f"local bundles in {bundle_dir}"
elif len(args) == 1 and re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args[0]):
    tag = args[0]
    base = f"https://github.com/0FL01/RustyTelega/releases/download/{tag}"
    try:
        sums = subprocess.run(["curl", "-fsSL", "--retry", "3", f"{base}/SHA256SUMS.txt"],
                              check=True, stdout=subprocess.PIPE, text=True).stdout
    except (OSError, subprocess.CalledProcessError, UnicodeError) as e:
        fail(f"cannot retrieve checksums for {tag}: {e}")
    found = {asset: [] for asset in assets.values()}
    for line in sums.splitlines():
        entry = re.fullmatch(r"([0-9a-f]{64}) [ *](.+)", line)
        if entry and entry[2] in found:
            found[entry[2]].append(entry[1])
        elif any(line.endswith(asset) for asset in found):
            fail("malformed bundle entry in SHA256SUMS.txt")
    for asset, checksums in found.items():
        if len(checksums) != 1:
            fail(f"expected exactly one sha256 for {asset} in {tag}")
    values = {arch: ("url", f"{base}/{asset}", found[asset][0])
              for arch, asset in assets.items()}
    output = manifest
    label = f"published RustyTelega release {tag}"
else:
    fail(usage)

result = prefix + marker
for i in (1, 2):
    arch = match[f"arch{i}"]
    kind, location, checksum = values[arch]
    # Quote hashes: YAML could otherwise interpret an all-digit hash as a number.
    result += (f"      - type: archive\n"
               f"        only-arches: [{arch}]\n"
               f"        {kind}: {location}\n"
               f'        sha256: "{checksum}"\n')
result += ("      - type: file\n"
           "        path: io.github.shinycake.Quill.metainfo.xml\n")
atomic_write(output, result)
print(f"Wrote {output} for {label}")
PY
