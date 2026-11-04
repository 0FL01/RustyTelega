#!/usr/bin/env python3
"""Resolve Telegram Desktop palette keys for Quill's built-in themes.

Usage: python3 -I gen_palette.py <tdesktop-checkout> key1 key2 ...

Prints one line per key: the key and its 0xRRGGBBAA value in Classic
(colors.palette), Day (day-blue), Tinted (night) and Night (night-green),
the order `src/app_theme/embedded.rs` keeps them in.
"""
import re
import sys
import zipfile
from pathlib import Path

LINE = re.compile(r"^\s*([A-Za-z0-9_]+)\s*:\s*([^;]+);")


def parse(text: str) -> dict:
    out = {}
    for line in text.splitlines():
        line = line.split("//", 1)[0]
        m = LINE.match(line)
        if m:
            out[m.group(1)] = m.group(2).strip()
    return out


def resolve(key: str, layers: list) -> int:
    for _ in range(64):
        value = None
        for layer in layers:
            if key in layer:
                value = layer[key]
                break
        if value is None:
            raise KeyError(key)
        if value.startswith("#"):
            hexpart = value[1:]
            if len(hexpart) == 6:
                hexpart += "ff"
            return int(hexpart, 16)
        # A reference: resolve it through every layer again.
        key = value
    raise RecursionError(key)


def main() -> int:
    root = Path(sys.argv[1])
    keys = sys.argv[2:]
    base = parse((root / "Telegram/lib_ui/ui/colors.palette").read_text())

    def theme(name: str) -> dict:
        path = root / f"Telegram/Resources/{name}.tdesktop-theme"
        with zipfile.ZipFile(path) as z:
            return parse(z.read("colors.tdesktop-theme").decode())

    themes = [[base], [theme("day-blue"), base], [theme("night"), base], [theme("night-green"), base]]
    for key in keys:
        try:
            values = [resolve(key, layers) for layers in themes]
        except KeyError:
            print(key, "MISSING")
            continue
        print(key, " ".join(f"0x{v:08x}" for v in values))
    return 0


if __name__ == "__main__":
    sys.exit(main())
