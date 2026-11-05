"""Regression checks for parity input sizes (no reference images needed)."""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent


class CaptureSizeTests(unittest.TestCase):
    def test_mismatched_capture_is_an_error_in_both_tools(self):
        with tempfile.TemporaryDirectory() as work:
            root = Path(work)
            ref, cap = root / "ref.png", root / "cap.png"
            Image.new("RGB", (1920, 1036)).save(ref)
            for size in ((1919, 1036), (1920, 1037)):
                Image.new("RGB", size).save(cap)
                for tool in ("align.py", "diff.py"):
                    with self.subTest(size=size, tool=tool):
                        result = subprocess.run(
                            [sys.executable, "-I", str(HERE / tool),
                             "--profile", "intro-phone", str(ref), str(cap),
                             str(root / "diff")],
                            capture_output=True, text=True, check=False,
                        )
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn("the capture is", result.stderr)
                        self.assertIn("the profile expects 1920x1036", result.stderr)


if __name__ == "__main__":
    unittest.main()
