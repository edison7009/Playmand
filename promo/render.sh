#!/usr/bin/env bash
# Record the promo with the Playmand video workflow: fixed clock, one PNG per frame.
# A machine with a GPU can leave the Vulkan overrides unset and Bevy will use it.
set -euo pipefail
cd "$(dirname "$0")"

if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "No DISPLAY or WAYLAND_DISPLAY. Bevy records from a real window." >&2
  exit 1
fi

# This repository's cloud check used lavapipe (Mesa's software Vulkan).
# On a GPU machine, do not force these.
if [[ -f /usr/share/vulkan/icd.d/lvp_icd.json && -z "${VK_ICD_FILENAMES:-}" ]]; then
  if ! ls /dev/dri >/dev/null 2>&1; then
    export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json
    export WGPU_BACKEND="${WGPU_BACKEND:-vulkan}"
    export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
    echo "No GPU node found; recording with lavapipe."
  fi
fi

FRAMES="${1:-frames}"
exec cargo run --release -- --record "$FRAMES"
