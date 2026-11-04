"""Tests for preparing the Flatpak manifest from release bundles."""

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


HERE = Path(__file__).resolve().parent
MANIFEST = "io.github.shinycake.Quill.yml"


class BumpTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        shutil.copy2(HERE / "bump.sh", self.root / "bump.sh")
        shutil.copy2(HERE / MANIFEST, self.root / MANIFEST)
        self.bundles = self.root / "_release"
        self.bundles.mkdir()
        for arch in ("x86_64", "aarch64"):
            (self.bundles / f"quill-linux-{arch}-bundle.tar.gz").write_bytes(arch.encode())

    def test_local_manifest_uses_both_bundles(self):
        original = (self.root / MANIFEST).read_bytes()
        output = self.root / "ci.yml"
        command = ["bash", str(self.root / "bump.sh"), "--local", str(self.bundles), str(output)]
        subprocess.run(command, check=True, capture_output=True)
        text = output.read_text()
        self.assertNotIn("        url:", text)
        for arch in ("x86_64", "aarch64"):
            name = f"quill-linux-{arch}-bundle.tar.gz"
            digest = hashlib.sha256((self.bundles / name).read_bytes()).hexdigest()
            self.assertIn(f'path: "_release/{name}"\n        sha256: "{digest}"', text)
        self.assertEqual((self.root / MANIFEST).read_bytes(), original)

        (self.bundles / "quill-linux-aarch64-bundle.tar.gz").unlink()
        self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
        self.assertEqual(output.read_text(), text)

    def test_published_tag_uses_fork(self):
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        curl = fake_bin / "curl"
        curl.write_text('#!/bin/sh\nprintf "%s" "$4" > "$URL_FILE"\ncat "$SUMS_FILE"\n')
        curl.chmod(0o755)
        sums = self.root / "sums"
        sums.write_text("".join(
            f"{hashlib.sha256((self.bundles / f'quill-linux-{arch}-bundle.tar.gz').read_bytes()).hexdigest()}  "
            f"quill-linux-{arch}-bundle.tar.gz\n" for arch in ("x86_64", "aarch64")
        ))
        url_file = self.root / "url"
        env = dict(os.environ, PATH=f"{fake_bin}{os.pathsep}{os.environ['PATH']}",
                   URL_FILE=str(url_file), SUMS_FILE=str(sums))
        subprocess.run(["bash", str(self.root / "bump.sh"), "v2026.1103.4954"],
                       env=env, check=True, capture_output=True)
        self.assertEqual(url_file.read_text(),
                         "https://github.com/0FL01/RustyTelega/releases/download/v2026.1103.4954/SHA256SUMS.txt")
        text = (self.root / MANIFEST).read_text()
        self.assertEqual(text.count("https://github.com/0FL01/RustyTelega/releases/download/"), 2)
        self.assertNotIn("github.com/shinycake/quill/releases", text)


if __name__ == "__main__":
    unittest.main()
