#!/usr/bin/env bash
# Encode recorded frames plus the original score. Matches the MP4 line in
# skills/playmand/references/video.md, with audio added.
set -euo pipefail
cd "$(dirname "$0")"

FRAMES="${1:-frames}"
OUT="${2:-playmand-promo.mp4}"
FPS="${FPS:-30}"
CRF="${CRF:-20}"
MUSIC="${MUSIC:-assets/music/promo.wav}"

if [[ ! -f "$FRAMES/frame-0000.png" ]]; then
  echo "no $FRAMES/frame-0000.png" >&2
  exit 1
fi

ffmpeg -y -framerate "$FPS" -i "$FRAMES/frame-%04d.png" -i "$MUSIC" \
  -map 0:v:0 -map 1:a:0 \
  -c:v libx264 -pix_fmt yuv420p -crf "$CRF" -preset slow \
  -c:a aac -b:a 160k -shortest -movflags +faststart \
  "$OUT"

ffprobe -hide_banner -show_streams -show_format "$OUT"
ls -lh "$OUT"
