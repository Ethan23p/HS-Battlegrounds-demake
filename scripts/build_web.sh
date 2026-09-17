#!/usr/bin/env bash
# Rebuilds bg-wasm, regenerates web/pkg/, and refreshes the browser's copy of
# the shop roster -- the steps every change to crates/bg-sim, crates/bg-wasm,
# or assets/roster.ron needs before the browser sees it.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build -p bg-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/bg_wasm.wasm
cp assets/roster.ron web/roster.ron
