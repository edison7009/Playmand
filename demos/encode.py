"""Encode recorded PNG frames into a looping animated WebP (or GIF) for the README.

python demos/encode.py <frames-dir> <out.webp|out.gif> [--width 720] [--fps 25]
Requires Pillow.
"""

import argparse
from pathlib import Path

from PIL import Image


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("frames")
    ap.add_argument("out")
    ap.add_argument("--width", type=int, default=720)
    ap.add_argument("--fps", type=int, default=25)
    ap.add_argument("--quality", type=int, default=82)
    args = ap.parse_args()

    paths = sorted(Path(args.frames).glob("frame-*.png"))
    if not paths:
        raise SystemExit(f"no frame-*.png files in {args.frames}")
    frames = []
    for p in paths:
        im = Image.open(p).convert("RGB")
        h = round(im.height * args.width / im.width)
        frames.append(im.resize((args.width, h), Image.LANCZOS))

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    duration = round(1000 / args.fps)
    if out.suffix == ".gif":
        frames = [f.quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE) for f in frames]
        frames[0].save(out, save_all=True, append_images=frames[1:], duration=duration, loop=0, optimize=True)
    else:
        frames[0].save(out, save_all=True, append_images=frames[1:], duration=duration, loop=0,
                       quality=args.quality, method=6)
    print(f"{out}: {len(frames)} frames, {out.stat().st_size / 1024:.0f} KB")


if __name__ == "__main__":
    main()
