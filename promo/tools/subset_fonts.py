#!/usr/bin/env python3
"""Build the promo fonts from the system Noto CJK collection.

The on-screen copy is Chinese, so the film needs a CJK face. Debian's
Noto Sans CJK is SIL OFL 1.1. A subset is a modified version, and "Noto"
is a reserved font name, so the files shipped here are renamed
Playmand Sans / Playmand Sans Bold. They are not a new design.

Noto Sans Mono is copied unmodified for the ASCII code card.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

from fontTools.ttLib import TTCollection, TTFont

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src" / "main.rs"
OUT = ROOT / "assets" / "fonts"
REGULAR_TTC = Path("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc")
BOLD_TTC = Path("/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc")
MONO = Path("/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf")


def sc_index(path: Path) -> int:
    collection = TTCollection(str(path))
    for index, font in enumerate(collection.fonts):
        family = font["name"].getDebugName(1) or ""
        if family.strip() in {"Noto Sans CJK SC", "Noto Sans Mono CJK SC"}:
            return index
    raise SystemExit(f"no Simplified Chinese face in {path}")


def charset() -> str:
    # Every non-ASCII character that can appear in the scene, plus the
    # punctuation the layout actually uses. ASCII comes from --unicodes.
    text = SRC.read_text(encoding="utf-8")
    extra = "·×—…「」"
    chars = sorted({ch for ch in text + extra if ord(ch) > 127})
    return "".join(chars)


def rename(path: Path, family: str, style: str) -> None:
    font = TTFont(str(path))
    full = f"{family} {style}"
    postscript = family.replace(" ", "") + "-" + style
    for record in font["name"].names:
        if record.nameID in (1, 16):
            record.string = family
        elif record.nameID in (2, 17):
            record.string = style
        elif record.nameID == 4:
            record.string = full
        elif record.nameID == 6:
            record.string = postscript
        elif record.nameID == 3:
            record.string = f"Playmand:{postscript}"
    font.save(str(path))


def subset(src: Path, dest: Path, family: str, style: str) -> None:
    index = sc_index(src)
    dest.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        sys.executable,
        "-m",
        "fontTools.subset",
        str(src),
        f"--font-number={index}",
        f"--output-file={dest}",
        f"--text={charset()}",
        "--unicodes=U+0020-007E,U+00A0,U+00B7,U+00D7,U+2013,U+2014,U+2026",
        "--layout-features=*",
        "--glyph-names",
        "--symbol-cmap",
        "--legacy-cmap",
        "--notdef-glyph",
        "--notdef-outline",
        "--recommended-glyphs",
        "--name-IDs=*",
        "--name-legacy",
        "--name-languages=*",
        "--drop-tables+=DSIG",
    ]
    subprocess.run(cmd, check=True)
    rename(dest, family, style)
    font = TTFont(str(dest))
    cmap = font.getBestCmap() or {}
    missing = [ch for ch in charset() if ord(ch) not in cmap]
    if missing:
        raise SystemExit(f"{dest.name} missing glyphs: {''.join(missing)}")
    print(f"{dest.name}: {dest.stat().st_size / 1024:.0f} KiB, face {font['name'].getDebugName(1)}")


def main() -> None:
    if not REGULAR_TTC.exists() or not BOLD_TTC.exists():
        raise SystemExit("install fonts-noto-cjk first")
    if not MONO.exists():
        raise SystemExit(f"missing {MONO}")
    subset(REGULAR_TTC, OUT / "PlaymandSans-Regular.ttf", "Playmand Sans", "Regular")
    subset(BOLD_TTC, OUT / "PlaymandSans-Bold.ttf", "Playmand Sans", "Bold")
    shutil.copyfile(MONO, OUT / "NotoSansMono-Regular.ttf")
    print("mono copied unmodified")


if __name__ == "__main__":
    sys.exit(main())
