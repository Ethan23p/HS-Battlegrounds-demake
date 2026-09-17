#!/usr/bin/env bash
# Rebuilds bg-wasm and regenerates web/pkg/ -- the two steps every change to
# crates/bg-sim or crates/bg-wasm needs before the browser sees it.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build -p bg-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/bg_wasm.wasm
