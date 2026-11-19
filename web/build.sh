#!/usr/bin/env bash
# Builds the demo to WebAssembly and writes the JS bindings into site/pkg.
# Needs the wasm32-unknown-unknown target and wasm-bindgen-cli 0.2.127
# (cargo install wasm-bindgen-cli --version 0.2.127, or set WASM_BINDGEN).
set -euo pipefail
cd "$(dirname "$0")"
BINDGEN=${WASM_BINDGEN:-wasm-bindgen}
# Small-size release settings for this build only.
export CARGO_PROFILE_RELEASE_OPT_LEVEL=s
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=abort
cargo build --release --locked --target wasm32-unknown-unknown
TARGET=${CARGO_TARGET_DIR:-target}
"$BINDGEN" --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir site/pkg --out-name playmand "$TARGET/wasm32-unknown-unknown/release/playmand_web.wasm"
# The harbor shot: its own package, fetched only when the visitor scrolls to it.
cargo build --release --locked --target wasm32-unknown-unknown --no-default-features --features harbor
"$BINDGEN" --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir site/pkg-harbor --out-name harbor "$TARGET/wasm32-unknown-unknown/release/playmand_web.wasm"
ls -l site/pkg site/pkg-harbor
