#!/usr/bin/env python3
"""Check an asset before building on it: glTF/GLB models, images (PNG, JPEG,
WebP) and audio (WAV, OGG). Python standard library only.

    python3 check_asset.py hero.glb --max-tris 60000 --expect-height 1.8
    python3 check_asset.py sprite.png --require-alpha --expect-size 512x512
    python3 check_asset.py rain.wav --max-peak -1
    python3 check_asset.py <file>... --json

It prints what the file really contains and warns about what commonly goes
wrong in a game or video. Each explicit limit that fails makes it exit 1, so
it can gate a build. Engine notes are for Bevy 0.20.1 (its glTF extension
table and default features); check them again for another version.
"""
import argparse
import array
import base64
import json
import math
import struct
import sys
import urllib.parse
import zlib
from pathlib import Path

# ------------------------------------------------------------------ Bevy 0.20.1 notes
GLTF_UNSUPPORTED = {
    "KHR_draco_mesh_compression": "Draco-compressed meshes do not load; re-export without Draco",
    "EXT_meshopt_compression": "meshopt-compressed buffers do not load; re-export without it",
    "KHR_mesh_quantization": "quantized vertex data is not supported; re-export with float attributes",
    "KHR_texture_basisu": "the extension syntax is not supported (KTX2 files are, by plain URI)",
    "EXT_texture_webp": "the extension syntax is not supported (WebP files are, with the 'webp' feature)",
    "EXT_mesh_gpu_instancing": "GPU instancing is ignored",
    "KHR_materials_variants": "material variants are ignored",
    "KHR_materials_sheen": "sheen is ignored",
    "KHR_materials_iridescence": "iridescence is ignored",
    "KHR_materials_dispersion": "dispersion is ignored",
    "KHR_animation_pointer": "pointer animations are ignored",
}
GLTF_NEEDS_FEATURE = {
    "KHR_materials_anisotropy": "pbr_anisotropy_texture",
    "KHR_materials_clearcoat": "pbr_multi_layer_material_textures",
    "KHR_materials_specular": "pbr_specular_textures",
    "KHR_materials_transmission": "pbr_transmission_textures",
}
IMAGE_NEEDS_FEATURE = {"image/jpeg": "jpeg", "image/webp": "webp"}


# ------------------------------------------------------------------ small helpers
def db(x):
    return -math.inf if x <= 0 else 20 * math.log10(x)


def image_size(data):
    """(format, width, height, has_alpha_channel) from an image's first bytes."""
    if data[:8] == b"\x89PNG\r\n\x1a\n":
        w, h, depth, ctype = struct.unpack(">IIBB", data[16:26])
        # Colour types 4 and 6 carry alpha; palette and RGB images can too, by a tRNS chunk.
        return "png", w, h, ctype in (4, 6) or b"tRNS" in data[:data.find(b"IDAT") if b"IDAT" in data else len(data)]
    if data[:2] == b"\xff\xd8":
        i = 2
        while i + 9 < len(data):
            if data[i] != 0xFF:
                i += 1
                continue
            marker = data[i + 1]
            length = struct.unpack(">H", data[i + 2:i + 4])[0]
            if marker in (0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF):
                h, w = struct.unpack(">HH", data[i + 5:i + 9])
                return "jpeg", w, h, False
            i += 2 + length
        return "jpeg", 0, 0, False
    if data[:4] == b"RIFF" and data[8:12] == b"WEBP":
        kind = data[12:16]
        if kind == b"VP8X":
            alpha = bool(data[20] & 0x10)
            w = 1 + int.from_bytes(data[24:27], "little")
            h = 1 + int.from_bytes(data[27:30], "little")
            return "webp", w, h, alpha
        if kind == b"VP8L":
            b = int.from_bytes(data[21:25], "little")
            return "webp", (b & 0x3FFF) + 1, ((b >> 14) & 0x3FFF) + 1, bool((b >> 28) & 1)
        if kind == b"VP8 ":
            w, h = struct.unpack("<HH", data[26:30])
            return "webp", w & 0x3FFF, h & 0x3FFF, False
    return "unknown", 0, 0, False


PNG_SCAN_LIMIT = 2_100_000  # pixels; pure-Python unfiltering beyond this gets slow


def png_alpha_stats(data):
    """Fraction of pixels that are fully opaque / fully clear, for 8-bit
    RGBA or grey+alpha PNGs (None for other layouts or very large images)."""
    w, h, depth, ctype, _, _, interlace = struct.unpack(">IIBBBBB", data[16:29])
    if depth != 8 or ctype not in (4, 6) or interlace or w * h > PNG_SCAN_LIMIT:
        return None
    channels = 4 if ctype == 6 else 2
    idat, i = b"", 8
    while i < len(data):
        length, kind = struct.unpack(">I4s", data[i:i + 8])
        if kind == b"IDAT":
            idat += data[i + 8:i + 8 + length]
        i += 12 + length
    raw = zlib.decompress(idat)
    stride = w * channels
    prev = bytearray(stride)
    opaque = clear = 0
    pos = 0
    for _ in range(h):
        f = raw[pos]
        line = bytearray(raw[pos + 1:pos + 1 + stride])
        pos += 1 + stride
        for x in range(stride):
            a = line[x - channels] if x >= channels else 0
            b = prev[x]
            c = prev[x - channels] if x >= channels else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        alphas = line[channels - 1::channels]
        opaque += sum(1 for v in alphas if v == 255)
        clear += sum(1 for v in alphas if v == 0)
        prev = line
    n = w * h
    return opaque / n, clear / n


# ------------------------------------------------------------------ glTF / GLB
def load_buffers(doc, binary, path, is_glb, warn):
    """Every buffer's bytes: the GLB binary chunk, data: URIs, or files beside a .gltf."""
    out = []
    for i, b in enumerate(doc.get("buffers", [])):
        uri = b.get("uri")
        if uri is None:
            out.append(binary if is_glb and i == 0 else b"")
        elif uri.startswith("data:"):
            out.append(base64.b64decode(uri.split(",", 1)[1]))
        else:
            f = path.parent / urllib.parse.unquote(uri)
            if f.exists():
                out.append(f.read_bytes())
            else:
                warn(f"buffer {i} ({uri}) is missing")
                out.append(b"")
    return out


def view_bytes(doc, buffers, index):
    bv = doc["bufferViews"][index]
    data = buffers[bv.get("buffer", 0)] if bv.get("buffer", 0) < len(buffers) else b""
    off = bv.get("byteOffset", 0)
    return data[off:off + bv["byteLength"]]


def load_gltf(path):
    data = path.read_bytes()
    if data[:4] == b"glTF":
        version, total = struct.unpack("<II", data[4:12])
        doc, binary, i = None, b"", 12
        while i < min(total, len(data)):
            length, kind = struct.unpack("<II", data[i:i + 8])
            chunk = data[i + 8:i + 8 + length]
            if kind == 0x4E4F534A:
                doc = json.loads(chunk.decode("utf-8"))
            elif kind == 0x004E4942:
                binary = chunk
            i += 8 + length
        return doc, binary, version, True
    return json.loads(data.decode("utf-8")), b"", 2, False


def mat_mul(a, b):
    return [sum(a[r + 4 * k] * b[k + 4 * c] for k in range(4)) for c in range(4) for r in range(4)]


def trs(node):
    if "matrix" in node:
        return list(node["matrix"])
    tx, ty, tz = node.get("translation", (0, 0, 0))
    qx, qy, qz, qw = node.get("rotation", (0, 0, 0, 1))
    sx, sy, sz = node.get("scale", (1, 1, 1))
    r = [1 - 2 * (qy * qy + qz * qz), 2 * (qx * qy + qz * qw), 2 * (qx * qz - qy * qw),
         2 * (qx * qy - qz * qw), 1 - 2 * (qx * qx + qz * qz), 2 * (qy * qz + qx * qw),
         2 * (qx * qz + qy * qw), 2 * (qy * qz - qx * qw), 1 - 2 * (qx * qx + qy * qy)]
    return [r[0] * sx, r[1] * sx, r[2] * sx, 0, r[3] * sy, r[4] * sy, r[5] * sy, 0,
            r[6] * sz, r[7] * sz, r[8] * sz, 0, tx, ty, tz, 1]


def check_gltf(path, args):
    doc, binary, version, is_glb = load_gltf(path)
    rep = {"file": str(path), "kind": "gltf", "warnings": [], "errors": []}
    warn, fail = rep["warnings"].append, rep["errors"].append
    if version != 2 or doc.get("asset", {}).get("version", "").split(".")[0] != "2":
        fail(f"not glTF 2.0 (asset.version {doc.get('asset', {}).get('version')})")
    rep["generator"] = doc.get("asset", {}).get("generator", "")
    acc, meshes, nodes = doc.get("accessors", []), doc.get("meshes", []), doc.get("nodes", [])
    # World transforms, from every scene root down.
    world = {}
    roots = []
    if doc.get("scenes"):
        for sc in doc["scenes"]:
            roots += sc.get("nodes", [])
    else:
        children = {c for n in nodes for c in n.get("children", [])}
        roots = [i for i in range(len(nodes)) if i not in children]
    stack = [(n, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]) for n in roots]
    while stack:
        n, parent = stack.pop()
        m = mat_mul(parent, trs(nodes[n]))
        world[n] = m
        stack += [(c, m) for c in nodes[n].get("children", [])]
    tris = verts = 0
    lo, hi = [math.inf] * 3, [-math.inf] * 3
    missing_normals = missing_uvs = 0
    materials = doc.get("materials", [])
    textured = {i for i, m in enumerate(materials)
                if "baseColorTexture" in m.get("pbrMetallicRoughness", {}) or "normalTexture" in m}
    for n, m in world.items():
        if "mesh" not in nodes[n]:
            continue
        for prim in meshes[nodes[n]["mesh"]].get("primitives", []):
            attrs = prim.get("attributes", {})
            pos = acc[attrs["POSITION"]] if "POSITION" in attrs else None
            if pos is None:
                continue
            count = pos.get("count", 0)
            verts += count
            mode = prim.get("mode", 4)
            if mode == 4:
                tris += (acc[prim["indices"]]["count"] if "indices" in prim else count) // 3
            if "NORMAL" not in attrs:
                missing_normals += 1
            if prim.get("material") in textured and "TEXCOORD_0" not in attrs:
                missing_uvs += 1
            if "min" in pos and "max" in pos:
                for corner in range(8):
                    p = [pos["max"][k] if corner >> k & 1 else pos["min"][k] for k in range(3)]
                    w = [m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r] for r in range(3)]
                    lo = [min(a, b) for a, b in zip(lo, w)]
                    hi = [max(a, b) for a, b in zip(hi, w)]
    rep.update(nodes=len(nodes), named_nodes=sum(1 for n in nodes if n.get("name")), meshes=len(meshes),
               triangles=tris, vertices=verts, materials=len(materials))
    if lo[0] < math.inf:
        size = [h - l for l, h in zip(lo, hi)]
        rep["size_m"] = [round(v, 3) for v in size]
        rep["min_y"] = round(lo[1], 3)
        rep["centre_xz"] = [round((lo[0] + hi[0]) / 2, 3), round((lo[2] + hi[2]) / 2, 3)]
        height = size[1]
        if height > 100:
            warn(f"{height:.0f} m tall: unless it is a building or a giant, it is in centimetres or millimetres; scale it to metres")
        elif height < 0.02:
            warn(f"{height * 100:.1f} cm tall: check the unit and scale")
        if height > 0 and abs(lo[1]) > 0.02 * height:
            warn(f"the lowest point is at y = {lo[1]:.3f} m, not 0: set the origin at the feet or base")
        if args.expect_height:
            if abs(height - args.expect_height) > args.tolerance * args.expect_height:
                fail(f"height {height:.3f} m, expected {args.expect_height} m (±{args.tolerance:.0%})")
    else:
        warn("no mesh with POSITION bounds: nothing to measure")
    if missing_normals:
        warn(f"{missing_normals} primitive(s) without normals: flat or wrong lighting")
    if missing_uvs:
        warn(f"{missing_uvs} textured primitive(s) without TEXCOORD_0: textures cannot map")
    if not materials and meshes:
        warn("no materials: everything renders in the default grey")
    if args.max_tris and tris > args.max_tris:
        fail(f"{tris} triangles, limit {args.max_tris}")
    # Textures and images.
    images = doc.get("images", [])
    buffers = load_buffers(doc, binary, path, is_glb, warn)
    info = []
    for i, im in enumerate(images):
        mime, data = im.get("mimeType"), b""
        if "bufferView" in im:
            data = view_bytes(doc, buffers, im["bufferView"])
        elif im.get("uri", "").startswith("data:"):
            data = base64.b64decode(im["uri"].split(",", 1)[1])
            mime = mime or im["uri"][5:].split(";")[0]
        else:
            uri = im.get("uri", "")
            if is_glb:
                warn(f"image {i} points outside the GLB ({uri}): it will be missing unless shipped beside it")
            f = path.parent / urllib.parse.unquote(uri)
            if f.exists():
                data = f.read_bytes()
            else:
                warn(f"image {i} ({uri}) is missing")
        fmt, w, h, _ = image_size(data)
        mime = mime or {"png": "image/png", "jpeg": "image/jpeg", "webp": "image/webp"}.get(fmt)
        info.append({"mime": mime, "size": [w, h], "bytes": len(data)})
        if mime in IMAGE_NEEDS_FEATURE:
            warn(f"image {i} is {mime}: Bevy needs the '{IMAGE_NEEDS_FEATURE[mime]}' feature")
        if args.max_texture and max(w, h) > args.max_texture:
            fail(f"image {i} is {w}x{h}, limit {args.max_texture}")
    rep["images"] = info
    rep["textures"] = len(doc.get("textures", []))
    # Rigs, animation, extensions.
    rep["skins"] = [len(s.get("joints", [])) for s in doc.get("skins", [])]
    anims = []
    for a in doc.get("animations", []):
        length = 0.0
        for s in a.get("samplers", []):
            mx = acc[s["input"]].get("max")
            if mx:
                length = max(length, mx[0])
        anims.append({"name": a.get("name", ""), "seconds": round(length, 3), "channels": len(a.get("channels", []))})
    rep["animations"] = anims
    used, required = doc.get("extensionsUsed", []), doc.get("extensionsRequired", [])
    rep["extensions"] = used
    for ext in used:
        if ext in GLTF_UNSUPPORTED:
            (fail if ext in required else warn)(f"{ext}: {GLTF_UNSUPPORTED[ext]}")
        elif ext in GLTF_NEEDS_FEATURE:
            warn(f"{ext}: Bevy needs the '{GLTF_NEEDS_FEATURE[ext]}' feature")
    return rep


# ------------------------------------------------------------------ images
def check_image(path, args):
    data = path.read_bytes()
    fmt, w, h, alpha = image_size(data)
    rep = {"file": str(path), "kind": "image", "format": fmt, "size": [w, h], "alpha_channel": alpha,
           "bytes": len(data), "warnings": [], "errors": []}
    warn, fail = rep["warnings"].append, rep["errors"].append
    if fmt == "unknown":
        fail("not a PNG, JPEG or WebP file")
        return rep
    if fmt == "png" and alpha:
        stats = png_alpha_stats(data)
        if stats is None:
            rep["transparency"] = "not measured (palette, 16-bit, interlaced, or over 2.1 megapixels)"
        else:
            opaque, clear = stats
            rep["opaque"], rep["clear"] = round(opaque, 4), round(clear, 4)
            if opaque == 1.0:
                warn("has an alpha channel but every pixel is opaque: the background is baked in")
                if args.require_alpha:
                    fail("no real transparency (every pixel opaque)")
    if args.require_alpha and not alpha:
        fail(f"{fmt} without an alpha channel")
    if args.expect_size:
        ew, eh = (int(v) for v in args.expect_size.lower().split("x"))
        if (w, h) != (ew, eh):
            fail(f"{w}x{h}, expected {ew}x{eh}")
    if args.max_texture and max(w, h) > args.max_texture:
        fail(f"{w}x{h}, limit {args.max_texture}")
    if fmt in ("jpeg", "webp"):
        warn(f"Bevy needs the '{fmt}' feature to load {fmt.upper()} images")
    return rep


# ------------------------------------------------------------------ audio
def read_wav(data):
    """(rate, channels, bits, is_float, sample bytes) from a RIFF WAVE file,
    including WAVE_FORMAT_EXTENSIBLE (common for 24-bit exports)."""
    if data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise ValueError("not a RIFF WAVE file")
    fmt, body, i = None, None, 12
    while i + 8 <= len(data):
        kind, length = data[i:i + 4], struct.unpack("<I", data[i + 4:i + 8])[0]
        chunk = data[i + 8:i + 8 + length]
        if kind == b"fmt ":
            fmt = chunk
        elif kind == b"data":
            body = chunk
        i += 8 + length + (length & 1)
    if fmt is None or body is None:
        raise ValueError("missing fmt or data chunk")
    tag, ch, rate = struct.unpack("<HHI", fmt[:8])
    bits = struct.unpack("<H", fmt[14:16])[0]
    if tag == 0xFFFE and len(fmt) >= 26:
        tag = struct.unpack("<H", fmt[24:26])[0]  # the sub-format GUID starts with the real tag
    if tag not in (1, 3):
        raise ValueError(f"unsupported WAV encoding (format tag {tag})")
    return rate, ch, bits, tag == 3, body


def samples_of(body, bits, is_float):
    """Samples as a compact array of floats in -1..1."""
    if is_float and bits == 32:
        return array.array("f", body[:len(body) // 4 * 4])
    if bits == 8:
        return array.array("f", ((b - 128) / 128.0 for b in body))
    if bits == 16:
        a = array.array("h", body[:len(body) // 2 * 2])
        return array.array("f", (v / 32768.0 for v in a))
    if bits == 24:
        return array.array("f", (int.from_bytes(body[i:i + 3], "little", signed=True) / 8388608.0
                                 for i in range(0, len(body) - 2, 3)))
    a = array.array("i", body[:len(body) // 4 * 4])
    return array.array("f", (v / 2147483648.0 for v in a))


def check_wav(path, args):
    rep = {"file": str(path), "kind": "audio", "format": "wav", "warnings": [], "errors": []}
    warn, fail = rep["warnings"].append, rep["errors"].append
    try:
        rate, ch, bits, is_float, body = read_wav(path.read_bytes())
    except ValueError as e:
        fail(f"unreadable WAV: {e}")
        return rep
    samples = samples_of(body, bits, is_float)
    n = len(samples) // max(1, ch)
    peak = max((abs(s) for s in samples), default=0.0)
    rms = math.sqrt(sum(s * s for s in samples) / len(samples)) if samples else 0.0
    clipped = sum(1 for s in samples if abs(s) >= 0.999)
    quiet = 10 ** (-60 / 20)
    loud = lambda f: any(abs(samples[f * ch + c]) > quiet for c in range(ch))
    lead = next((f for f in range(n) if loud(f)), n)
    tail = next((k for k, f in enumerate(range(n - 1, -1, -1)) if loud(f)), n)
    finite = lambda v: round(db(v), 2) if v > 0 else None  # silence has no level; JSON has no -inf
    rep.update(rate=rate, channels=ch, bits=bits, float=is_float, seconds=round(n / rate, 3) if rate else None,
               peak_dbfs=finite(peak), rms_dbfs=finite(rms), clipped_samples=clipped,
               leading_silence=round(lead / rate, 3), trailing_silence=round(tail / rate, 3))
    if peak == 0:
        warn("completely silent")
    if clipped:
        warn(f"{clipped} samples at full scale: it clips; lower the gain")
    if 0 < peak and db(peak) < -24:
        warn(f"peaks at {db(peak):.1f} dBFS: very quiet")
    if lead / rate > 0.05 and peak > 0:
        warn(f"{lead / rate:.2f} s of silence at the start: a one-shot will feel late")
    if args.max_peak is not None and peak > 0 and db(peak) > args.max_peak:
        fail(f"peak {db(peak):.2f} dBFS, limit {args.max_peak} dBFS")
    rep["note"] = "Bevy plays WAV only with the 'wav' feature; its default audio format is Ogg Vorbis"
    return rep


def check_ogg(path, args):
    data = path.read_bytes()
    rep = {"file": str(path), "kind": "audio", "format": "ogg", "warnings": [], "errors": []}
    if data[:4] != b"OggS":
        rep["errors"].append("not an Ogg file")
        return rep
    body = data[27 + data[26]:]  # page header: 27 bytes, then the segment table
    if body[1:7] == b"vorbis":
        codec, ch, rate = "vorbis", body[11], struct.unpack("<I", body[12:16])[0]
    elif body[:8] == b"OpusHead":
        codec, ch, rate = "opus", body[9], 48000
        rep["warnings"].append("Opus in Ogg: Bevy's default 'vorbis' decoder does not play it")
    else:
        codec, ch, rate = "unknown", 0, 0
        rep["warnings"].append("unknown codec in the Ogg container")
    last = data.rfind(b"OggS")
    granule = struct.unpack("<q", data[last + 6:last + 14])[0]
    rep.update(codec=codec, channels=ch, rate=rate, seconds=round(granule / rate, 3) if rate else None)
    return rep


# ------------------------------------------------------------------ main
def check(path, args):
    ext = path.suffix.lower()
    if ext in (".glb", ".gltf"):
        return check_gltf(path, args)
    if ext in (".png", ".jpg", ".jpeg", ".webp"):
        return check_image(path, args)
    if ext == ".wav":
        return check_wav(path, args)
    if ext in (".ogg", ".oga"):
        return check_ogg(path, args)
    return {"file": str(path), "kind": "unknown", "warnings": [], "errors": [f"unsupported file type {ext}"]}


def show(rep):
    lines = [f"{rep['file']} ({rep['kind']})"]
    skip = {"file", "kind", "warnings", "errors"}
    for k, v in rep.items():
        if k not in skip:
            lines.append(f"  {k}: {v}")
    lines += [f"  warning: {w}" for w in rep["warnings"]]
    lines += [f"  FAIL: {e}" for e in rep["errors"]]
    return "\n".join(lines)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("files", nargs="+", type=Path)
    p.add_argument("--max-tris", type=int)
    p.add_argument("--expect-height", type=float, help="metres, for models")
    p.add_argument("--tolerance", type=float, default=0.15)
    p.add_argument("--max-texture", type=int, help="largest image side in pixels")
    p.add_argument("--require-alpha", action="store_true")
    p.add_argument("--expect-size", help="WxH, for images")
    p.add_argument("--max-peak", type=float, help="dBFS, for audio")
    p.add_argument("--json", action="store_true")
    args = p.parse_args(argv)
    reports = [check(f, args) for f in args.files]
    print(json.dumps(reports, indent=2) if args.json else "\n".join(show(r) for r in reports))
    return 1 if any(r["errors"] for r in reports) else 0


if __name__ == "__main__":
    sys.exit(main())
