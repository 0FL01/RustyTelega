#!/usr/bin/env python3
"""Check that text runs sit where Telegram Desktop puts them.

Usage: python3 -I align.py <reference.png> <capture.png>

For each text run of the main window it finds the shift of the capture
(within +-3 px) that best matches the reference. A run whose best shift is
not (0, 0) is a geometry bug, not antialiasing. It also compares the ink of
the run (mean brightness above the background): a rasteriser that draws
heavier or lighter strokes shows up as a ratio away from 1.00.
Exit status 1 when any run is shifted.
"""
import sys

from PIL import Image, ImageChops

# Runs of the 1919x1034 Night reference (x0, y0, x1, y1).
RUNS = {
    "search placeholder": (96, 42, 160, 60),
    "rail label": (8, 118, 64, 132),
    "list name 1": (158, 90, 300, 106),
    "list date 1": (455, 90, 516, 106),
    "list preview 1": (160, 113, 515, 130),
    "list name 2": (158, 152, 250, 168),
    "list preview 2": (160, 175, 515, 192),
    "header name": (540, 34, 600, 50),
    "header status": (540, 55, 690, 70),
    "pinned title": (544, 86, 660, 101),
    "pinned text": (544, 104, 830, 119),
    "message line 1": (590, 254, 995, 270),
    "message line 2": (590, 272, 860, 288),
    "message time": (935, 422, 998, 436),
    "reply name": (636, 463, 720, 478),
    "reply link": (636, 481, 850, 496),
    "voice status": (646, 536, 745, 551),
    "voice time": (903, 541, 965, 556),
    "file name": (646, 689, 1003, 704),
    "file duration": (646, 711, 686, 726),
    "gif time": (828, 953, 892, 969),
    "composer placeholder": (574, 1005, 690, 1022),
}


def score(a, b):
    hist = ImageChops.difference(a, b).histogram()
    return sum(i * v for i, v in enumerate(hist)) / (a.size[0] * a.size[1])


def ink(img):
    hist = img.histogram()
    n = img.size[0] * img.size[1]
    bg = max(range(256), key=lambda v: hist[v])
    return sum((v - bg) * c for v, c in enumerate(hist) if v > bg) / n


def main() -> int:
    ref = Image.open(sys.argv[1]).convert("L")
    cap = Image.open(sys.argv[2]).convert("L")
    shifted = 0
    print(f"{'run':<22} {'shift':>8} {'diff':>6} {'ink':>6}")
    for name, (x0, y0, x1, y1) in RUNS.items():
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
