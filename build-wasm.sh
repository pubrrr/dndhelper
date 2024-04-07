#!/usr/bin/env bash

set -xe

echo "Building:"
# RUSTFLAGS due to https://github.com/mvlabat/bevy_egui/issues/269#issuecomment-2022279609
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --target wasm32-unknown-unknown --bin dndhelper --features "bevy"

echo "Generating WASM JS:"
wasm-bindgen --out-name dndhelper_wasm --out-dir ./public/wasm --target web target/wasm32-unknown-unknown/release/dndhelper.wasm

echo "Generating nations assets"
cargo run --release --bin generate_assets_file

echo "Copying assets:"
cp -r ./assets ./public/assets

echo "done building wasm"