#!/usr/bin/env python3
"""Compare a Quill capture with the Telegram Desktop reference, region by region.

Usage: python3 -I diff.py [--no-masks] <reference.png> <capture.png> <out-dir>

Both images are cropped to the reference size from the top-left corner. For
each region of the main window it prints the share of pixels whose colour
differs by more than a small tolerance, and writes:
  <out>/heat.png      differing pixels in red over a dimmed reference
  <out>/side-<region>.png  reference and capture of one region, side by side
  <out>/blend.png     50/50 overlay (misaligned edges show up doubled)
Avatars and photos are masked, they are fixture content, not chrome;
--no-masks compares them too (the demo crops them from the reference when
QUILL_PARITY_DONOR is set). The mouse pointer in the reference is always
masked. The table also gives the share at the strict tolerances and the
largest summed delta of each region.
"""
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

TOLERANCE = 24  # summed RGB delta below this counts as equal (AA, rounding)
STRICT = (0, 8)  # also reported: exact match and near-exact

# Regions of the 1919x1034 Night reference (x0, y0, x1, y1).
REGIONS = {
    "titlebar": (0, 0, 1919, 24),
    "folder-rail": (0, 24, 72, 1034),
    "chat-list": (72, 24, 525, 1034),
    "header": (526, 24, 1919, 79),
    "pinned": (526, 79, 1919, 128),
    "history": (526, 128, 1919, 990),
    "composer": (526, 990, 1919, 1034),
}

# Fixture content that cannot match: avatars in the list and the history,
# photos and the GIF.
MASKS = [
    (82, 86, 128, 318),     # chat list avatars
    (536, 128, 576, 990),   # history avatars
    (580, 128, 902, 238),   # photo at the top of the history
    (580, 800, 902, 980),   # GIF
]
POINTER = (378, 702, 398, 730)  # the mouse pointer in the reference


def main() -> int:
    args = sys.argv[1:]
    masked = "--no-masks" not in args
    args = [a for a in args if a != "--no-masks"]
    ref_path, cap_path, out = (Path(a) for a in args[:3])
    out.mkdir(parents=True, exist_ok=True)
    ref = Image.open(ref_path).convert("RGB")
    cap = Image.open(cap_path).convert("RGB").crop((0, 0) + ref.size)

    diff = ImageChops.difference(ref, cap)
    r, g, b = diff.split()
    total = ImageChops.add(ImageChops.add(r, g), b)  # saturates at 255
    mask = Image.new("L", ref.size, 255)
    draw = ImageDraw.Draw(mask)
    for box in (MASKS if masked else []) + [POINTER]:
        draw.rectangle(box, fill=0)
    total = ImageChops.multiply(total, mask)
    bad = total.point(lambda v: 255 if v > TOLERANCE else 0)

    def share(img, box, tol):
        hist = img.crop(box).histogram()
        n = (box[2] - box[0]) * (box[3] - box[1])
        return 100 * sum(hist[tol + 1:]) / n

    tols = (TOLERANCE,) + STRICT
    whole = (0, 0) + ref.size
    print(f"{'region':<12} " + " ".join(f"{f'>{t}':>8}" for t in tols) + f" {'max':>5}")
    for name, box in list(REGIONS.items()) + [("total", whole)]:
        cols = " ".join(f"{share(total, box, t):7.2f}%" for t in tols)
        print(f"{name:<12} {cols} {total.crop(box).getextrema()[1]:5d}")

    heat = Image.blend(ref, Image.new("RGB", ref.size), 0.6)
    heat.paste(Image.new("RGB", ref.size, (255, 0, 0)), mask=bad)
    heat.save(out / "heat.png")
    Image.blend(ref, cap, 0.5).save(out / "blend.png")

    rows = []
    for name, box in REGIONS.items():
        w, h = box[2] - box[0], box[3] - box[1]
        if w > h:  # horizontal strips: stack reference over capture
            pair = Image.new("RGB", (w, 2 * h + 4), (255, 0, 255))
            pair.paste(ref.crop(box), (0, 0))
            pair.paste(cap.crop(box), (0, h + 4))
        else:
            pair = Image.new("RGB", (2 * w + 4, h), (255, 0, 255))
            pair.paste(ref.crop(box), (0, 0))
            pair.paste(cap.crop(box), (w + 4, 0))
        pair.save(out / f"side-{name}.png")
        rows.append(pair)
    return 0


if __name__ == "__main__":
    sys.exit(main())
