#!/usr/bin/env python3
"""Compare a Quill capture with the Telegram Desktop reference, region by region.

Usage: python3 -I diff.py [--profile NAME] [--masks] <reference.png> <capture.png> <out-dir>

Both images must have the profile's exact size. For each region of the
profile (profiles.py; "main" by default) it prints the share of pixels whose colour
differs by more than a small tolerance, and writes:
  <out>/heat.png      differing pixels in red over a dimmed reference
  <out>/side-<region>.png  reference and capture of one region, side by side
  <out>/blend.png     50/50 overlay (misaligned edges show up doubled)
The demo cuts avatars, photos and the GIF frame from the reference
(QUILL_PARITY_DONOR, set by capture.sh), so everything is compared; --masks
leaves those pictures out, for a capture made without the reference. The
mouse pointer in the reference is always masked. The table also gives the share at the strict tolerances and the
largest summed delta of each region.
"""
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import profiles  # noqa: E402

TOLERANCE = 24  # summed RGB delta below this counts as equal (AA, rounding)
STRICT = (0, 8)  # also reported: exact match and near-exact

def main() -> int:
    profile, args = profiles.take(sys.argv[1:])
    masked = "--masks" in args
    args = [a for a in args if a not in ("--masks", "--no-masks")]
    ref_path, cap_path, out = (Path(a) for a in args[:3])
    out.mkdir(parents=True, exist_ok=True)
    ref = Image.open(ref_path).convert("RGB")
    profiles.check_size(profile, ref, "the reference")
    cap = Image.open(cap_path).convert("RGB")
    profiles.check_size(profile, cap, "the capture")

    diff = ImageChops.difference(ref, cap)
    r, g, b = diff.split()
    total = ImageChops.add(ImageChops.add(r, g), b)  # saturates at 255
    mask = Image.new("L", ref.size, 255)
    draw = ImageDraw.Draw(mask)
    for box in (profile["masks"] if masked else []) + profile["always"]:
        draw.rectangle(box, fill=0)
    if masked:
        for box in profile.get("keep_ellipses", []):
            draw.ellipse(box, fill=255)
    total = ImageChops.multiply(total, mask)
    bad = total.point(lambda v: 255 if v > TOLERANCE else 0)

    def share(img, box, tol):
        hist = img.crop(box).histogram()
        n = (box[2] - box[0]) * (box[3] - box[1])
        return 100 * sum(hist[tol + 1:]) / n

    tols = (TOLERANCE,) + STRICT
    whole = (0, 0) + ref.size
    print(f"{'region':<12} " + " ".join(f"{f'>{t}':>8}" for t in tols) + f" {'max':>5}")
    for name, box in list(profile["regions"].items()) + [("total", whole)]:
        cols = " ".join(f"{share(total, box, t):7.2f}%" for t in tols)
        print(f"{name:<12} {cols} {total.crop(box).getextrema()[1]:5d}")

    heat = Image.blend(ref, Image.new("RGB", ref.size), 0.6)
    heat.paste(Image.new("RGB", ref.size, (255, 0, 0)), mask=bad)
    heat.save(out / "heat.png")
    Image.blend(ref, cap, 0.5).save(out / "blend.png")

    rows = []
    for name, box in profile["regions"].items():
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
