"""Check the asset checker on small files made here: a GLB, PNGs and a WAV."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import struct
import tempfile
import unittest
import wave
import zlib


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("check_asset", ROOT / "skills/playmand/scripts/check_asset.py")
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


def glb(doc, binary=b""):
    """A GLB file from a glTF document and a binary chunk."""
    js = json.dumps(doc).encode()
    js += b" " * (-len(js) % 4)
    binary += b"\0" * (-len(binary) % 4)
    chunks = struct.pack("<II", len(js), 0x4E4F534A) + js
    if binary:
        chunks += struct.pack("<II", len(binary), 0x004E4942) + binary
    return b"glTF" + struct.pack("<II", 2, 12 + len(chunks)) + chunks


def triangle_doc(height, lowest=0.0, **extra):
    doc = {
        "asset": {"version": "2.0"},
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "translation": [0, lowest, 0], "name": "Body"}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1}}]}],
        "accessors": [
            {"count": 3, "type": "VEC3", "componentType": 5126, "min": [0, 0, 0], "max": [1, height, 0]},
            {"count": 3, "type": "VEC3", "componentType": 5126},
        ],
        "materials": [{"name": "Steel"}],
    }
    doc.update(extra)
    return doc


def png(width, height, alpha_of):
    """An 8-bit RGBA PNG; alpha_of(x, y) gives each pixel's alpha."""
    rows = b"".join(b"\0" + b"".join(bytes((40, 40, 40, alpha_of(x, y))) for x in range(width)) for y in range(height))
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b""))


def args(*overrides):
    values = dict(max_tris=None, expect_height=None, tolerance=0.15, max_texture=None, require_alpha=False,
                  expect_size=None, max_peak=None, json=False)
    values.update(overrides)
    return checker.argparse.Namespace(**values)


class CheckAssetTests(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())

    def write(self, name, data):
        path = self.dir / name
        path.write_bytes(data)
        return path

    def test_glb_size_triangles_and_height_limit(self):
        path = self.write("hero.glb", glb(triangle_doc(1.8)))
        rep = checker.check(path, args(("expect_height", 1.8), ("max_tris", 10)))
        self.assertEqual(rep["triangles"], 1)
        self.assertEqual(rep["size_m"], [1.0, 1.8, 0.0])
        self.assertEqual(rep["errors"], [])
        rep = checker.check(path, args(("expect_height", 1.0)))
        self.assertTrue(any("height 1.800" in e for e in rep["errors"]))

    def test_glb_in_centimetres_and_floating_origin_are_flagged(self):
        # A 1.8 m character exported in centimetres is 180 units tall.
        rep = checker.check(self.write("cm.glb", glb(triangle_doc(180.0))), args())
        self.assertTrue(any("centimetres" in w for w in rep["warnings"]))
        rep = checker.check(self.write("float.glb", glb(triangle_doc(2.0, lowest=0.5))), args())
        self.assertTrue(any("origin" in w for w in rep["warnings"]))

    def test_required_draco_fails_and_feature_extensions_warn(self):
        doc = triangle_doc(1.0, extensionsUsed=["KHR_draco_mesh_compression", "KHR_materials_clearcoat"],
                           extensionsRequired=["KHR_draco_mesh_compression"])
        rep = checker.check(self.write("draco.glb", glb(doc)), args())
        self.assertTrue(any("Draco" in e for e in rep["errors"]))
        self.assertTrue(any("pbr_multi_layer_material_textures" in w for w in rep["warnings"]))
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(checker.main([str(self.dir / "draco.glb")]), 1)

    def test_embedded_image_size_and_jpeg_feature(self):
        jpeg = b"\xff\xd8\xff\xc0\x00\x11\x08" + struct.pack(">HH", 256, 512) + b"\x03" + b"\0" * 9
        doc = triangle_doc(1.0, images=[{"bufferView": 0, "mimeType": "image/jpeg"}],
                           bufferViews=[{"buffer": 0, "byteLength": len(jpeg)}], buffers=[{"byteLength": len(jpeg)}])
        rep = checker.check(self.write("tex.glb", glb(doc, jpeg)), args(("max_texture", 256)))
        self.assertEqual(rep["images"][0]["size"], [512, 256])
        self.assertTrue(any("'jpeg' feature" in w for w in rep["warnings"]))
        self.assertTrue(any("limit 256" in e for e in rep["errors"]))

    def test_fake_transparency_is_caught(self):
        fake = self.write("fake.png", png(8, 8, lambda x, y: 255))
        rep = checker.check(fake, args(("require_alpha", True)))
        self.assertEqual(rep["opaque"], 1.0)
        self.assertTrue(rep["errors"])
        real = self.write("real.png", png(8, 8, lambda x, y: 255 if 2 <= x < 6 else 0))
        rep = checker.check(real, args(("require_alpha", True), ("expect_size", "8x8")))
        self.assertEqual((rep["opaque"], rep["clear"]), (0.5, 0.5))
        self.assertEqual(rep["errors"], [])

    def test_wav_peak_clipping_and_leading_silence(self):
        buf = io.BytesIO()
        with wave.open(buf, "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(8000)
            frames = [0] * 800 + [32767, -32768] * 100 + [1000] * 800
            w.writeframes(struct.pack(f"<{len(frames)}h", *frames))
        rep = checker.check(self.write("hit.wav", buf.getvalue()), args(("max_peak", -1.0)))
        self.assertEqual(rep["clipped_samples"], 200)
        self.assertAlmostEqual(rep["leading_silence"], 0.1, places=3)
        self.assertTrue(any("clips" in w for w in rep["warnings"]))
        self.assertTrue(any("limit -1.0" in e for e in rep["errors"]))

    def test_children_without_scenes_count_once(self):
        doc = triangle_doc(1.0)
        del doc["scenes"]
        doc["nodes"] = [{"children": [1], "scale": [2, 2, 2]}, {"mesh": 0}]
        rep = checker.check(self.write("noscene.glb", glb(doc)), args())
        self.assertEqual(rep["triangles"], 1)
        self.assertEqual(rep["size_m"][1], 2.0)

    def test_palette_png_with_trns_has_alpha(self):
        def chunk(kind, data):
            return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
        data = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 1, 8, 3, 0, 0, 0))
                + chunk(b"PLTE", b"\0\0\0\xff\xff\xff") + chunk(b"tRNS", b"\0")
                + chunk(b"IDAT", zlib.compress(b"\0\0\1")) + chunk(b"IEND", b""))
        rep = checker.check(self.write("palette.png", data), args(("require_alpha", True)))
        self.assertTrue(rep["alpha_channel"])
        self.assertEqual(rep["errors"], [])

    def test_24_bit_extensible_wav_and_silence(self):
        def wav(tag, bits, frames, extensible=False):
            ch, rate = 1, 8000
            block = ch * bits // 8
            if extensible:
                fmt = struct.pack("<HHIIHHHHI", 0xFFFE, ch, rate, rate * block, block, bits, 22, bits, 0) \
                    + struct.pack("<H", tag) + b"\0" * 14
            else:
                fmt = struct.pack("<HHIIHH", tag, ch, rate, rate * block, block, bits)
            body = b"".join(v.to_bytes(bits // 8, "little", signed=True) for v in frames)
            riff = b"WAVE" + b"fmt " + struct.pack("<I", len(fmt)) + fmt + b"data" + struct.pack("<I", len(body)) + body
            return b"RIFF" + struct.pack("<I", len(riff)) + riff
        loud = wav(1, 24, [0] * 80 + [4194304] * 800, extensible=True)  # half scale = -6 dBFS
        rep = checker.check(self.write("daw.wav", loud), args())
        self.assertEqual((rep["bits"], rep["errors"]), (24, []))
        self.assertAlmostEqual(rep["peak_dbfs"], -6.02, places=1)
        silent = checker.check(self.write("silent.wav", wav(1, 16, [0] * 100)), args())
        self.assertIsNone(silent["peak_dbfs"])
        json.dumps(silent, allow_nan=False)  # valid JSON: no -Infinity

    def test_ogg_vorbis_codec_and_length(self):
        def page(granule, body):
            header = b"OggS" + bytes([0, 0]) + struct.pack("<q", granule) + b"\0" * 12 + bytes([1, len(body)])
            return header + body
        ident = b"\x01vorbis" + struct.pack("<I", 0) + bytes([2]) + struct.pack("<I", 44100) + b"\0" * 13
        rep = checker.check(self.write("rain.ogg", page(0, ident) + page(44100 * 3, b"\0" * 4)), args())
        self.assertEqual((rep["codec"], rep["channels"], rep["rate"], rep["seconds"]), ("vorbis", 2, 44100, 3.0))


if __name__ == "__main__":
    unittest.main()
