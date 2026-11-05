#!/usr/bin/env python3
"""Write a Quill sprite pack from tdesktop's emoji list and sheets.

Usage: python3 -I write_config.py <list.tsv> <tdesktop emoji dir> <out dir>

list.tsv is the driver's output (index, code points). The pack is what
src/emoji_sets_style/pack.rs reads: config.json mapping each emoji (without
U+FE0F) to (sheet, cell), and emoji_<sheet>.webp numbered from 0 (tdesktop
numbers its files from 1).
"""
import json
import shutil
import sys
from pathlib import Path

CELL, COLUMNS, ROWS = 72, 32, 16  # lib_ui/ui/emoji_config.cpp


def main() -> int:
    listing, sheets, out = (Path(a) for a in sys.argv[1:4])
    index = {}
    for line in listing.read_text().splitlines():
        number, points = line.split("\t")[:2]
        n = int(number)
        emoji = "".join(chr(int(p, 16)) for p in points.split("-")).replace("️", "")
        index.setdefault(emoji, [n >> 9, n & 511])
    count = max(sheet for sheet, _ in index.values()) + 1
    for sheet in range(count):
        shutil.copyfile(sheets / f"emoji_{sheet + 1}.webp", out / f"emoji_{sheet}.webp")
    shutil.copyfile(sheets / "set0_preview.webp", out / "preview.webp")
    config = {
        "version": 1,
        "cell": CELL,
        "columns": COLUMNS,
        "rows": ROWS,
        "sheets": count,
        "index": index,
    }
    (out / "config.json").write_text(json.dumps(config, ensure_ascii=False, sort_keys=True))
    print(f"{len(index)} emoji on {count} sheets -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
