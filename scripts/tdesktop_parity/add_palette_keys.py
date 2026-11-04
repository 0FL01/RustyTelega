#!/usr/bin/env python3
"""Append tdesktop palette keys to Quill's palette.

Usage: python3 -I add_palette_keys.py <tdesktop-checkout> <keys-file>

Each line of <keys-file> is `Variant|tdesktopName|Group|Label`. The keys are
appended to `src/app_theme/keys.rs`, with a `fallback` arm when tdesktop's
`colors.palette` defines the key by one Quill already has, and their values
are appended to every built-in palette in `src/app_theme/embedded.rs`:
Classic, Day, Tinted and Night from tdesktop's themes, Quill Light and Quill
Dark from Classic and Tinted. Run from the repository root.
"""
import re
import subprocess
import sys
from pathlib import Path


def main() -> int:
    td, keys_file = sys.argv[1], Path(sys.argv[2])
    rows = [line.split("|") for line in keys_file.read_text().splitlines() if line.strip()]
    names = [row[1] for row in rows]
    out = subprocess.run(
        [sys.executable, "-I", str(Path(__file__).with_name("gen_palette.py")), td, *names],
        capture_output=True, text=True, check=True,
    ).stdout
    values = {}
    for line in out.splitlines():
        key, *cols = line.split()
        if cols == ["MISSING"]:
            raise SystemExit(f"unknown tdesktop key: {key}")
        values[key] = [int(col, 16) for col in cols]

    keys = Path("src/app_theme/keys.rs")
    text = keys.read_text()
    last = list(re.finditer(r'    \w+ = "\w+", \w+, "[^"]*";\n', text))[-1]
    added = "".join(f'    {v} = "{n}", {g}, "{l}";\n' for v, n, g, l in rows)
    text = text[: last.end()] + added + text[last.end():]

    palette = {}
    for line in (Path(td) / "Telegram/lib_ui/ui/colors.palette").read_text().splitlines():
        line = line.split("//", 1)[0]
        m = re.match(r"^\s*([A-Za-z0-9_]+)\s*:\s*([^;]+);", line)
        if m:
            palette[m.group(1)] = m.group(2).strip()
    by_name = {n: v for v, n in re.findall(r'    (\w+) = "(\w+)", \w+, "', text)}
    arms = ""
    for variant, name, _group, _label in rows:
        ref, hops = palette.get(name, ""), 0
        while ref and not ref.startswith("#") and ref not in by_name and hops < 16:
            ref, hops = palette.get(ref, ""), hops + 1
        if ref and not ref.startswith("#") and ref in by_name:
            arms += f"            PaletteKey::{variant} => Some(PaletteKey::{by_name[ref]}),\n"
    anchor = "            _ => None,\n        }\n    }\n"
    if arms:
        text = text.replace(anchor, arms + anchor, 1)
    keys.write_text(text)

    embedded = Path("src/app_theme/embedded.rs")
    text = embedded.read_text()
    columns = {"QUILL_LIGHT": 0, "QUILL_DARK": 2, "CLASSIC": 0, "DAY": 1, "TINTED": 2, "NIGHT": 3}
    for array, column in columns.items():
        m = re.search(r"const " + array + r": \[u32; PaletteKey::COUNT\] = \[\n(.*?)\n\];", text, re.S)
        items = re.findall(r"0x[0-9a-f]{8}", m.group(1)) + [f"0x{values[n][column]:08x}" for n in names]
        body = "\n".join("    " + ", ".join(items[i : i + 8]) + "," for i in range(0, len(items), 8))
        text = text[: m.start(1)] + body + text[m.end(1):]
    embedded.write_text(text)
    print(f"added {len(rows)} keys")
    return 0


if __name__ == "__main__":
    sys.exit(main())
