"""brand_colour.py: card colours read from icons."""

import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "brand_colour", ROOT / "skills" / "playmand" / "scripts" / "brand_colour.py"
)
bc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bc)


def png(width, height, pixel, filter_type=0):
    """An RGBA PNG; `pixel(x, y)` returns (r, g, b, a)."""
    raw = b""
    for y in range(height):
        row = bytes(c for x in range(width) for c in pixel(x, y))
        if filter_type == 1:  # Sub: store the difference from the pixel to the left
            row = bytes((row[i] - (row[i - 4] if i >= 4 else 0)) & 255 for i in range(len(row)))
        raw += bytes([filter_type]) + row

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def channels(colour):
    return [int(v) for v in colour[4:-1].split()]


class BrandColourTests(unittest.TestCase):
    def colour(self, data, suffix=".png"):
        with tempfile.TemporaryDirectory() as scratch:
            path = Path(scratch) / f"icon{suffix}"
            (path.write_bytes if isinstance(data, bytes) else path.write_text)(data)
            return bc.brand_colour(path)

    def test_filled_square_keeps_its_hue_and_its_own_colours(self):
        colour = self.colour(png(8, 8, lambda x, y: (217, 119, 87, 255)))
        r, g, b = channels(colour["from"])
        self.assertTrue(r > g > b)
        self.assertFalse(colour["neutral"])
        self.assertFalse(colour["glyph"])

    def test_white_text_reads_on_every_card(self):
        # A pale brand is darkened until white text on it reaches 3:1.
        for rgb in ((250, 230, 120), (180, 240, 200), (200, 220, 255)):
            colour = self.colour(png(4, 4, lambda x, y: (*rgb, 255)))
            r, g, b = (c / 255 for c in channels(colour["from"]))
            h, l, s = bc.colorsys.rgb_to_hls(r, g, b)
            self.assertGreaterEqual(bc.contrast_with_white(h * 360, s, l), 2.9, rgb)

    def test_black_white_and_grey_logos_get_the_slate_card(self):
        colour = self.colour(png(6, 6, lambda x, y: (250, 250, 250, 255) if (x + y) % 2 else (20, 20, 20, 255)))
        self.assertEqual(colour, bc.NEUTRAL)

    def test_one_colour_glyph_on_transparency_is_drawn_white(self):
        mark = self.colour(png(8, 8, lambda x, y: (40, 90, 230, 255) if x == y else (0, 0, 0, 0)))
        self.assertTrue(mark["glyph"])
        dark = self.colour(png(8, 8, lambda x, y: (10, 10, 10, 255) if x == y else (0, 0, 0, 0)))
        self.assertTrue(dark["neutral"] and dark["glyph"])

    def test_several_colours_stay_as_they_are(self):
        palette = [(234, 67, 53, 255), (52, 168, 83, 255), (66, 133, 244, 255), (251, 188, 5, 255)]
        colour = self.colour(png(8, 8, lambda x, y: palette[x % 4] if y % 2 else (0, 0, 0, 0)))
        self.assertFalse(colour["glyph"])

    def test_png_row_filters_are_undone(self):
        plain = self.colour(png(6, 6, lambda x, y: (30 + x * 20, 120, 200, 255)))
        sub = self.colour(png(6, 6, lambda x, y: (30 + x * 20, 120, 200, 255), filter_type=1))
        self.assertEqual(plain, sub)

    def test_svg_fill_colours_are_read(self):
        svg = '<svg xmlns="http://www.w3.org/2000/svg"><path fill="#D97757" d="M0 0h1v1z"/></svg>'
        colour = self.colour(svg, ".svg")
        self.assertTrue(colour["glyph"])
        r, g, b = channels(colour["from"])
        self.assertTrue(r > g > b)
        white = '<svg fill="#FFFFFF"><path d="M0 0h1v1z"/></svg>'
        self.assertTrue(self.colour(white, ".svg")["neutral"])

    def test_output_has_no_percent_signs(self):
        colour = self.colour(png(4, 4, lambda x, y: (40, 160, 90, 255)))
        self.assertNotIn("%", colour["from"] + colour["to"])


if __name__ == "__main__":
    unittest.main()
