#!/usr/bin/env python3
"""Check that text runs sit where Telegram Desktop puts them.

Usage: python3 -I align.py [--profile NAME] <reference.png> <capture.png>

For each text run of the profile (profiles.py; "main" by default) it finds the shift of the capture
(within +-3 px) that best matches the reference. A run whose best shift is
not (0, 0) is a geometry bug, not antialiasing. It also compares the ink of
the run (mean brightness above the background): a rasteriser that draws
heavier or lighter strokes shows up as a ratio away from 1.00.
Exit status 1 when any run is shifted.
"""
import sys
from pathlib import Path

from PIL import Image, ImageChops

sys.path.insert(0, str(Path(__file__).resolve().parent))
import profiles  # noqa: E402


def score(a, b):
    hist = ImageChops.difference(a, b).histogram()
    return sum(i * v for i, v in enumerate(hist)) / (a.size[0] * a.size[1])


def ink(img):
    hist = img.histogram()
    n = img.size[0] * img.size[1]
    bg = max(range(256), key=lambda v: hist[v])
    return sum((v - bg) * c for v, c in enumerate(hist) if v > bg) / n


def main() -> int:
    profile, args = profiles.take(sys.argv[1:])
    ref = Image.open(args[0]).convert("L")
    cap = Image.open(args[1]).convert("L")
    profiles.check_size(profile, ref, "the reference")
    profiles.check_size(profile, cap, "the capture")
    shifted = 0
    print(f"{'run':<22} {'shift':>8} {'diff':>6} {'ink':>6}")
    for name, (x0, y0, x1, y1) in profile["runs"].items():
        a = ref.crop((x0, y0, x1, y1))
        best = None
        for dy in range(-3, 4):
            for dx in range(-3, 4):
                b = cap.crop((x0 + dx, y0 + dy, x1 + dx, y1 + dy))
                s = score(a, b)
                if best is None or s < best[0] - 1e-9 or (
                    abs(s - best[0]) < 1e-9 and abs(dx) + abs(dy) < abs(best[1]) + abs(best[2])
                ):
                    best = (s, dx, dy)
        s, dx, dy = best
        b = cap.crop((x0 + dx, y0 + dy, x1 + dx, y1 + dy))
        ref_ink = ink(a)
        ratio = ink(b) / ref_ink if ref_ink else 0.0
        mark = "" if (dx, dy) == (0, 0) else "  <- shifted"
        shifted += (dx, dy) != (0, 0)
        print(f"{name:<22} {f'({dx:+d},{dy:+d})':>8} {s:6.2f} {ratio:6.2f}{mark}")
    print(f"shifted runs: {shifted}")
    return 1 if shifted else 0


if __name__ == "__main__":
    sys.exit(main())
