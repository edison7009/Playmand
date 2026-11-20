#!/usr/bin/env python3
"""A card colour for an app's provider or brand, read from its own icon.

    python brand_colour.py icon.png [icon.svg ...]

Prints one JSON object per icon: `from` and `to` (the card's gradient stops,
as `rgb(r g b)`), `neutral` (black, white or grey logo: use a slate card) and
`glyph` (a one-colour mark on transparency: draw it white on its own colour).
Standard library only. Reads PNG (8-bit grey, RGB, palette, with or without
alpha; not interlaced) and SVG (fill and stop colours in the markup).

The rule matches the preview kit's `brand.js` and the app code it was ported
to: the most common saturated hue, weighted by saturation, with lightness and
saturation settled into a band where white text reads on every brand alike.
"""

import colorsys
import json
import re
import struct
import sys
import zlib
from pathlib import Path

def rgb(h, s, l):
    r, g, b = colorsys.hls_to_rgb((h % 360) / 360, l, s)
    return f"rgb({round(r * 255)} {round(g * 255)} {round(b * 255)})"


# Black, white and grey logos get a slate card.
NEUTRAL = {"from": rgb(220, 0.09, 0.3), "to": rgb(220, 0.1, 0.22), "neutral": True, "glyph": False}


def contrast_with_white(h, s, l):
    def linear(c):
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = colorsys.hls_to_rgb((h % 360) / 360, l, s)
    luminance = 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    return 1.05 / (luminance + 0.05)


def png_pixels(data):
    """RGBA tuples of a non-interlaced 8-bit PNG."""
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos, idat, palette, alpha = 8, b"", None, None
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + length]
        if kind == b"IHDR":
            width, height, depth, colour, _, _, interlace = struct.unpack(">IIBBBBB", body)
        elif kind == b"PLTE":
            palette = [tuple(body[i:i + 3]) for i in range(0, len(body), 3)]
        elif kind == b"tRNS":
            alpha = body
        elif kind == b"IDAT":
            idat += body
        pos += 12 + length
    if depth != 8 or interlace:
        raise ValueError("only 8-bit, non-interlaced PNG is read")
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[colour]
    raw, stride = zlib.decompress(idat), width * channels
    rows, prev, i = [], bytearray(stride), 0
    for _ in range(height):
        kind, line = raw[i], bytearray(raw[i + 1:i + 1 + stride])
        i += 1 + stride
        for x in range(stride):
            a = line[x - channels] if x >= channels else 0
            b, c = prev[x], prev[x - channels] if x >= channels else 0
            if kind == 1:
                line[x] = (line[x] + a) & 255
            elif kind == 2:
                line[x] = (line[x] + b) & 255
            elif kind == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif kind == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line)
        prev = line
    for line in rows:
        for x in range(width):
            px = line[x * channels:(x + 1) * channels]
            if colour == 0:
                yield px[0], px[0], px[0], 255
            elif colour == 2:
                yield px[0], px[1], px[2], 255
            elif colour == 3:
                r, g, b = palette[px[0]]
                yield r, g, b, alpha[px[0]] if alpha and px[0] < len(alpha) else 255
            elif colour == 4:
                yield px[0], px[0], px[0], px[1]
            else:
                yield tuple(px)


def svg_pixels(text):
    """Stand-in pixels for an SVG: each fill or stop colour counts once.
    Colourless shapes (fill="none", currentColor) are skipped; an SVG drawn
    with one colour counts as a glyph on transparency."""
    colours = re.findall(r'(?:fill|stop-color|color)\s*[:=]\s*"?\s*#([0-9a-fA-F]{3,8})\b', text)
    pixels = []
    for hexa in colours:
        if len(hexa) in (3, 4):
            hexa = "".join(ch * 2 for ch in hexa)
        r, g, b = (int(hexa[i:i + 2], 16) for i in (0, 2, 4))
        pixels.append((r, g, b, 255))
    if not pixels:
        pixels.append((0, 0, 0, 255))
    # One shape colour on a transparent canvas: pad with transparency.
    return pixels + [(0, 0, 0, 0)] * len(pixels)


def brand(pixels):
    bins = [[0.0, 0.0, 0.0, 0.0] for _ in range(36)]
    hues, solid, coloured, dark, total = [], 0, 0, 0, 0
    for r, g, b, a in pixels:
        total += 1
        if a < 140:
            continue
        solid += 1
        h, l, s = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
        h *= 360
        if s < 0.25 and l < 0.3:
            dark += 1
        if s < 0.25 or l < 0.12 or l > 0.92:
            continue
        coloured += 1
        hues.append(h)
        w = s * (1 - abs(l - 0.5))
        bin_ = bins[int(h // 10) % 36]
        bin_[0] += w
        bin_[1] += h * w
        bin_[2] += s * w
        bin_[3] += l * w
    thin = solid < total * 0.6
    if not solid or coloured / solid < 0.08:
        return dict(NEUTRAL, glyph=bool(thin and solid and dark / solid > 0.5))
    pooled = [bins[i - 1][0] * 0.5 + bins[i][0] + bins[(i + 1) % 36][0] * 0.5 for i in range(36)]
    w, hsum, ssum, lsum = bins[pooled.index(max(pooled))]
    if not w:
        return dict(NEUTRAL)
    h = hsum / w
    near = sum(1 for x in hues if min(abs(x - h), 360 - abs(x - h)) < 25)
    s = min(max(ssum / w, 0.5), 0.82)
    l = min(max(lsum / w, 0.36), 0.5) - (0.07 if 40 < h < 170 else 0)  # yellows, greens read lighter
    # Then darken until white text has at least 3:1 contrast (WCAG, large bold text).
    while contrast_with_white(h, s, l) < 3 and l > 0.2:
        l -= 0.02
    return {
        "from": rgb(h, s, l),
        "to": rgb(h + 4, min(s + 0.04, 0.9), l - 0.08),
        "neutral": False,
        "glyph": bool(thin and near / solid > 0.75),
    }


def brand_colour(path):
    path = Path(path)
    if path.suffix.lower() == ".svg":
        return brand(svg_pixels(path.read_text(encoding="utf-8", errors="replace")))
    return brand(png_pixels(path.read_bytes()))


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for arg in sys.argv[1:]:
        print(json.dumps({"icon": arg, **brand_colour(arg)}, ensure_ascii=False))
